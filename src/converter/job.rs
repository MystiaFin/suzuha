use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use amane::Service;

use super::compress::{self, Level, Speed};
use super::documents::{Document, Kind};
use super::formats::Format;
use super::{Queue, Status, Task};

// libreoffice can hang on a file it chokes on, which would hold up the whole queue
const DOCUMENT_TIMEOUT: Duration = Duration::from_secs(300);

// converts the waiting files one after another on a thread of its own
pub fn start() {
    {
        let mut queue = Queue::write();

        if queue.running {
            return;
        }

        queue.running = true;
    }

    thread::spawn(run);
}

fn run() {
    loop {
        let next = {
            let mut queue = Queue::write();

            let task = queue.task;

            let waiting = queue.files.iter_mut().find(|file| file.status == Status::Waiting);

            match waiting {
                Some(file) => {
                    file.status = Status::Converting;
                    file.progress = None;

                    Some((file.path.clone(), task))
                }

                None => {
                    queue.running = false;

                    None
                }
            }
        };

        let Some((input, task)) = next else {
            return;
        };

        let status = match task {
            Task::Convert(format) => convert(&input, format),
            Task::Compress(level, speed) => compress(&input, level, speed),
            Task::Document(document) => convert_document(&input, document),
        };

        // files are only added while this runs, so the same path still names the same file
        let mut queue = Queue::write();

        for file in &mut queue.files {
            if file.path == input && file.status == Status::Converting {
                file.status = status;

                break;
            }
        }
    }
}

fn convert(input: &Path, format: &Format) -> Status {
    let output = free_path(input, "", format.extension);

    ffmpeg(input, format.arguments, &output)
}

// kept beside the original in the same format, as name-compressed
fn compress(input: &Path, level: Level, speed: Speed) -> Status {
    let extension = input.extension().unwrap_or_default().to_string_lossy();

    let Some(arguments) = compress::arguments(&extension, level, speed) else {
        return Status::Failed;
    };

    let output = free_path(input, "-compressed", &extension);

    ffmpeg(input, &arguments, &output)
}

fn ffmpeg(input: &Path, arguments: &[&str], output: &Path) -> Status {
    let duration = duration(input);

    // arguments are passed straight to ffmpeg, so file names need no quoting
    let child = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-nostats", "-progress", "pipe:1", "-n", "-i"])
        .arg(input)
        .args(arguments)
        .arg(output)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();

    let Ok(mut child) = child else {
        return Status::Failed;
    };

    /*
     * the progress report has to be read to the end even without a
     * duration, or ffmpeg stalls once the pipe is full
     */
    if let Some(stdout) = child.stdout.take() {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let Some(duration) = duration else {
                continue;
            };

            // "N/A" before the first frame is out
            let Some(Ok(micros)) = line.strip_prefix("out_time_us=").map(str::parse::<f32>) else {
                continue;
            };

            let seconds = micros / 1_000_000.0;

            set_progress(input, (seconds / duration).clamp(0.0, 1.0));
        }
    }

    let finished = child.wait();

    if finished.is_ok_and(|status| status.success()) {
        return Status::Done;
    }

    // the name was free before, so whatever is there is ffmpeg's half-written file
    let _ = fs::remove_file(output);

    Status::Failed
}

// in seconds, none for a still image or anything ffprobe can't time
fn duration(input: &Path) -> Option<f32> {
    let probed = Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1"])
        .arg(input)
        .stderr(Stdio::null())
        .output()
        .ok()?;

    let seconds: f32 = String::from_utf8_lossy(&probed.stdout).trim().parse().ok()?;

    // a still image reports a single frame's length
    if seconds < 0.5 {
        return None;
    }

    Some(seconds)
}

fn set_progress(input: &Path, progress: f32) {
    let mut queue = Queue::write();

    for file in &mut queue.files {
        if file.path == input && file.status == Status::Converting {
            file.progress = Some(progress);

            break;
        }
    }
}

/*
 * soffice names its output after the input and overwrites what is there,
 * so it writes into a hidden folder first and the file is moved out to a free name
 */
fn convert_document(input: &Path, document: &Document) -> Status {
    let Some(folder) = input.parent() else {
        return Status::Failed;
    };

    let scratch = folder.join(format!(".amane-converting-{}", process::id()));

    if fs::create_dir_all(&scratch).is_err() {
        return Status::Failed;
    }

    let status = soffice(input, document, &scratch);

    let _ = fs::remove_dir_all(&scratch);

    status
}

fn soffice(input: &Path, document: &Document, scratch: &Path) -> Status {
    // a profile of its own, so an open libreoffice window doesn't take the job over
    let profile = env::temp_dir().join("amane-libreoffice");

    let mut command = Command::new("soffice");

    command
        .arg(format!("-env:UserInstallation=file://{}", profile.display()))
        .args(["--headless", "--norestore"]);

    let from_pdf = input.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"));

    // a pdf opens as a drawing unless writer is told to read it as text
    if from_pdf && document.kind == Kind::Text {
        command.arg("--infilter=writer_pdf_import");
    }

    // soffice starts helpers of its own, a group of their own lets a timeout stop them all
    let child = command
        .arg("--convert-to")
        .arg(document.extension)
        .arg("--outdir")
        .arg(scratch)
        .arg(input)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn();

    let Ok(mut child) = child else {
        return Status::Failed;
    };

    let deadline = Instant::now() + DOCUMENT_TIMEOUT;

    while child.try_wait().is_ok_and(|exited| exited.is_none()) {
        if Instant::now() > deadline {
            let _ = Command::new("kill").arg("-KILL").arg(format!("-{}", child.id())).status();

            let _ = child.wait();

            return Status::Failed;
        }

        thread::sleep(Duration::from_millis(200));
    }

    // a conversion libreoffice can't do still exits fine, it just writes nothing
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();

    let written = scratch.join(format!("{stem}.{}", document.extension));

    if !written.exists() {
        return Status::Failed;
    }

    let output = free_path(input, "", document.extension);

    if fs::rename(&written, &output).is_err() {
        return Status::Failed;
    }

    Status::Done
}

// beside the input, with -1, -2 and so on added when the name is taken
fn free_path(input: &Path, suffix: &str, extension: &str) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();

    let stem = format!("{stem}{suffix}");

    let mut path = input.with_file_name(format!("{stem}.{extension}"));

    let mut number = 1;

    while path.exists() {
        path = input.with_file_name(format!("{stem}-{number}.{extension}"));

        number += 1;
    }

    path
}
