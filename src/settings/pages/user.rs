use std::env;

use amane::{
    Center, Column, Image, Padding, Rectangle, Row, Service, Start, Text, TextInput, Weight,
    Widget, children,
};

use super::super::button::{self, Style};
use super::super::field;
use super::super::page::Page;
use crate::fonts;
use crate::profile::Profile;

const AVATAR: f32 = 72.0;
const AVATAR_ICON: &str = "󰀄";

const PROFILE_HEIGHT: f32 = 128.0;

// the text input's name, which keeps what was typed between redraws
pub const NAME_INPUT: &str = "profile-name";

// the picture and name the lock screen shows; they change at once, they aren't settings
pub fn build(page: &mut Page) {
    let profile = profile(page);

    page.row(PROFILE_HEIGHT, profile);

    page.end_group();

    let login = env::var("USER").unwrap_or_default();

    let input = TextInput::new(NAME_INPUT)
        .size(14.0)
        .color(page.theme.text)
        .placeholder(&login)
        .on_change(Profile::set_name);

    let name = field::view(
        page,
        "Display name",
        "Name shown on the lock screen; this does not change your system account",
        input,
    );

    page.row(super::super::row::TALL_HEIGHT, name);

    page.end_group();
}

fn profile(page: &Page) -> Rectangle {
    let theme = page.theme;

    let profile = Profile::read();

    let login = env::var("USER").unwrap_or_default();

    let has_picture = profile.picture().is_some();

    let labels = Column::new(children![
        Text::new(profile.name())
            .size(17.0)
            .font(fonts::BODY)
            .weight(Weight::SemiBold)
            .color(theme.text),
        Text::new(format!("@{login}"))
            .size(11.0)
            .font(fonts::BODY)
            .color(theme.muted_text),
        Text::new("Used by the lock screen")
            .size(11.0)
            .font(fonts::BODY)
            .color(theme.secondary_text),
    ])
    .gap(2.0);

    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    let mut buttons_width = button::width("Change photo");

    if has_picture {
        buttons.push(Box::new(button::view(
            theme,
            "Default",
            Style::Plain,
            true,
            Profile::clear_picture,
        )));

        buttons_width += button::width("Default") + 8.0;
    }

    buttons.push(Box::new(button::view(
        theme,
        "Change photo",
        Style::Plain,
        true,
        Profile::choose_picture,
    )));

    let labels_width = page.width - 40.0 - AVATAR - 32.0 - buttons_width;

    let labels = Rectangle::new()
        .width(labels_width)
        .height(AVATAR)
        .align_child(Start, Center)
        .child(labels);

    let row = Row::new(children![
        avatar(&profile, page),
        labels,
        Row::new(buttons).gap(8.0)
    ])
    .gap(16.0)
    .align(Center);

    Rectangle::new()
        .width(page.width)
        .height(PROFILE_HEIGHT)
        .padding(Padding {
            top: 0.0,
            right: 20.0,
            bottom: 0.0,
            left: 20.0,
        })
        .align_child(Start, Center)
        .child(row)
}

fn avatar(profile: &Profile, page: &Page) -> Rectangle {
    let theme = page.theme;

    let circle = Rectangle::new()
        .width(AVATAR)
        .height(AVATAR)
        .radius(AVATAR / 2.0)
        .fill(theme.selected_surface)
        .clip();

    if let Some(picture) = profile.picture() {
        if Image::loaded(picture) {
            let pixels = AVATAR as u32 * 2;

            return circle.fill(Image::cover(picture).thumbnail(pixels, pixels));
        }
    }

    circle.align_child(Center, Center).child(
        Text::new(AVATAR_ICON)
            .size(32.0)
            .font(fonts::NERD)
            .tight()
            .color(theme.secondary_text),
    )
}
