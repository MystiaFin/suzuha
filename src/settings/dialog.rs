use amane::{
    Center, Color, Column, End, Padding, Parent, Rectangle, Row, Service, Start, Text, Weight,
    children,
};

use super::button::{self, Style};
use super::{Confirm, Settings, Shown, field};
use crate::fonts;
use crate::theme::Theme;

const WARNING_ICON: &str = "󰀪";

// the window dimmed behind a card that asks before something risky is staged
pub fn view(theme: &Theme, confirm: &Confirm) -> Rectangle {
    let card = match confirm {
        Confirm::Integration {
            key,
            title,
            warning,
        } => integration(theme, key, title, warning),
        Confirm::Reset => reset(theme),
    };

    // takes every click, so nothing behind it can be pressed
    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(Color::rgba(0, 0, 0, 0x8c))
        .on_click(|_| {})
        .align_child(Center, Center)
        .child(card)
}

fn integration(theme: &Theme, key: &'static str, title: &str, warning: &str) -> Rectangle {
    let icon = Rectangle::new()
        .width(38.0)
        .height(38.0)
        .radius(12.0)
        .fill(theme.selected_surface)
        .align_child(Center, Center)
        .child(
            Text::new(WARNING_ICON)
                .size(20.0)
                .font(fonts::NERD)
                .tight()
                .color(theme.danger),
        );

    let heading = Row::new(children![
        icon,
        title_text(theme, &format!("Enable {title} integration?"))
    ])
    .gap(12.0)
    .align(Center);

    let continue_button = button::view(theme, "Continue", Style::Danger, true, move || {
        Settings::stage(key, true);

        close();
    });

    let content = Column::new(children![
        heading,
        body(warning, theme.secondary_text, 11.0, 500.0),
        body(
            "This only stages the change. The integration is enabled when you press Apply.",
            theme.muted_text,
            10.0,
            500.0,
        ),
        buttons(theme, continue_button),
    ])
    .gap(12.0);

    card(theme, 500.0, 250.0, content)
}

fn reset(theme: &Theme) -> Rectangle {
    let reset_button = button::view(theme, "Reset", Style::Danger, true, || {
        Settings::stage_defaults();

        field::fill();

        close();
    });

    let content = Column::new(children![
        title_text(theme, "Reset all settings?"),
        body(
            "This stages every setting at its default. Nothing changes until you press Apply.",
            theme.secondary_text,
            11.0,
            460.0,
        ),
        buttons(theme, reset_button),
    ])
    .gap(12.0);

    card(theme, 460.0, 190.0, content)
}

fn card(theme: &Theme, width: f32, height: f32, content: Column) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(height)
        .radius(20.0)
        .fill(theme.surface)
        .padding(Padding {
            top: 22.0,
            right: 22.0,
            bottom: 22.0,
            left: 22.0,
        })
        .align_child(Start, Center)
        .child(content)
}

fn title_text(theme: &Theme, text: &str) -> Text {
    Text::new(text)
        .size(17.0)
        .font(fonts::BODY)
        .weight(Weight::Bold)
        .color(theme.text)
}

// wrapped inside the card, with room for three lines
fn body(text: &str, color: amane::Color, size: f32, width: f32) -> Rectangle {
    Rectangle::new()
        .width(width - 44.0)
        .height(size * 4.5)
        .child(
            Text::new(text)
                .size(size)
                .font(fonts::BODY)
                .color(color)
                .wrap(),
        )
}

// cancel and the risky one, on the right
fn buttons(theme: &Theme, confirm: Rectangle) -> Rectangle {
    let cancel = button::view(theme, "Cancel", Style::Plain, true, close);

    Rectangle::new()
        .width(Parent)
        .height(40.0)
        .align_child(End, Center)
        .child(Row::new(children![cancel, confirm]).gap(8.0))
}

pub fn close() {
    Shown::write().confirm = None;
}
