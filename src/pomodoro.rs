use std::time::{Duration, Instant};

use amane::{
    Center, Color, Column, Key, Parent, Pointer, Rectangle, Row, Service, SpaceBetween, Text,
    Weight, Window, children,
};

use crate::fonts;
use crate::theme::{self, Theme};

// what open_window and close_window know this window by
const NAME: &str = "pomodoro";

const WIDTH: f32 = 360.0;
const HEIGHT: f32 = 320.0;

const MARGIN: f32 = 22.0;

const FOCUS: Duration = Duration::from_secs(25 * 60);
const BREAK: Duration = Duration::from_secs(5 * 60);

const TIMER_ICON: &str = "󰔛";
const CLOSE_ICON: &str = "󰅖";

const SOUND: &str = "/run/current-system/sw/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga";

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Focus,
    Break,
}

impl Phase {
    fn length(self) -> Duration {
        match self {
            Phase::Focus => FOCUS,
            Phase::Break => BREAK,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Phase::Focus => "Focus",
            Phase::Break => "Break",
        }
    }
}

pub struct Timer {
    phase: Phase,

    // time left while paused
    left: Duration,

    // when the phase ends, only while running
    ends: Option<Instant>,

    hovered: Option<String>,
}

impl Service for Timer {
    fn new() -> Self {
        Self {
            phase: Phase::Focus,
            left: FOCUS,
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
    // the next phase waits for start, so a break never begins unnoticed
    fn finish(&mut self) {
        let message = match self.phase {
            Phase::Focus => "Focus done, time for a break",
            Phase::Break => "Break over, back to focus",
        };

        amane::spawn(&format!("notify-send -a Pomodoro Pomodoro '{message}'"));
        amane::spawn(&format!("pw-play {SOUND}"));

        self.phase = match self.phase {
            Phase::Focus => Phase::Break,
            Phase::Break => Phase::Focus,
        };

        self.left = self.phase.length();
        self.ends = None;
    }
}

pub fn open() {
    amane::open_window(NAME, view);
}

fn close() {
    amane::close_window(NAME);
}

fn toggle() {
    let mut timer = Timer::write();

    timer.ends = match timer.ends {
        Some(_) => None,
        None => Some(Instant::now() + timer.left),
    };
}

fn reset() {
    let mut timer = Timer::write();

    timer.left = timer.phase.length();
    timer.ends = None;
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

    let inner_width = WIDTH - MARGIN * 2.0;

    let content = Column::new(children![
        header(&theme, inner_width),
        countdown(&theme),
        controls(&theme),
    ])
    .gap(20.0)
    .align(Center);

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
        .radius(12.0)
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

fn countdown(theme: &Theme) -> Column {
    let timer = Timer::read();

    // rounded up, so 0:00 only shows once the phase is over
    let seconds = timer.left.as_secs() + u64::from(timer.left.subsec_nanos() > 0);

    let time = format!("{:02}:{:02}", seconds / 60, seconds % 60);

    let phase = timer.phase.label();

    drop(timer);

    Column::new(children![
        Text::new(phase).size(14.0).font(fonts::BODY).weight(Weight::SemiBold).color(theme.accent),
        Text::new(time).size(56.0).font(fonts::BODY).weight(Weight::Bold).color(theme.text),
    ])
    .gap(4.0)
    .align(Center)
}

fn controls(theme: &Theme) -> Row {
    let start_label = if Timer::read().ends.is_some() { "Pause" } else { "Start" };

    Row::new(children![
        button(theme, "Reset", false, reset),
        button(theme, start_label, true, toggle),
    ])
    .gap(10.0)
}

fn button(theme: &Theme, label: &'static str, primary: bool, on_click: fn()) -> Rectangle {
    let hover_name = format!("button:{label}");

    let (fill, text) = if primary {
        (theme.accent, theme.on_accent)
    } else {
        (theme.surface, theme.text)
    };

    let fill = if hovered(&hover_name) {
        theme::mix(fill, text, 0.08)
    } else {
        fill
    };

    Rectangle::new()
        .width(96.0)
        .height(40.0)
        .radius(12.0)
        .fill(fill)
        .cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_click(move |_| on_click())
        .align_child(Center, Center)
        .child(Text::new(label).size(13.0).font(fonts::BODY).weight(Weight::SemiBold).color(text))
}
