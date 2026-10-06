use amane::{Center, Pointer, Rectangle, Row, Text, Widget};

use super::{Page, hover, hovered, select};
use crate::fonts;
use crate::motion::{self, FAST_SPATIAL};
use crate::overlay::Overlay;
use crate::theme::Theme;

pub const HEIGHT: f32 = 38.0;

// the chosen tab is wider, and round at the ends
const WIDTH: f32 = 44.0;
const ACTIVE_WIDTH: f32 = 56.0;

// a connected group: tabs sit close, with small corners where they meet
const GAP: f32 = 2.0;
const INNER_RADIUS: f32 = 6.0;

struct Tab {
    page: Page,
    name: &'static str,
    icon: &'static str,
}

const TABS: [Tab; 3] = [
    Tab {
        page: Page::Notifications,
        name: "notifications",
        icon: "󰂚",
    },
    Tab {
        page: Page::Wifi,
        name: "wifi",
        icon: "󰖩",
    },
    Tab {
        page: Page::Bluetooth,
        name: "bluetooth",
        icon: "󰂯",
    },
];

pub fn view(overlay: &Overlay, theme: &Theme, width: f32) -> Row {
    let mut tabs: Vec<Box<dyn Widget>> = Vec::new();

    for (index, tab) in TABS.iter().enumerate() {
        tabs.push(Box::new(button(overlay, theme, tab, index)));
    }

    Row::new(tabs)
        .width(width)
        .height(HEIGHT)
        .gap(GAP)
        .justify(Center)
        .align(Center)
}

fn button(overlay: &Overlay, theme: &Theme, tab: &Tab, index: usize) -> Rectangle {
    let active = overlay.page == tab.page;

    let hover_name = format!("tab:{}", tab.name);

    let (target_width, target_radius) = if active {
        (ACTIVE_WIDTH, HEIGHT / 2.0)
    } else {
        (WIDTH, INNER_RADIUS)
    };

    let width = motion::follow(&format!("{hover_name}:width"), target_width, FAST_SPATIAL);
    let radius = motion::follow(&format!("{hover_name}:radius"), target_radius, FAST_SPATIAL);

    // the group's two ends stay fully round
    let left = if index == 0 { HEIGHT / 2.0 } else { radius };
    let right = if index == TABS.len() - 1 { HEIGHT / 2.0 } else { radius };

    let fill = if active {
        theme.accent
    } else if hovered(overlay, &hover_name) {
        theme.border
    } else {
        theme.surface
    };

    let icon_color = if active { theme.on_accent } else { theme.text };

    let page = tab.page;

    Rectangle::new()
        .width(width)
        .height(HEIGHT)
        .radius_top_left(left)
        .radius_bottom_left(left)
        .radius_top_right(right)
        .radius_bottom_right(right)
        .fill(fill)
        .cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_click(move |_| select(page))
        .align_child(Center, Center)
        .child(
            Text::new(tab.icon)
                .size(18.0)
                .font(fonts::NERD)
                .tight()
                .color(icon_color),
        )
}
