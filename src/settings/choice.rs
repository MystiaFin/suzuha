use amane::{
    Center, Column, Padding, Pointer, Rectangle, Row, Service, SpaceBetween, Start, Text, Weight,
    Widget, children,
};

use super::page::Page;
use super::{Settings, Shown, high_surface, hover, hovered, row};
use crate::fonts;
use crate::motion;
use crate::theme;

const WIDTH: f32 = 164.0;
const RADIUS: f32 = 12.0;

const MENU_GAP: f32 = 8.0;
const MENU_PADDING: f32 = 4.0;

const CHEVRON_DOWN: &str = "󰅀";
const CHEVRON_UP: &str = "󰅃";
const CHECK: &str = "󰄬";

// a setting that is one of a few words, picked from a menu that drops over the rows below
pub fn add(
    page: &mut Page,
    key: &'static str,
    title: &str,
    detail: &str,
    options: &'static [(&'static str, &'static str)],
) {
    let theme = page.theme;

    let staged = String::from(Settings::read().staged(key));

    let open = Shown::read().choice == Some(key);

    // the saved word's label, or the first option's for a word that isn't one
    let mut label = options[0].0;

    for (option_label, value) in options {
        if *value == staged {
            label = option_label;
        }
    }

    let selector = selector(page, key, label, open);

    let top = page.y;

    let row = row::view(
        theme,
        page.width,
        row::HEIGHT,
        title,
        detail,
        selector,
        WIDTH,
    );

    page.row(row::HEIGHT, row);

    // the menu fades out after closing, so it stays until it is gone
    let amount = motion::fade(
        &format!("settings-menu:{key}"),
        if open { 1.0 } else { 0.0 },
    );

    if amount < 0.01 {
        return;
    }

    let x = page.width - row::SIDE - WIDTH;
    let y = top + (row::HEIGHT - row::CONTROL_HEIGHT) / 2.0 + row::CONTROL_HEIGHT + MENU_GAP;

    let menu = menu(page, key, options, &staged, open);

    let height = options.len() as f32 * row::CONTROL_HEIGHT + MENU_PADDING * 2.0;

    page.menu(
        menu.opacity(amount)
            .scale(0.97 + 0.03 * amount)
            .translate(x, y),
        y + height,
    );
}

fn selector(page: &Page, key: &'static str, label: &str, open: bool) -> Rectangle {
    let theme = page.theme;

    let chevron = if open { CHEVRON_UP } else { CHEVRON_DOWN };

    let content = Row::new(children![
        Text::new(label)
            .size(13.0)
            .font(fonts::BODY)
            .weight(Weight::SemiBold)
            .color(theme.accent),
        Text::new(chevron)
            .size(14.0)
            .font(fonts::NERD)
            .tight()
            .color(theme.secondary_text),
    ])
    .width(WIDTH - 16.0 - 12.0)
    .justify(SpaceBetween)
    .align(Center);

    let hover_name = format!("choice:{key}");

    let fill = if hovered(&hover_name) {
        theme::mix(high_surface(theme), theme.text, 0.08)
    } else {
        high_surface(theme)
    };

    let mut selector = Rectangle::new()
        .width(WIDTH)
        .height(row::CONTROL_HEIGHT)
        .radius(RADIUS)
        .fill(fill)
        .cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_click(move |_| toggle(key))
        .padding(Padding {
            top: 0.0,
            right: 12.0,
            bottom: 0.0,
            left: 16.0,
        })
        .align_child(Start, Center)
        .child(content);

    if open {
        selector = selector.border(2.0, theme.accent);
    }

    selector
}

fn menu(
    page: &Page,
    key: &'static str,
    options: &'static [(&'static str, &'static str)],
    staged: &str,
    open: bool,
) -> Rectangle {
    let theme = page.theme;

    let option_width = WIDTH - MENU_PADDING * 2.0;

    let mut items: Vec<Box<dyn Widget>> = Vec::new();

    for (label, value) in options {
        let selected = *value == staged;

        let hover_name = format!("choice:{key}:{value}");

        let fill = if selected {
            theme.selected_surface
        } else if hovered(&hover_name) {
            theme.hover_surface
        } else {
            amane::Color::TRANSPARENT
        };

        let (color, weight) = if selected {
            (theme.accent, Weight::SemiBold)
        } else {
            (theme.text, Weight::Regular)
        };

        let mut content: Vec<Box<dyn Widget>> = vec![Box::new(
            Text::new(*label)
                .size(13.0)
                .font(fonts::BODY)
                .weight(weight)
                .color(color),
        )];

        if selected {
            content.push(Box::new(
                Text::new(CHECK)
                    .size(13.0)
                    .font(fonts::NERD)
                    .tight()
                    .color(theme.accent),
            ));
        }

        let mut item = Rectangle::new()
            .width(option_width)
            .height(row::CONTROL_HEIGHT)
            .radius(8.0)
            .fill(fill)
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 0.0,
                left: 11.0,
            })
            .align_child(Start, Center)
            .child(
                Row::new(content)
                    .width(option_width - 21.0)
                    .justify(SpaceBetween)
                    .align(Center),
            );

        // a closing menu is only drawn, it takes no clicks
        if open {
            let value = *value;

            item = item
                .cursor(Pointer)
                .on_hover(move |inside| hover(hover_name.clone(), inside))
                .on_click(move |_| pick(key, value));
        }

        items.push(Box::new(item));
    }

    let height = options.len() as f32 * row::CONTROL_HEIGHT + MENU_PADDING * 2.0;

    Rectangle::new()
        .width(WIDTH)
        .height(height)
        .radius(RADIUS)
        .fill(high_surface(theme))
        .padding(Padding {
            top: MENU_PADDING,
            right: MENU_PADDING,
            bottom: MENU_PADDING,
            left: MENU_PADDING,
        })
        .child(Column::new(items))
}

// one menu is open at a time, opening another closes it
fn toggle(key: &'static str) {
    let mut shown = Shown::write();

    shown.choice = if shown.choice == Some(key) {
        None
    } else {
        Some(key)
    };
}

fn pick(key: &'static str, value: &'static str) {
    Settings::stage(key, value);

    Shown::write().choice = None;
}
