use amane::{Center, Column, Padding, Rectangle, Row, Start, Text, Weight, Widget, children};

use crate::fonts;
use crate::theme::Theme;

// most rows are this tall, ones with a slider or a text field a little taller
pub const HEIGHT: f32 = 96.0;
pub const TALL_HEIGHT: f32 = 104.0;

pub const SIDE: f32 = 24.0;

// the controls on the right are this tall
pub const CONTROL_HEIGHT: f32 = 40.0;

// its title and a line about it on the left, its control on the right
pub fn view(
    theme: &Theme,
    width: f32,
    height: f32,
    title: &str,
    detail: &str,
    control: impl Widget + 'static,
    control_width: f32,
) -> Rectangle {
    let labels_width = width - SIDE * 3.0 - control_width;

    let row = Row::new(children![
        labels(theme, title, detail, labels_width),
        control
    ])
    .gap(SIDE)
    .align(Center);

    Rectangle::new()
        .width(width)
        .height(height)
        .padding(Padding {
            top: 0.0,
            right: SIDE,
            bottom: 0.0,
            left: SIDE,
        })
        .align_child(Start, Center)
        .child(row)
}

fn labels(theme: &Theme, title: &str, detail: &str, width: f32) -> Rectangle {
    let title = Text::new(title)
        .size(16.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text)
        .elide();

    let detail = Text::new(detail)
        .size(12.0)
        .font(fonts::BODY)
        .color(theme.muted_text)
        .elide();

    Rectangle::new()
        .width(width)
        .height(44.0)
        .align_child(Start, Center)
        .child(Column::new(children![title, detail]).gap(4.0))
}
