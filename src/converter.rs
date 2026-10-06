mod compress;
mod documents;
mod formats;
mod job;

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use amane::{
    Center, Color, Column, Full, Key, Parent, Pointer, Rectangle, Row, ScrollArea, Service,
    SpaceBetween, Stack, Start, Text, Weight, Widget, Window, children,
};

use compress::{LEVELS, Level, SPEEDS, Speed};
use documents::{DOCUMENTS, Document, Kind};
use formats::{FORMATS, Format, Group};

use crate::fonts;
use crate::motion;
use crate::overlay::control_center::wave;
use crate::theme::{self, Theme};

// what open_window and close_window know this window by
const NAME: &str = "converter";

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 600.0;

const MARGIN: f32 = 22.0;

const TABS_HEIGHT: f32 = 48.0;
const CONTROL_HEIGHT: f32 = 34.0;
const CONTROL_GAP: f32 = 12.0;

// connected toggles sit close, with small corners where they meet
const TOGGLE_GAP: f32 = 2.0;
const TOGGLE_INNER_RADIUS: f32 = 6.0;
const GAP: f32 = 16.0;

const PROGRESS_WIDTH: f32 = 64.0;

// how long the bar takes to fill once when there is no real progress to show
const SWEEP_MS: u128 = 1500;

const CONVERTER_ICON: &str = "󰓡";
const COMPRESS_ICON: &str = "󰛀";
const DOCUMENTS_ICON: &str = "󰈙";
const DROP_ICON: &str = "󰉍";
const CLOSE_ICON: &str = "󰅖";

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Convert,
    Compress,
    Documents,
}

const TABS: [(Page, &str, &str); 3] = [
    (Page::Convert, CONVERTER_ICON, "Convert"),
    (Page::Compress, COMPRESS_ICON, "Compress"),
    (Page::Documents, DOCUMENTS_ICON, "Documents"),
];

// what the job thread does to every waiting file, fixed when it starts
#[derive(Clone, Copy)]
pub enum Task {
    Convert(&'static Format),
    Compress(Level, Speed),
    Document(&'static Document),
}

struct Words {
    action: &'static str,
    working: &'static str,
    finished: &'static str,
}

const CONVERT_WORDS: Words = Words { action: "Convert", working: "Converting", finished: "converted" };
const COMPRESS_WORDS: Words = Words { action: "Compress", working: "Compressing", finished: "compressed" };

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

    // from 0 to 1 while converting, none when ffmpeg can't tell how long it is
    progress: Option<f32>,
}

pub struct Queue {
    files: Vec<File>,

    page: Page,

    format: &'static Format,
    level: Level,
    speed: Speed,
    document: &'static Document,

    task: Task,

    // true while the job thread works through the waiting files
    running: bool,

    hovered: Option<String>,
}

impl Service for Queue {
    fn new() -> Self {
        Self {
            files: Vec::new(),
            page: Page::Convert,
            format: &FORMATS[0],
            level: Level::Medium,
            speed: Speed::Medium,
            document: &DOCUMENTS[0],
            task: Task::Convert(&FORMATS[0]),
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

        queue.files.push(File { path, status: Status::Waiting, progress: None });
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

// files that are done or failed go back in line for what the open page sets up
fn start() {
    {
        let mut queue = Queue::write();

        queue.task = match queue.page {
            Page::Convert => Task::Convert(queue.format),
            Page::Compress => Task::Compress(queue.level, queue.speed),
            Page::Documents => Task::Document(queue.document),
        };

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

    let page = Queue::read().page;

    let controls = match page {
        Page::Convert => Column::new(children![
            format_row(&theme, Group::Image, "Image"),
            format_row(&theme, Group::Video, "Video"),
            format_row(&theme, Group::Audio, "Audio"),
        ]),

        Page::Compress => Column::new(children![level_row(&theme), speed_row(&theme)]),

        Page::Documents => Column::new(children![
            document_row(&theme, Kind::Text, "Text"),
            document_row(&theme, Kind::Sheet, "Sheet"),
            document_row(&theme, Kind::Slides, "Slides"),
        ]),
    };

    let control_rows = match page {
        Page::Convert | Page::Documents => 3.0,
        Page::Compress => 2.0,
    };

    let controls_height = control_rows * CONTROL_HEIGHT + (control_rows - 1.0) * CONTROL_GAP;

    // what is left once the header, tabs, controls and footer have their room
    let taken = MARGIN * 2.0 + 40.0 + TABS_HEIGHT + controls_height + 40.0 + GAP * 4.0;

    let drop_height = (height - taken).max(120.0);

    let content = Column::new(children![
        header(&theme, inner_width),
        tabs(&theme, inner_width),
        drop_zone(&theme, inner_width, drop_height),
        controls.gap(CONTROL_GAP),
        footer(&theme, inner_width),
    ])
    .gap(GAP);

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

// a pill that slides under the open page's label
fn tabs(theme: &Theme, width: f32) -> Stack {
    let open = Queue::read().page;

    let tab_width = (width - 8.0) / TABS.len() as f32;
    let tab_height = TABS_HEIGHT - 8.0;

    let mut open_index = 0;
    let mut labels: Vec<Box<dyn Widget>> = Vec::new();

    for (index, (page, icon, label)) in TABS.into_iter().enumerate() {
        let selected = page == open;

        if selected {
            open_index = index;
        }

        let hover_name = format!("tab:{label}");

        let amount = motion::fade(&format!("converter-{hover_name}"), if selected { 1.0 } else { 0.0 });

        let color = theme::mix(theme.secondary_text, theme.on_accent, amount);

        let fill = if !selected && hovered(&hover_name) {
            theme.hover_surface
        } else {
            Color::TRANSPARENT
        };

        let content = Row::new(children![
            Text::new(icon).size(16.0).font(fonts::NERD).tight().color(color),
            Text::new(label).size(14.0).font(fonts::BODY).weight(Weight::SemiBold).color(color),
        ])
        .gap(8.0)
        .align(Center);

        labels.push(Box::new(
            Rectangle::new()
                .width(tab_width)
                .height(tab_height)
                .radius(Full)
                .fill(fill)
                .cursor(Pointer)
                .on_hover(move |inside| hover(hover_name.clone(), inside))
                .on_click(move |_| Queue::write().page = page)
                .align_child(Center, Center)
                .child(content),
        ));
    }

    let x = motion::follow("converter-tab", open_index as f32 * tab_width, motion::DEFAULT_SPATIAL);

    let track = Rectangle::new().width(width).height(TABS_HEIGHT).radius(Full).fill(theme.surface);

    let indicator = Rectangle::new()
        .width(tab_width)
        .height(tab_height)
        .radius(Full)
        .fill(theme.accent)
        .translate(4.0 + x, 4.0);

    let labels = Rectangle::new()
        .width(width)
        .height(TABS_HEIGHT)
        .padding(4.0)
        .child(Row::new(labels));

    Stack::new(children![track, indicator, labels]).width(width).height(TABS_HEIGHT)
}

// takes files dropped from a file manager, and lists them once there are some
fn drop_zone(theme: &Theme, width: f32, height: f32) -> Rectangle {
    let queue = Queue::read();

    let zone = Rectangle::new()
        .width(width)
        .height(height)
        .radius(28.0)
        .fill(theme.surface)
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
    let name = file.path.file_name().unwrap_or_default().to_string_lossy();

    let name = Rectangle::new()
        .width(width - 110.0)
        .height(28.0)
        .align_child(Start, Center)
        .child(Text::new(name).size(13.0).font(fonts::BODY).color(theme.text).elide());

    let (label, color) = match file.status {
        Status::Waiting => ("Waiting", theme.muted_text),
        Status::Done => ("Done", theme.success),
        Status::Failed => ("Failed", theme.danger),
        Status::Converting => {
            return Row::new(children![name, progress(theme, file.progress)])
                .width(width)
                .justify(SpaceBetween)
                .align(Center);
        }
    };

    let status = Text::new(label).size(12.0).font(fonts::BODY).weight(Weight::SemiBold).color(color);

    Row::new(children![name, status])
        .width(width)
        .justify(SpaceBetween)
        .align(Center)
}

/*
 * a wavy bar and how far along, or a bar that keeps filling
 * when the length is unknown, like a document or a still image
 */
fn progress(theme: &Theme, progress: Option<f32>) -> Row {
    let Some(progress) = progress else {
        amane::request_frame();

        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|since| since.as_millis())
            .unwrap_or(0);

        let sweep = (millis % SWEEP_MS) as f32 / SWEEP_MS as f32;

        return Row::new(children![wave::view(PROGRESS_WIDTH, sweep, true, theme.accent, theme.border)]);
    };

    let percent = Text::new(format!("{:.0}%", progress * 100.0))
        .size(12.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.accent);

    Row::new(children![wave::view(PROGRESS_WIDTH, progress, true, theme.accent, theme.border), percent])
        .gap(8.0)
        .align(Center)
}

fn format_row(theme: &Theme, group: Group, label: &str) -> Row {
    let chosen = Queue::read().format.extension;

    let formats: Vec<&'static Format> = FORMATS.iter().filter(|format| format.group == group).collect();

    let mut chips: Vec<Box<dyn Widget>> = Vec::new();

    for (index, format) in formats.iter().copied().enumerate() {
        let name = format!("format:{}", format.extension);

        let selected = format.extension == chosen;

        let ends = (index == 0, index == formats.len() - 1);

        chips.push(Box::new(toggle(theme, name, &format.extension.to_uppercase(), selected, ends, move || {
            Queue::write().format = format
        })));
    }

    option_row(theme, label, chips)
}

fn document_row(theme: &Theme, kind: Kind, label: &str) -> Row {
    let chosen = Queue::read().document.extension;

    let documents: Vec<&'static Document> = DOCUMENTS.iter().filter(|document| document.kind == kind).collect();

    let mut chips: Vec<Box<dyn Widget>> = Vec::new();

    for (index, document) in documents.iter().copied().enumerate() {
        let name = format!("document:{}", document.extension);

        let selected = document.extension == chosen;

        let ends = (index == 0, index == documents.len() - 1);

        chips.push(Box::new(toggle(theme, name, &document.extension.to_uppercase(), selected, ends, move || {
            Queue::write().document = document
        })));
    }

    option_row(theme, label, chips)
}

fn level_row(theme: &Theme) -> Row {
    let chosen = Queue::read().level;

    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    for (index, (level, label)) in LEVELS.into_iter().enumerate() {
        let name = format!("level:{label}");

        let ends = (index == 0, index == LEVELS.len() - 1);

        buttons.push(Box::new(toggle(theme, name, label, level == chosen, ends, move || Queue::write().level = level)));
    }

    option_row(theme, "Strength", buttons)
}

fn speed_row(theme: &Theme) -> Row {
    let chosen = Queue::read().speed;

    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    for (index, (speed, label)) in SPEEDS.into_iter().enumerate() {
        let name = format!("speed:{label}");

        let ends = (index == 0, index == SPEEDS.len() - 1);

        buttons.push(Box::new(toggle(theme, name, label, speed == chosen, ends, move || Queue::write().speed = speed)));
    }

    option_row(theme, "Speed", buttons)
}

fn option_row(theme: &Theme, label: &str, buttons: Vec<Box<dyn Widget>>) -> Row {
    let label = Rectangle::new()
        .width(64.0)
        .height(CONTROL_HEIGHT)
        .align_child(Start, Center)
        .child(Text::new(label).size(12.0).font(fonts::BODY).color(theme.muted_text));

    Row::new(children![label, Row::new(buttons).gap(TOGGLE_GAP)]).align(Center)
}

/*
 * one of a connected group: the group's two ends are fully round, the
 * corners between buttons small, and a picked one morphs into a filled pill
 */
fn toggle(
    theme: &Theme,
    name: String,
    label: &str,
    selected: bool,
    (first, last): (bool, bool),
    on_click: impl Fn() + 'static,
) -> Rectangle {
    let amount = motion::fade(&format!("converter-{name}"), if selected { 1.0 } else { 0.0 });

    let resting = if hovered(&name) {
        theme.hover_surface
    } else {
        theme.surface
    };

    let fill = theme::mix(resting, theme.accent, amount);
    let text = theme::mix(theme.secondary_text, theme.on_accent, amount);

    let inner = TOGGLE_INNER_RADIUS + (CONTROL_HEIGHT / 2.0 - TOGGLE_INNER_RADIUS) * amount;

    let left = if first { CONTROL_HEIGHT / 2.0 } else { inner };
    let right = if last { CONTROL_HEIGHT / 2.0 } else { inner };

    let width = label.len() as f32 * 8.0 + 32.0;

    Rectangle::new()
        .width(width)
        .height(CONTROL_HEIGHT)
        .radius_top_left(left)
        .radius_bottom_left(left)
        .radius_top_right(right)
        .radius_bottom_right(right)
        .fill(fill)
        .cursor(Pointer)
        .on_hover(move |inside| hover(name.clone(), inside))
        .on_click(move |_| on_click())
        .align_child(Center, Center)
        .child(Text::new(label).size(12.0).font(fonts::BODY).weight(Weight::SemiBold).color(text))
}

fn words(task: Task) -> Words {
    match task {
        Task::Convert(_) | Task::Document(_) => CONVERT_WORDS,
        Task::Compress(..) => COMPRESS_WORDS,
    }
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

    // a running job keeps its own words, the open page only says what comes next
    let words = if running {
        words(queue.task)
    } else {
        match queue.page {
            Page::Convert | Page::Documents => CONVERT_WORDS,
            Page::Compress => COMPRESS_WORDS,
        }
    };

    drop(queue);

    let mut summary = format!("{finished} of {total} {}", words.finished);

    if failed > 0 {
        summary.push_str(&format!(", {failed} failed"));
    }

    let summary = Text::new(summary).size(12.0).font(fonts::BODY).color(theme.muted_text);

    let start_label = if running { words.working } else { words.action };

    let buttons = Row::new(children![
        button(theme, "Clear", false, total > 0, clear),
        button(theme, start_label, true, total > 0 && !running, start),
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
        .radius(Full)
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
