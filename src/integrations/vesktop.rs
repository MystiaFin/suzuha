use std::fs;

use super::{config_home, fill, hex};
use crate::theme::Theme;

const TEMPLATE: &str = include_str!("vesktop.css");

// the block this owns inside quick css, everything around it is kept
const START: &str = "/* amane-theme:start */";
const END: &str = "/* amane-theme:end */";

// vencord's quick css; rewritten in place, since vesktop watches the file itself
pub fn export(theme: &Theme) {
    let path = format!("{}/vesktop/settings/quickCss.css", config_home());

    let accent = theme.accent;

    let accent_rgb = format!("{}, {}, {}", accent.red(), accent.green(), accent.blue());

    let scheme = if theme.light { "light" } else { "dark" };

    let names = [
        ("@COLOR_SCHEME@", String::from(scheme)),
        ("@BACKGROUND@", hex(theme.background)),
        ("@SURFACE@", hex(theme.surface)),
        ("@HOVER@", hex(theme.hover_surface)),
        ("@SELECTED@", hex(theme.selected_surface)),
        ("@BORDER@", hex(theme.border)),
        ("@TEXT@", hex(theme.text)),
        ("@SECONDARY@", hex(theme.secondary_text)),
        ("@MUTED@", hex(theme.muted_text)),
        ("@ACCENT_HOVER@", hex(theme.accent_hover)),
        ("@ACCENT_RGB@", accent_rgb),
        ("@ACCENT@", hex(accent)),
        ("@ON_ACCENT@", hex(theme.on_accent)),
        ("@SUCCESS@", hex(theme.success)),
        ("@DANGER@", hex(theme.danger)),
    ];

    let block = format!("{START}\n{}\n{END}\n", fill(TEMPLATE, &names).trim());

    let existing = fs::read_to_string(&path).unwrap_or_default();

    let kept = without_block(&existing);

    let text = if kept.is_empty() {
        block
    } else {
        format!("{kept}\n\n{block}")
    };

    let _ = fs::write(&path, text);
}

// the file with the old block cut out
fn without_block(text: &str) -> String {
    let (Some(start), Some(end)) = (text.find(START), text.find(END)) else {
        return String::from(text.trim());
    };

    if end < start {
        return String::from(text.trim());
    }

    let before = &text[..start];
    let after = &text[end + END.len()..];

    format!("{before}{after}").trim().to_string()
}
