use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

use amane::Service;

use super::formats::Format;
use super::{Queue, Status};

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

            let format = queue.format;

            let waiting = queue.files.iter_mut().find(|file| file.status == Status::Waiting);

            match waiting {
                Some(file) => {
                    file.status = Status::Converting;

                    Some((file.path.clone(), format))
                }

                None => {
                    queue.running = false;

                    None
                }
            }
        };

        let Some((input, format)) = next else {
            return;
        };

        let status = convert(&input, format);

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
    let output = free_path(input, format.extension);

    // arguments are passed straight to ffmpeg, so file names need no quoting
    let finished = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-n", "-i"])
        .arg(input)
        .args(format.arguments)
        .arg(&output)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if finished.is_ok_and(|status| status.success()) {
        return Status::Done;
    }

    // the name was free before, so whatever is there is ffmpeg's half-written file
    let _ = fs::remove_file(&output);

    Status::Failed
}

// beside the input, with -1, -2 and so on added when the name is taken
fn free_path(input: &Path, extension: &str) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();

    let mut path = input.with_file_name(format!("{stem}.{extension}"));

    let mut number = 1;

    while path.exists() {
        path = input.with_file_name(format!("{stem}-{number}.{extension}"));

        number += 1;
    }

    path
}
