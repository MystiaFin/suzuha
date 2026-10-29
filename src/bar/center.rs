use std::cell::RefCell;

use amane::{
    Audio, Center, Media, Padding, Parent, Pointer, Rectangle, Row, Scroll, Service, Size, Start,
    Text, Widget, children,
};

use super::{pill, ring, timer};
use crate::clock::Clock;
use crate::settings::Settings;
use crate::fonts;
use crate::motion::{self, Spring};
use crate::overlay::Overlay;
use crate::theme::Theme;

const MEDIA_ICON: &str = "󰎈";

// longer song text is cut off
const MEDIA_CHARACTERS: usize = 32;

const MEDIA_TEXT_SIZE: f32 = 13.0;

// the room at the pill's ends, and between the rings and the song
const PADDING: f32 = 10.0;
const SECTION_GAP: f32 = 12.0;

const RING_SIZE: f32 = 18.0;
const RING_THICKNESS: f32 = 4.0;

// how far one wheel step moves a level
const STEP: i32 = 5;

thread_local! {
    // kept outside services, a service write from view() would draw frames forever
    static WIDTH: RefCell<Option<Spring>> = const { RefCell::new(None) };
}

pub fn view(theme: &Theme, width: f32) -> Row {
    let settings = Settings::read();

    let show_audio = settings.flag("bar_audio");
    let show_media = settings.flag("bar_media");
    let show_clock = settings.flag("bar_clock");

    // the clock reads the settings again, so this read ends first
    drop(settings);

    let mut items: Vec<Box<dyn Widget>> = Vec::new();

    if show_audio || show_media {
        items.push(Box::new(media(theme, show_audio, show_media)));
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

// the speaker and microphone rings, then the song, in one pill that opens the control center
fn media(theme: &Theme, show_audio: bool, show_song: bool) -> Rectangle {
    let mut sections: Vec<Box<dyn Widget>> = Vec::new();

    // the padding at both ends
    let mut full = PADDING * 2.0;

    if show_audio {
        sections.push(Box::new(rings(theme)));

        // two rings 8px apart
        full += RING_SIZE * 2.0 + 8.0;
    }

    if show_audio && show_song {
        full += SECTION_GAP;
    }

    if show_song {
        let (song, song_width) = song(theme);

        sections.push(Box::new(song));

        full += song_width;
    }

    let width = follow_width(full);

    // always as wide as the pill is going to be, so a shrinking pill clips it instead of squeezing it
    let content = Rectangle::new()
        .width(full)
        .height(Parent)
        .align_child(Start, Center)
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: PADDING,
        })
        .child(Row::new(sections).gap(SECTION_GAP).align(Center));

    pill::view(width, theme.surface)
        .clip()
        .cursor(Pointer)
        .on_click(|_| Overlay::toggle_control_center())
        .align_child(Start, Center)
        .child(content)
}

// scroll over a ring to change its level
fn rings(theme: &Theme) -> Row {
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

    Row::new(children![speaker, microphone]).gap(8.0).align(Center)
}

// the icon and "artist - title", or "No media" while nothing plays, and how wide they are
fn song(theme: &Theme) -> (Row, f32) {
    let media = Media::read();

    let text = if media.title().is_empty() {
        String::from("No media")
    } else if media.artist().is_empty() {
        String::from(media.title())
    } else {
        format!("{} - {}", media.artist(), media.title())
    };

    let text = shorten(&text);

    let icon = Text::new(MEDIA_ICON)
        .size(14.0)
        .font(fonts::NERD)
        .color(theme.success);

    let label = pill::label(&text, MEDIA_TEXT_SIZE, theme.text);

    // the icon, a 7px gap, then the text, each as wide as the font really draws it
    let width = measure(&icon) + 7.0 + measure(&label);

    let row = Row::new(children![icon, label]).gap(7.0).align(Center);

    (row, width)
}

// song text past MEDIA_CHARACTERS is cut off
fn shorten(text: &str) -> String {
    if text.chars().count() <= MEDIA_CHARACTERS {
        return String::from(text);
    }

    // one character is left for the ellipsis
    let kept: String = text.chars().take(MEDIA_CHARACTERS - 1).collect();

    format!("{}…", kept.trim_end())
}

// the width a widget asks for, from its real font measurements for text
fn measure(widget: &dyn Widget) -> f32 {
    match widget.width() {
        Size::Fixed(width) => width,
        Size::Parent => 0.0,
    }
}

// springs toward the width the pill wants, so it grows and shrinks as the song changes
fn follow_width(target: f32) -> f32 {
    WIDTH.with_borrow_mut(|width| {
        let spring = width.get_or_insert_with(|| motion::size(target));

        spring.to(target);

        spring.value()
    })
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
