mod center;
mod motion;
mod pill;
mod reveal;
mod star;
mod ring;
mod system;
mod timer;
mod workspaces;

use amane::{
    Color, Full, Layer, LayerWindow, Monitor, Parent, Rectangle, Row, Service, Stack, Vertical,
    Zone, children,
};

use crate::lock_screen::Curtain;
use crate::motion::{self as shell_motion, DEFAULT_SPATIAL};
use crate::settings::Settings;
use crate::theme;

use reveal::Reveal;

const CORNER_RADIUS: f32 = 16.0;

pub const ITEM_HEIGHT: f32 = 26.0;

// what is left of an auto-hidden bar, for the pointer to find
const TRIGGER: f32 = 2.0;

// the bar's height from the settings
pub fn height() -> f32 {
    Settings::read().number("bar_height")
}

// the room the bar keeps free for itself, none while it hides
pub fn reserved() -> f32 {
    if Settings::read().flag("bar_auto_hide") {
        return 0.0;
    }

    height()
}

// the room kept free above the rest of the screen
pub fn reserved_top() -> f32 {
    if Settings::read().text("bar_position") == "bottom" {
        return 0.0;
    }

    reserved()
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = theme::current();

    let settings = Settings::read();

    let height = settings.number("bar_height");
    let bottom = settings.text("bar_position") == "bottom";
    let hides = settings.flag("bar_auto_hide");

    drop(settings);

    // how far the bar is out, always all the way unless it hides
    let target = if !hides || Reveal::read().inside { 1.0 } else { 0.0 };

    let shown = shell_motion::follow(&format!("bar:{}", monitor.name), target, DEFAULT_SPATIAL);

    let window_height = (height * shown).max(TRIGGER);

    // three equal thirds, so the middle section stays centered whatever the sides hold
    let third = monitor.width as f32 / 3.0;

    let sections = Row::new(children![
        workspaces::view(monitor, &theme, third),
        center::view(&theme, third),
        system::view(&theme, third),
    ]);

    // taller than the bar and clipped, so only the corners away from the edge come out rounded
    let background = Rectangle::new()
        .width(Parent)
        .height(height + CORNER_RADIUS)
        .radius(CORNER_RADIUS)
        .fill(theme.background);

    let background = if bottom {
        background.translate(0.0, -CORNER_RADIUS)
    } else {
        background
    };

    let content = Rectangle::new()
        .width(Parent)
        .height(height)
        .child(sections);

    // a hiding bar slides toward its edge, the part past the window is clipped
    let hidden = height - window_height;

    let slide = if bottom { 0.0 } else { -hidden };

    let bar = Stack::new(children![background, content])
        .width(Parent)
        .height(height);

    let edge = if bottom {
        Vertical::Bottom
    } else {
        Vertical::Top
    };

    let space = if hides { Zone::Ignore } else { Zone::Reserve };

    // a hiding bar leaves no black strip behind
    let behind = if hides { Color::TRANSPARENT } else { Color::BLACK };

    // black behind the rounded corners, so the screen's corners look rounded too
    LayerWindow::new()
        .width(Full)
        .height(window_height)
        .anchor_vertical(edge)
        .layer(Layer::Top)
        .space(space)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(behind)
                .opacity(Curtain::read().items.value())
                .clip()
                .on_hover(reveal::hover)
                .child(Rectangle::new().width(Parent).height(height).translate(0.0, slide).child(bar)),
        )
}
