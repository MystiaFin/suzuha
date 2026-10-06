mod btop;
mod cava;
mod gtk;
mod spotify;
mod terminal;
mod tmux;
mod vesktop;

use std::env;
use std::fs;
use std::path::Path;

use amane::{Color, Service};

use crate::settings::Settings;
use crate::theme::{self, Theme};

// the settings key and the writer of each program that takes the shell's colors
const PROGRAMS: [(&str, fn(&Theme)); 7] = [
    ("integration_gtk", gtk::export),
    ("integration_terminal", terminal::export),
    ("integration_tmux", tmux::export),
    ("integration_vesktop", vesktop::export),
    ("integration_spotify", spotify::export),
    ("integration_btop", btop::export),
    ("integration_cava", cava::export),
];

/*
 * looks at the theme once a second and writes the colors out to every
 * program turned on in the settings, only when the colors or the programs
 * changed; it runs on its own thread, so drawing never waits on the files
 */
#[derive(Default)]
pub struct Export {
    // the colors and programs written last, so an unchanged theme writes nothing
    written: String,
}

impl Service for Export {
    fn new() -> Self {
        Self::default()
    }

    // nothing a window shows changes here
    fn update(&mut self) -> bool {
        let theme = theme::current();

        let settings = Settings::read();

        let mut turned_on = Vec::new();

        for (key, export) in PROGRAMS {
            if settings.flag(key) {
                turned_on.push((key, export));
            }
        }

        drop(settings);

        let mut fingerprint = fingerprint(&theme);

        for (key, _) in &turned_on {
            fingerprint.push_str(key);
        }

        if fingerprint == self.written {
            return false;
        }

        self.written = fingerprint;

        for (_, export) in turned_on {
            export(&theme);
        }

        false
    }
}

fn fingerprint(theme: &Theme) -> String {
    let colors = [
        theme.background,
        theme.surface,
        theme.hover_surface,
        theme.selected_surface,
        theme.border,
        theme.text,
        theme.secondary_text,
        theme.muted_text,
        theme.accent,
        theme.accent_hover,
        theme.on_accent,
        theme.success,
        theme.danger,
    ];

    let mut text = format!("{}", theme.light);

    for color in colors {
        text.push_str(&hex(color));
    }

    text
}

// "#rrggbb", never with alpha
pub fn hex(color: Color) -> String {
    format!("#{:02x}{:02x}{:02x}", color.red(), color.green(), color.blue())
}

pub fn home() -> String {
    env::var("HOME").expect("failed to find home: HOME is not set")
}

pub fn config_home() -> String {
    env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home()))
}

// where the generated files go, next to the settings
pub fn state_home() -> String {
    format!("{}/.local/state/amane", home())
}

/*
 * written next to the file and renamed over it, so a program reading it
 * never sees half a file; a failed write only leaves the old colors
 */
pub fn write(path: &str, text: &str) {
    let path = Path::new(path);

    if let Some(folder) = path.parent() {
        let _ = fs::create_dir_all(folder);
    }

    let partial = path.with_extension("partial");

    if fs::write(&partial, text).is_ok() {
        let _ = fs::rename(&partial, path);
    }
}

// every @NAME@ in the template swapped for its color
pub fn fill(template: &str, names: &[(&str, String)]) -> String {
    let mut text = String::from(template);

    for (name, value) in names {
        text = text.replace(name, value);
    }

    text
}
