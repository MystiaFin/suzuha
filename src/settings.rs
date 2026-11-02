mod button;
mod choice;
mod dialog;
mod field;
mod navigation;
mod page;
mod pages;
mod row;
mod slider;
mod store;
mod switch;

use amane::{
    Center, Color, Column, End, Key, Parent, Pointer, Rectangle, Row, ScrollArea, Service,
    SpaceBetween, Stack, Text, TextInput, Weight, Widget, Window, children,
};

pub use store::Settings;

use crate::fonts;
use crate::profile::Profile;
use crate::theme::{self, Theme};

// what open_window and close_window know this window by
const NAME: &str = "settings";

const WIDTH: f32 = 980.0;
const HEIGHT: f32 = 680.0;

const MARGIN: f32 = 22.0;

const HEADER_HEIGHT: f32 = 52.0;
const HEADER_GAP: f32 = 16.0;

const SECTION_HEIGHT: f32 = 46.0;
const SECTION_GAP: f32 = 16.0;

const FOOTER_HEIGHT: f32 = 50.0;
const FOOTER_GAP: f32 = 10.0;

// the room each part gets in a window of the size the compositor gave
struct Layout {
    width: f32,

    // below the header, beside the list of pages
    body_height: f32,

    content_width: f32,

    // what is left for the page itself, longer pages scroll inside it
    scroll_height: f32,
}

impl Layout {
    // a tiling compositor can give another size than asked for, the opening size counts until it says
    fn measure() -> Self {
        let (width, height) = match amane::window_size() {
            (0.0, _) | (_, 0.0) => (WIDTH, HEIGHT),
            size => size,
        };

        let body_height = height - MARGIN * 2.0 - HEADER_HEIGHT - HEADER_GAP;

        Self {
            width,
            body_height,
            content_width: width - MARGIN * 3.0 - navigation::WIDTH,
            scroll_height: body_height - SECTION_HEIGHT - SECTION_GAP - FOOTER_GAP - FOOTER_HEIGHT,
        }
    }
}

const SETTINGS_ICON: &str = "󰒓";
const CLOSE_ICON: &str = "󰅖";

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    User,
    Appearance,
    Colors,
    Launcher,
    Wallpaper,
    Bar,
    Behavior,
    Floating,
    Weather,
    Integrations,
    About,
}

// every page in the list's order, with its icon, label and the line under its title
const PAGES: [(Page, &str, &str, &str); 11] = [
    (
        Page::User,
        "󰀄",
        "User info",
        "Choose the name and profile picture shown on the lock screen.",
    ),
    (
        Page::Appearance,
        "󰍹",
        "Appearance",
        "Tune the shell's surfaces and blur.",
    ),
    (
        Page::Colors,
        "󰏘",
        "Colors",
        "Control palette selection and how dynamic wallpaper colors are generated.",
    ),
    (
        Page::Launcher,
        "󰍉",
        "Launcher",
        "Configure search results, command mode, and launcher presentation.",
    ),
    (
        Page::Wallpaper,
        "󰸉",
        "Wallpaper",
        "Choose the source directory, transitions, shuffle, and palette behavior.",
    ),
    (
        Page::Bar,
        "󰍜",
        "Status bar",
        "Control placement, modules, clock formatting, and workspace presentation.",
    ),
    (
        Page::Behavior,
        "󰒓",
        "Behavior",
        "Change dismissal rules, launcher behavior, and global motion preferences.",
    ),
    (
        Page::Floating,
        "󰖲",
        "Floating widgets",
        "Control desktop-only visibility, placement, scale, opacity, and cards.",
    ),
    (
        Page::Weather,
        "󰖐",
        "Weather",
        "Choose where the weather comes from and how it reads.",
    ),
    (
        Page::Integrations,
        "󰌹",
        "Integrations",
        "Keep supported applications in sync with the current shell palette.",
    ),
    (
        Page::About,
        "󰋼",
        "About",
        "Inspect the running setup or reset settings.",
    ),
];

// something risky waiting for a yes before it is staged
pub enum Confirm {
    Integration {
        key: &'static str,
        title: &'static str,
        warning: &'static str,
    },
    Reset,
}

// what the window shows besides the settings themselves
pub struct Shown {
    page: Page,

    // the dropdown that is open, named by its setting
    choice: Option<&'static str>,

    hovered: Option<String>,

    confirm: Option<Confirm>,
}

impl Service for Shown {
    fn new() -> Self {
        Self {
            page: Page::User,
            choice: None,
            hovered: None,
            confirm: None,
        }
    }

    // it only changes through input
    fn listen() {}
}

pub fn hover(name: String, inside: bool) {
    let mut shown = Shown::write();

    if inside {
        shown.hovered = Some(name);
    } else if shown.hovered.as_ref() == Some(&name) {
        shown.hovered = None;
    }
}

pub fn hovered(name: &str) -> bool {
    Shown::read().hovered.as_deref() == Some(name)
}

// controls sit a step lighter than the cards they are on
pub fn high_surface(theme: &Theme) -> Color {
    let amount = if theme.light { 0.08 } else { 0.06 };

    theme::mix(theme.surface, theme.text, amount)
}

// the shell's settings, opened with open_window from the launcher or `amane ipc call settings`
pub fn view() -> Window {
    let theme = theme::current();

    let layout = Layout::measure();

    let shown = Shown::read();

    let page = shown.page;
    let confirm = shown
        .confirm
        .as_ref()
        .map(|confirm| dialog::view(&theme, confirm));

    drop(shown);

    let body = Row::new(children![
        navigation::view(&theme, page, layout.body_height),
        content(&theme, page, &layout),
    ])
    .gap(MARGIN);

    let frame = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(theme.background)
        .padding(MARGIN)
        .child(Column::new(children![header(&theme, layout.width), body]).gap(HEADER_GAP));

    let mut layers: Vec<Box<dyn Widget>> = vec![Box::new(frame)];

    if let Some(confirm) = confirm {
        layers.push(Box::new(confirm));
    }

    Window::new()
        .title("Settings")
        .size(WIDTH, HEIGHT)
        .on_key(key_pressed)
        .child(Stack::new(layers).width(Parent).height(Parent))
}

fn header(theme: &Theme, width: f32) -> Row {
    let icon = Rectangle::new()
        .width(40.0)
        .height(40.0)
        .radius(12.0)
        .fill(theme.selected_surface)
        .align_child(Center, Center)
        .child(
            Text::new(SETTINGS_ICON)
                .size(21.0)
                .font(fonts::NERD)
                .tight()
                .color(theme.accent),
        );

    let title = Text::new("Settings")
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
        .child(
            Text::new(CLOSE_ICON)
                .size(18.0)
                .font(fonts::NERD)
                .tight()
                .color(theme.secondary_text),
        );

    Row::new(children![
        Row::new(children![icon, title]).gap(12.0).align(Center),
        close
    ])
    .width(width - MARGIN * 2.0)
    .height(HEADER_HEIGHT)
    .justify(SpaceBetween)
    .align(Center)
}

// the page's title and line, its groups, and apply at the bottom
fn content(theme: &Theme, shown: Page, layout: &Layout) -> Column {
    let mut title = "";
    let mut description = "";

    for (page, _, label, line) in PAGES {
        if page == shown {
            title = label;
            description = line;
        }
    }

    let section = Column::new(children![
        Text::new(title)
            .size(20.0)
            .font(fonts::BODY)
            .weight(Weight::Bold)
            .color(theme.text),
        Text::new(description)
            .size(11.0)
            .font(fonts::BODY)
            .color(theme.muted_text)
            .elide(),
    ])
    .gap(2.0);

    let section = Rectangle::new()
        .width(layout.content_width)
        .height(SECTION_HEIGHT)
        .child(section);

    let mut page = page::Page::new(theme, layout.content_width);

    pages::build(shown, &mut page);

    // each page keeps its own scroll position, named by its title
    let scroll = ScrollArea::new(title, page.finish())
        .width(layout.content_width)
        .height(layout.scroll_height);

    let scroll = Rectangle::new()
        .width(layout.content_width)
        .height(layout.scroll_height + FOOTER_GAP)
        .child(scroll);

    Column::new(children![
        section,
        scroll,
        footer(theme, layout.content_width)
    ])
    .gap(SECTION_GAP)
}

// "unsaved changes" beside apply, which is faded while there is nothing to apply
fn footer(theme: &Theme, width: f32) -> Rectangle {
    let changed = Settings::read().changed();

    let mut items: Vec<Box<dyn Widget>> = Vec::new();

    if changed {
        let note = Text::new("Unsaved changes")
            .size(11.0)
            .font(fonts::BODY)
            .color(theme.muted_text);

        items.push(Box::new(note));
    }

    items.push(Box::new(button::view(
        theme,
        "Apply",
        button::Style::Primary,
        changed,
        Settings::apply,
    )));

    Rectangle::new()
        .width(width)
        .height(FOOTER_HEIGHT)
        .align_child(End, End)
        .child(Row::new(items).gap(12.0).align(Center))
}

// escape backs out of a question, then a menu, then the window
fn key_pressed(key: Key) {
    if key != Key::Escape {
        return;
    }

    let mut shown = Shown::write();

    if shown.confirm.is_some() {
        shown.confirm = None;

        return;
    }

    if shown.choice.is_some() {
        shown.choice = None;

        return;
    }

    drop(shown);

    close();
}

// what wasn't applied is thrown away
fn close() {
    Settings::discard();

    amane::close_window(NAME);
}

// opens the window, or does nothing while it is open
pub fn open() {
    // a window closed from its title bar can leave a draft behind
    Settings::discard();

    let typed_name = String::from(Profile::read().typed_name());

    TextInput::set_text(pages::NAME_INPUT, &typed_name);

    field::fill();

    {
        let mut shown = Shown::write();

        shown.choice = None;
        shown.confirm = None;
    }

    amane::open_window(NAME, view);
}

// opens the window on one page, like colors from the launcher's color command
pub fn open_on(page: Page) {
    Shown::write().page = page;

    open();
}

pub fn ipc(_arguments: &[String]) -> String {
    open();

    String::from("ok")
}
