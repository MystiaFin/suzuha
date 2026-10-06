use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use amane::Service;

// the mixed sink desktop and mic sound both loop into, for wf-recorder's one audio device
const MIX_SINK: &str = "amane_rec";

#[derive(Clone, Copy, PartialEq)]
pub enum Audio {
    None,
    Desktop,
    DesktopMic,
    Mic,
}

pub const AUDIO: [Audio; 4] = [Audio::None, Audio::Desktop, Audio::Mic, Audio::DesktopMic];

impl Audio {
    pub fn label(self) -> &'static str {
        match self {
            Audio::None => "No sound",
            Audio::Desktop => "Desktop sound",
            Audio::DesktopMic => "Desktop + mic",
            Audio::Mic => "Mic only",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Audio::None => "󰝟",
            Audio::Desktop => "󰕾",
            Audio::DesktopMic => "󰋋",
            Audio::Mic => "󰍬",
        }
    }

    fn flag(self) -> String {
        match self {
            Audio::None => String::new(),
            Audio::Desktop => String::from("--audio=@DEFAULT_MONITOR@"),
            Audio::DesktopMic => format!("--audio={MIX_SINK}.monitor"),
            Audio::Mic => String::from("--audio=@DEFAULT_SOURCE@"),
        }
    }
}

pub struct Recorder {
    // wf-recorder, only while recording
    child: Option<Child>,

    started: Instant,

    // set once stop is asked for, so an exit before that counts as a failure
    stopping: bool,

    // pactl modules loaded for the desktop and mic mix, unloaded when recording ends
    modules: Vec<String>,

    pub audio: Audio,

}

impl Service for Recorder {
    fn new() -> Self {
        Self {
            child: None,
            started: Instant::now(),
            stopping: false,
            modules: Vec::new(),
            audio: Audio::None,
        }
    }

    // ticks the elapsed time, and notices wf-recorder exiting
    fn update(&mut self) -> bool {
        let Some(child) = self.child.as_mut() else {
            return false;
        };

        if child.try_wait().is_ok_and(|exited| exited.is_none()) {
            return true;
        }

        self.child = None;

        for module in self.modules.drain(..) {
            amane::spawn(&format!("pactl unload-module {module}"));
        }

        let message = if self.stopping {
            "Recording saved to ~/Videos"
        } else {
            "Recording failed, is wf-recorder installed?"
        };

        amane::spawn(&format!("notify-send -a Recorder Recorder '{message}'"));

        true
    }
}

pub fn toggle() {
    let mut recorder = Recorder::write();

    if let Some(child) = recorder.child.as_ref() {
        // SIGINT lets wf-recorder finish writing the file; update() reaps it
        amane::spawn(&format!("kill -INT {}", child.id()));

        recorder.stopping = true;

        return;
    }

    if recorder.audio == Audio::DesktopMic {
        recorder.modules = mix();
    }

    // ponytail: parses niri's text output for the focused monitor, use --json if that format changes
    let script = format!(
        "mkdir -p ~/Videos && exec wf-recorder -o \"$(niri msg focused-output | sed -n 's/.*(\\(.*\\))/\\1/p;q')\" {} -f ~/Videos/$(date +%F_%H-%M-%S).mp4",
        recorder.audio.flag()
    );

    let child = Command::new("sh")
        .args(["-c", &script])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    recorder.child = child.ok();
    recorder.started = Instant::now();
    recorder.stopping = false;
}

// ponytail: pactl loopback mix, switch to gpu-screen-recorder if the two sources drift apart
fn mix() -> Vec<String> {
    let sink = format!("sink_name={MIX_SINK}");
    let into = format!("sink={MIX_SINK}");

    let loads: [&[&str]; 3] = [
        &["module-null-sink", &sink],
        &["module-loopback", "source=@DEFAULT_MONITOR@", &into],
        &["module-loopback", "source=@DEFAULT_SOURCE@", &into],
    ];

    let mut modules = Vec::new();

    for arguments in loads {
        let output = Command::new("pactl").arg("load-module").args(arguments).output();

        if let Ok(output) = output
            && output.status.success()
        {
            modules.push(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
    }

    modules
}

// like "01:23" while recording, for the bar tray and the record button
pub fn status() -> Option<String> {
    let recorder = Recorder::read();

    recorder.child.as_ref()?;

    Some(time(recorder.started.elapsed()))
}

fn time(gone: Duration) -> String {
    let seconds = gone.as_secs();

    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

pub fn ipc(_arguments: &[String]) -> String {
    toggle();

    String::from("ok")
}
