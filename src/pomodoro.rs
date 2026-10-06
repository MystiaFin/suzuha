mod ring;

use std::cell::RefCell;
use std::time::{Duration, Instant};

use amane::{
    Center, Color, Column, Full, Key, Parent, Pointer, Rectangle, Row, Service, SpaceBetween,
    Stack, Text, Weight, Widget, Window, children,
};

use crate::fonts;
use crate::motion::{self, Glide};
use crate::theme::{self, Theme};

// what open_window and close_window know this window by
const NAME: &str = "pomodoro";

const WIDTH: f32 = 460.0;
const HEIGHT: f32 = 600.0;

const MARGIN: f32 = 22.0;

// the space between the mode buttons, the ring and the controls
const GAP: f32 = 24.0;

const CHIP_HEIGHT: f32 = 40.0;

// connected mode buttons sit close, with small corners where they meet
const CHIP_GAP: f32 = 2.0;
const CHIP_INNER_RADIUS: f32 = 6.0;

const PLAY_HEIGHT: f32 = 64.0;

// a long break comes after this many focus rounds
const ROUNDS: u32 = 4;

pub const TIMER_ICON: &str = "󰔛";
const CLOSE_ICON: &str = "󰅖";
pub const PLAY_ICON: &str = "\u{f040a}";
pub const PAUSE_ICON: &str = "\u{f03e4}";
const RESET_ICON: &str = "\u{f0453}";

const SOUND: &str = "/run/current-system/sw/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga";

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Focus,
    ShortBreak,
    LongBreak,
}

const MODES: [Mode; 3] = [Mode::Focus, Mode::ShortBreak, Mode::LongBreak];

thread_local! {
    // kept outside services, a service write from view() would draw frames forever
    static MORPHS: RefCell<Vec<Glide>> = const { RefCell::new(Vec::new()) };
}

impl Mode {
    fn length(self) -> Duration {
        let minutes = match self {
            Mode::Focus => 25,
            Mode::ShortBreak => 5,
            Mode::LongBreak => 15,
        };

        Duration::from_secs(minutes * 60)
    }

    fn label(self) -> &'static str {
        match self {
            Mode::Focus => "Focus",
            Mode::ShortBreak => "Short break",
            Mode::LongBreak => "Long break",
        }
    }
}

pub struct Timer {
    mode: Mode,

    // focus rounds finished, for when the long break comes
    rounds: u32,

    // time left while paused
    left: Duration,

    // when the mode ends, only while running
    ends: Option<Instant>,

    hovered: Option<String>,
}

impl Service for Timer {
    fn new() -> Self {
        Self {
            mode: Mode::Focus,
            rounds: 0,
            left: Mode::Focus.length(),
            ends: None,
            hovered: None,
        }
    }

    // keeps counting with the window closed
    fn update(&mut self) -> bool {
        let Some(ends) = self.ends else {
            return false;
        };

        self.left = ends.saturating_duration_since(Instant::now());

        if self.left.is_zero() {
            self.finish();
        }

        true
    }
}

impl Timer {
    // the next mode waits for start, so a break never begins unnoticed
    fn finish(&mut self) {
        let message = match self.mode {
            Mode::Focus => "Focus done, time for a break",
            _ => "Break over, back to focus",
        };

        amane::spawn(&format!("notify-send -a Pomodoro Pomodoro '{message}'"));
        amane::spawn(&format!("pw-play {SOUND}"));

        let next = match self.mode {
            Mode::Focus => {
                self.rounds += 1;

                if self.rounds % ROUNDS == 0 {
                    Mode::LongBreak
                } else {
                    Mode::ShortBreak
                }
            }

            _ => Mode::Focus,
        };

        self.choose(next);
    }

    fn choose(&mut self, mode: Mode) {
        self.mode = mode;
        self.left = mode.length();
        self.ends = None;
    }
}

pub fn open() {
    amane::open_window(NAME, view);
}

fn close() {
    amane::close_window(NAME);
}

pub fn toggle() {
    let mut timer = Timer::write();

    timer.ends = match timer.ends {
        Some(_) => None,
        None => Some(Instant::now() + timer.left),
    };
}

fn reset() {
    let mut timer = Timer::write();

    let mode = timer.mode;

    timer.choose(mode);
}

fn hover(name: String, inside: bool) {
    let mut timer = Timer::write();

    if inside {
        timer.hovered = Some(name);
    } else if timer.hovered.as_ref() == Some(&name) {
        timer.hovered = None;
    }
}

fn hovered(name: &str) -> bool {
    Timer::read().hovered.as_deref() == Some(name)
}

fn key_pressed(key: Key) {
    match key {
        Key::Escape => close(),
        Key::Space => toggle(),
        _ => {}
    }
}

pub fn view() -> Window {
    let theme = theme::current();

    let (width, height) = match amane::window_size() {
        (0.0, _) | (_, 0.0) => (WIDTH, HEIGHT),
        size => size,
    };

    let inner_width = width - MARGIN * 2.0;
    let inner_height = height - MARGIN * 2.0;

    // what the header, mode buttons, controls and gaps leave for the ring
    let room = inner_height - 40.0 - CHIP_HEIGHT - PLAY_HEIGHT - GAP * 3.0;

    let ring_size = room.min(inner_width).clamp(180.0, 380.0);

    let body = Column::new(children![
        mode_row(&theme),
        dial(&theme, ring_size),
        controls(&theme),
    ])
    .gap(GAP)
    .align(Center);

    let body = Rectangle::new()
        .width(inner_width)
        .height(inner_height - 40.0)
        .align_child(Center, Center)
        .child(body);

    let content = Column::new(children![header(&theme, inner_width), body]);

    let frame = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(theme.background)
        .padding(MARGIN)
        .child(content);

    Window::new()
        .title("Pomodoro")
        .size(WIDTH, HEIGHT)
        .on_key(key_pressed)
        .child(frame)
}

fn header(theme: &Theme, width: f32) -> Row {
    let icon = Rectangle::new()
        .width(40.0)
        .height(40.0)
        .radius(12.0)
        .fill(theme.selected_surface)
        .align_child(Center, Center)
        .child(Text::new(TIMER_ICON).size(21.0).font(fonts::NERD).tight().color(theme.accent));

    let title = Text::new("Pomodoro")
        .size(21.0)
        .font(fonts::BODY)
        .weight(Weight::Bold)
        .color(theme.text);

    let close_fill = if hovered("close") {
        theme.hover_surface
    } else {
        Color::TRANSPARENT
    };

    let close = Rectangle::new()
        .width(40.0)
        .height(40.0)
        .radius(Full)
        .fill(close_fill)
        .cursor(Pointer)
        .on_hover(|inside| hover(String::from("close"), inside))
        .on_click(|_| close())
        .align_child(Center, Center)
        .child(Text::new(CLOSE_ICON).size(18.0).font(fonts::NERD).tight().color(theme.secondary_text));

    Row::new(children![Row::new(children![icon, title]).gap(12.0).align(Center), close])
        .width(width)
        .height(40.0)
        .justify(SpaceBetween)
        .align(Center)
}

/*
 * a connected button group: its two ends are fully round, the corners
 * between buttons small, and the chosen mode turns into a pill; picking
 * one starts it over, paused
 */
fn mode_row(theme: &Theme) -> Row {
    let chosen = Timer::read().mode;

    let mut chips: Vec<Box<dyn Widget>> = Vec::new();

    for (index, mode) in MODES.into_iter().enumerate() {
        chips.push(Box::new(mode_chip(theme, index, mode, mode == chosen)));
    }

    Row::new(chips).gap(CHIP_GAP)
}

fn mode_chip(theme: &Theme, index: usize, mode: Mode, chosen: bool) -> Rectangle {
    let label = mode.label();

    let hover_name = format!("mode:{label}");

    let (fill, text) = if chosen {
        (theme.accent, theme.on_accent)
    } else if hovered(&hover_name) {
        (theme.hover_surface, theme.text)
    } else {
        (theme.surface, theme.secondary_text)
    };

    let pill = CHIP_HEIGHT / 2.0;

    let inner = morph(index, if chosen { pill } else { CHIP_INNER_RADIUS });

    let left = if index == 0 { pill } else { inner };
    let right = if index == MODES.len() - 1 { pill } else { inner };

    Rectangle::new()
        .width(label.len() as f32 * 8.0 + 36.0)
        .height(CHIP_HEIGHT)
        .radius_top_left(left)
        .radius_bottom_left(left)
        .radius_top_right(right)
        .radius_bottom_right(right)
        .fill(fill)
        .cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_click(move |_| Timer::write().choose(mode))
        .align_child(Center, Center)
        .child(Text::new(label).size(13.0).font(fonts::BODY).weight(Weight::SemiBold).color(text))
}

pub fn running() -> bool {
    Timer::read().ends.is_some()
}

// like "Focus  24:13" while a round is under way, for the bar
pub fn status() -> Option<String> {
    let timer = Timer::read();

    let started = timer.ends.is_some() || timer.left != timer.mode.length();

    if !started {
        return None;
    }

    Some(format!("{}  {}", timer.mode.label(), time(timer.left)))
}

// rounded up, so 00:00 only shows once the mode is over
fn time(left: Duration) -> String {
    let seconds = left.as_secs() + u64::from(left.subsec_nanos() > 0);

    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

// the time left inside a wavy ring of how much has gone by
fn dial(theme: &Theme, size: f32) -> Stack {
    let timer = Timer::read();

    let gone = 1.0 - timer.left.as_secs_f32() / timer.mode.length().as_secs_f32();

    let left = time(timer.left);

    let running = timer.ends.is_some();

    let state = if running { "Running" } else { "Paused" };

    drop(timer);

    let readout = Column::new(children![
        Text::new(left).size(size * 0.24).font(fonts::BODY).weight(Weight::Bold).tight().color(theme.text),
        Text::new(state).size(14.0).font(fonts::BODY).weight(Weight::SemiBold).color(theme.accent),
    ])
    .gap(8.0)
    .align(Center);

    let middle = Rectangle::new()
        .width(size)
        .height(size)
        .align_child(Center, Center)
        .child(readout);

    Stack::new(children![ring::view(size, gone, running, theme.accent, theme.selected_surface), middle])
        .width(size)
        .height(size)
}

// reset beside a big play button that squares off while running
fn controls(theme: &Theme) -> Row {
    let running = Timer::read().ends.is_some();

    let (icon, radius) = if running {
        (PAUSE_ICON, 22.0)
    } else {
        (PLAY_ICON, PLAY_HEIGHT / 2.0)
    };

    let play_fill = if hovered("play") {
        theme::mix(theme.accent, theme.on_accent, 0.08)
    } else {
        theme.accent
    };

    let play = Rectangle::new()
        .width(112.0)
        .height(PLAY_HEIGHT)
        .radius(morph(MODES.len(), radius))
        .fill(play_fill)
        .cursor(Pointer)
        .on_hover(|inside| hover(String::from("play"), inside))
        .on_click(|_| toggle())
        .align_child(Center, Center)
        .child(Text::new(icon).size(30.0).font(fonts::NERD).tight().color(theme.on_accent));

    let reset_fill = if hovered("reset") {
        theme::mix(theme.selected_surface, theme.text, 0.08)
    } else {
        theme.selected_surface
    };

    let reset = Rectangle::new()
        .width(PLAY_HEIGHT)
        .height(PLAY_HEIGHT)
        .radius(Full)
        .fill(reset_fill)
        .cursor(Pointer)
        .on_hover(|inside| hover(String::from("reset"), inside))
        .on_click(|_| reset())
        .align_child(Center, Center)
        .child(Text::new(RESET_ICON).size(24.0).font(fonts::NERD).tight().color(theme.text));

    Row::new(children![reset, play]).gap(12.0).align(Center)
}

/*
 * a corner radius that glides to its target, one per shape: the mode
 * buttons by index, then the play button
 */
fn morph(index: usize, radius: f32) -> f32 {
    MORPHS.with_borrow_mut(|morphs| {
        while morphs.len() <= index {
            morphs.push(motion::spatial(radius, motion::DEFAULT_SPATIAL));
        }

        morphs[index].to(radius);

        morphs[index].value()
    })
}
