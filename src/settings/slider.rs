use amane::{Center, End, Full, Point, Pointer, Rectangle, Row, Service, Stack, Text, children};

use super::page::Page;
use super::{Settings, row};
use crate::fonts;

const TRACK_WIDTH: f32 = 170.0;
const TRACK_HEIGHT: f32 = 4.0;
const THUMB: f32 = 16.0;

const VALUE_WIDTH: f32 = 64.0;
const VALUE_GAP: f32 = 8.0;

// the range a slider covers, how far one step moves it, and how its value reads
pub struct Range {
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub label: fn(f32) -> String,
}

// a setting that is a number
pub fn add(page: &mut Page, key: &'static str, title: &str, detail: &str, range: Range) {
    let theme = page.theme;

    let value = Settings::read().staged_number(key);

    let label = Text::new((range.label)(value))
        .size(12.0)
        .font(fonts::BODY)
        .color(theme.secondary_text);

    let label = Rectangle::new()
        .width(VALUE_WIDTH)
        .height(row::CONTROL_HEIGHT)
        .align_child(End, Center)
        .child(label);

    let control = Row::new(children![track(page, key, value, range), label])
        .gap(VALUE_GAP)
        .align(Center);

    let width = TRACK_WIDTH + VALUE_GAP + VALUE_WIDTH;

    let row = row::view(
        theme,
        page.width,
        row::TALL_HEIGHT,
        title,
        detail,
        control,
        width,
    );

    page.row(row::TALL_HEIGHT, row);
}

// pressing anywhere on it sets the value under the pointer, and dragging follows
fn track(page: &Page, key: &'static str, value: f32, range: Range) -> Stack {
    let theme = page.theme;

    let height = row::CONTROL_HEIGHT;

    let share = ((value - range.min) / (range.max - range.min)).clamp(0.0, 1.0);

    let filled = Rectangle::new()
        .width(TRACK_WIDTH * share)
        .height(TRACK_HEIGHT)
        .radius(Full)
        .fill(theme.accent);

    let line = Rectangle::new()
        .width(TRACK_WIDTH)
        .height(TRACK_HEIGHT)
        .radius(Full)
        .fill(theme.border)
        .translate(0.0, (height - TRACK_HEIGHT) / 2.0)
        .child(filled);

    let thumb_x = (TRACK_WIDTH - THUMB) * share;

    let thumb = Rectangle::new()
        .width(THUMB)
        .height(THUMB)
        .radius(Full)
        .fill(theme.accent)
        .translate(thumb_x, (height - THUMB) / 2.0);

    let area = Rectangle::new()
        .width(TRACK_WIDTH)
        .height(height)
        .cursor(Pointer)
        .on_drag(move |point| drag(point, key, &range));

    Stack::new(children![line, thumb, area])
        .width(TRACK_WIDTH)
        .height(height)
}

fn drag(point: Point, key: &'static str, range: &Range) {
    let share = (point.x / TRACK_WIDTH).clamp(0.0, 1.0);

    let raw = range.min + share * (range.max - range.min);

    let steps = ((raw - range.min) / range.step).round();

    let value = range.min + steps * range.step;

    // every move reports, but the draft only needs a change when the value does
    if (value - Settings::read().staged_number(key)).abs() < range.step / 2.0 {
        return;
    }

    Settings::stage(key, format!("{value:.3}"));
}

// labels for the ranges above, like "1.25×", "40 px" or "30 min"
pub fn times(value: f32) -> String {
    format!("{value:.2}×")
}

pub fn plain(value: f32) -> String {
    format!("{value:.2}")
}

pub fn count(value: f32) -> String {
    format!("{value:.0}")
}

pub fn pixels(value: f32) -> String {
    format!("{value:.0} px")
}

pub fn milliseconds(value: f32) -> String {
    format!("{value:.0} ms")
}

pub fn minutes(value: f32) -> String {
    format!("{value:.0} min")
}
