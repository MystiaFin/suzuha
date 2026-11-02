use std::cell::RefCell;
use std::time::{Duration, Instant};

use amane::{Notification, Notifications, Padding, Rectangle, Service, Stack, Widget, children};

use super::panel::Panel;
use super::utility::notifications::{self, CardStyle};
use super::{Overlay, PanelView, Region};
use crate::liquid::{self, Blob};
use crate::motion::{self, DEFAULT_SPATIAL, FAST_SPATIAL, Glide, Spring};
use crate::theme::Theme;

const WIDTH: f32 = 380.0;
const RADIUS: f32 = 30.0;

// the stack reaches past the screen's right and bottom edges, so it melts into the corner
const EDGE_OVERLAP: f32 = 40.0;
const BOTTOM_OVERLAP: f32 = 24.0;

// how far past its own width it slides to let go of the edge
const HIDDEN_MARGIN: f32 = 32.0;

// the stack never grows closer than this to the top
const TOP_ROOM: f32 = 112.0;

const PADDING: Padding = Padding {
    top: 16.0,
    right: EDGE_OVERLAP + 16.0,
    bottom: BOTTOM_OVERLAP + 8.0,
    left: 16.0,
};

const CARD_WIDTH: f32 = WIDTH - PADDING.left - PADDING.right;

const GAP: f32 = 8.0;

// how long a popup stays while the pointer is away from it
const TIMEOUT: Duration = Duration::from_secs(6);

// the line along a card's bottom that runs out with its time
const COUNTDOWN_HEIGHT: f32 = 3.0;
const COUNTDOWN_INSET: f32 = 12.0;
const COUNTDOWN_BOTTOM: f32 = 6.0;

const POPUP_CARD: CardStyle = CardStyle {
    padding: Padding {
        top: 14.0,
        right: 42.0,
        bottom: 14.0,
        left: 14.0,
    },
    radius: 12.0,
    min_height: 98.0,
    icon_size: 52.0,
    icon_margin: 9.0,
    icon_radius: 16.0,
    icon_gap: 13.0,
    bell_size: 23.0,
    text_gap: 3.0,
    summary_size: 14.0,
    close_size: 25.0,
    close_inset: 10.0,
    close_glyph_size: 14.0,
    close: Overlay::close_popup,
    popup: true,
};

thread_local! {
    /*
     * the countdowns and slides change every frame, so they live here and
     * not in the overlay service, where each change would draw another frame
     */
    static POPUPS: RefCell<Popups> = RefCell::new(Popups::new());
}

struct Popup {
    id: u32,

    // counts down while the pointer is away
    left: Duration,

    // how far it is slid out to the right, in card widths
    slide: Glide,

    leaving: bool,
}

impl Popup {
    // slides in from the right
    fn new(id: u32) -> Self {
        let mut slide = motion::spatial(1.0, DEFAULT_SPATIAL);

        slide.to(0.0);

        Self {
            id,
            left: TIMEOUT,
            slide,
            leaving: false,
        }
    }

    // and back out the same way, a little quicker
    fn leave(&mut self) {
        self.slide = motion::spatial(self.slide.value(), FAST_SPATIAL);
        self.slide.to(1.0);

        self.leaving = true;
    }
}

struct Popups {
    // oldest first
    list: Vec<Popup>,

    // the notifications there on the last frame, anything not in it is new
    known: Vec<u32>,

    last_frame: Instant,

    panel: Panel,

    // none while hidden, so the next opening starts at the right height
    height: Option<Spring>,
}

impl Popups {
    fn new() -> Self {
        Self {
            list: Vec::new(),
            known: Vec::new(),
            last_frame: Instant::now(),
            panel: Panel::new(),
            height: None,
        }
    }

    fn update(&mut self, overlay: &Overlay, listed: &[u32]) {
        let now = Instant::now();

        let elapsed = now - self.last_frame;

        self.last_frame = now;

        // gone from the list, gone from the stack
        self.list.retain(|popup| listed.contains(&popup.id));

        let mut counting = false;

        for popup in &mut self.list {
            if popup.leaving {
                continue;
            }

            if overlay.hovered_popup != Some(popup.id) {
                popup.left = popup.left.saturating_sub(elapsed);

                counting = true;
            }

            if popup.left.is_zero() || overlay.closed_popups.contains(&popup.id) {
                popup.leave();
            }
        }

        // a countdown moves the line on every card, so it keeps drawing frames
        if counting {
            amane::request_frame();
        }

        self.list
            .retain(|popup| !popup.leaving || popup.slide.value() < 1.0);

        // nothing pops up while the panel with every notification is open
        for id in listed {
            if !self.known.contains(id) && !overlay.utility.shown {
                self.list.push(Popup::new(*id));
            }
        }

        self.known = listed.to_vec();

        let wanted = !self.list.is_empty() && !overlay.utility.shown;

        if wanted && !self.panel.shown {
            self.panel.show();
        } else if !wanted && self.panel.shown {
            self.panel.hide();
        }
    }

    // none while fully hidden
    fn view(
        &mut self,
        overlay: &Overlay,
        theme: &Theme,
        notifications: &Notifications,
        screen: Region,
    ) -> Option<PanelView> {
        let progress = self.panel.progress.value();

        if progress <= 0.001 {
            self.height = None;

            return None;
        }

        let room = screen.height - TOP_ROOM - PADDING.top - PADDING.bottom;

        let (cards, cards_height, moving) = self.cards(overlay, theme, notifications, room);

        let wanted_height = PADDING.top + cards_height + PADDING.bottom;

        let spring = self
            .height
            .get_or_insert_with(|| motion::size(wanted_height));

        spring.to(wanted_height);

        let height = spring.value();

        let settled = height == wanted_height;

        let hidden = WIDTH + HIDDEN_MARGIN;

        let x = screen.width - WIDTH + EDGE_OVERLAP + hidden * (1.0 - progress);
        let y = screen.height + BOTTOM_OVERLAP - height;

        let content = content(cards, height, moving || !settled);

        let blob = Blob::new(x, y, WIDTH, height).radius(RADIUS).child(content);

        let input = Region {
            x,
            y,
            width: WIDTH,
            height,
        };

        // fully out it starts at its resting edge, and its melted corner spreads a little past that
        let reach_width = WIDTH - EDGE_OVERLAP + liquid::CONNECTION;

        let tallest = f32::max(height, wanted_height);

        let reach_top = screen.height + BOTTOM_OVERLAP - tallest - liquid::CONNECTION;

        let reach = Region {
            x: screen.width - reach_width,
            y: reach_top,
            width: reach_width,
            height: screen.height - reach_top,
        };

        Some(PanelView { blob, input, reach })
    }

    /*
     * the newest popups that fit, laid out from the bottom up: a card keeps
     * its place while older ones above it leave, and slides up when a new
     * one arrives under it
     */
    fn cards(
        &self,
        overlay: &Overlay,
        theme: &Theme,
        notifications: &Notifications,
        room: f32,
    ) -> (Vec<Placed>, f32, bool) {
        let mut cards = Vec::new();

        let mut total = 0.0;
        let mut moving = false;

        for popup in self.list.iter().rev() {
            let Some(notification) = find(notifications, popup.id) else {
                continue;
            };

            let (card, card_height) =
                notifications::card(overlay, theme, notification, CARD_WIDTH, &POPUP_CARD);

            let below = if cards.is_empty() { 0.0 } else { total + GAP };

            if below + card_height > room && !cards.is_empty() {
                break;
            }

            total = below + card_height;

            let offset = popup.slide.value();

            let from_bottom =
                motion::follow(&format!("popup:{}", popup.id), total, DEFAULT_SPATIAL);

            if offset > 0.0 || from_bottom != total {
                moving = true;
            }

            let card = Stack::new(children![card, countdown(theme, popup, card_height)])
                .width(CARD_WIDTH)
                .height(card_height);

            cards.push(Placed {
                card,
                height: card_height,
                id: popup.id,
                x: offset * CARD_WIDTH,
                from_bottom,
            });
        }

        (cards, total, moving)
    }
}

// a card and where it goes, its top counted up from the bottom of the stack
struct Placed {
    card: Stack,
    height: f32,
    id: u32,
    x: f32,
    from_bottom: f32,
}

pub fn view(overlay: &Overlay, theme: &Theme, screen: Region) -> Option<PanelView> {
    let notifications = Notifications::read();

    let mut listed = Vec::new();

    for notification in notifications.list() {
        listed.push(notification.id());
    }

    POPUPS.with_borrow_mut(|popups| {
        popups.update(overlay, &listed);

        popups.view(overlay, theme, &notifications, screen)
    })
}

fn find(notifications: &Notifications, id: u32) -> Option<&Notification> {
    notifications
        .list()
        .iter()
        .find(|notification| notification.id() == id)
}

// the cards hang from the bottom of the stack, and only get clipped while they move
fn content(cards: Vec<Placed>, height: f32, moving: bool) -> Rectangle {
    let area_height = height - PADDING.top - PADDING.bottom;

    let mut layers: Vec<Box<dyn Widget>> = Vec::new();

    for placed in cards {
        let id = placed.id;

        let item = Rectangle::new()
            .width(CARD_WIDTH)
            .height(placed.height)
            .translate(placed.x, area_height - placed.from_bottom)
            .on_hover(move |inside| Overlay::hover_popup(id, inside))
            .child(placed.card);

        layers.push(Box::new(item));
    }

    let mut area = Rectangle::new()
        .width(CARD_WIDTH)
        .height(area_height)
        .child(Stack::new(layers));

    if moving {
        area = area.clip();
    }

    Rectangle::new()
        .width(WIDTH)
        .height(height)
        .padding(PADDING)
        .child(area)
}

fn countdown(theme: &Theme, popup: &Popup, card_height: f32) -> Rectangle {
    let full = CARD_WIDTH - COUNTDOWN_INSET * 2.0;

    let share = popup.left.as_secs_f32() / TIMEOUT.as_secs_f32();

    Rectangle::new()
        .width(full * share)
        .height(COUNTDOWN_HEIGHT)
        .radius(COUNTDOWN_HEIGHT / 2.0)
        .fill(theme.accent)
        .translate(
            COUNTDOWN_INSET,
            card_height - COUNTDOWN_BOTTOM - COUNTDOWN_HEIGHT,
        )
}
