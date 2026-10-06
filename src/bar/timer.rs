use std::cell::RefCell;

use amane::{Center, Parent, Pointer, Rectangle, Row, Text, children};

use super::pill;
use crate::fonts;
use crate::motion::{self, Glide};
use crate::pomodoro;
use crate::theme::Theme;

// milliseconds the pill takes to open and close
const DURATION: u64 = 420;

const TEXT_SIZE: f32 = 13.0;

// the pill's width, and the last text shown so it stays while the pill closes
struct Shown {
    width: Glide,

    text: String,
}

thread_local! {
    // kept outside services, a service write from view() would draw frames forever
    static SHOWN: RefCell<Option<Shown>> = const { RefCell::new(None) };
}

/*
 * the running pomodoro, in an accent pill that grows from nothing
 * and pushes its neighbours apart; none once it has closed
 */
pub fn view(theme: &Theme) -> Option<Rectangle> {
    let status = pomodoro::status();

    SHOWN.with_borrow_mut(|shown| {
        let shown = shown.get_or_insert_with(|| Shown {
            width: motion::spatial(0.0, DURATION),
            text: String::new(),
        });

        if let Some(text) = status.as_ref() {
            shown.text.clone_from(text);
        }

        // the icon is about 9px wide, then a 7px gap and 20px of padding
        let full = 9.0 + 7.0 + pill::text_width(&shown.text, TEXT_SIZE) + 20.0;

        let target = if status.is_some() { full } else { 0.0 };

        shown.width.to(target);

        let width = shown.width.value();

        if width < 1.0 {
            return None;
        }

        // what the timer is doing right now, playing or paused
        let icon = if pomodoro::running() {
            pomodoro::PLAY_ICON
        } else {
            pomodoro::PAUSE_ICON
        };

        let icon = Text::new(icon)
            .size(14.0)
            .font(fonts::NERD)
            .color(theme.on_accent);

        let label = pill::label(&shown.text, TEXT_SIZE, theme.on_accent);

        // the content fades in as the pill opens, and is clipped while it is narrow
        let content = Rectangle::new()
            .width(full)
            .height(Parent)
            .opacity(width / full)
            .align_child(Center, Center)
            .child(Row::new(children![icon, label]).gap(7.0).align(Center));

        let pill = pill::view(width, theme.accent)
            .clip()
            .cursor(Pointer)
            .on_click(|_| pomodoro::toggle())
            .align_child(Center, Center)
            .child(content);

        Some(pill)
    })
}
