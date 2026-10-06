mod results;

use std::cell::{Cell, RefCell};

use amane::{
    Apps, Center, Column, Image, Key, Padding, Parent, Pointer, Rectangle, Row, Scroll, Service,
    Stack, Start, Text, TextInput, Widget, children,
};

use super::state::LIST_DURATION;
use super::{Overlay, PanelView, Region};
use crate::fonts;
use crate::liquid::{self, Blob};
use crate::motion::{self, Spring};
use crate::settings::Settings;
use crate::theme::Theme;

use results::{Entry, Kind};

// the text input's name, which keeps what was typed between redraws
const INPUT: &str = "launcher";

const ROW_HEIGHT: f32 = 60.0;
const FIELD_HEIGHT: f32 = 46.0;

const RADIUS: f32 = 30.0;
const INNER_RADIUS: f32 = 12.0;

// the panel reaches this far past the screen's bottom edge, so it stays melted into it
const EDGE_OVERLAP: f32 = 40.0;

// how far past its own height it slides to let go of the edge
const HIDDEN_MARGIN: f32 = 32.0;

// a hidden launcher is a little narrower, it widens as it rises
const CLOSED_WIDTH: f32 = 0.96;

const PADDING: Padding = Padding {
    top: 12.0,
    right: 14.0,
    bottom: 56.0,
    left: 14.0,
};

// the gap between the list and the search field
const GAP: f32 = 8.0;

const FIXED_HEIGHT: f32 = PADDING.top + GAP + FIELD_HEIGHT + PADDING.bottom;

const ICON_SIZE: f32 = 38.0;

thread_local! {
    // how many rows fit last frame, so the keys scroll by what is really shown
    static ROWS_SHOWN: Cell<usize> = const { Cell::new(1) };

    /*
     * the panel's height springs toward what the results need; kept outside
     * the service because the view sets its target, and a service write
     * from the view would draw frames forever
     */
    static HEIGHT: RefCell<Option<Spring>> = const { RefCell::new(None) };
}

// none while fully hidden
pub fn view(overlay: &Overlay, theme: &Theme, screen: Region) -> Option<PanelView> {
    let progress = overlay.launcher.progress.value();

    if progress <= 0.001 {
        return None;
    }

    let entries = results::find(&overlay.query, &overlay.sessions);

    let full_width = Settings::read().number("launcher_width").min(screen.width - 80.0);
    let width = full_width * (CLOSED_WIDTH + (1.0 - CLOSED_WIDTH) * progress);

    let rows = visible_rows(screen, entries.len()).max(1);

    ROWS_SHOWN.set(rows);

    let height = follow_height(FIXED_HEIGHT + rows as f32 * ROW_HEIGHT);

    let hidden = height + HIDDEN_MARGIN;

    let x = (screen.width - width) / 2.0;
    let y = screen.height - height + EDGE_OVERLAP + hidden * (1.0 - progress);

    let blob = Blob::new(x, y, width, height)
        .radius(RADIUS)
        .child(content(overlay, theme, &entries, width, height));

    let input = Region {
        x,
        y,
        width,
        height,
    };

    // as tall as it can ever grow, so the window keeps its size while results change
    let tallest = FIXED_HEIGHT + max_rows(screen) as f32 * ROW_HEIGHT;

    let reach_height = tallest - EDGE_OVERLAP + liquid::CONNECTION;
    let reach_width = full_width + liquid::CONNECTION * 2.0;

    let reach = Region {
        x: (screen.width - reach_width) / 2.0,
        y: screen.height - reach_height,
        width: reach_width,
        height: reach_height,
    };

    Some(PanelView { blob, input, reach })
}

fn follow_height(target: f32) -> f32 {
    HEIGHT.with_borrow_mut(|height| {
        let spring = height.get_or_insert_with(|| motion::size(target));

        spring.to(target);

        spring.value()
    })
}

// the rows the settings ask for, as many as fit on the screen
fn max_rows(screen: Region) -> usize {
    let wanted = Settings::read().number("launcher_rows") as usize;

    let fitting = ((screen.height - 60.0 - FIXED_HEIGHT) / ROW_HEIGHT).floor() as usize;

    wanted.min(fitting).max(1)
}

fn visible_rows(screen: Region, count: usize) -> usize {
    count.min(max_rows(screen))
}

fn content(
    overlay: &Overlay,
    theme: &Theme,
    entries: &[Entry],
    width: f32,
    height: f32,
) -> Rectangle {
    let inner_width = width - PADDING.left - PADDING.right;

    let list_height = height - FIXED_HEIGHT;

    let field = Rectangle::new()
        .width(inner_width)
        .height(FIELD_HEIGHT)
        .radius(INNER_RADIUS)
        .fill(theme.surface)
        .padding(Padding {
            top: 0.0,
            right: 14.0,
            bottom: 0.0,
            left: 14.0,
        })
        .align_child(Start, Center)
        .child(
            TextInput::new(INPUT)
                .size(15.0)
                .color(theme.text)
                .placeholder("Type > for command palette...")
                .focused()
                .on_change(query_changed)
                .on_submit(|_| launch_selected()),
        );

    let list = list(overlay, theme, entries, inner_width, list_height);

    Rectangle::new()
        .width(width)
        .height(Parent)
        .padding(PADDING)
        .child(Column::new(children![list, field]).gap(GAP))
}

// the rows that fit, with the selection highlight sliding behind them
fn list(
    overlay: &Overlay,
    theme: &Theme,
    entries: &[Entry],
    width: f32,
    height: f32,
) -> Rectangle {
    let area = Rectangle::new()
        .width(width)
        .height(height)
        .clip()
        .on_scroll(scrolled);

    if entries.is_empty() {
        let empty = Text::new(results::empty_text(&overlay.query))
            .size(14.0)
            .font(fonts::BODY)
            .color(theme.secondary_text);

        return area.align_child(Center, Center).child(empty);
    }

    // both counted in rows from the first result, so they slide together when the list scrolls
    let scroll = overlay.scroll.value();
    let highlight_row = overlay.highlight.value();

    let highlight = Rectangle::new()
        .width(width)
        .height(ROW_HEIGHT)
        .radius(INNER_RADIUS)
        .fill(theme.selected_surface)
        .translate(0.0, (highlight_row - scroll) * ROW_HEIGHT);

    // only the rows in view are built, plus one that is sliding in
    let top = scroll.max(0.0).floor() as usize;

    let last = (top + ROWS_SHOWN.get() + 1).min(entries.len());

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    for index in top..last {
        rows.push(Box::new(row(overlay, theme, &entries[index], index, width)));
    }

    let sliding = (scroll - top as f32) * ROW_HEIGHT;

    let rows = Rectangle::new()
        .width(width)
        .height(ROW_HEIGHT * (last - top) as f32)
        .translate(0.0, -sliding)
        .child(Column::new(rows));

    area.child(Stack::new(children![highlight, rows]))
}

fn row(overlay: &Overlay, theme: &Theme, entry: &Entry, index: usize, width: f32) -> Rectangle {
    let hovered = overlay.hovered_row == Some(index) && overlay.selected != index;

    let fill = if hovered {
        theme.hover_surface
    } else {
        amane::Color::TRANSPARENT
    };

    // a tmux session has no icon, so its name starts at the edge
    let is_session = matches!(entry.kind, Kind::Tmux(_));

    let show_icon = Settings::read().flag("launcher_icons") && !is_session;

    let name_width = if show_icon {
        width - 62.0 - 12.0
    } else {
        width - 24.0
    };

    let mut labels: Vec<Box<dyn Widget>> = vec![Box::new(
        Text::new(&entry.name)
            .size(15.0)
            .font(fonts::BODY)
            .color(theme.text)
            .elide(),
    )];

    if let Some(description) = &entry.description {
        let description = Text::new(description)
            .size(12.0)
            .font(fonts::BODY)
            .color(theme.secondary_text)
            .elide();

        labels.push(Box::new(description));
    }

    let name = Rectangle::new()
        .width(name_width)
        .height(ROW_HEIGHT)
        .align_child(Start, Center)
        .child(Column::new(labels).gap(2.0));

    let content: Box<dyn Widget> = if show_icon {
        Box::new(Row::new(children![icon(entry, theme), name]).gap(12.0).align(Center))
    } else {
        Box::new(name)
    };

    Rectangle::new()
        .width(width)
        .height(ROW_HEIGHT)
        .radius(INNER_RADIUS)
        .fill(fill)
        .cursor(Pointer)
        .on_hover(move |inside| hover_row(index, inside))
        .on_click(move |_| {
            Overlay::write().selected = index;

            launch_selected();
        })
        .padding(Padding {
            top: 0.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        })
        .align_child(Start, Center)
        .child(Column::new(vec![content]))
}

// the app's own icon, a glyph for commands, or empty space to keep names lined up
fn icon(entry: &Entry, theme: &Theme) -> Rectangle {
    let slot = Rectangle::new()
        .width(ICON_SIZE)
        .height(ICON_SIZE)
        .align_child(Center, Center);

    if let Some(glyph) = entry.glyph {
        return slot.child(Text::new(glyph).size(24.0).font(fonts::NERD).color(theme.text));
    }

    let Kind::App(index) = entry.kind else {
        return slot;
    };

    let apps = Apps::read();

    let Some(path) = apps.list()[index].icon_path() else {
        return slot;
    };

    let readable = matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("png" | "jpg" | "jpeg" | "svg")
    );

    if !readable {
        return slot;
    }

    // twice the size, so it stays sharp on a scaled screen
    let pixels = (ICON_SIZE * 2.0) as u32;

    slot.fill(Image::contain(path).thumbnail(pixels, pixels))
}

pub fn ipc(arguments: &[String]) -> String {
    let command = arguments.first().map(String::as_str).unwrap_or("toggle");

    let mut overlay = Overlay::write();

    match command {
        "show" => show(&mut overlay),
        "showTmux" => {
            show(&mut overlay);

            search_tmux(&mut overlay);
        }
        "hide" => overlay.launcher.hide(),
        _ if overlay.launcher.shown => overlay.launcher.hide(),
        _ => show(&mut overlay),
    }

    String::from("ok")
}

// opens at the top of the list, on an empty search unless the last one is remembered
fn show(overlay: &mut Overlay) {
    overlay.power_menu.hide();
    overlay.control_center.hide();
    super::utility::close(overlay);
    overlay.launcher.show();

    // remembering keeps the last search, and the field still shows it
    if !Settings::read().flag("launcher_remember_query") {
        overlay.query.clear();

        TextInput::set_text(INPUT, "");
    }

    reset_list(overlay);
}

// switches the search to tmux sessions, as if "!" was typed
fn search_tmux(overlay: &mut Overlay) {
    overlay.query = String::from("!");
    overlay.sessions = results::read_sessions();

    TextInput::set_text(INPUT, "!");

    reset_list(overlay);
}

// the window's keys while the launcher is open; letters and enter go to the search field
pub fn key_pressed(key: Key) {
    let mut overlay = Overlay::write();

    if !overlay.launcher.shown {
        return;
    }

    let count = results::find(&overlay.query, &overlay.sessions).len();

    match key {
        Key::Down if count > 0 => {
            let next = (overlay.selected + 1) % count;

            select(&mut overlay, next, count);
        }

        Key::Up if count > 0 => {
            let previous = (overlay.selected + count - 1) % count;

            select(&mut overlay, previous, count);
        }

        // the first escape clears the search, the second closes
        Key::Escape if !overlay.query.is_empty() && Settings::read().flag("launcher_escape_clears") => {
            overlay.query.clear();

            TextInput::set_text(INPUT, "");

            reset_list(&mut overlay);
        }

        Key::Escape => overlay.launcher.hide(),

        _ => {}
    }
}

/*
 * moves the selection and scrolls just enough to keep it in the list;
 * the highlight slides to the selection's row
 */
fn select(overlay: &mut Overlay, index: usize, count: usize) {
    overlay.selected = index;

    // the rows really shown, which can be fewer than the most there is room for
    let shown = ROWS_SHOWN.get();

    if index < overlay.first {
        overlay.first = index;
    }

    if index >= overlay.first + shown {
        overlay.first = index + 1 - shown;
    }

    // a shorter list may leave the old top past its end
    overlay.first = overlay.first.min(count.saturating_sub(shown));

    let first = overlay.first as f32;

    overlay.scroll.to(first);
    overlay.highlight.to(index as f32);
}

// a new search starts at the top, without sliding there from the old results
fn reset_list(overlay: &mut Overlay) {
    overlay.selected = 0;
    overlay.first = 0;

    overlay.scroll = motion::spatial(0.0, LIST_DURATION);
    overlay.highlight = motion::spatial(0.0, LIST_DURATION);
}

fn query_changed(query: String) {
    let mut overlay = Overlay::write();

    // tmux sessions are listed once, when the search switches to them
    if query.starts_with('!') && !overlay.query.starts_with('!') {
        overlay.sessions = results::read_sessions();
    }

    overlay.query = query;

    reset_list(&mut overlay);
}

// wheel down shows later results, the selection stays where it was
fn scrolled(scroll: Scroll) {
    let mut overlay = Overlay::write();

    let count = results::find(&overlay.query, &overlay.sessions).len();

    let last_top = count.saturating_sub(ROWS_SHOWN.get());

    if scroll.y > 0.0 {
        overlay.first = (overlay.first + 1).min(last_top);
    } else {
        overlay.first = overlay.first.saturating_sub(1);
    }

    let first = overlay.first as f32;

    overlay.scroll.to(first);
}

fn hover_row(index: usize, inside: bool) {
    let mut overlay = Overlay::write();

    if inside {
        overlay.hovered_row = Some(index);
    } else if overlay.hovered_row == Some(index) {
        overlay.hovered_row = None;
    }
}

fn launch_selected() {
    let mut overlay = Overlay::write();

    let entries = results::find(&overlay.query, &overlay.sessions);

    let Some(entry) = entries.get(overlay.selected) else {
        return;
    };

    let close = Settings::read().flag("launcher_close_on_launch");

    match &entry.kind {
        Kind::App(index) => {
            Apps::read().list()[*index].launch();

            if close {
                overlay.launcher.hide();
            }
        }

        Kind::Settings => {
            overlay.launcher.hide();

            crate::settings::open();
        }

        Kind::Colors => {
            overlay.launcher.hide();

            crate::settings::open_on(crate::settings::Page::Colors);
        }

        Kind::Wallpapers => {
            overlay.launcher.hide();

            // the picker takes the overlay's place, so this write ends first
            drop(overlay);

            crate::wallpaper::picker::ipc(&[String::from("show")]);
        }

        Kind::Converter => {
            overlay.launcher.hide();

            crate::converter::open();
        }

        Kind::TmuxCommand => search_tmux(&mut overlay),

        Kind::Tmux(session) => {
            if results::attach(session) && close {
                overlay.launcher.hide();
            }
        }
    }
}
