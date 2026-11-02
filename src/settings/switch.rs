use amane::{Full, Pointer, Rectangle, Service};

use super::page::Page;
use super::{Settings, row};
use crate::motion::{self, FAST_SPATIAL};

const WIDTH: f32 = 48.0;
const HEIGHT: f32 = 28.0;

// the knob grows when switched on
const KNOB: f32 = 16.0;
const CHECKED_KNOB: f32 = 24.0;

// how far the knob sits from the left end when off, and from the right end when on
const OFF_INSET: f32 = 6.0;
const ON_INSET: f32 = 2.0;

// a setting that is on or off
pub fn add(page: &mut Page, key: &'static str, title: &str, detail: &str) {
    add_with(page, key, title, detail, move || {
        Settings::stage_toggle(key)
    });
}

// the same with its own answer to a click, like asking first
pub fn add_with(
    page: &mut Page,
    key: &'static str,
    title: &str,
    detail: &str,
    on_toggle: impl Fn() + 'static,
) {
    let checked = Settings::read().staged_flag(key);

    let control = view(page, key, checked, on_toggle);

    let row = row::view(
        page.theme,
        page.width,
        row::HEIGHT,
        title,
        detail,
        control,
        WIDTH,
    );

    page.row(row::HEIGHT, row);
}

// a capsule with a knob that slides right when on
fn view(page: &Page, key: &str, checked: bool, on_toggle: impl Fn() + 'static) -> Rectangle {
    let theme = page.theme;

    let (target_size, target_x) = if checked {
        (CHECKED_KNOB, WIDTH - CHECKED_KNOB - ON_INSET)
    } else {
        (KNOB, OFF_INSET)
    };

    let size = motion::follow(
        &format!("settings-switch:{key}:size"),
        target_size,
        FAST_SPATIAL,
    );
    let x = motion::follow(&format!("settings-switch:{key}:x"), target_x, FAST_SPATIAL);

    let (fill, knob_color) = if checked {
        (theme.accent, theme.on_accent)
    } else {
        (theme.selected_surface, theme.secondary_text)
    };

    let knob = Rectangle::new()
        .width(size)
        .height(size)
        .radius(Full)
        .fill(knob_color)
        .translate(x, (HEIGHT - size) / 2.0);

    let mut track = Rectangle::new()
        .width(WIDTH)
        .height(HEIGHT)
        .radius(Full)
        .fill(fill)
        .cursor(Pointer)
        .on_click(move |_| on_toggle())
        .child(knob);

    if !checked {
        track = track.border(2.0, theme.border);
    }

    track
}
