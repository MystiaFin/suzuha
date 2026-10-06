use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use amane::Service;

// every setting and the value it has until changed
const DEFAULTS: [(&str, &str); 69] = [
    // appearance
    ("blur_strength", "1"),
    ("reduce_transparency", "false"),
    // colors
    ("scheme", "dynamic"),
    ("color_mode", "auto"),
    ("manual_accent", "false"),
    ("accent", "#89b4fa"),
    ("saturation", "1"),
    ("contrast", "1"),
    ("follow_wallpaper", "true"),
    // bar
    ("bar_position", "top"),
    ("bar_height", "40"),
    ("bar_auto_hide", "false"),
    ("workspace_style", "pill"),
    ("bar_logo", "true"),
    ("bar_workspaces", "true"),
    ("bar_workspace_name", "true"),
    ("bar_audio", "true"),
    ("bar_media", "true"),
    ("bar_clock", "true"),
    ("bar_battery", "true"),
    ("bar_memory", "true"),
    ("bar_tray", "true"),
    ("clock_24_hour", "false"),
    ("clock_seconds", "false"),
    // launcher
    ("launcher_width", "620"),
    ("launcher_rows", "9"),
    ("launcher_descriptions", "false"),
    ("launcher_icons", "true"),
    ("launcher_remember_query", "false"),
    ("launcher_commands", "true"),
    ("command_settings", "true"),
    ("command_colors", "true"),
    ("command_tmux", "true"),
    ("command_wallpapers", "true"),
    ("command_converter", "true"),
    // wallpaper
    ("wallpaper_folder", "~/Pictures/Wallpapers"),
    ("wallpaper_transition", "circle"),
    ("wallpaper_duration", "1200"),
    ("wallpaper_shuffle", "false"),
    ("wallpaper_shuffle_minutes", "30"),
    // behavior
    ("launcher_close_on_launch", "true"),
    ("launcher_escape_clears", "true"),
    ("click_outside_dismiss", "false"),
    ("animation_speed", "1"),
    ("reduce_motion", "false"),
    // floating widgets
    ("floating_visibility", "desktop"),
    ("floating_scale", "1"),
    ("floating_opacity", "1"),
    ("floating_lock_placement", "false"),
    ("widget_clock", "true"),
    ("widget_weather", "true"),
    ("widget_cpu_temperature", "true"),
    ("widget_cpu_usage", "true"),
    ("widget_gpu_temperature", "true"),
    ("widget_uv", "true"),
    ("widget_humidity", "true"),
    ("widget_air_quality", "true"),
    // weather
    ("weather_place", ""),
    ("weather_latitude", ""),
    ("weather_longitude", ""),
    ("weather_unit", "celsius"),
    ("weather_minutes", "15"),
    // integrations
    ("integration_gtk", "false"),
    ("integration_terminal", "false"),
    ("integration_tmux", "false"),
    ("integration_vesktop", "false"),
    ("integration_spotify", "false"),
    ("integration_btop", "false"),
    ("integration_cava", "false"),
];

/*
 * the shell's settings, kept across restarts as lines like "bar_height=40";
 * the settings window changes a draft, which the shell only sees once applied
 */
pub struct Settings {
    values: HashMap<String, String>,

    draft: HashMap<String, String>,
}

impl Service for Settings {
    fn new() -> Self {
        let mut values = HashMap::new();

        for (key, value) in DEFAULTS {
            values.insert(String::from(key), String::from(value));
        }

        let saved = fs::read_to_string(path()).unwrap_or_default();

        // the color mode used to have a file of its own
        let old_mode = fs::read_to_string(old_file("mode")).unwrap_or_default();

        if let Some(slot) = values.get_mut("color_mode") {
            if matches!(old_mode.trim(), "light" | "dark") {
                *slot = String::from(old_mode.trim());
            }
        }

        // so did the weather's location, as lines like "latitude=6.18"
        let old_weather = fs::read_to_string(old_file("weather")).unwrap_or_default();

        for line in old_weather.lines() {
            let Some((name, value)) = line.split_once('=') else {
                continue;
            };

            let key = match name.trim() {
                "name" => "weather_place",
                "latitude" => "weather_latitude",
                "longitude" => "weather_longitude",
                _ => continue,
            };

            values.insert(String::from(key), String::from(value.trim()));
        }

        // keys that are no longer settings are dropped
        for line in saved.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };

            if let Some(slot) = values.get_mut(key) {
                *slot = String::from(value);
            }
        }

        Self {
            draft: values.clone(),
            values,
        }
    }

    // it only changes through input
    fn listen() {}
}

impl Settings {
    // what the shell uses
    pub fn text(&self, key: &str) -> &str {
        self.values.get(key).expect("failed to find setting: unknown key")
    }

    pub fn flag(&self, key: &str) -> bool {
        self.text(key) == "true"
    }

    // a value that doesn't parse falls back to 1, which every number here can live with
    pub fn number(&self, key: &str) -> f32 {
        self.text(key).parse().unwrap_or(1.0)
    }

    // what the settings window shows, applied or not
    pub fn staged(&self, key: &str) -> &str {
        self.draft.get(key).expect("failed to find setting: unknown key")
    }

    pub fn staged_flag(&self, key: &str) -> bool {
        self.staged(key) == "true"
    }

    pub fn staged_number(&self, key: &str) -> f32 {
        self.staged(key).parse().unwrap_or(1.0)
    }

    // whether the draft holds something the shell doesn't use yet
    pub fn changed(&self) -> bool {
        self.draft != self.values
    }

    // into the draft only, applied later
    pub fn stage(key: &str, value: impl ToString) {
        let mut settings = Self::write();

        let slot = settings.draft.get_mut(key).expect("failed to find setting: unknown key");

        *slot = value.to_string();
    }

    pub fn stage_toggle(key: &str) {
        let on = Self::read().staged_flag(key);

        Self::stage(key, !on);
    }

    // every setting back to its default, still to be applied
    pub fn stage_defaults() {
        let mut settings = Self::write();

        for (key, value) in DEFAULTS {
            settings.draft.insert(String::from(key), String::from(value));
        }
    }

    // straight to the shell and the file, for switches outside the settings window
    pub fn set(key: &str, value: impl ToString) {
        let mut settings = Self::write();

        let value = value.to_string();

        settings.values.insert(String::from(key), value.clone());
        settings.draft.insert(String::from(key), value);

        settings.save();
    }

    pub fn apply() {
        let mut settings = Self::write();

        settings.values = settings.draft.clone();

        settings.save();
    }

    // the draft starts over from what the shell uses
    pub fn discard() {
        let mut settings = Self::write();

        settings.draft = settings.values.clone();
    }

    // losing the file only means the next start uses the defaults
    fn save(&self) {
        let mut text = String::new();

        for (key, _) in DEFAULTS {
            text.push_str(&format!("{key}={}\n", self.values[key]));
        }

        let path = path();

        let folder = path.parent().expect("failed to find the state folder");

        let _ = fs::create_dir_all(folder);
        let _ = fs::write(&path, text);
    }
}

// where a setting lived before it moved in here
fn old_file(name: &str) -> PathBuf {
    let home = env::var("HOME").expect("failed to find home: HOME is not set");

    PathBuf::from(format!("{home}/.local/state/amane/{name}"))
}

fn path() -> PathBuf {
    let home = env::var("HOME").expect("failed to find home: HOME is not set");

    PathBuf::from(format!("{home}/.local/state/amane/settings"))
}
