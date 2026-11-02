mod curtain;
mod logind;
mod password;

use std::thread;

use amane::{
    Center, Color, Column, Image, LayerWindow, Lock, Monitor, Parent, Rectangle, Service, Stack,
    Text, Weight, children,
};

use crate::clock::Clock;
use crate::fonts;
use crate::motion::{self, DEFAULT_SPATIAL};
use crate::wallpaper::Wallpaper;

pub use curtain::Curtain;
pub use logind::Logind;

use password::Typing;

const WHITE: Color = Color::rgb(0xf7, 0xf7, 0xf7);

// the wallpaper is darkened this much, so white text reads on any picture
pub const DIM: u8 = 122;

const LOCK_ICON: &str = "󰌾";

// on every monitor while the session is locked
pub fn view(monitor: &Monitor) -> LayerWindow {
    let width = monitor.width as f32;
    let height = monitor.height as f32;

    let background = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(Image::cover(&Wallpaper::read().shown));

    let dim = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(Color::rgba(0, 0, 0, DIM));

    // the compositor sizes lock screens to the monitor, so the window's own size is never used
    LayerWindow::new()
        .width(1.0)
        .height(1.0)
        .child(Stack::new(children![
            background,
            dim,
            clock(width, height),
            unlock(width, height),
        ]))
}

// the time, big, with the date under it
fn clock(width: f32, height: f32) -> Rectangle {
    let clock = Clock::read();

    let time_size = (width * 0.075).clamp(78.0, 112.0);

    let column = Column::new(children![
        Text::new(clock.time(false))
            .size(time_size)
            .font(fonts::BODY)
            .weight(Weight::SemiBold)
            .color(WHITE),
        Text::new(clock.date())
            .size(16.0)
            .font(fonts::BODY)
            .color(Color::rgba(0xff, 0xff, 0xff, 0xd9)),
    ])
    .gap(2.0)
    .align(Center);

    let top = (height * 0.18).max(72.0);

    Rectangle::new()
        .width(width)
        .height(time_size * 1.4 + 24.0)
        .translate(0.0, top)
        .align_child(Center, Center)
        .child(column)
}

/*
 * the password block rises in once a key is pressed; until then only a
 * hint at the bottom says what to do
 */
fn unlock(width: f32, height: f32) -> Stack {
    let lock = Lock::read();

    let awake = Typing::read().typing || lock.checking() || lock.failed();

    let shown = motion::follow("lock:awake", if awake { 1.0 } else { 0.0 }, DEFAULT_SPATIAL);

    let block = Rectangle::new()
        .width(width)
        .height(password::HEIGHT)
        .translate(0.0, height * 0.52 + 10.0 * (1.0 - shown))
        .opacity(shown)
        .align_child(Center, Center)
        .child(password::view(&lock));

    let hint = Text::new(format!("{LOCK_ICON}   Type your password to unlock"))
        .size(11.0)
        .font(fonts::BODY)
        .color(Color::rgba(0xff, 0xff, 0xff, 0xb8));

    let hint = Rectangle::new()
        .width(width)
        .height(20.0)
        .translate(0.0, height - 48.0)
        .opacity(1.0 - shown)
        .align_child(Center, Center)
        .child(hint);

    Stack::new(children![block, hint])
        .width(width)
        .height(height)
}

// locks the session, the same as `loginctl lock-session`
pub fn ipc(_arguments: &[String]) -> String {
    thread::spawn(logind::start);

    String::from("ok")
}
