use std::cell::RefCell;

use amane::{
    Battery, Center, Color, End, Memory, Padding, Parent, Pointer, Rectangle, Row, Service,
    SpaceBetween, Stack, Text, Widget, children,
};

use super::{pill, ring};
use crate::fonts;
use crate::motion::{self, Glide};
use crate::overlay::Overlay;
use crate::recorder;
use crate::settings::Settings;
use crate::theme::Theme;

// the memory icon, and the notification, wifi and bluetooth tray icons
const MEMORY_ICON: &str = "󰍛";
const TRAY_ICONS: [&str; 3] = ["󰂚", "󰖩", "󰂯"];

const RING_SIZE: f32 = 24.0;
const RING_THICKNESS: f32 = 3.0;

// 50px of icons with 10px of padding on each side
const TRAY_WIDTH: f32 = 70.0;
const TRAY_PADDING: f32 = 10.0;

// with the row's 10px gap this keeps 15px from the screen edge
const EDGE: f32 = 5.0;

const RECORDING_DOT: &str = "\u{f0765}";

// milliseconds the tray takes to grow and shrink around the recording time
const RECORDING_DURATION: u64 = 420;

thread_local! {
    // kept outside services, a service write from view() would draw frames forever
    static RECORDING: RefCell<Option<(Glide, String)>> = const { RefCell::new(None) };
}

pub fn view(theme: &Theme, width: f32) -> Row {
    let settings = Settings::read();

    let mut items: Vec<Box<dyn Widget>> = Vec::new();

    if settings.flag("bar_battery") {
        if let Some(battery) = battery(theme) {
            items.push(Box::new(battery));
        }
    }

    if settings.flag("bar_memory") {
        items.push(Box::new(memory(theme)));
    }

    if settings.flag("bar_tray") {
        items.push(Box::new(tray(theme)));
    }

    items.push(Box::new(Rectangle::new().width(EDGE).height(1.0)));

    Row::new(items)
        .width(width)
        .height(Parent)
        .gap(10.0)
        .justify(End)
        .align(Center)
}

// a ring with a small icon inside, then the value
fn indicator(value: f32, color: Color, icon: &str, text: &str, theme: &Theme) -> Row {
    let ring = ring::view(RING_SIZE, RING_THICKNESS, value, color, theme.border);

    let icon = Rectangle::new()
        .width(RING_SIZE)
        .height(RING_SIZE)
        .align_child(Center, Center)
        .child(Text::new(icon).size(10.0).font(fonts::MATERIAL).color(theme.text));

    let ring_with_icon = Stack::new(children![ring, icon]);

    Row::new(children![ring_with_icon, pill::label(text, 14.0, theme.text)])
        .gap(6.0)
        .align(Center)
}

// none on a desktop without a battery
fn battery(theme: &Theme) -> Option<Row> {
    let battery = Battery::read();

    if !battery.present() {
        return None;
    }

    let percent = battery.percent();

    let color = if battery.charging() {
        theme.success
    } else if percent > 20 {
        theme.accent
    } else {
        theme.danger
    };

    let icon = battery_icon(percent, battery.charging());

    let text = format!("{percent}%");

    let value = f32::from(percent) / 100.0;

    Some(indicator(value, color, icon, &text, theme))
}

// one glyph per 10%
fn battery_icon(percent: u8, charging: bool) -> &'static str {
    if charging {
        return "󰚥";
    }

    match percent {
        91.. => "󰁹",
        81..=90 => "󰂂",
        71..=80 => "󰂁",
        61..=70 => "󰂀",
        51..=60 => "󰁿",
        41..=50 => "󰁾",
        31..=40 => "󰁽",
        21..=30 => "󰁼",
        11..=20 => "󰁻",
        _ => "󰁺",
    }
}

fn memory(theme: &Theme) -> Row {
    let memory = Memory::read();

    let usage = f32::from(memory.percent()) / 100.0;

    let color = if usage < 0.5 {
        theme.success
    } else if usage < 0.8 {
        theme.accent
    } else {
        theme.danger
    };

    // the used amount, like "5.2G"
    let gibibytes = memory.used_kib() as f32 / 1024.0 / 1024.0;

    let text = format!("{gibibytes:.1}G");

    indicator(usage, color, MEMORY_ICON, &text, theme)
}

// opens the utility center, and grows to the left with the time while a recording runs
fn tray(theme: &Theme) -> Rectangle {
    let mut icons: Vec<Box<dyn Widget>> = Vec::new();

    for icon in TRAY_ICONS {
        let text = Text::new(icon).size(15.0).font(fonts::NERD).color(theme.on_accent);

        icons.push(Box::new(text));
    }

    // spread across the inside, so the spacing matches however wide amane measures each glyph
    let row = Row::new(icons)
        .width(TRAY_WIDTH - TRAY_PADDING * 2.0)
        .justify(SpaceBetween)
        .align(Center);

    let (extra, recording) = recording(theme);

    let content = Row::new(children![recording, row]).align(Center);

    pill::view(TRAY_WIDTH + extra, theme.accent)
        .clip()
        .cursor(Pointer)
        .on_click(|_| Overlay::toggle_utility())
        .align_child(End, Center)
        .padding(Padding {
            top: 0.0,
            right: TRAY_PADDING,
            bottom: 0.0,
            left: 0.0,
        })
        .child(content)
}

/*
 * the dot and running time in front of the tray icons, and how much wider it makes the pill;
 * the width glides, and the last time stays while it closes
 */
fn recording(theme: &Theme) -> (f32, Rectangle) {
    let status = recorder::status();

    RECORDING.with_borrow_mut(|shown| {
        let (width, text) =
            shown.get_or_insert_with(|| (motion::spatial(0.0, RECORDING_DURATION), String::new()));

        if let Some(time) = status.as_ref() {
            text.clone_from(time);
        }

        // 10px of padding, the dot, a 6px gap, the time, then 8px before the icons
        let full = 10.0 + 8.0 + 6.0 + pill::text_width(text, 13.0) + 8.0;

        width.to(if status.is_some() { full } else { 0.0 });

        let extra = width.value();

        let dot = Text::new(RECORDING_DOT).size(8.0).font(fonts::NERD).color(theme.on_accent);

        let segment = Rectangle::new()
            .width(extra)
            .height(Parent)
            .opacity(extra / full)
            .align_child(Center, Center)
            .child(
                Row::new(children![dot, pill::label(text, 13.0, theme.on_accent)])
                    .gap(6.0)
                    .align(Center),
            );

        (extra, segment)
    })
}
