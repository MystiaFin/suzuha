use amane::{Center, Column, Padding, Rectangle, Row, Start, Text, Weight, Widget, children};

use super::super::page::Page;
use super::super::slider::{self, Range};
use super::super::{choice, switch};
use crate::fonts;

const SWATCH_WIDTH: f32 = 28.0;
const SWATCH_HEIGHT: f32 = 44.0;
const SWATCH_GAP: f32 = 7.0;

const PREVIEW_HEIGHT: f32 = 116.0;

const ACCENTS: [(&str, &str); 7] = [
    ("Blue", "#89b4fa"),
    ("Rose", "#f38ba8"),
    ("Peach", "#fab387"),
    ("Green", "#a6e3a1"),
    ("Teal", "#94e2d5"),
    ("Purple", "#cba6f7"),
    ("Pink", "#f5c2e7"),
];

pub fn build(page: &mut Page) {
    let preview = preview(page);

    page.card(PREVIEW_HEIGHT, preview);

    choice::add(
        page,
        "scheme",
        "Color scheme",
        "Choose the palette family used by the shell",
        &[
            ("Dynamic", "dynamic"),
            ("Gruvbox", "gruvbox"),
            ("Catppuccin", "catppuccin"),
        ],
    );

    choice::add(
        page,
        "color_mode",
        "Color mode",
        "Let the wallpaper decide, or force light/dark dynamic colors",
        &[("Auto", "auto"), ("Light", "light"), ("Dark", "dark")],
    );

    switch::add(
        page,
        "manual_accent",
        "Manual accent",
        "Override the wallpaper-derived accent in the dynamic palette",
    );

    choice::add(
        page,
        "accent",
        "Accent preset",
        "Pick the manual accent used when the override is enabled",
        &ACCENTS,
    );

    page.end_group();

    slider::add(
        page,
        "saturation",
        "Saturation",
        "Tune how colorful the generated dynamic palette feels",
        Range {
            min: 0.55,
            max: 1.45,
            step: 0.05,
            label: slider::times,
        },
    );

    slider::add(
        page,
        "contrast",
        "Contrast",
        "Increase or soften tonal separation in the dynamic palette",
        Range {
            min: 0.75,
            max: 1.35,
            step: 0.05,
            label: slider::times,
        },
    );

    switch::add(
        page,
        "follow_wallpaper",
        "Follow wallpaper colors",
        "Regenerate the dynamic palette when the wallpaper changes",
    );

    page.end_group();
}

// the colors the shell draws with right now
fn preview(page: &Page) -> Rectangle {
    let theme = page.theme;

    let colors = [
        theme.background,
        theme.surface,
        theme.selected_surface,
        theme.accent,
        theme.success,
        theme.danger,
    ];

    let mut swatches: Vec<Box<dyn Widget>> = Vec::new();

    for color in colors {
        let swatch = Rectangle::new()
            .width(SWATCH_WIDTH)
            .height(SWATCH_HEIGHT)
            .radius(8.0)
            .fill(color)
            .border(1.0, theme.border);

        swatches.push(Box::new(swatch));
    }

    let swatches_width = colors.len() as f32 * (SWATCH_WIDTH + SWATCH_GAP) - SWATCH_GAP;

    let labels = Column::new(children![
        Text::new("Current palette")
            .size(16.0)
            .font(fonts::BODY)
            .weight(Weight::SemiBold)
            .color(theme.text),
        Text::new("Live preview of the colors currently used by the shell")
            .size(12.0)
            .font(fonts::BODY)
            .color(theme.muted_text)
            .elide(),
    ])
    .gap(3.0);

    let labels = Rectangle::new()
        .width(page.width - 40.0 - 12.0 - swatches_width)
        .height(SWATCH_HEIGHT)
        .align_child(Start, Center)
        .child(labels);

    Rectangle::new()
        .width(page.width)
        .height(PREVIEW_HEIGHT)
        .radius(20.0)
        .fill(theme.surface)
        .padding(Padding {
            top: 0.0,
            right: 20.0,
            bottom: 0.0,
            left: 20.0,
        })
        .align_child(Start, Center)
        .child(
            Row::new(children![labels, Row::new(swatches).gap(SWATCH_GAP)])
                .gap(12.0)
                .align(Center),
        )
}
