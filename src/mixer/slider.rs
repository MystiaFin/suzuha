use amane::{Color, Full, Point, Pointer, Rectangle, Scroll, Stack, Widget};

use crate::motion;

use super::{Level, hover};

const RADIUS: f32 = 18.0;

// the fill never gets shorter than this, so the icon always sits on it
pub const LEAD: f32 = 64.0;

// a short bar riding just inside the end of the fill, this share of the slider's height
const HANDLE_WIDTH: f32 = 4.0;
pub const HANDLE_INSET: f32 = 12.0;
const HANDLE_SHARE: f32 = 0.42;

// a muted handle fades back
const MUTED_HANDLE_OPACITY: f32 = 0.6;

// how far one wheel notch moves a slider
const STEP: i32 = 5;

pub struct Slider {
    pub level: Level,

    pub width: f32,
    pub height: f32,

    // the empty track, the part up to the level, and the bar riding its end
    pub track: Color,
    pub fill: Color,

    // none for an app, whose level and mute button already mark the end of its row
    pub handle: Option<Color>,
}

/*
 * a thick track whose fill glides to the level, with an optional short bar
 * riding the end of the fill; pressing sets the level under the pointer,
 * dragging follows and the wheel moves it a step at a time
 */
pub fn view(slider: Slider, inside: Vec<Box<dyn Widget>>) -> Stack {
    let layers = layers(&slider, inside);

    Stack::new(layers).width(slider.width).height(slider.height)
}

// the slider's layers with what goes inside it, so a button can still go on top
pub fn layers(slider: &Slider, inside: Vec<Box<dyn Widget>>) -> Vec<Box<dyn Widget>> {
    let level = &slider.level;

    let key = level.channel.key();

    let target = f32::from(level.volume.min(100)) / 100.0;

    let share = motion::follow(&format!("{key}:level"), target, motion::FAST_SPATIAL);

    let fill_width = LEAD + share * (slider.width - LEAD);

    let background = Rectangle::new()
        .width(slider.width)
        .height(slider.height)
        .radius(RADIUS)
        .fill(slider.track);

    let filled = Rectangle::new()
        .width(fill_width)
        .height(slider.height)
        .radius(RADIUS)
        .fill(slider.fill);

    let travel = slider.width - LEAD;

    let dragged = level.clone();
    let scrolled = level.clone();

    let hover_name = format!("{key}:row");

    let area = Rectangle::new()
        .width(slider.width)
        .height(slider.height)
        .cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_drag(move |point| drag(point, travel, &dragged))
        .on_scroll(move |scroll| scroll_level(scroll, &scrolled));

    let mut layers: Vec<Box<dyn Widget>> = vec![Box::new(background), Box::new(filled)];

    if let Some(color) = slider.handle {
        layers.push(Box::new(handle(slider, color, fill_width)));
    }

    layers.extend(inside);
    layers.push(Box::new(area));

    layers
}

fn handle(slider: &Slider, color: Color, fill_width: f32) -> Rectangle {
    let height = slider.height * HANDLE_SHARE;

    let opacity = if slider.level.muted {
        MUTED_HANDLE_OPACITY
    } else {
        1.0
    };

    Rectangle::new()
        .width(HANDLE_WIDTH)
        .height(height)
        .radius(Full)
        .fill(color)
        .opacity(opacity)
        .translate(fill_width - HANDLE_INSET, (slider.height - height) / 2.0)
}

// the pointer's place past the lead, where the fill starts growing
fn drag(point: Point, travel: f32, level: &Level) {
    // a window too narrow for the slider leaves nothing to drag along
    if travel <= 0.0 {
        return;
    }

    let along = (point.x - LEAD) / travel;

    let wanted = (along.clamp(0.0, 1.0) * 100.0).round() as u8;

    // every move reports, but the sound server only needs a call when the number changes
    if wanted == level.volume {
        return;
    }

    level.channel.set(wanted);
}

// wheel up is a negative y, which turns the level up
fn scroll_level(scroll: Scroll, level: &Level) {
    let change = if scroll.y < 0.0 { STEP } else { -STEP };

    let changed = (i32::from(level.volume) + change).clamp(0, 100);

    level.channel.set(changed as u8);
}
