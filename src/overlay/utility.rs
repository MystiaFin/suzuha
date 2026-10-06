mod bluetooth;
mod brightness;
mod calendar;
mod header;
pub mod notifications;
mod record;
mod switch;
mod tabs;
mod wifi;

use amane::{Bluetooth, Column, Key, Network, Padding, Rectangle, Service, Stack, Widget, children};

use super::{Overlay, PanelView, Region};
use crate::liquid::{self, Blob};
use crate::motion::{self, DEFAULT_SPATIAL};
use crate::theme::Theme;

const WIDTH: f32 = 360.0;
const MAX_HEIGHT: f32 = 800.0;
const RADIUS: f32 = 30.0;

// the panel reaches this far past the screen's right edge, so it stays melted into it
const EDGE_OVERLAP: f32 = 40.0;

// a little above the window's top, so it melts into the bar's edge too
const TOP: f32 = -10.0;

// how far past its own width it slides to let go of the edge
const HIDDEN_MARGIN: f32 = 32.0;

const PADDING: Padding = Padding {
    top: 28.0,
    right: EDGE_OVERLAP + 16.0,
    bottom: 16.0,
    left: 16.0,
};

const INNER_WIDTH: f32 = WIDTH - PADDING.left - PADDING.right;

const GAP: f32 = 12.0;

// the pages side by side, in the order of the tabs
#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Notifications,
    Wifi,
    Bluetooth,
    Record,
}

impl Page {
    fn index(self) -> f32 {
        match self {
            Page::Notifications => 0.0,
            Page::Wifi => 1.0,
            Page::Bluetooth => 2.0,
            Page::Record => 3.0,
        }
    }
}

// none while fully hidden
pub fn view(overlay: &Overlay, theme: &Theme, screen: Region) -> Option<PanelView> {
    let progress = overlay.utility.progress.value();

    if progress <= 0.001 {
        return None;
    }

    let height = MAX_HEIGHT.min(screen.height - 96.0);

    let hidden = WIDTH + HIDDEN_MARGIN;

    let x = screen.width - WIDTH + EDGE_OVERLAP + hidden * (1.0 - progress);

    let blob = Blob::new(x, TOP, WIDTH, height)
        .radius(RADIUS)
        .child(content(overlay, theme, height));

    let input = Region {
        x,
        y: TOP,
        width: WIDTH,
        height,
    };

    // fully out it starts at its resting edge, and its melted corners spread a little past that
    let reach_width = WIDTH - EDGE_OVERLAP + liquid::CONNECTION;

    let reach = Region {
        x: screen.width - reach_width,
        y: 0.0,
        width: reach_width,
        height: TOP + height + liquid::CONNECTION,
    };

    Some(PanelView { blob, input, reach })
}

fn content(overlay: &Overlay, theme: &Theme, height: f32) -> Rectangle {
    let fixed = tabs::HEIGHT + brightness::HEIGHT + calendar::HEIGHT + GAP * 3.0;

    let pages_height = height - PADDING.top - PADDING.bottom - fixed;

    let column = Column::new(children![
        tabs::view(overlay, theme, INNER_WIDTH),
        brightness::view(overlay, theme, INNER_WIDTH),
        pages(overlay, theme, pages_height),
        calendar::view(overlay, theme, INNER_WIDTH),
    ])
    .gap(GAP);

    Rectangle::new()
        .width(WIDTH)
        .height(height)
        .padding(PADDING)
        .on_hover(Overlay::hover_utility)
        .child(column)
}

// every page is a full width apart, and they all slide together to the chosen one
fn pages(overlay: &Overlay, theme: &Theme, height: f32) -> Rectangle {
    let shown = motion::follow("utility-page", overlay.page.index(), DEFAULT_SPATIAL);

    let mut layers: Vec<Box<dyn Widget>> = Vec::new();

    for page in [Page::Notifications, Page::Wifi, Page::Bluetooth, Page::Record] {
        let offset = page.index() - shown;

        // only pages at least partly in view are built
        if offset.abs() >= 1.0 {
            continue;
        }

        let content = match page {
            Page::Notifications => notifications::view(overlay, theme, INNER_WIDTH, height),
            Page::Wifi => wifi::view(overlay, theme, INNER_WIDTH, height),
            Page::Bluetooth => bluetooth::view(overlay, theme, INNER_WIDTH, height),
            Page::Record => record::view(overlay, theme, INNER_WIDTH, height),
        };

        let slot = Rectangle::new()
            .width(INNER_WIDTH)
            .height(height)
            .translate(offset * INNER_WIDTH, 0.0)
            .child(content);

        layers.push(Box::new(slot));
    }

    let viewport = Rectangle::new()
        .width(INNER_WIDTH)
        .height(height)
        .child(Stack::new(layers));

    // a clip is a gpu layer, so only while a page slides past the edge
    if shown != overlay.page.index() {
        return viewport.clip();
    }

    viewport
}

// what opening the panel or turning to a page starts, like a fresh wifi scan
pub fn opened(overlay: &mut Overlay) {
    close_password(overlay);

    match overlay.page {
        Page::Wifi => Network::scan(),
        Page::Bluetooth => Bluetooth::start_scan(),
        Page::Notifications | Page::Record => {}
    }
}

// every way the panel closes goes through here, so a bluetooth scan never outlives it
pub fn close(overlay: &mut Overlay) {
    if overlay.utility.shown && overlay.page == Page::Bluetooth {
        Bluetooth::stop_scan();
    }

    overlay.utility.hide();
}

// "toggle", "show" or "hide", then optionally the page to show
pub fn ipc(arguments: &[String]) -> String {
    let command = arguments.first().map(String::as_str).unwrap_or("toggle");

    let page = match arguments.get(1).map(String::as_str) {
        Some("notifications") => Some(Page::Notifications),
        Some("wifi") => Some(Page::Wifi),
        Some("bluetooth") => Some(Page::Bluetooth),
        Some("record") => Some(Page::Record),
        _ => None,
    };

    if let Some(page) = page {
        Overlay::write().page = page;
    }

    let shown = Overlay::read().utility.shown;

    let wanted = match command {
        "show" => true,
        "hide" => false,
        _ => !shown,
    };

    if wanted != shown {
        Overlay::toggle_utility();
    }

    String::from("ok")
}

pub fn select(page: Page) {
    let mut overlay = Overlay::write();

    if overlay.page == page {
        return;
    }

    if overlay.page == Page::Bluetooth {
        Bluetooth::stop_scan();
    }

    overlay.page = page;

    opened(&mut overlay);
}

// a password field takes every key, so it can be typed into without a click first
pub fn wants_keyboard(overlay: &Overlay) -> bool {
    overlay.utility.shown && overlay.password_for.is_some()
}

// escape closes an open password field first, then the panel
pub fn key_pressed(key: Key) {
    let mut overlay = Overlay::write();

    if !overlay.utility.shown || key != Key::Escape {
        return;
    }

    if overlay.password_for.is_some() {
        close_password(&mut overlay);

        return;
    }

    close(&mut overlay);
}

pub fn close_password(overlay: &mut Overlay) {
    overlay.password_for = None;
    overlay.show_password = false;

    wifi::clear_password();
}

// remembers which control the pointer is on, for hover colors
pub fn hover(name: String, inside: bool) {
    let mut overlay = Overlay::write();

    if inside {
        overlay.hovered = Some(name);
    } else if overlay.hovered.as_ref() == Some(&name) {
        overlay.hovered = None;
    }
}

pub fn hovered(overlay: &Overlay, name: &str) -> bool {
    overlay.hovered.as_deref() == Some(name)
}

// where a hover color fades to: 1 while the pointer is on the control
pub fn fade_target(overlay: &Overlay, name: &str) -> f32 {
    if hovered(overlay, name) { 1.0 } else { 0.0 }
}
