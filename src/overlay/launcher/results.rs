use std::env;

use amane::{Apps, Service};

use crate::settings::Settings;

// "> " opens the command palette, "!" lists tmux sessions
const COMMAND_PREFIX: &str = ">";
const TMUX_PREFIX: &str = "!";

pub enum Kind {
    // its place in Apps::list()
    App(usize),

    // opens the settings window, on its colors page for the color command
    Settings,
    Colors,

    // opens the wallpaper picker
    Wallpapers,

    // opens the file converter
    Converter,

    // switches the search to tmux sessions
    TmuxCommand,

    Tmux(String),
}

pub struct Entry {
    pub name: String,

    // an app's description, shown under its name when turned on
    pub description: Option<String>,

    // a nerd font glyph, for entries that have no app icon
    pub glyph: Option<&'static str>,

    pub kind: Kind,
}

pub fn empty_text(query: &str) -> &'static str {
    if query.starts_with(TMUX_PREFIX) {
        return "No tmux sessions found";
    }

    if query.starts_with(COMMAND_PREFIX) {
        return "No commands found";
    }

    "No applications found"
}

pub fn find(query: &str, sessions: &[String]) -> Vec<Entry> {
    if let Some(search) = query.strip_prefix(TMUX_PREFIX) {
        return tmux(&search.trim().to_lowercase(), sessions);
    }

    // with command mode off, ">" is searched for like any other letter
    if let Some(search) = query.strip_prefix(COMMAND_PREFIX) {
        if Settings::read().flag("launcher_commands") {
            return commands(&search.trim().to_lowercase());
        }
    }

    apps(&query.trim().to_lowercase())
}

// already sorted by name, so only the filter is left
fn apps(search: &str) -> Vec<Entry> {
    let apps = Apps::read();

    let described = Settings::read().flag("launcher_descriptions");

    let mut found = Vec::new();

    for (index, app) in apps.list().iter().enumerate() {
        if !app.name().to_lowercase().contains(search) {
            continue;
        }

        let description = if described {
            app.description().map(String::from)
        } else {
            None
        };

        found.push(Entry {
            name: String::from(app.name()),
            description,
            glyph: None,
            kind: Kind::App(index),
        });
    }

    found
}

// the commands turned on in the settings
fn commands(search: &str) -> Vec<Entry> {
    let all = [
        ("Settings", "\u{f0493}", "command_settings", Kind::Settings),
        ("Color scheme", "\u{f03d8}", "command_colors", Kind::Colors),
        ("Tmux sessions", "\u{f018d}", "command_tmux", Kind::TmuxCommand),
        ("Wallpapers", "\u{f02e9}", "command_wallpapers", Kind::Wallpapers),
        ("Converter", "\u{f04e1}", "command_converter", Kind::Converter),
    ];

    let settings = Settings::read();

    let mut found = Vec::new();

    for (name, glyph, key, kind) in all {
        if !settings.flag(key) {
            continue;
        }

        if !name.to_lowercase().contains(search) {
            continue;
        }

        found.push(Entry {
            name: String::from(name),
            description: None,
            glyph: Some(glyph),
            kind,
        });
    }

    found
}

fn tmux(search: &str, sessions: &[String]) -> Vec<Entry> {
    let mut found = Vec::new();

    for session in sessions {
        if !session.to_lowercase().contains(search) {
            continue;
        }

        found.push(Entry {
            name: session.clone(),
            description: None,
            glyph: None,
            kind: Kind::Tmux(session.clone()),
        });
    }

    found
}

pub fn read_sessions() -> Vec<String> {
    let listed = amane::output("tmux list-sessions -F '#{session_name}'");

    let mut sessions = Vec::new();

    for line in listed.lines() {
        let name = line.trim();

        if !name.is_empty() {
            sessions.push(String::from(name));
        }
    }

    sessions
}

// false when there is no $TERMINAL to attach in
pub fn attach(session: &str) -> bool {
    let Ok(terminal) = env::var("TERMINAL") else {
        return false;
    };

    amane::spawn(&format!("{terminal} -- tmux attach-session -t '{session}'"));

    true
}
