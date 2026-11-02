use amane::{
    Bluetooth, BluetoothDevice, Center, Column, End, Padding, Pointer, Rectangle, Row, ScrollArea,
    Service, Stack, Start, Text, Weight, Widget, children,
};

use super::fade_target;
use super::header::{self, Header};
use super::hover;
use crate::fonts;
use crate::motion;
use crate::overlay::Overlay;
use crate::theme::{self, Theme};

const LIST: &str = "bluetooth-devices";

const GAP: f32 = 10.0;
const CARD_GAP: f32 = 7.0;

const CARD_HEIGHT: f32 = 54.0;
const CARD_PADDING: f32 = 9.0;

const ICON_SIZE: f32 = 34.0;
const FORGET_SIZE: f32 = 20.0;

const BLUETOOTH_ICON: &str = "󰂯";
const FORGET_ICON: &str = "󰅖";

// icons for the kinds of device bluez names, anything else gets the bluetooth icon
const DEVICE_ICONS: [(&str, &str); 6] = [
    ("audio", "󰋋"),
    ("input-mouse", "󰍽"),
    ("input-keyboard", "󰌌"),
    ("input-gaming", "󰊴"),
    ("phone", "󰏲"),
    ("computer", "󰍹"),
];

pub fn view(overlay: &Overlay, theme: &Theme, width: f32, height: f32) -> Column {
    let bluetooth = Bluetooth::read();

    let powered = bluetooth.powered();

    let status = if !bluetooth.available() {
        "No adapter"
    } else if bluetooth.scanning() {
        "Scanning for devices…"
    } else {
        "Ready"
    };

    let title = if powered {
        "Bluetooth"
    } else {
        "Bluetooth off"
    };

    let header = Header {
        name: "bluetooth",
        icon: BLUETOOTH_ICON,
        title,
        status,
        status_color: theme.muted_text,
        active: powered,
    };

    let header = header::view(overlay, theme, header, width, move || {
        Bluetooth::set_powered(!powered)
    });

    let list_height = height - header::HEIGHT - GAP;

    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(header)];

    if powered {
        parts.push(Box::new(list(
            overlay,
            theme,
            &bluetooth,
            width,
            list_height,
        )));
    }

    Column::new(parts).gap(GAP)
}

// paired devices first, then the ones a scan found
fn list(
    overlay: &Overlay,
    theme: &Theme,
    bluetooth: &Bluetooth,
    width: f32,
    height: f32,
) -> Rectangle {
    let area = Rectangle::new().width(width).height(height);

    if bluetooth.devices().is_empty() {
        let text = if bluetooth.scanning() {
            "Scanning…"
        } else {
            "No devices found"
        };

        let empty = Text::new(text)
            .size(12.0)
            .font(fonts::BODY)
            .color(theme.muted_text);

        return area.align_child(Center, Center).child(empty);
    }

    let mut paired = Vec::new();
    let mut found = Vec::new();

    for device in bluetooth.devices() {
        if device.paired() {
            paired.push(device);
        } else {
            found.push(device);
        }
    }

    paired.extend(found);

    let mut cards: Vec<Box<dyn Widget>> = Vec::new();

    for device in &paired {
        cards.push(Box::new(card(overlay, theme, device, width)));
    }

    let total = CARD_HEIGHT * cards.len() as f32 + CARD_GAP * (cards.len() - 1) as f32;

    let column = Column::new(cards).width(width).height(total).gap(CARD_GAP);

    area.child(ScrollArea::new(LIST, column))
}

// clicking connects or disconnects a paired device, and pairs a new one
fn card(overlay: &Overlay, theme: &Theme, device: &BluetoothDevice, width: f32) -> Stack {
    let connected = device.connected();

    let inner_width = width - CARD_PADDING * 2.0;

    let icon = Rectangle::new()
        .width(ICON_SIZE)
        .height(ICON_SIZE)
        .radius(8.0)
        .fill(theme.selected_surface)
        .align_child(Center, Center)
        .child(
            Text::new(device_icon(device.icon()))
                .size(16.0)
                .font(fonts::NERD)
                .tight()
                .color(theme.accent),
        );

    let (status, status_color) = if connected {
        ("Connected", theme.success)
    } else if device.paired() {
        ("Paired", theme.muted_text)
    } else {
        ("Available", theme.muted_text)
    };

    let weight = if connected {
        Weight::SemiBold
    } else {
        Weight::Regular
    };

    let name = Text::new(device.name())
        .size(12.0)
        .font(fonts::BODY)
        .weight(weight)
        .color(theme.text)
        .elide();

    let status = Text::new(status)
        .size(9.0)
        .font(fonts::BODY)
        .color(status_color);

    let battery = match device.battery() {
        Some(percent) => format!("{percent}%"),
        None => String::new(),
    };

    let battery = Text::new(battery)
        .size(9.0)
        .font(fonts::BODY)
        .color(theme.muted_text);

    // what is left beside the icon, the battery and the forget button
    let text_width = inner_width - ICON_SIZE - 32.0 - FORGET_SIZE - CARD_PADDING * 3.0;

    let name = Rectangle::new().width(text_width).height(18.0).child(name);

    let text = Rectangle::new()
        .width(text_width)
        .height(ICON_SIZE)
        .align_child(Start, Center)
        .child(Column::new(children![name, status]));

    let battery = Rectangle::new()
        .width(32.0)
        .height(ICON_SIZE)
        .align_child(End, Center)
        .child(battery);

    let fill = if connected {
        theme.selected_surface
    } else {
        theme.surface
    };

    let body = Rectangle::new()
        .width(width)
        .height(CARD_HEIGHT)
        .radius(12.0)
        .fill(fill)
        .padding(Padding {
            top: CARD_PADDING,
            right: CARD_PADDING,
            bottom: CARD_PADDING,
            left: CARD_PADDING,
        })
        .child(
            Row::new(children![icon, text, battery])
                .gap(CARD_PADDING)
                .align(Center),
        );

    let path = String::from(device.path());
    let paired = device.paired();

    let click = Rectangle::new()
        .width(width)
        .height(CARD_HEIGHT)
        .cursor(Pointer)
        .on_click(move |_| card_clicked(&path, paired, connected));

    let mut layers: Vec<Box<dyn Widget>> = vec![Box::new(body), Box::new(click)];

    // over the click area, so forgetting doesn't also connect
    if paired {
        layers.push(Box::new(forget_button(
            overlay,
            theme,
            device.path(),
            width,
        )));
    }

    Stack::new(layers).width(width).height(CARD_HEIGHT)
}

fn forget_button(overlay: &Overlay, theme: &Theme, device: &str, card_width: f32) -> Rectangle {
    let hover_name = format!("bluetooth:{device}:forget");

    let amount = motion::fade(&hover_name, fade_target(overlay, &hover_name));

    let glyph = Text::new(FORGET_ICON)
        .size(13.0)
        .font(fonts::NERD)
        .tight()
        .color(theme::mix(theme.muted_text, theme.danger, amount));

    let device = String::from(device);

    Rectangle::new()
        .width(FORGET_SIZE)
        .height(CARD_HEIGHT)
        .translate(card_width - CARD_PADDING - FORGET_SIZE, 0.0)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_click(move |_| Bluetooth::forget(&device))
        .child(glyph)
}

fn card_clicked(device: &str, paired: bool, connected: bool) {
    if !paired {
        Bluetooth::pair(device);
    } else if connected {
        Bluetooth::disconnect(device);
    } else {
        Bluetooth::connect(device);
    }
}

// bluez names kinds like "audio-headset" or "input-mouse"
fn device_icon(kind: &str) -> &'static str {
    for (prefix, icon) in DEVICE_ICONS {
        if kind.starts_with(prefix) {
            return icon;
        }
    }

    BLUETOOTH_ICON
}
