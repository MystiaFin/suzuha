use amane::{
    Center, Column, Padding, Pointer, Rectangle, Row, ScrollArea, Service, Start, Text, Weight,
    Widget, children,
};

use super::header;
use super::{hover, hovered};
use crate::fonts;
use crate::overlay::Overlay;
use crate::recorder::{self, AUDIO, Audio, Recorder};
use crate::theme::{self, Theme};

const LIST: &str = "record-modes";

const GAP: f32 = 10.0;
const CARD_GAP: f32 = 7.0;

const CARD_HEIGHT: f32 = 50.0;
const CARD_PADDING: f32 = 9.0;
const ICON_SIZE: f32 = 32.0;

// the header's icon, like the one in header.rs
const HEADER_PADDING: f32 = 10.0;
const HEADER_ICON_SIZE: f32 = 36.0;

const BUTTON_HEIGHT: f32 = 48.0;

const RECORD_ICON: &str = "󰑊";
const STOP_ICON: &str = "󰓛";
const CHECK: &str = "󰄬";

// a header like the other pages, the sound modes as cards, and the record button at the bottom
pub fn view(overlay: &Overlay, theme: &Theme, width: f32, height: f32) -> Column {
    let chosen = Recorder::read().audio;

    let status = recorder::status();

    let title = title(theme, status.as_deref(), width);

    // the sound can't change mid-recording, so the cards stop taking clicks then
    let locked = status.is_some();

    let mut cards: Vec<Box<dyn Widget>> = Vec::new();

    for audio in AUDIO {
        cards.push(Box::new(card(overlay, theme, audio, audio == chosen, locked, width)));
    }

    let count = AUDIO.len() as f32;

    let total = count * CARD_HEIGHT + (count - 1.0) * CARD_GAP;

    let column = Column::new(cards).width(width).height(total).gap(CARD_GAP);

    // everything between the header and the button, scrolling when the screen is short
    let area = Rectangle::new()
        .width(width)
        .height((height - header::HEIGHT - BUTTON_HEIGHT - GAP * 2.0).max(0.0))
        .child(ScrollArea::new(LIST, column));

    Column::new(children![title, area, button(overlay, theme, status, width)]).gap(GAP)
}

// header.rs's look, without its switch since the button below starts and stops
fn title(theme: &Theme, status: Option<&str>, width: f32) -> Rectangle {
    let (icon_fill, text, color) = match status {
        Some(time) => (theme.danger, format!("Recording · {time}"), theme.danger),
        None => (theme.accent, String::from("Ready · saves to ~/Videos"), theme.muted_text),
    };

    let icon = Rectangle::new()
        .width(HEADER_ICON_SIZE)
        .height(HEADER_ICON_SIZE)
        .radius(12.0)
        .fill(icon_fill)
        .align_child(Center, Center)
        .child(Text::new(RECORD_ICON).size(18.0).font(fonts::NERD).tight().color(theme.on_accent));

    let text_width = width - HEADER_PADDING * 3.0 - HEADER_ICON_SIZE;

    let title = Text::new("Screen recorder")
        .size(13.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text);

    let status = Rectangle::new()
        .width(text_width)
        .height(16.0)
        .child(Text::new(text).size(10.0).font(fonts::BODY).color(color).elide());

    let text = Rectangle::new()
        .width(text_width)
        .height(HEADER_ICON_SIZE)
        .align_child(Start, Center)
        .child(Column::new(children![title, status]));

    Rectangle::new()
        .width(width)
        .height(header::HEIGHT)
        .radius(16.0)
        .fill(theme.surface)
        .padding(Padding {
            top: HEADER_PADDING,
            right: HEADER_PADDING,
            bottom: HEADER_PADDING,
            left: HEADER_PADDING,
        })
        .child(Row::new(children![icon, text]).gap(HEADER_PADDING).align(Center))
}

// one sound mode, tinted with a check when chosen
fn card(overlay: &Overlay, theme: &Theme, audio: Audio, selected: bool, locked: bool, width: f32) -> Rectangle {
    let hover_name = format!("record:audio:{}", audio.label());

    let fill = if selected {
        theme.selected_surface
    } else if !locked && hovered(overlay, &hover_name) {
        theme.hover_surface
    } else {
        theme.surface
    };

    let icon = Rectangle::new()
        .width(ICON_SIZE)
        .height(ICON_SIZE)
        .radius(8.0)
        .fill(theme.selected_surface)
        .align_child(Center, Center)
        .child(Text::new(audio.icon()).size(16.0).font(fonts::NERD).tight().color(theme.accent));

    let weight = if selected { Weight::SemiBold } else { Weight::Regular };

    let text_width = width - CARD_PADDING * 4.0 - ICON_SIZE - 16.0;

    let name = Rectangle::new()
        .width(text_width)
        .height(ICON_SIZE)
        .align_child(Start, Center)
        .child(Text::new(audio.label()).size(12.0).font(fonts::BODY).weight(weight).color(theme.text));

    let check = if selected { CHECK } else { "" };

    let check = Text::new(check).size(14.0).font(fonts::NERD).tight().color(theme.accent);

    let card = Rectangle::new()
        .width(width)
        .height(CARD_HEIGHT)
        .radius(12.0)
        .fill(fill)
        .padding(Padding {
            top: CARD_PADDING,
            right: CARD_PADDING * 2.0,
            bottom: CARD_PADDING,
            left: CARD_PADDING,
        })
        .child(Row::new(children![icon, name, check]).gap(CARD_PADDING).align(Center));

    // the other modes fade back while recording
    if locked {
        return card.opacity(if selected { 1.0 } else { 0.5 });
    }

    card.cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_click(move |_| Recorder::write().audio = audio)
}

// accent to start, red with the time to stop
fn button(overlay: &Overlay, theme: &Theme, status: Option<String>, width: f32) -> Rectangle {
    let hover_name = String::from("record:button");

    let (icon, label, fill) = match status {
        Some(time) => (STOP_ICON, format!("Stop · {time}"), theme.danger),
        None => (RECORD_ICON, String::from("Start recording"), theme.accent),
    };

    let fill = if hovered(overlay, &hover_name) {
        theme::mix(fill, theme.on_accent, 0.12)
    } else {
        fill
    };

    let content = Row::new(children![
        Text::new(icon).size(16.0).font(fonts::NERD).tight().color(theme.on_accent),
        Text::new(label)
            .size(13.0)
            .font(fonts::BODY)
            .weight(Weight::SemiBold)
            .color(theme.on_accent),
    ])
    .gap(8.0)
    .align(Center);

    Rectangle::new()
        .width(width)
        .height(BUTTON_HEIGHT)
        .radius(BUTTON_HEIGHT / 2.0)
        .fill(fill)
        .cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_click(|_| recorder::toggle())
        .align_child(Center, Center)
        .child(content)
}
