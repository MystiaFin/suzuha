use amane::{
    Brightness, Center, Color, End, Point, Pointer, Rectangle, Row, Service, Stack, Start, Text,
    Weight, children,
};

use super::{fade_target, hover};
use crate::fonts;
use crate::motion::{self, DEFAULT_SPATIAL};
use crate::overlay::Overlay;
use crate::theme::{self, Mode, Theme};

pub const HEIGHT: f32 = 46.0;

const GAP: f32 = 10.0;

const MODE_WIDTH: f32 = 42.0;
const MODE_HEIGHT: f32 = 38.0;

const SLIDER_RADIUS: f32 = 16.0;

// the track starts past the sun icon and ends before the percentage
const TRACK_LEFT: f32 = 48.0;
const TRACK_HEIGHT: f32 = 6.0;
const TRACK_GAP: f32 = 12.0;

const THUMB_WIDTH: f32 = 4.0;
const THUMB_HEIGHT: f32 = 20.0;

const SIDE_PADDING: f32 = 14.0;
const PERCENT_WIDTH: f32 = 38.0;

const MOON_ICON: &str = "󰖔";
const SUN_ICON: &str = "󰖙";
const BRIGHTNESS_ICON: &str = "󰃠";

// the light and dark switch beside the screen brightness slider
pub fn view(overlay: &Overlay, theme: &Theme, width: f32) -> Row {
    let slider_width = width - MODE_WIDTH - GAP;

    Row::new(children![
        mode_button(overlay, theme),
        slider(theme, slider_width)
    ])
    .gap(GAP)
    .align(Center)
}

/*
 * shows a moon while light, to go dark, and a sun while dark; they turn
 * and shrink into each other as the mode changes
 */
fn mode_button(overlay: &Overlay, theme: &Theme) -> Rectangle {
    let hover_name = String::from("mode");

    let amount = motion::fade(&hover_name, fade_target(overlay, &hover_name));

    let light = motion::follow(
        "mode:light",
        if theme.light { 1.0 } else { 0.0 },
        DEFAULT_SPATIAL,
    );

    let fill = theme::mix(theme.selected_surface, theme.accent, amount);
    let icon_color = theme::mix(theme.accent, theme.on_accent, amount);

    let moon = turning_icon(MOON_ICON, icon_color, light, -90.0);
    let sun = turning_icon(SUN_ICON, icon_color, 1.0 - light, 90.0);

    let showing_light = theme.light;

    Rectangle::new()
        .width(MODE_WIDTH)
        .height(MODE_HEIGHT)
        .radius(12.0)
        .fill(fill)
        .cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_click(move |_| Mode::toggle(showing_light))
        .child(
            Stack::new(children![moon, sun])
                .width(MODE_WIDTH)
                .height(MODE_HEIGHT),
        )
}

// fully there at 1, and at 0 small, faded and turned away by the given degrees
fn turning_icon(icon: &str, color: Color, shown: f32, turned: f32) -> Rectangle {
    let glyph = Text::new(icon)
        .size(19.0)
        .font(fonts::NERD)
        .tight()
        .color(color);

    Rectangle::new()
        .width(MODE_WIDTH)
        .height(MODE_HEIGHT)
        .align_child(Center, Center)
        .opacity(shown)
        .scale(0.35 + 0.65 * shown)
        .rotate(turned * (1.0 - shown))
        .child(glyph)
}

// pressing anywhere on it sets the brightness under the pointer, and dragging follows
fn slider(theme: &Theme, width: f32) -> Stack {
    let percent = Brightness::read().percent();

    let value = f32::from(percent) / 100.0;

    let track_width = width - TRACK_LEFT - TRACK_GAP - PERCENT_WIDTH - SIDE_PADDING;

    let filled = Rectangle::new()
        .width(track_width * value)
        .height(TRACK_HEIGHT)
        .radius(TRACK_HEIGHT / 2.0)
        .fill(theme.accent);

    let track = Rectangle::new()
        .width(track_width)
        .height(TRACK_HEIGHT)
        .radius(TRACK_HEIGHT / 2.0)
        .fill(theme.border)
        .child(filled);

    // kept inside the track at both ends
    let thumb_x = (value * track_width - THUMB_WIDTH / 2.0).clamp(0.0, track_width - THUMB_WIDTH);

    let thumb = Rectangle::new()
        .width(THUMB_WIDTH)
        .height(THUMB_HEIGHT)
        .radius(THUMB_WIDTH / 2.0)
        .fill(theme.accent)
        .translate(thumb_x, (TRACK_HEIGHT - THUMB_HEIGHT) / 2.0);

    let track = Rectangle::new()
        .width(track_width)
        .height(HEIGHT)
        .translate(TRACK_LEFT, 0.0)
        .align_child(Start, Center)
        .child(
            Stack::new(children![track, thumb])
                .width(track_width)
                .height(TRACK_HEIGHT),
        );

    let icon = Rectangle::new()
        .width(TRACK_LEFT)
        .height(HEIGHT)
        .translate(SIDE_PADDING, 0.0)
        .align_child(Start, Center)
        .child(
            Text::new(BRIGHTNESS_ICON)
                .size(18.0)
                .font(fonts::NERD)
                .tight()
                .color(theme.text),
        );

    let label = Text::new(format!("{percent}%"))
        .size(12.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text);

    let label = Rectangle::new()
        .width(PERCENT_WIDTH)
        .height(HEIGHT)
        .translate(width - SIDE_PADDING - PERCENT_WIDTH, 0.0)
        .align_child(End, Center)
        .child(label);

    let background = Rectangle::new()
        .width(width)
        .height(HEIGHT)
        .radius(SLIDER_RADIUS)
        .fill(theme.surface)
        .cursor(Pointer)
        .on_drag(move |point| drag(point, track_width));

    Stack::new(children![background, icon, track, label])
        .width(width)
        .height(HEIGHT)
}

// the pointer is measured from the slider's corner, the track starts further in
fn drag(point: Point, track_width: f32) {
    let along = (point.x - TRACK_LEFT) / track_width;

    let percent = (along.clamp(0.01, 1.0) * 100.0).round() as u8;

    // every move reports, but the backlight only needs a call when the number changes
    if percent == Brightness::read().percent() {
        return;
    }

    Brightness::set(percent);
}
