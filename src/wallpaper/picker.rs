mod card;
mod state;

use amane::{
    Center, Color, Column, Full, Key, Keyboard, Layer, LayerWindow, Monitor, Parent, Rectangle,
    Service, Stack, Text, Vertical, Widget, Zone, children,
};

use card::Place;

use super::Wallpaper;
use crate::fonts;
use crate::settings::Settings;
use crate::theme;

pub use state::{Picker, list_folder};

// the band's height, leaving at least this much of the screen above and below
const HEIGHT: f32 = 380.0;
const SCREEN_GAP: f32 = 72.0;

// the cards' largest size, and how much of the band's width one may take
const CARD_WIDTH: f32 = 360.0;
const CARD_HEIGHT: f32 = 240.0;
const CARD_SHARE: f32 = 0.34;

const MARGIN: f32 = 24.0;

// how many cards fit along the carousel, and how far past each side it runs
const VISIBLE_CARDS: f32 = 7.0;
const OVERHANG: f32 = 0.12;

const TINT: f32 = 0.48;

/*
 * the wallpaper behind the band, blurred and tinted, with the wallpapers as
 * cards sliding along it; above everything, and only open while shown
 */
pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = theme::current();

    let picker = Picker::read();

    let screen_width = monitor.width as f32;
    let screen_height = monitor.height as f32;

    let height = f32::min(HEIGHT, screen_height - SCREEN_GAP);

    let opacity = picker.opacity.value();

    let visible = picker.shown || opacity > 0.0;

    let keyboard = if picker.shown {
        Keyboard::Exclusive
    } else {
        Keyboard::None
    };

    // as tall as the screen and moved up, so it lines up with the wallpaper around the band
    let top = (screen_height - height) / 2.0;

    let opaque = Settings::read().flag("reduce_transparency");

    let backdrop = Rectangle::new()
        .width(screen_width)
        .height(screen_height)
        .fill(super::blurred(screen_width, screen_height))
        .translate(0.0, -top);

    // an alpha in the color, since fading the rectangle would give it a canvas of its own
    let tint = if opaque { 1.0 } else { TINT };

    let tint_alpha = (tint * 255.0).round() as u8;

    let surface = theme.surface;

    let tint = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(Color::rgba(
            surface.red(),
            surface.green(),
            surface.blue(),
            tint_alpha,
        ));

    let content: Box<dyn Widget> = if picker.files.is_empty() {
        Box::new(empty(&theme))
    } else {
        Box::new(carousel(&picker, screen_width, height))
    };

    let band = Stack::new(vec![Box::new(backdrop), Box::new(tint), content])
        .width(Parent)
        .height(Parent);

    LayerWindow::new()
        .width(Full)
        .height(height)
        .anchor_vertical(Vertical::Middle)
        .layer(Layer::Overlay)
        .space(Zone::Ignore)
        .keyboard(keyboard)
        .namespace("wallpaper-picker")
        .visible(visible)
        .on_key(key_pressed)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .opacity(opacity)
                .child(band),
        )
}

/*
 * like a carousel on a straight line: the line runs a little past both
 * sides, cards are spread evenly along it, and each gets smaller and
 * darker the further it is from the middle; the list wraps around
 */
fn carousel(picker: &Picker, width: f32, height: f32) -> Stack {
    let count = picker.files.len() as f32;

    let inside = width - MARGIN * 2.0;

    let line = inside * (1.0 + OVERHANG * 2.0);
    let spacing = line / VISIBLE_CARDS;

    let card_width = f32::min(CARD_WIDTH, width * CARD_SHARE);
    let card_height = f32::min(CARD_HEIGHT, height - SCREEN_GAP);

    let position = picker.position.value();

    let mut places = Vec::new();

    for (index, path) in picker.files.iter().enumerate() {
        // how far from the middle, the short way round
        let distance = (index as f32 - position + count / 2.0).rem_euclid(count) - count / 2.0;

        let prominence = 1.0 - distance.abs() * spacing / (line / 2.0);

        if prominence < 0.0 {
            continue;
        }

        let place = Place {
            x: width / 2.0 + distance * spacing,
            y: height / 2.0,
            width: card_width,
            height: card_height,
            prominence,
            offset: distance.round() as i64,
        };

        places.push((path, place));
    }

    // the middle card is drawn last, so it lies on top
    places.sort_by(|first, second| first.1.prominence.total_cmp(&second.1.prominence));

    let mut cards: Vec<Box<dyn Widget>> = Vec::new();

    for (path, place) in places {
        cards.push(Box::new(card::view(path, place)));
    }

    Stack::new(cards).width(Parent).height(Parent)
}

fn empty(theme: &theme::Theme) -> Rectangle {
    let title = Text::new("No wallpapers found")
        .size(16.0)
        .font(fonts::BODY)
        .color(theme.text);

    let folder = Text::new(state::folder())
        .size(11.0)
        .font(fonts::BODY)
        .color(theme.muted_text);

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .align_child(Center, Center)
        .child(Column::new(children![title, folder]).gap(8.0))
}

pub fn ipc(arguments: &[String]) -> String {
    let command = arguments.first().map(String::as_str).unwrap_or("toggle");

    let current = Wallpaper::read().path.clone();

    let mut picker = Picker::write();

    match command {
        "show" => picker.show(&current),
        "hide" => picker.hide(),
        _ if picker.shown => picker.hide(),
        _ => picker.show(&current),
    }

    String::from("ok")
}

fn key_pressed(key: Key) {
    let mut picker = Picker::write();

    if !picker.shown {
        return;
    }

    match key {
        Key::Left | Key::Character('h') => picker.slide(-1),
        Key::Right | Key::Character('l') => picker.slide(1),
        Key::Escape => picker.hide(),

        Key::Enter => {
            if let Some(path) = picker.selected() {
                super::choose(path);
            }

            picker.hide();
        }

        _ => {}
    }
}
