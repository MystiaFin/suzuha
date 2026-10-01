mod analysis;
mod search;

use std::process::Command;
use std::thread;
use std::time::Duration;

use amane::Service;

use crate::wallpaper::Wallpaper;
use crate::settings::Settings;

use analysis::{Analysis, Area};

pub use search::{Name, Request, arrange};

// the small copy's longest side; enough to tell calm areas from busy ones
const SAMPLE: u32 = 160;

// how often the chosen wallpaper is checked for a change
const CHECK: Duration = Duration::from_millis(500);

// how often a running wallpaper transition is checked for its end
const SETTLE: Duration = Duration::from_millis(16);

// the chosen wallpaper, read once per change for where the desktop cards fit
#[derive(Default)]
pub struct Placement {
    path: String,

    // none until the first wallpaper is read, or when it can't be
    analysis: Option<Analysis>,
}

/*
 * the wallpaper is shrunk and read off the draw thread, since that takes
 * a few hundred milliseconds; the view only looks things up in the result
 */
impl Service for Placement {
    fn new() -> Self {
        Self::default()
    }

    fn listen() {
        loop {
            let path = Wallpaper::read().path.clone();

            let placed = Self::read().path.clone();

            // locked cards stay where the first wallpaper put them
            let locked = !placed.is_empty() && Settings::read().flag("floating_lock_placement");

            if path != placed && !locked {
                let analysis = analyse(&path);

                // the cards move once the new wallpaper has finished coming in
                while Wallpaper::read().shown != path && Wallpaper::read().path == path {
                    thread::sleep(SETTLE);
                }

                // another wallpaper was picked meanwhile, so this reading is already stale
                if Wallpaper::read().path != path {
                    continue;
                }

                let mut placement = Self::write();

                placement.path = path;
                placement.analysis = analysis;
            }

            thread::sleep(CHECK);
        }
    }
}

impl Placement {
    pub fn analysis(&self) -> Option<&Analysis> {
        self.analysis.as_ref()
    }

    // a wallpaper was tried, even if it could not be read
    pub fn settled(&self) -> bool {
        !self.path.is_empty()
    }
}

/*
 * the part of the small copy a screen this shape shows: the wallpaper
 * covers the screen, so its longer side is cut evenly on both ends
 */
pub fn crop(analysis: &Analysis, screen_width: f32, screen_height: f32) -> Area {
    let width = analysis.width as f32;
    let height = analysis.height as f32;

    let screen_ratio = screen_width / screen_height;

    if width / height > screen_ratio {
        let shown = (height * screen_ratio).round().max(1.0) as usize;

        return Area {
            x: (analysis.width - shown) / 2,
            y: 0,
            width: shown,
            height: analysis.height,
        };
    }

    let shown = (width / screen_ratio).round().max(1.0) as usize;

    Area {
        x: 0,
        y: (analysis.height - shown) / 2,
        width: analysis.width,
        height: shown,
    }
}

// imagemagick reads every format and orientation, and hands back a plain ppm
fn analyse(path: &str) -> Option<Analysis> {
    let size = format!("{SAMPLE}x{SAMPLE}");

    let output = Command::new("magick")
        .args([path, "-auto-orient", "-resize", &size, "-colorspace", "sRGB", "-depth", "8", "ppm:-"])
        .output()
        .ok()?;

    if !output.status.success() {
        eprintln!("amane: failed to read the wallpaper for the desktop cards: {path}");

        return None;
    }

    let (width, height, pixels) = ppm(&output.stdout)?;

    Analysis::new(pixels, width, height)
}

// "P6\n160 90\n255\n" and then the rgb bytes
fn ppm(bytes: &[u8]) -> Option<(usize, usize, &[u8])> {
    let mut fields = Vec::new();
    let mut start = 0;
    let mut index = 0;

    while fields.len() < 4 && index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            if index > start {
                fields.push(std::str::from_utf8(&bytes[start..index]).ok()?);
            }

            start = index + 1;
        }

        index += 1;
    }

    let [kind, width, height, _] = fields.as_slice() else {
        return None;
    };

    if *kind != "P6" {
        return None;
    }

    Some((width.parse().ok()?, height.parse().ok()?, &bytes[start..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_ppm_and_crops() {
        let mut bytes = b"P6\n4 2\n255\n".to_vec();
        bytes.extend([0u8; 4 * 2 * 3]);

        let (width, height, pixels) = ppm(&bytes).expect("failed to read ppm");

        assert_eq!((width, height, pixels.len()), (4, 2, 24));

        let analysis = Analysis::new(pixels, width, height).expect("failed to analyse");

        // a 2:1 picture on a square screen shows its middle half
        assert_eq!(crop(&analysis, 100.0, 100.0), Area { x: 1, y: 0, width: 2, height: 2 });
    }
}
