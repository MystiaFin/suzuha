use amane::Service;

use super::super::page::Page;
use super::super::{Confirm, Settings, Shown, switch};

// the key, title, what the row says, and what turning it on does
const INTEGRATIONS: [(&str, &str, &str, &str); 7] = [
    (
        "integration_gtk",
        "GTK",
        "GTK 3/4 and desktop preferences",
        "This writes generated GTK themes, changes gtk-theme, icon-theme and color-scheme through dconf, replaces ~/.config/gtk-4.0/gtk.css with a symlink, and restarts xdg-desktop-portal-gnome.",
    ),
    (
        "integration_terminal",
        "Terminals",
        "Kitty and foot",
        "This overwrites the kitty and foot palette files, reloads kitty windows, signals all running foot processes, and writes color escape sequences to their terminals.",
    ),
    (
        "integration_tmux",
        "tmux",
        "Status line and pane colors",
        "This overwrites tmux-colors.conf and asks running tmux servers to source that file.",
    ),
    (
        "integration_vesktop",
        "Vesktop",
        "Vencord Quick CSS",
        "This edits ~/.config/vesktop/settings/quickCss.css. Content outside the managed theme block is kept, but the file itself is rewritten.",
    ),
    (
        "integration_spotify",
        "Spotify",
        "Spicetify color stylesheet",
        "This creates and overwrites the Spotify stylesheet under your cache directory.",
    ),
    (
        "integration_btop",
        "btop",
        "Generated terminal monitor theme",
        "This creates and overwrites ~/.config/btop/themes/amane.theme.",
    ),
    (
        "integration_cava",
        "Cava",
        "Generated visualizer theme",
        "This creates and overwrites ~/.config/cava/themes/amane.",
    ),
];

// turning one on asks first, turning it off doesn't
pub fn build(page: &mut Page) {
    for (key, title, detail, warning) in INTEGRATIONS {
        switch::add_with(page, key, title, detail, move || {
            if Settings::read().staged_flag(key) {
                Settings::stage(key, false);

                return;
            }

            Shown::write().confirm = Some(Confirm::Integration { key, title, warning });
        });
    }

    page.end_group();
}
