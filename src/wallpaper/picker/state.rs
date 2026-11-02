use std::env;
use std::fs;

use amane::Service;

use crate::motion::{self, Glide};
use crate::settings::Settings;

// the formats amane can decode
const EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];

// how long the cards take to slide one place
const SLIDE: u64 = 320;

// the wallpaper picker, a band of cards across the middle of the screen
pub struct Picker {
    pub shown: bool,

    // every wallpaper in the folder, sorted by name
    pub files: Vec<String>,

    // which card is in the middle, counted without wrapping, so a slide past the end keeps going
    pub position: Glide,
    pub target: i64,

    // fades the band in and out, 0 to 1
    pub opacity: Glide,
}

impl Service for Picker {
    fn new() -> Self {
        Self {
            shown: false,
            files: Vec::new(),
            position: motion::spatial(0.0, SLIDE),
            target: 0,
            opacity: motion::effects(0.0, motion::DEFAULT_EFFECTS),
        }
    }

    // it only changes through input
    fn listen() {}
}

impl Picker {
    // opens on the wallpaper that is on screen, with the folder read again
    pub fn show(&mut self, current: &str) {
        self.files = list_folder();

        let index = self
            .files
            .iter()
            .position(|file| file == current)
            .unwrap_or(0);

        self.target = index as i64;
        self.position = motion::spatial(index as f32, SLIDE);

        self.shown = true;
        self.opacity.to(1.0);
    }

    pub fn hide(&mut self) {
        self.shown = false;
        self.opacity.to(0.0);
    }

    // by any number of cards, left when negative
    pub fn slide(&mut self, cards: i64) {
        if self.files.len() < 2 {
            return;
        }

        self.target += cards;
        self.position.to(self.target as f32);
    }

    pub fn selected(&self) -> Option<&String> {
        let count = self.files.len() as i64;

        if count == 0 {
            return None;
        }

        let index = self.target.rem_euclid(count);

        self.files.get(index as usize)
    }
}

// the folder from the settings, where a leading ~ means home
pub fn folder() -> String {
    let home = env::var("HOME").expect("failed to find home: HOME is not set");

    let folder = String::from(Settings::read().text("wallpaper_folder"));

    match folder.strip_prefix('~') {
        Some(rest) => format!("{home}{rest}"),
        None => folder,
    }
}

pub fn list_folder() -> Vec<String> {
    let Ok(entries) = fs::read_dir(folder()) else {
        return Vec::new();
    };

    let mut files = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_lowercase)
            .unwrap_or_default();

        if path.is_file() && EXTENSIONS.contains(&extension.as_str()) {
            files.push(path.display().to_string());
        }
    }

    // by name whatever its case, like a file manager lists them
    files.sort_by_key(|file| file.to_lowercase());

    files
}
