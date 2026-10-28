use amane::{Notification, Notifications, Service};

use super::panel::Panel;
use super::utility::{self, Page};
use crate::motion::{self, Glide};

// how long the power menu's hover fill and the launcher's list take to move
const FILL_DURATION: u64 = 340;
pub const LIST_DURATION: u64 = 240;

// which panels are out, shared by the bar that opens them and the overlay that draws them
pub struct Overlay {
    pub power_menu: Panel,

    // how far each power menu button's hover color has filled it, 0 to 1
    pub action_fills: Vec<Glide>,

    // leaving a panel for the bar keeps it open
    pub bar_hovered: bool,

    pub control_center: Panel,

    pub launcher: Panel,

    pub query: String,

    // the chosen result, and the first result the list shows
    pub selected: usize,
    pub first: usize,

    pub hovered_row: Option<usize>,

    // which result the highlight is on, and which one is at the top of the list, both sliding
    pub highlight: Glide,
    pub scroll: Glide,

    pub sessions: Vec<String>,

    pub utility: Panel,

    pub page: Page,

    // the control under the pointer in the utility center, like "tab:wifi"
    pub hovered: Option<String>,

    // the network whose password is being typed
    pub password_for: Option<String>,
    pub show_password: bool,

    // the month the calendar shows, counted from this month
    pub calendar_month: i64,

    // the network last asked to join, shown as connecting until it is up
    pub joining: Option<String>,

    // the popup under the pointer, whose countdown waits
    // the media player the control center shows, by bus name; none shows the active one
    pub player: Option<String>,

    // slides the media card in from the side it was switched toward, 0 once settled
    pub player_switch: Glide,

    pub hovered_popup: Option<u32>,

    // popups closed by hand, which leave before their time
    pub closed_popups: Vec<u32>,

    // notifications closed in the list, fading out before they are dismissed
    pub dismissing: Vec<u32>,
}

impl Service for Overlay {
    fn new() -> Self {
        let mut action_fills = Vec::new();

        for _ in 0..super::power_menu::ACTION_COUNT {
            action_fills.push(motion::spatial(0.0, FILL_DURATION));
        }

        Self {
            power_menu: Panel::new(),
            action_fills,
            bar_hovered: false,
            control_center: Panel::new(),
            launcher: Panel::new(),
            query: String::new(),
            selected: 0,
            first: 0,
            hovered_row: None,
            highlight: motion::spatial(0.0, LIST_DURATION),
            scroll: motion::spatial(0.0, LIST_DURATION),
            sessions: Vec::new(),
            utility: Panel::new(),
            page: Page::Notifications,
            hovered: None,
            password_for: None,
            show_password: false,
            joining: None,
            calendar_month: 0,
            player: None,
            player_switch: motion::spatial(0.0, motion::FAST_SPATIAL),
            hovered_popup: None,
            closed_popups: Vec::new(),
            dismissing: Vec::new(),
        }
    }

    // only changes through input
    fn listen() {}
}

impl Overlay {
    // only one panel is out at a time
    pub fn toggle_power_menu() {
        let mut overlay = Self::write();

        overlay.launcher.hide();
        overlay.control_center.hide();
        utility::close(&mut overlay);
        overlay.power_menu.toggle();
    }

    pub fn hide_power_menu() {
        Self::write().power_menu.hide();
    }

    pub fn toggle_utility() {
        let mut overlay = Self::write();

        overlay.launcher.hide();
        overlay.power_menu.hide();
        overlay.control_center.hide();

        if overlay.utility.shown {
            utility::close(&mut overlay);

            return;
        }

        overlay.utility.show();

        utility::opened(&mut overlay);
    }

    pub fn toggle_control_center() {
        let mut overlay = Self::write();

        overlay.launcher.hide();
        overlay.power_menu.hide();
        utility::close(&mut overlay);
        overlay.control_center.toggle();
    }

    pub fn hover_control_center(inside: bool) {
        let mut overlay = Self::write();

        overlay.control_center.hovered = inside;

        if inside {
            overlay.control_center.was_hovered = true;
        }

        overlay.dismiss_if_left();
    }

    pub fn hover_power_menu(inside: bool) {
        let mut overlay = Self::write();

        overlay.power_menu.hovered = inside;

        if inside {
            overlay.power_menu.was_hovered = true;
        }

        overlay.dismiss_if_left();
    }

    pub fn hover_utility(inside: bool) {
        let mut overlay = Self::write();

        overlay.utility.hovered = inside;

        if inside {
            overlay.utility.was_hovered = true;
        }

        overlay.dismiss_if_left();
    }

    pub fn hover_popup(id: u32, inside: bool) {
        let mut overlay = Self::write();

        if inside {
            overlay.hovered_popup = Some(id);
        } else if overlay.hovered_popup == Some(id) {
            overlay.hovered_popup = None;
        }
    }

    // only the popup goes, the notification stays in the list
    pub fn close_popup(id: u32) {
        let listed: Vec<u32> = Notifications::read().list().iter().map(Notification::id).collect();

        let mut overlay = Self::write();

        // ids of notifications already gone are dropped, so the list stays short
        overlay.closed_popups.retain(|closed| listed.contains(closed));

        overlay.closed_popups.push(id);
    }

    pub fn hover_bar(inside: bool) {
        let mut overlay = Self::write();

        overlay.bar_hovered = inside;

        overlay.dismiss_if_left();
    }

    /*
     * a panel closes once the pointer has been on it and then left it and
     * the bar; not while a password is being typed, it would be lost
     */
    fn dismiss_if_left(&mut self) {
        let bar_hovered = self.bar_hovered;

        let typing = self.password_for.is_some();

        let mut panels = vec![&mut self.power_menu, &mut self.control_center];

        if !typing {
            panels.push(&mut self.utility);
        }

        for panel in panels {
            let left = panel.was_hovered && !panel.hovered && !bar_hovered;

            if panel.shown && left {
                panel.hide();
            }
        }
    }
}
