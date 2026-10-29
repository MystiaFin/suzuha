mod hsl;
pub mod oklch;
mod mode;
mod scheme;

use amane::{Color, Palette, Service};

use hsl::Hsl;
use scheme::{CATPPUCCIN, GRUVBOX, Scheme};

use crate::settings::Settings;

pub use mode::Mode;

// no tone gets more saturated than this
const MAX_SATURATION: f32 = 0.82;

// green and red to start from, pulled a little toward the accent
const SUCCESS_SEED: Color = Color::rgb(0xa6, 0xe3, 0xa1);
const DANGER_SEED: Color = Color::rgb(0xf3, 0x8b, 0xa8);

// the colors every component draws with, rebuilt from the wallpaper each frame
pub struct Theme {
    pub light: bool,

    pub background: Color,
    pub surface: Color,
    pub hover_surface: Color,
    pub selected_surface: Color,
    pub border: Color,

    pub text: Color,
    pub secondary_text: Color,
    pub muted_text: Color,

    pub accent: Color,
    pub accent_hover: Color,
    pub on_accent: Color,

    pub success: Color,
    pub danger: Color,
}

// how the dynamic palette is tuned in the settings
struct Tuning {
    saturation: f32,
    contrast: f32,
}

// every color is the wallpaper's darkest or most vivid color, re-lit, unless a fixed scheme is picked
pub fn current() -> Theme {
    let light = Mode::light();

    for_mode(light)
}

// the theme as it would be in light or dark, none to decide from the wallpaper
pub fn for_mode(forced: Option<bool>) -> Theme {
    let settings = Settings::read();

    match settings.text("scheme") {
        "gruvbox" => return fixed(&GRUVBOX),
        "catppuccin" => return fixed(&CATPPUCCIN),
        _ => {}
    }

    let tuning = Tuning {
        saturation: settings.number("saturation"),
        contrast: settings.number("contrast"),
    };

    // a hand-picked accent replaces the wallpaper's
    let manual_accent = if settings.flag("manual_accent") {
        Some(Color::from(settings.text("accent")))
    } else {
        None
    };

    drop(settings);

    let palette = Palette::read();

    // a choice made by hand wins, otherwise a bright wallpaper gets a light theme
    let light = forced.unwrap_or(palette.light());

    let base = palette.background();
    let seed = manual_accent.unwrap_or(palette.accent());

    let seed_saturation = hsl::from_color(seed).saturation;

    let tone = |color: Color, lightness: f32, min_saturation: f32| {
        tone(color, lightness, min_saturation, &tuning)
    };

    let background = pick(light, 0.94, 0.075);
    let surface = pick(light, 0.88, 0.12);

    let background = tone(base, background, pick(light, 0.08, 0.30));
    let surface = tone(base, surface, pick(light, 0.10, 0.28));

    let accent = tone(seed, pick(light, 0.42, 0.68), seed_saturation.max(0.58));
    let accent_hover = tone(seed, pick(light, 0.34, 0.78), seed_saturation.max(0.48));

    let text = tone(base, pick(light, 0.10, 0.91), 0.10);
    let secondary_text = tone(base, pick(light, 0.30, 0.72), 0.14);
    let muted_text = tone(base, pick(light, 0.42, 0.52), 0.16);

    let border = mix(surface, accent, pick(light, 0.26, 0.32));
    let hover_surface = mix(surface, accent, pick(light, 0.12, 0.20));
    let selected_surface = mix(surface, accent, pick(light, 0.20, 0.15));

    let success = tone(mix(SUCCESS_SEED, accent, 0.20), pick(light, 0.42, 0.70), 0.48);
    let danger = tone(mix(DANGER_SEED, accent, 0.20), pick(light, 0.42, 0.70), 0.48);

    // on a light theme the bar's accent pill is darkened so its text stays readable
    let accent = if light {
        mix(accent, Color::BLACK, 0.22)
    } else {
        accent
    };

    let on_accent = if luminance(accent) > 0.179 {
        tone(base, 0.08, 0.12)
    } else {
        tone(base, 0.96, 0.08)
    };

    Theme {
        light,
        background,
        surface,
        hover_surface,
        selected_surface,
        border,
        text,
        secondary_text,
        muted_text,
        accent,
        accent_hover,
        on_accent,
        success,
        danger,
    }
}

// a fixed scheme is always dark; selected sits between the surface and its hover
fn fixed(scheme: &Scheme) -> Theme {
    Theme {
        light: false,
        background: scheme.background,
        surface: scheme.surface,
        hover_surface: scheme.hover_surface,
        selected_surface: mix(scheme.surface, scheme.hover_surface, 0.5),
        border: scheme.border,
        text: scheme.text,
        secondary_text: scheme.secondary_text,
        muted_text: scheme.muted_text,
        accent: scheme.accent,
        accent_hover: scheme.accent_hover,
        on_accent: scheme.on_accent,
        success: scheme.success,
        danger: scheme.danger,
    }
}

fn pick(light: bool, when_light: f32, when_dark: f32) -> f32 {
    if light { when_light } else { when_dark }
}

/*
 * keeps the color's hue, sets its lightness, and keeps it at least a little
 * colorful; contrast pushes lightness away from the middle
 */
fn tone(color: Color, lightness: f32, min_saturation: f32, tuning: &Tuning) -> Color {
    let original = hsl::from_color(color);

    let saturation = original.saturation.min(MAX_SATURATION).max(min_saturation);

    let saturation = (saturation * tuning.saturation).clamp(0.0, 1.0);

    let lightness = 0.5 + (lightness - 0.5) * tuning.contrast;

    hsl::to_color(Hsl {
        hue: original.hue,
        saturation,
        lightness: lightness.clamp(0.02, 0.98),
    })
}

/*
 * amount 0 gives first, 1 gives second; each channel is weighted by its
 * color's alpha, so mixing from transparent fades in instead of passing black
 */
pub fn mix(first: Color, second: Color, amount: f32) -> Color {
    let first_alpha = f32::from(first.alpha()) / 255.0;
    let second_alpha = f32::from(second.alpha()) / 255.0;

    let alpha = first_alpha + (second_alpha - first_alpha) * amount;

    if alpha == 0.0 {
        return Color::TRANSPARENT;
    }

    let blend = |from: u8, to: u8| {
        let from = f32::from(from) * first_alpha;
        let to = f32::from(to) * second_alpha;

        ((from + (to - from) * amount) / alpha).round() as u8
    };

    Color::rgba(
        blend(first.red(), second.red()),
        blend(first.green(), second.green()),
        blend(first.blue(), second.blue()),
        (alpha * 255.0).round() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_fades_in_from_transparent() {
        let border = Color::rgb(200, 100, 50);

        assert_eq!(mix(Color::TRANSPARENT, border, 0.0).alpha(), 0);

        let half = mix(Color::TRANSPARENT, border, 0.5);

        assert_eq!((half.red(), half.green(), half.blue(), half.alpha()), (200, 100, 50, 128));

        assert_eq!(mix(Color::BLACK, Color::WHITE, 0.5).red(), 128);
    }
}

// relative luminance, to choose dark or light text on the accent
fn luminance(color: Color) -> f32 {
    let linear = |channel: u8| {
        let value = f32::from(channel) / 255.0;

        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };

    linear(color.red()) * 0.2126 + linear(color.green()) * 0.7152 + linear(color.blue()) * 0.0722
}

pub fn with_opacity(color: Color, opacity: f32) -> Color {
    let alpha = (opacity * 255.0).round() as u8;

    Color::rgba(color.red(), color.green(), color.blue(), alpha)
}

// a color at a new lightness with the untuned rules, for palettes other programs use
pub fn retone(color: Color, lightness: f32, min_saturation: f32) -> Color {
    let untuned = Tuning {
        saturation: 1.0,
        contrast: 1.0,
    };

    tone(color, lightness, min_saturation, &untuned)
}
