use amane::{Center, Column, Padding, Point, Pointer, Rectangle, Stack, Text, Weight, children};

use crate::fonts;
use crate::theme::Theme;

const WIDTH: f32 = 48.0;
const MARGIN: f32 = 4.0;
const GAP: f32 = 8.0;

const ICON_SIZE: f32 = 30.0;
const LABEL_HEIGHT: f32 = 18.0;

// the track and its round handle are equally wide
const TRACK_WIDTH: f32 = 22.0;
const HANDLE: f32 = 22.0;

// one level with its setter, like the speaker volume
pub struct Fader {
    pub icon: &'static str,
    pub level: u8,
    pub muted: bool,
    pub set: fn(u8),
    pub toggle_mute: fn(),
}

// the icon mutes, the track is pressed or dragged to a level, the percentage sits below
pub fn view(fader: Fader, theme: &Theme, height: f32) -> Rectangle {
    let track_height = height - MARGIN * 2.0 - ICON_SIZE - LABEL_HEIGHT - GAP * 2.0;

    let icon = Rectangle::new()
        .width(ICON_SIZE)
        .height(ICON_SIZE)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_click(move |_| (fader.toggle_mute)())
        .child(
            Text::new(fader.icon)
                .size(19.0)
                .font(fonts::NERD)
                .tight()
                .color(theme.text),
        );

    let label = Rectangle::new()
        .width(WIDTH)
        .height(LABEL_HEIGHT)
        .align_child(Center, Center)
        .child(
            Text::new(format!("{}%", fader.level))
                .size(14.0)
                .font(fonts::BODY)
                .weight(Weight::SemiBold)
                .color(theme.text),
        );

    let column = Column::new(children![icon, track(&fader, theme, track_height), label])
        .gap(GAP)
        .align(Center);

    Rectangle::new()
        .width(WIDTH)
        .height(height)
        .padding(Padding {
            top: MARGIN,
            right: 0.0,
            bottom: MARGIN,
            left: 0.0,
        })
        .child(column)
}

/*
 * the fill starts a handle tall, so the handle always sits on it, and
 * grows with the level; the handle rides its top
 */
fn track(fader: &Fader, theme: &Theme, height: f32) -> Stack {
    let value = f32::from(fader.level.min(100)) / 100.0;

    let travel = height - HANDLE;

    let filled_height = HANDLE + value * travel;

    let fill_color = if fader.muted {
        theme.muted_text
    } else {
        theme.accent
    };

    let track_x = (WIDTH - TRACK_WIDTH) / 2.0;

    let background = Rectangle::new()
        .width(TRACK_WIDTH)
        .height(height)
        .radius(TRACK_WIDTH / 2.0)
        .fill(theme.border)
        .translate(track_x, 0.0);

    let filled = Rectangle::new()
        .width(TRACK_WIDTH)
        .height(filled_height)
        .radius(TRACK_WIDTH / 2.0)
        .fill(fill_color)
        .translate(track_x, height - filled_height);

    let handle = Rectangle::new()
        .width(HANDLE)
        .height(HANDLE)
        .radius(HANDLE / 2.0)
        .fill(theme.text)
        .translate(track_x, (1.0 - value) * travel);

    let level = fader.level;
    let set = fader.set;

    let area = Rectangle::new()
        .width(WIDTH)
        .height(height)
        .cursor(Pointer)
        .on_drag(move |point| drag(point, travel, level, set));

    Stack::new(children![background, filled, handle, area])
        .width(WIDTH)
        .height(height)
}

// the pointer's height on the track, measured from the handle's middle
fn drag(point: Point, travel: f32, level: u8, set: fn(u8)) {
    let along = 1.0 - (point.y - HANDLE / 2.0) / travel;

    let wanted = (along.clamp(0.0, 1.0) * 100.0).round() as u8;

    // every move reports, but the sound server only needs a call when the number changes
    if wanted == level {
        return;
    }

    set(wanted);
}
