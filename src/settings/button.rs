use amane::{Center, Pointer, Rectangle, Text, Weight};

use super::{high_surface, hover, hovered, row};
use crate::fonts;
use crate::theme::{self, Theme};

pub enum Style {
    Plain,
    Primary,
    Danger,
}

// about how wide one letter of a button's label is
const LETTER: f32 = 7.6;

// a rounded button sized to its label; a disabled one is faded and takes no clicks
pub fn view(
    theme: &Theme,
    label: &str,
    style: Style,
    enabled: bool,
    on_click: impl Fn() + 'static,
) -> Rectangle {
    let (fill, text) = match style {
        Style::Plain => (high_surface(theme), theme.text),
        Style::Primary => (theme.accent, theme.on_accent),
        Style::Danger => (theme.danger, theme.on_accent),
    };

    let hover_name = format!("button:{label}");

    // a faint layer of the label's color while the pointer is on it
    let fill = if enabled && hovered(&hover_name) {
        theme::mix(fill, text, 0.08)
    } else {
        fill
    };

    let mut button = Rectangle::new()
        .width(width(label))
        .height(row::CONTROL_HEIGHT)
        .radius(12.0)
        .fill(fill)
        .align_child(Center, Center)
        .child(
            Text::new(label)
                .size(13.0)
                .font(fonts::BODY)
                .weight(Weight::SemiBold)
                .color(text),
        );

    if !enabled {
        return button.opacity(0.42);
    }

    button = button
        .cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_click(move |_| on_click());

    button
}

pub fn width(label: &str) -> f32 {
    label.chars().count() as f32 * LETTER + 32.0
}
