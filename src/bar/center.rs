use amane::{
    Audio, Center, Media, Parent, Pointer, Rectangle, Row, Scroll, Service, Text, Widget,
    children,
};

use super::{pill, ring, timer};
use crate::clock::Clock;
use crate::settings::Settings;
use crate::fonts;
use crate::overlay::Overlay;
use crate::theme::Theme;

const MEDIA_ICON: &str = "󰎈";

// longer song text is cut off
const MEDIA_TEXT_WIDTH: f32 = 180.0;

const RING_SIZE: f32 = 18.0;
const RING_THICKNESS: f32 = 4.0;

// how far one wheel step moves a level
const STEP: i32 = 5;

pub fn view(theme: &Theme, width: f32) -> Row {
    let settings = Settings::read();

    let show_audio = settings.flag("bar_audio");
    let show_media = settings.flag("bar_media");
    let show_clock = settings.flag("bar_clock");

    // the clock reads the settings again, so this read ends first
    drop(settings);

    let mut items: Vec<Box<dyn Widget>> = Vec::new();

    if show_audio {
        items.push(Box::new(audio(theme)));
    }

    if show_media {
        items.push(Box::new(media(theme)));
    }

    if let Some(timer) = timer::view(theme) {
        items.push(Box::new(timer));
    }

    if show_clock {
        items.push(Box::new(clock(theme)));
    }

    Row::new(items)
        .width(width)
        .height(Parent)
        .gap(2.0)
        .justify(Center)
        .align(Center)
}

// the speaker and microphone rings, scroll over one to change its level, click to open the control center
fn audio(theme: &Theme) -> Rectangle {
    let audio = Audio::read();

    let speaker_color = if audio.muted() {
        theme.muted_text
    } else {
        theme.accent
    };

    let microphone_color = if audio.microphone_muted() {
        theme.muted_text
    } else {
        theme.accent
    };

    let speaker_value = f32::from(audio.volume()) / 100.0;
    let microphone_value = f32::from(audio.microphone_volume()) / 100.0;

    let speaker = Rectangle::new()
        .width(RING_SIZE)
        .height(RING_SIZE)
        .on_scroll(scroll_volume)
        .child(ring::view(RING_SIZE, RING_THICKNESS, speaker_value, speaker_color, theme.border));

    let microphone = Rectangle::new()
        .width(RING_SIZE)
        .height(RING_SIZE)
        .on_scroll(scroll_microphone)
        .child(ring::view(
            RING_SIZE,
            RING_THICKNESS,
            microphone_value,
            microphone_color,
            theme.border,
        ));

    // two rings 8px apart, plus 16px of padding
    let width = RING_SIZE * 2.0 + 8.0 + 16.0;

    pill::view(width, theme.surface)
        .cursor(Pointer)
        .on_click(|_| Overlay::toggle_control_center())
        .align_child(Center, Center)
        .child(Row::new(children![speaker, microphone]).gap(8.0).align(Center))
}

// "artist - title", or "No media" while nothing plays
fn media(theme: &Theme) -> Rectangle {
    let media = Media::read();

    let text = if media.title().is_empty() {
        String::from("No media")
    } else if media.artist().is_empty() {
        String::from(media.title())
    } else {
        format!("{} - {}", media.artist(), media.title())
    };

    let text_width = pill::text_width(&text, 13.0).min(MEDIA_TEXT_WIDTH);

    let icon = Text::new(MEDIA_ICON)
        .size(14.0)
        .font(fonts::NERD)
        .color(theme.success);

    let label = Rectangle::new()
        .width(text_width)
        .height(Parent)
        .align_child(amane::Start, Center)
        .child(pill::label(&text, 13.0, theme.text).elide());

    // the icon is about 9px wide, then a 7px gap and 20px of padding
    let width = 9.0 + 7.0 + text_width + 20.0;

    pill::view(width, theme.surface)
        .cursor(Pointer)
        .on_click(|_| Media::play_pause())
        .align_child(Center, Center)
        .child(Row::new(children![icon, label]).gap(7.0).align(Center))
}

// "09:05 PM  •  Tuesday, 29 Sep 2026", plain text with 8px in front, no pill
fn clock(theme: &Theme) -> Row {
    let clock = Clock::read();

    let seconds = Settings::read().flag("clock_seconds");

    let text = format!("{}  •  {}", clock.time(seconds), clock.date());

    Row::new(children![
        Rectangle::new().width(8.0).height(1.0),
        pill::label(&text, 15.0, theme.text),
    ])
    .align(Center)
}

fn scroll_volume(scroll: Scroll) {
    let volume = i32::from(Audio::read().volume());

    Audio::set_volume(step(volume, scroll));
}

fn scroll_microphone(scroll: Scroll) {
    let volume = i32::from(Audio::read().microphone_volume());

    Audio::set_microphone_volume(step(volume, scroll));
}

// wheel up is a negative y, which turns the level up
fn step(level: i32, scroll: Scroll) -> u8 {
    let change = if scroll.y < 0.0 { STEP } else { -STEP };

    (level + change).clamp(0, 100) as u8
}
