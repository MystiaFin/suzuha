mod formats;
mod job;

use std::path::PathBuf;

use amane::{
    Center, Color, Column, Key, Parent, Pointer, Rectangle, Row, ScrollArea, Service,
    SpaceBetween, Start, Text, Weight, Widget, Window, children,
};

use formats::{FORMATS, Format, Group};

use crate::fonts;
use crate::theme::{self, Theme};

// what open_window and close_window know this window by
const NAME: &str = "converter";

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 600.0;

const MARGIN: f32 = 22.0;

const CONVERTER_ICON: &str = "󰓡";
const DROP_ICON: &str = "󰉍";
const CLOSE_ICON: &str = "󰅖";

#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    Waiting,
    Converting,
    Done,
    Failed,
}

pub struct File {
    path: PathBuf,
    status: Status,
}

pub struct Queue {
    files: Vec<File>,

    format: &'static Format,

    // true while the job thread works through the waiting files
    running: bool,

    hovered: Option<String>,
}

impl Service for Queue {
    fn new() -> Self {
        Self {
            files: Vec::new(),
            format: &FORMATS[0],
            running: false,
            hovered: None,
        }
    }

    // it only changes through input and the job thread
    fn listen() {}
}

pub fn open() {
    amane::open_window(NAME, view);
}

// a conversion that is running carries on with the window closed
fn close() {
    amane::close_window(NAME);
}

fn add(paths: Vec<PathBuf>) {
    let mut queue = Queue::write();

    for path in paths {
        // folders and the like are left out, ffmpeg only reads files
        if !path.is_file() {
            continue;
        }

        queue.files.push(File { path, status: Status::Waiting });
    }
}

// finished and failed files go, so only what is left to do stays
fn clear() {
    let mut queue = Queue::write();

    queue.files.retain(|file| matches!(file.status, Status::Waiting | Status::Converting));

    // nothing is converting while stopped, so everything goes
    if !queue.running {
        queue.files.clear();
    }
}

// files that are done or failed go back in line for the new format
fn convert() {
    {
        let mut queue = Queue::write();

        for file in &mut queue.files {
            if matches!(file.status, Status::Done | Status::Failed) {
                file.status = Status::Waiting;
            }
        }
    }

    job::start();
}

fn hover(name: String, inside: bool) {
    let mut queue = Queue::write();

    if inside {
        queue.hovered = Some(name);
    } else if queue.hovered.as_ref() == Some(&name) {
        queue.hovered = None;
    }
}

fn hovered(name: &str) -> bool {
    Queue::read().hovered.as_deref() == Some(name)
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

    let inner_width = width - MARGIN * 2.0;

    // what is left once the header, formats and footer have their room
    let drop_height = (height - MARGIN * 2.0 - 40.0 - 3.0 * 34.0 - 40.0 - 16.0 * 6.0).max(120.0);

    let content = Column::new(children![
        header(&theme, inner_width),
        drop_zone(&theme, inner_width, drop_height),
        format_row(&theme, Group::Image, "Image"),
        format_row(&theme, Group::Video, "Video"),
        format_row(&theme, Group::Audio, "Audio"),
        footer(&theme, inner_width),
    ])
    .gap(16.0);

    let frame = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(theme.background)
        .padding(MARGIN)
        .child(content);

    Window::new()
        .title("Converter")
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
        .child(Text::new(CONVERTER_ICON).size(21.0).font(fonts::NERD).tight().color(theme.accent));

    let title = Text::new("Converter")
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

// takes files dropped from a file manager, and lists them once there are some
fn drop_zone(theme: &Theme, width: f32, height: f32) -> Rectangle {
    let queue = Queue::read();

    let zone = Rectangle::new()
        .width(width)
        .height(height)
        .radius(16.0)
        .fill(theme.surface)
        .border(1.0, theme.border)
        .on_drop(add);

    if queue.files.is_empty() {
        let hint = Column::new(children![
            Text::new(DROP_ICON).size(40.0).font(fonts::NERD).tight().color(theme.muted_text),
            Text::new("Drop files here").size(14.0).font(fonts::BODY).color(theme.secondary_text),
        ])
        .gap(10.0)
        .align(Center);

        return zone.align_child(Center, Center).child(hint);
    }

    let row_width = width - 24.0;

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for file in &queue.files {
        rows.push(Box::new(file_row(theme, file, row_width)));
    }

    let list = ScrollArea::new("converter_files", Column::new(rows).gap(4.0))
        .width(row_width)
        .height(height - 24.0);

    zone.padding(12.0).child(list)
}

fn file_row(theme: &Theme, file: &File, width: f32) -> Row {
    let (label, color) = match file.status {
        Status::Waiting => ("Waiting", theme.muted_text),
        Status::Converting => ("Converting", theme.accent),
        Status::Done => ("Done", theme.success),
        Status::Failed => ("Failed", theme.danger),
    };

    let name = file.path.file_name().unwrap_or_default().to_string_lossy();

    let name = Rectangle::new()
        .width(width - 110.0)
        .height(28.0)
        .align_child(Start, Center)
        .child(Text::new(name).size(13.0).font(fonts::BODY).color(theme.text).elide());

    let status = Text::new(label).size(12.0).font(fonts::BODY).weight(Weight::SemiBold).color(color);

    Row::new(children![name, status])
        .width(width)
        .justify(SpaceBetween)
        .align(Center)
}

fn format_row(theme: &Theme, group: Group, label: &str) -> Row {
    let chosen = Queue::read().format.extension;

    let mut chips: Vec<Box<dyn Widget>> = Vec::new();

    for format in FORMATS {
        if format.group != group {
            continue;
        }

        chips.push(Box::new(chip(theme, format, format.extension == chosen)));
    }

    let label = Rectangle::new()
        .width(64.0)
        .height(34.0)
        .align_child(Start, Center)
        .child(Text::new(label).size(12.0).font(fonts::BODY).color(theme.muted_text));

    Row::new(children![label, Row::new(chips).gap(8.0)]).align(Center)
}

fn chip(theme: &Theme, format: &'static Format, chosen: bool) -> Rectangle {
    let hover_name = format!("format:{}", format.extension);

    let (fill, text) = if chosen {
        (theme.accent, theme.on_accent)
    } else if hovered(&hover_name) {
        (theme.hover_surface, theme.text)
    } else {
        (theme.surface, theme.secondary_text)
    };

    let width = format.extension.len() as f32 * 9.0 + 28.0;

    Rectangle::new()
        .width(width)
        .height(34.0)
        .radius(10.0)
        .fill(fill)
        .cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_click(move |_| Queue::write().format = format)
        .align_child(Center, Center)
        .child(
            Text::new(format.extension.to_uppercase())
                .size(12.0)
                .font(fonts::BODY)
                .weight(Weight::SemiBold)
                .color(text),
        )
}

// how far the list is, beside clear and convert
fn footer(theme: &Theme, width: f32) -> Row {
    let queue = Queue::read();

    let total = queue.files.len();

    let mut finished = 0;
    let mut failed = 0;

    for file in &queue.files {
        match file.status {
            Status::Done => finished += 1,
            Status::Failed => failed += 1,
            _ => {}
        }
    }

    let running = queue.running;

    drop(queue);

    let mut summary = format!("{finished} of {total} converted");

    if failed > 0 {
        summary.push_str(&format!(", {failed} failed"));
    }

    let summary = Text::new(summary).size(12.0).font(fonts::BODY).color(theme.muted_text);

    let convert_label = if running { "Converting" } else { "Convert" };

    let buttons = Row::new(children![
        button(theme, "Clear", false, total > 0, clear),
        button(theme, convert_label, true, total > 0 && !running, convert),
    ])
    .gap(10.0);

    Row::new(children![summary, buttons])
        .width(width)
        .height(40.0)
        .justify(SpaceBetween)
        .align(Center)
}

fn button(theme: &Theme, label: &'static str, primary: bool, enabled: bool, on_click: fn()) -> Rectangle {
    let hover_name = format!("button:{label}");

    let (fill, text) = if primary {
        (theme.accent, theme.on_accent)
    } else {
        (theme.surface, theme.text)
    };

    let fill = if enabled && hovered(&hover_name) {
        theme::mix(fill, text, 0.08)
    } else {
        fill
    };

    let button = Rectangle::new()
        .width(label.len() as f32 * 7.6 + 32.0)
        .height(40.0)
        .radius(12.0)
        .fill(fill)
        .align_child(Center, Center)
        .child(Text::new(label).size(13.0).font(fonts::BODY).weight(Weight::SemiBold).color(text));

    if !enabled {
        return button.opacity(0.42);
    }

    button
        .cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_click(move |_| on_click())
}

pub fn ipc(_arguments: &[String]) -> String {
    open();

    String::from("ok")
}
