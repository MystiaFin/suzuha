use amane::{
    Center, Color, Column, End, Padding, Rectangle, Row, SpaceBetween, Start, Text, Weight, Widget,
    children,
};

use crate::fonts;
use crate::theme::Theme;

pub const WIDTH: f32 = 190.0;
pub const HEIGHT: f32 = 125.0;

pub const RADIUS: f32 = 20.0;
const PADDING: f32 = 18.0;

const BAR_HEIGHT: f32 = 5.0;

// how much of the card's color covers the wallpaper behind it
const SURFACE_ALPHA: u8 = 200;

// the rounded, see-through base every desktop card sits on
pub fn background(theme: &Theme, width: f32, height: f32) -> Rectangle {
    let surface = theme.surface;

    Rectangle::new()
        .width(width)
        .height(height)
        .radius(RADIUS)
        .fill(Color::rgba(
            surface.red(),
            surface.green(),
            surface.blue(),
            SURFACE_ALPHA,
        ))
        .padding(Padding {
            top: PADDING,
            right: PADDING,
            bottom: PADDING,
            left: PADDING,
        })
}

// one reading, like "CPU TEMPERATURE" over a thermometer and "72°C", with a bar under it
pub struct Reading<'a> {
    pub label: &'a str,
    pub icon: &'a str,
    pub value: String,

    // 0 to 1 for the bar
    pub amount: f32,

    pub color: Color,
}

pub fn view(theme: &Theme, reading: Reading) -> Rectangle {
    let inner_width = WIDTH - PADDING * 2.0;

    let label = Text::new(reading.label)
        .size(13.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.secondary_text);

    let icon = Text::new(reading.icon)
        .size(28.0)
        .font(fonts::NERD)
        .tight()
        .color(theme.secondary_text);

    let value = Text::new(reading.value)
        .size(34.0)
        .font(fonts::BODY)
        .weight(Weight::Bold)
        .color(theme.text);

    let middle = Row::new(children![icon, value])
        .width(inner_width)
        .justify(SpaceBetween)
        .align(Center);

    let filled = Rectangle::new()
        .width(inner_width * reading.amount.clamp(0.0, 1.0))
        .height(BAR_HEIGHT)
        .radius(BAR_HEIGHT / 2.0)
        .fill(reading.color);

    let bar = Rectangle::new()
        .width(inner_width)
        .height(BAR_HEIGHT)
        .radius(BAR_HEIGHT / 2.0)
        .fill(theme.border)
        .child(filled);

    let rows: Vec<Box<dyn Widget>> = vec![Box::new(label), Box::new(middle), Box::new(bar)];

    background(theme, WIDTH, HEIGHT)
        .align_child(Start, End)
        .child(
            Column::new(rows)
                .width(inner_width)
                .height(HEIGHT - PADDING * 2.0)
                .justify(SpaceBetween),
        )
}
