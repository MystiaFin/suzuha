mod entry;
mod slider;

use amane::{
    Apps, Audio, AudioApp, Center, Color, Column, DesktopApp, End, Full, Image, Key, Parent,
    Pointer, Rectangle, Row, ScrollArea, Service, SpaceBetween, Stack, Start, Text, Weight, Widget,
    Window, children,
};

use crate::app_icon;
use crate::fonts;
use crate::motion;
use crate::theme::{self, Theme};

use slider::{HANDLE_INSET, LEAD, Slider};

// what open_window and close_window know this window by
const NAME: &str = "mixer";

const LIST: &str = "mixer-apps";

const WIDTH: f32 = 480.0;
const HEIGHT: f32 = 620.0;

const MARGIN: f32 = 22.0;

const HEADER_HEIGHT: f32 = 44.0;

// the space between the header, the speakers and the apps
const GAP: f32 = 20.0;

const LABEL_HEIGHT: f32 = 18.0;
const LABEL_GAP: f32 = 10.0;

// the speakers get a card of their own, with a big level and a big slider
const HERO_PADDING: f32 = 20.0;
const HERO_READOUT: f32 = 52.0;
const HERO_SLIDER: f32 = 52.0;
const HERO_GAP: f32 = 16.0;
const HERO_HEIGHT: f32 = HERO_PADDING * 2.0 + HERO_READOUT + HERO_GAP + HERO_SLIDER;
const HERO_MUTE: f32 = 52.0;

// every app is one wide slider with its icon, name and level inside
const ROW_HEIGHT: f32 = 56.0;
const ROW_GAP: f32 = 10.0;
const ROW_INSET: f32 = 10.0;
const APP_ICON_SIZE: f32 = 36.0;
const ROW_MUTE: f32 = 36.0;
const PERCENT_WIDTH: f32 = 44.0;

// how much accent an app's fill mixes in, how much grey once muted, and how much more under the pointer
const FILL_TONE: f32 = 0.3;
const MUTED_TONE: f32 = 0.18;
const HOVER_TONE: f32 = 0.1;

// a muted app's icon fades back
const MUTED_ICON_OPACITY: f32 = 0.45;

// a muted button's red tint, how strongly it lights up under the pointer, and its glyph's share of its size
const MUTED_TINT: f32 = 0.18;
const BUTTON_HOVER: f32 = 0.8;
const GLYPH_SHARE: f32 = 0.45;

pub const MIXER_ICON: &str = "\u{f1543}";
const CLOSE_ICON: &str = "󰅖";
const LOUD_ICON: &str = "\u{f057e}";
const MEDIUM_ICON: &str = "\u{f0580}";
const QUIET_ICON: &str = "\u{f057f}";
const MUTED_ICON: &str = "\u{f075f}";
const SILENT_ICON: &str = "\u{f075b}";

// apps whose desktop entry can't be found get a note
const APP_ICON: &str = "󰝚";

#[derive(Default)]
pub struct Mixer {
    // the button or slider under the pointer, none when it's over neither
    hovered: Option<String>,
}

impl Service for Mixer {
    fn new() -> Self {
        Self::default()
    }
}

// what a slider and its mute button control
#[derive(Clone)]
enum Channel {
    Speakers,

    App(AudioApp),
}

impl Channel {
    fn set(&self, volume: u8) {
        match self {
            Channel::Speakers => Audio::set_volume(volume),
            Channel::App(app) => Audio::set_app_volume(app, volume),
        }
    }

    fn toggle_mute(&self) {
        match self {
            Channel::Speakers => Audio::toggle_mute(),
            Channel::App(app) => Audio::toggle_app_mute(app),
        }
    }

    // a key for hover and glide names, so every slider's parts are told apart
    fn key(&self) -> String {
        match self {
            Channel::Speakers => String::from("speakers"),
            Channel::App(app) => format!("app:{}:{:?}", app.name(), app.process()),
        }
    }
}

// what a slider and its mute button show and control
#[derive(Clone)]
struct Level {
    channel: Channel,

    // 0 to 100
    volume: u8,

    muted: bool,
}

pub fn open() {
    amane::open_window(NAME, view);
}

fn close() {
    amane::close_window(NAME);
}

pub fn ipc(_arguments: &[String]) -> String {
    open();

    String::from("ok")
}

fn hover(name: String, inside: bool) {
    let mut mixer = Mixer::write();

    if inside {
        mixer.hovered = Some(name);
    } else if mixer.hovered.as_ref() == Some(&name) {
        mixer.hovered = None;
    }
}

fn hovered(name: &str) -> bool {
    Mixer::read().hovered.as_deref() == Some(name)
}

// 0 to 1, fading in while the pointer is over it
fn hover_amount(name: &str) -> f32 {
    motion::fade(name, if hovered(name) { 1.0 } else { 0.0 })
}

fn key_pressed(key: Key) {
    if key == Key::Escape {
        close();
    }
}

pub fn view() -> Window {
    let theme = theme::current();

    let (width, height) = match amane::window_size() {
        (0.0, _) | (_, 0.0) => (WIDTH, HEIGHT),
        size => size,
    };

    let content_width = width - MARGIN * 2.0;
    let inner_height = height - MARGIN * 2.0;

    let audio = Audio::read();

    let speakers = hero(&theme, audio.volume(), audio.muted(), content_width);

    // what the header, the speakers, the label and the gaps leave for the apps
    let used = HEADER_HEIGHT + HERO_HEIGHT + LABEL_HEIGHT + LABEL_GAP + GAP * 2.0;

    let apps = Column::new(children![
        label(&theme, "Apps", content_width),
        app_list(&theme, audio.apps(), content_width, inner_height - used),
    ])
    .gap(LABEL_GAP);

    let header = header(&theme, audio.apps().len(), content_width);

    drop(audio);

    let content = Column::new(children![header, speakers, apps])
        .width(content_width)
        .gap(GAP);

    let middle = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .align_child(Center, Start)
        .child(content);

    let frame = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(theme.background)
        .padding(MARGIN)
        .child(middle);

    Window::new()
        .title("Mixer")
        .size(WIDTH, HEIGHT)
        .on_key(key_pressed)
        .child(frame)
}

fn header(theme: &Theme, app_count: usize, width: f32) -> Row {
    let icon = Rectangle::new()
        .width(HEADER_HEIGHT)
        .height(HEADER_HEIGHT)
        .radius(14.0)
        .fill(theme.selected_surface)
        .align_child(Center, Center)
        .child(
            Text::new(MIXER_ICON)
                .size(22.0)
                .font(fonts::NERD)
                .tight()
                .color(theme.accent),
        );

    let title = Text::new("Mixer")
        .size(21.0)
        .font(fonts::BODY)
        .weight(Weight::Bold)
        .color(theme.text);

    let status = match app_count {
        0 => String::from("Nothing playing"),
        1 => String::from("1 app"),
        count => format!("{count} apps"),
    };

    let status = Text::new(status)
        .size(12.0)
        .font(fonts::BODY)
        .color(theme.muted_text);

    let titles = Column::new(children![title, status]);

    let close_fill = theme::mix(
        Color::TRANSPARENT,
        theme.hover_surface,
        hover_amount("close"),
    );

    let close = Rectangle::new()
        .width(40.0)
        .height(40.0)
        .radius(Full)
        .fill(close_fill)
        .cursor(Pointer)
        .on_hover(|inside| hover(String::from("close"), inside))
        .on_click(|_| close())
        .align_child(Center, Center)
        .child(
            Text::new(CLOSE_ICON)
                .size(18.0)
                .font(fonts::NERD)
                .tight()
                .color(theme.secondary_text),
        );

    Row::new(children![
        Row::new(children![icon, titles]).gap(12.0).align(Center),
        close
    ])
    .width(width)
    .height(HEADER_HEIGHT)
    .justify(SpaceBetween)
    .align(Center)
}

fn label(theme: &Theme, text: &str, width: f32) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(LABEL_HEIGHT)
        .align_child(Start, Center)
        .child(
            Text::new(text)
                .size(13.0)
                .font(fonts::BODY)
                .weight(Weight::SemiBold)
                .color(theme.secondary_text),
        )
}

// the speakers: their name and a big level over a big slider, with a mute toggle
fn hero(theme: &Theme, volume: u8, muted: bool, width: f32) -> Rectangle {
    let inner_width = width - HERO_PADDING * 2.0;

    let (number, number_color) = if muted {
        (String::from("Muted"), theme.secondary_text)
    } else {
        (format!("{volume}"), theme.text)
    };

    let mut figure: Vec<Box<dyn Widget>> = vec![Box::new(
        Text::new(number)
            .size(34.0)
            .font(fonts::BODY)
            .weight(Weight::Bold)
            .tight()
            .color(number_color),
    )];

    if !muted {
        figure.push(Box::new(
            Text::new("%")
                .size(17.0)
                .font(fonts::BODY)
                .weight(Weight::SemiBold)
                .color(theme.secondary_text),
        ));
    }

    let name = Text::new("Speakers")
        .size(13.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.secondary_text);

    let readout = Column::new(children![name, Row::new(figure).gap(2.0).align(End)]).gap(4.0);

    let level = Level {
        channel: Channel::Speakers,
        volume,
        muted,
    };

    let mute = mute_button(theme, &level, HERO_MUTE, theme.background);

    let top = Row::new(children![readout, mute])
        .width(inner_width)
        .height(HERO_READOUT)
        .justify(SpaceBetween)
        .align(Center);

    let fill = if muted {
        theme.muted_text
    } else {
        theme.accent
    };

    let slider = Slider {
        level,
        width: inner_width,
        height: HERO_SLIDER,
        track: theme.background,
        fill,
        handle: Some(theme.on_accent),
    };

    let glyph = Rectangle::new()
        .width(LEAD - HANDLE_INSET)
        .height(HERO_SLIDER)
        .align_child(Center, Center)
        .child(
            Text::new(level_icon(&slider.level))
                .size(22.0)
                .font(fonts::NERD)
                .tight()
                .color(theme.on_accent),
        );

    Rectangle::new()
        .width(width)
        .height(HERO_HEIGHT)
        .radius(28.0)
        .fill(theme.selected_surface)
        .padding(HERO_PADDING)
        .child(
            Column::new(children![top, slider::view(slider, vec![Box::new(glyph)])]).gap(HERO_GAP),
        )
}

// every app playing sound, oldest first, scrolling once they don't fit
fn app_list(theme: &Theme, apps: &[AudioApp], width: f32, height: f32) -> Rectangle {
    entry::forget(apps);

    let area = Rectangle::new().width(width).height(height);

    if apps.is_empty() {
        return area.align_child(Center, Center).child(empty(theme));
    }

    let desktop = Apps::read();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for app in apps {
        rows.push(Box::new(app_row(theme, &desktop, app, width)));
    }

    let total = ROW_HEIGHT * rows.len() as f32 + ROW_GAP * (rows.len() - 1) as f32;

    let column = Column::new(rows).width(width).height(total).gap(ROW_GAP);

    area.child(ScrollArea::new(LIST, column))
}

fn empty(theme: &Theme) -> Column {
    let badge = Rectangle::new()
        .width(64.0)
        .height(64.0)
        .radius(Full)
        .fill(theme.surface)
        .align_child(Center, Center)
        .child(
            Text::new(SILENT_ICON)
                .size(26.0)
                .font(fonts::NERD)
                .tight()
                .color(theme.muted_text),
        );

    let title = Text::new("Nothing is playing")
        .size(14.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text);

    let detail = Text::new("Apps show up here while they make sound")
        .size(12.0)
        .font(fonts::BODY)
        .color(theme.muted_text);

    Column::new(children![badge, title, detail])
        .gap(8.0)
        .align(Center)
}

// a tonal slider the width of the list, with the app's icon, name and level inside it
fn app_row(theme: &Theme, desktop: &Apps, app: &AudioApp, width: f32) -> Stack {
    let level = Level {
        channel: Channel::App(app.clone()),
        volume: app.volume(),
        muted: app.muted(),
    };

    let entry = entry::find(desktop, app);

    // electron apps all play as "Chromium", so the entry's name says which one it is
    let name = entry.map_or(app.name(), DesktopApp::name);

    let lift = hover_amount(&format!("{}:row", level.channel.key())) * HOVER_TONE;

    let fill = if level.muted {
        theme::mix(theme.surface, theme.muted_text, MUTED_TONE + lift)
    } else {
        theme::mix(theme.surface, theme.accent, FILL_TONE + lift)
    };

    let icon = Rectangle::new()
        .width(LEAD - HANDLE_INSET)
        .height(ROW_HEIGHT)
        .align_child(Center, Center)
        .child(app_icon(theme, entry, level.muted));

    let text_color = if level.muted {
        theme.muted_text
    } else {
        theme.text
    };

    // a window too narrow for the name leaves it no room instead of a negative width
    let name_width = f32::max(
        width - LEAD - ROW_INSET - PERCENT_WIDTH - ROW_MUTE - ROW_INSET,
        0.0,
    );

    let name = Rectangle::new()
        .width(name_width)
        .height(ROW_HEIGHT)
        .translate(LEAD + ROW_INSET / 2.0, 0.0)
        .align_child(Start, Center)
        .child(
            Text::new(name)
                .size(14.0)
                .font(fonts::BODY)
                .weight(Weight::SemiBold)
                .color(text_color)
                .elide(),
        );

    let percent = Rectangle::new()
        .width(PERCENT_WIDTH)
        .height(ROW_HEIGHT)
        .translate(width - ROW_INSET - ROW_MUTE - PERCENT_WIDTH, 0.0)
        .align_child(End, Center)
        .child(
            Text::new(format!("{}%", level.volume))
                .size(13.0)
                .font(fonts::BODY)
                .weight(Weight::SemiBold)
                .color(theme.secondary_text),
        );

    let mute = mute_button(theme, &level, ROW_MUTE, Color::TRANSPARENT).translate(
        width - ROW_INSET / 2.0 - ROW_MUTE,
        (ROW_HEIGHT - ROW_MUTE) / 2.0,
    );

    let slider = Slider {
        level,
        width,
        height: ROW_HEIGHT,
        track: theme.surface,
        fill,
        handle: None,
    };

    let mut layers = slider::layers(
        &slider,
        vec![Box::new(icon), Box::new(name), Box::new(percent)],
    );

    // over the drag area, so muting doesn't also move the level
    layers.push(Box::new(mute));

    Stack::new(layers).width(width).height(ROW_HEIGHT)
}

// the app's own icon, or a note when it has none; faded while muted
fn app_icon(theme: &Theme, entry: Option<&DesktopApp>, muted: bool) -> Rectangle {
    let icon = Rectangle::new()
        .width(APP_ICON_SIZE)
        .height(APP_ICON_SIZE)
        .radius(10.0)
        .align_child(Center, Center);

    let icon = match entry.and_then(app_icon::path) {
        // twice the size, so it stays sharp on a scaled screen
        Some(path) => {
            let pixels = (APP_ICON_SIZE * 2.0) as u32;

            icon.fill(Image::contain(path).thumbnail(pixels, pixels))
        }

        None => icon.fill(theme.selected_surface).child(
            Text::new(APP_ICON)
                .size(18.0)
                .font(fonts::NERD)
                .tight()
                .color(theme.accent),
        ),
    };

    icon.opacity(if muted { MUTED_ICON_OPACITY } else { 1.0 })
}

fn level_icon(level: &Level) -> &'static str {
    match level.volume {
        _ if level.muted => MUTED_ICON,
        0..=33 => QUIET_ICON,
        34..=66 => MEDIUM_ICON,
        _ => LOUD_ICON,
    }
}

// round, showing the level's icon; muted, it turns red
fn mute_button(theme: &Theme, level: &Level, size: f32, rest: Color) -> Rectangle {
    let hover_name = format!("{}:mute", level.channel.key());

    let lift = hover_amount(&hover_name);

    let (base, glyph_color) = if level.muted {
        (theme::mix(rest, theme.danger, MUTED_TINT), theme.danger)
    } else {
        (rest, theme.text)
    };

    let fill = theme::mix(base, theme.hover_surface, lift * BUTTON_HOVER);

    let channel = level.channel.clone();

    Rectangle::new()
        .width(size)
        .height(size)
        .radius(Full)
        .fill(fill)
        .cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_click(move |_| channel.toggle_mute())
        .align_child(Center, Center)
        .child(
            Text::new(level_icon(level))
                .size(size * GLYPH_SHARE)
                .font(fonts::NERD)
                .tight()
                .color(glyph_color),
        )
}
