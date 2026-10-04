pub mod control_center;
pub mod launcher;
pub mod dismiss;
mod panel;
mod popups;
pub mod power_menu;
mod region;
mod state;
pub mod utility;

use amane::{
    Color, Horizontal, InputArea, Key, Keyboard, Layer, LayerWindow, Margin, Monitor, Parent,
    Rectangle, Service, Stack, Vertical, Zone, children,
};

use crate::bar;
use crate::liquid::{self, Blob, Placement};
use crate::lock_screen::Curtain;
use crate::theme;

pub use region::Region;
pub use state::Overlay;

// what a panel hands the overlay while it is out, all in screen coordinates
pub struct PanelView {
    pub blob: Blob,

    // where it is right now, which is where it takes the pointer
    pub input: Region,

    // everything it can cover once fully out, melted edges included
    pub reach: Region,
}

/*
 * the window only covers what the open panels can reach: every pixel of
 * it runs the liquid shader each frame, and a full screen of them is
 * more than the gpu manages within one refresh.
 * it stays mapped, 1 pixel big, with every panel away, because mapping a
 * window takes a round trip with the compositor, long enough to miss an opening
 */
pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = theme::current();

    let overlay = Overlay::read();

    // the area below the bar, which a window that respects the bar is placed in
    let screen = Region {
        x: 0.0,
        y: 0.0,
        width: monitor.width as f32,
        height: monitor.height as f32 - bar::reserved(),
    };

    let mut panels: Vec<PanelView> = Vec::new();

    if let Some(panel) = power_menu::view(&overlay, &theme) {
        panels.push(panel);
    }

    if let Some(panel) = launcher::view(&overlay, &theme, screen) {
        panels.push(panel);
    }

    if let Some(panel) = control_center::view(&overlay, &theme, screen) {
        panels.push(panel);
    }

    if let Some(panel) = utility::view(&overlay, &theme, screen) {
        panels.push(panel);
    }

    if let Some(panel) = popups::view(&overlay, &theme, screen) {
        panels.push(panel);
    }

    let Some(first) = panels.first() else {
        return empty(screen);
    };

    /*
     * the launcher and a password field are typed into, so they take the
     * keyboard while open; the utility center takes it once clicked, for escape
     */
    let keyboard = if overlay.launcher.shown || utility::wants_keyboard(&overlay) {
        Keyboard::Exclusive
    } else if overlay.utility.shown {
        Keyboard::OnDemand
    } else {
        Keyboard::None
    };

    let mut reach = first.reach;

    for panel in &panels {
        reach = reach.join(panel.reach);
    }

    // a fraction cut off the window's corner would leave a sliver between a panel and its edge
    let reach = reach.within(screen).snapped();

    let mut blobs = Vec::new();
    let mut areas = Vec::new();

    for panel in panels {
        areas.push(input_area(panel.input.within(reach), reach));

        blobs.push(panel.blob);
    }

    let placement = Placement {
        x: reach.x,
        y: reach.y,
        screen_width: screen.width,
        screen_height: screen.height,
    };

    let margin = Margin {
        top: reach.y as i32,
        right: 0,
        bottom: 0,
        left: reach.x as i32,
    };

    LayerWindow::new()
        .width(reach.width)
        .height(reach.height)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Left)
        .margin(margin)
        .layer(Layer::Top)
        .space(Zone::Respect)
        .keyboard(keyboard)
        .on_key(key_pressed)
        .input_region(areas)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .opacity(Curtain::read().items.value())
                .child(liquid::view(theme.background, blobs, placement)),
        )
}

// only one panel is out at a time, so only one of these acts on the key
fn key_pressed(key: Key) {
    launcher::key_pressed(key);
    utility::key_pressed(key);
}

// an input area is counted from the window's corner, not the screen's
fn input_area(region: Region, window: Region) -> InputArea {
    InputArea {
        x: (region.x - window.x) as i32,
        y: (region.y - window.y) as i32,
        width: region.width as i32,
        height: region.height as i32,
    }
}

/*
 * the 1 pixel still draws the liquid invisibly and one nearly clear pixel:
 * the first draw builds the shader and the vector renderer on the gpu, and
 * that took long enough to swallow the first panel's opening
 */
fn empty(screen: Region) -> LayerWindow {
    let placement = Placement {
        x: 0.0,
        y: 0.0,
        screen_width: screen.width,
        screen_height: screen.height,
    };

    LayerWindow::new()
        .width(1.0)
        .height(1.0)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Left)
        .layer(Layer::Top)
        .space(Zone::Respect)
        .click_through()
        .child(Stack::new(children![
            liquid::view(Color::TRANSPARENT, Vec::new(), placement),
            Rectangle::new().width(1.0).height(1.0).fill(Color::rgba(0, 0, 0, 1)),
        ]))
}
