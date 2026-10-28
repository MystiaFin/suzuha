use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::thread;

use amane::{
    Apps, Center, Color, Column, Image, Notification, Notifications, Padding, Parent, Pointer,
    Rectangle, Row, ScrollArea, Service, Size, SpaceBetween, Stack, Start, Text, Weight, Widget,
    children,
};

use super::{fade_target, hover};
use crate::clock::Clock;
use crate::fonts;
use crate::motion::{self, FAST_EFFECTS, FAST_SPATIAL};
use crate::overlay::Overlay;
use crate::theme::{self, Theme};

const LIST: &str = "notifications";

const TITLE_HEIGHT: f32 = 30.0;

const GAP: f32 = 10.0;

// a little past the fade and the close, so the space is shut before the card goes
const DISMISS_DELAY: u64 = FAST_EFFECTS + FAST_SPATIAL + 50;

// what sets a card in the list apart from one popping up at the screen's edge
pub struct CardStyle {
    pub padding: Padding,
    pub radius: f32,

    // a shorter card grows to this, its content centered
    pub min_height: f32,

    pub icon_size: f32,
    pub icon_margin: f32,
    pub icon_radius: f32,
    pub icon_gap: f32,
    pub bell_size: f32,

    pub text_gap: f32,
    pub summary_size: f32,

    pub close_size: f32,
    pub close_inset: f32,
    pub close_glyph_size: f32,
    pub close: fn(u32),

    /*
     * a popup names the app above the summary and centers its icon;
     * a card in the list says the app and the time below
     */
    pub popup: bool,
}

const LIST_CARD: CardStyle = CardStyle {
    padding: Padding {
        top: 10.0,
        right: 36.0,
        bottom: 10.0,
        left: 12.0,
    },
    radius: 16.0,
    min_height: 0.0,
    icon_size: 38.0,
    icon_margin: 7.0,
    icon_radius: 8.0,
    icon_gap: 10.0,
    bell_size: 17.0,
    text_gap: 2.0,
    summary_size: 13.0,
    close_size: 15.0,
    close_inset: 12.0,
    close_glyph_size: 15.0,
    close: dismiss,
    popup: false,
};

const BUTTON_HEIGHT: f32 = 26.0;
const BUTTON_PADDING: f32 = 9.0;
const BUTTON_GAP: f32 = 6.0;
const ACTIONS_GAP: f32 = 8.0;

const BELL_ICON: &str = "󰂚";
const CLOSE_ICON: &str = "󰅖";

pub fn view(overlay: &Overlay, theme: &Theme, width: f32, height: f32) -> Column {
    let notifications = Notifications::read();

    let list_height = height - TITLE_HEIGHT - GAP;

    Column::new(children![
        title_row(overlay, theme, &notifications, width),
        list(overlay, theme, &notifications, width, list_height),
    ])
    .gap(GAP)
}

fn title_row(overlay: &Overlay, theme: &Theme, notifications: &Notifications, width: f32) -> Row {
    let title = Text::new("Notifications")
        .size(15.0)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text);

    let mut parts: Vec<Box<dyn Widget>> = vec![Box::new(title)];

    if !notifications.list().is_empty() {
        let hover_name = String::from("notifications:clear");

        let amount = motion::fade(&hover_name, fade_target(overlay, &hover_name));

        let clear = Text::new("Clear all")
            .size(11.0)
            .font(fonts::BODY)
            .color(theme::mix(theme.muted_text, theme.danger, amount));

        let clear = Rectangle::new()
            .width(size_of(&clear))
            .height(TITLE_HEIGHT)
            .align_child(Start, Center)
            .cursor(Pointer)
            .on_hover(move |inside| hover(hover_name.clone(), inside))
            .on_click(|_| Notifications::clear())
            .child(clear);

        parts.push(Box::new(clear));
    }

    Row::new(parts)
        .width(width)
        .height(TITLE_HEIGHT)
        .justify(SpaceBetween)
        .align(Center)
}

// newest on top
fn list(
    overlay: &Overlay,
    theme: &Theme,
    notifications: &Notifications,
    width: f32,
    height: f32,
) -> Rectangle {
    let area = Rectangle::new().width(width).height(height);

    if notifications.list().is_empty() {
        let empty = Text::new("No notifications")
            .size(13.0)
            .font(fonts::BODY)
            .color(theme.muted_text);

        return area.align_child(Center, Center).child(empty);
    }

    let mut cards: Vec<Box<dyn Widget>> = Vec::new();
    let mut total = 0.0;

    // how much of the cards above is still open, so a gap closes along with the card before it
    let mut open_above: f32 = 0.0;

    for notification in notifications.list().iter().rev() {
        let id = notification.id();

        let (card, card_height) = card(overlay, theme, notification, width, &LIST_CARD);

        let leaving = overlay.dismissing.contains(&id);

        let opacity = motion::fade(&format!("notification:{id}:opacity"), if leaving { 0.0 } else { 1.0 });

        // the space only starts closing once the card has faded out
        let faded = leaving && opacity == 0.0;

        let open_target = if faded { 0.0 } else { 1.0 };

        let open = motion::follow(&format!("notification:{id}:open"), open_target, FAST_SPATIAL);

        let gap = GAP * open_above.min(1.0);

        let slot_height = (gap + card_height) * open;

        let slot = Rectangle::new().width(width).height(slot_height);

        // a faded card leaves only its space behind, so its buttons can't be hit while it closes
        if faded {
            cards.push(Box::new(slot));
        } else {
            let padding = Padding {
                top: gap,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            };

            cards.push(Box::new(slot.padding(padding).opacity(opacity).child(card)));
        }

        total += slot_height;

        open_above += open;
    }

    let column = Column::new(cards).width(width).height(total);

    area.child(ScrollArea::new(LIST, column))
}

// the icon beside the text, the buttons under both, and a close button in the corner
pub fn card(
    overlay: &Overlay,
    theme: &Theme,
    notification: &Notification,
    width: f32,
    style: &CardStyle,
) -> (Stack, f32) {
    let padding = style.padding;

    let inner_width = width - padding.left - padding.right;

    let text_width = inner_width - style.icon_size - style.icon_gap;

    let (text, text_height) = text_column(theme, notification, text_width, style);

    let top_height = f32::max(style.icon_size, text_height);

    let mut top = Row::new(children![icon(theme, notification, style), text])
        .height(top_height)
        .gap(style.icon_gap);

    // a popup centers its icon beside the text
    if style.popup {
        top = top.align(Center);
    }

    let mut rows: Vec<Box<dyn Widget>> = vec![Box::new(top)];

    let mut inner_height = top_height;

    if !notification.actions().is_empty() {
        rows.push(Box::new(actions(overlay, theme, notification, style)));

        inner_height += ACTIONS_GAP + BUTTON_HEIGHT;
    }

    let fitted_height = padding.top + inner_height + padding.bottom;

    let height = f32::max(style.min_height, fitted_height);

    // what the minimum height adds, half of it above the content
    let centering = (height - fitted_height) / 2.0;

    let body = Rectangle::new()
        .width(width)
        .height(height)
        .radius(style.radius)
        .fill(theme.surface)
        .padding(padding)
        .align_child(Start, Center)
        .child(Column::new(rows).height(inner_height).gap(ACTIONS_GAP));

    let mut layers: Vec<Box<dyn Widget>> = vec![Box::new(body)];

    // the top of the card runs its default action, the buttons below keep their own clicks
    if notification.has_default_action() {
        let id = notification.id();

        let click = Rectangle::new()
            .width(width)
            .height(padding.top + centering + top_height)
            .cursor(Pointer)
            .on_click(move |_| Notifications::click(id));

        layers.push(Box::new(click));
    }

    let close = close_button(overlay, theme, notification.id(), width, style);

    layers.push(Box::new(close));

    (Stack::new(layers).width(width).height(height), height)
}

// the summary, up to two lines of body, then which app sent it and when
fn text_column(
    theme: &Theme,
    notification: &Notification,
    width: f32,
    style: &CardStyle,
) -> (Column, f32) {
    let summary = Text::new(notification.summary())
        .size(style.summary_size)
        .font(fonts::BODY)
        .weight(Weight::SemiBold)
        .color(theme.text)
        .elide();

    let mut parts: Vec<(Text, f32)> = Vec::new();

    if style.popup {
        let app = Text::new(notification.app_name())
            .size(9.0)
            .font(fonts::BODY)
            .weight(Weight::SemiBold)
            .color(theme.accent)
            .elide();

        parts.push(fitted(app, width));
    }

    parts.push(fitted(summary, width));

    let body = plain_text(notification.body());

    if !body.trim().is_empty() {
        let body = Text::new(body)
            .size(11.0)
            .font(fonts::BODY)
            .color(theme.muted_text)
            .wrap()
            .max_lines(2)
            .elide();

        parts.push(fitted(body, width));
    }

    if !style.popup {
        let received = Clock::read().hours_minutes(notification.received());

        let source = Text::new(format!("{}  •  {received}", notification.app_name()))
            .size(9.0)
            .font(fonts::BODY)
            .color(theme.secondary_text)
            .elide();

        parts.push(fitted(source, width));
    }

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();
    let mut total = 0.0;

    for (text, height) in parts {
        total += height;

        rows.push(Box::new(Rectangle::new().width(width).height(height).child(text)));
    }

    total += style.text_gap * (rows.len() - 1) as f32;

    (Column::new(rows).width(width).height(total).gap(style.text_gap), total)
}

fn fitted(text: Text, width: f32) -> (Text, f32) {
    let height = text.height_in(width);

    (text, height)
}

// the sender's icon when one can be found, a bell otherwise
fn icon(theme: &Theme, notification: &Notification, style: &CardStyle) -> Rectangle {
    let slot = Rectangle::new()
        .width(style.icon_size)
        .height(style.icon_size)
        .radius(style.icon_radius)
        .fill(theme.selected_surface)
        .align_child(Center, Center);

    let Some(path) = icon_path(notification) else {
        let bell = Text::new(BELL_ICON)
            .size(style.bell_size)
            .font(fonts::NERD)
            .tight()
            .color(theme.accent);

        return slot.child(bell);
    };

    let inner = style.icon_size - style.icon_margin * 2.0;

    // twice the size, so it stays sharp on a scaled screen
    let pixels = (inner * 2.0) as u32;

    slot.child(
        Rectangle::new()
            .width(inner)
            .height(inner)
            .fill(Image::contain(path).thumbnail(pixels, pixels)),
    )
}

fn icon_path(notification: &Notification) -> Option<PathBuf> {
    let path = find_icon(notification)?;

    // other formats, like xpm in old themes, get the bell
    let extension = path.extension()?.to_str()?.to_lowercase();

    if ["png", "jpg", "jpeg", "svg"].contains(&extension.as_str()) {
        Some(path)
    } else {
        None
    }
}

/*
 * the notification's own image, like a sender's avatar, comes first;
 * the icon is a file path, a file url, or a name from the icon theme; a
 * name is looked up through the installed apps, and with no icon at all
 * the app sending it may still have one
 */
fn find_icon(notification: &Notification) -> Option<PathBuf> {
    if let Some(path) = file_path(notification.image()) {
        return Some(path);
    }

    let icon = notification.icon();

    if let Some(path) = file_path(icon) {
        return Some(path);
    }

    let apps = Apps::read();

    for app in apps.list() {
        let same_icon = !icon.is_empty() && app.icon() == Some(icon);

        let same_app = icon.is_empty() && app.name().eq_ignore_ascii_case(notification.app_name());

        if same_icon || same_app {
            return app.icon_path().map(PathBuf::from);
        }
    }

    None
}

// none for an icon name or an empty string
fn file_path(text: &str) -> Option<PathBuf> {
    if let Some(url) = text.strip_prefix("file://") {
        return Some(PathBuf::from(decode_url(url)));
    }

    if text.starts_with('/') {
        return Some(PathBuf::from(text));
    }

    None
}

// a file url writes spaces and other bytes as %20 and the like
fn decode_url(url: &str) -> OsString {
    let bytes = url.as_bytes();

    let mut decoded = Vec::with_capacity(bytes.len());

    let mut index = 0;

    while index < bytes.len() {
        let escaped = bytes
            .get(index + 1..index + 3)
            .filter(|_| bytes[index] == b'%')
            .and_then(|hex| std::str::from_utf8(hex).ok())
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());

        if let Some(byte) = escaped {
            decoded.push(byte);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    OsString::from_vec(decoded)
}

// one button for each action the sender offered
fn actions(
    overlay: &Overlay,
    theme: &Theme,
    notification: &Notification,
    style: &CardStyle,
) -> Rectangle {
    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    for action in notification.actions() {
        let id = notification.id();
        let key = String::from(action.key());

        let hover_name = format!("notification:{id}:{key}");

        let amount = motion::fade(&hover_name, fade_target(overlay, &hover_name));

        let label = Text::new(action.label())
            .size(10.0)
            .font(fonts::BODY)
            .weight(Weight::SemiBold)
            .color(theme.text);

        let button = Rectangle::new()
            .width(size_of(&label) + BUTTON_PADDING * 2.0)
            .height(BUTTON_HEIGHT)
            .radius(8.0)
            .fill(theme::mix(theme.border, theme.selected_surface, amount))
            .align_child(Center, Center)
            .cursor(Pointer)
            .on_hover(move |inside| hover(hover_name.clone(), inside))
            .on_click(move |_| Notifications::invoke(id, &key))
            .child(label);

        buttons.push(Box::new(button));
    }

    let buttons = Row::new(buttons).gap(BUTTON_GAP);

    // lined up with the text, past the icon
    Rectangle::new()
        .width(Parent)
        .height(BUTTON_HEIGHT)
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: style.icon_size + style.icon_gap,
        })
        .child(buttons)
}

fn close_button(
    overlay: &Overlay,
    theme: &Theme,
    id: u32,
    card_width: f32,
    style: &CardStyle,
) -> Rectangle {
    let hover_name = format!("notification:{id}:close");

    let amount = motion::fade(&hover_name, fade_target(overlay, &hover_name));

    let glyph = Text::new(CLOSE_ICON)
        .size(style.close_glyph_size)
        .font(fonts::NERD)
        .tight()
        .color(theme::mix(theme.muted_text, theme.danger, amount));

    // a popup's button also lights up behind the glyph
    let backdrop = if style.popup {
        theme::mix(Color::TRANSPARENT, theme.border, amount)
    } else {
        Color::TRANSPARENT
    };

    let size = style.close_size;
    let inset = style.close_inset;

    let close = style.close;

    Rectangle::new()
        .width(size)
        .height(size)
        .translate(card_width - inset - size, inset)
        .radius(8.0)
        .fill(backdrop)
        .align_child(Center, Center)
        .cursor(Pointer)
        .on_hover(move |inside| hover(hover_name.clone(), inside))
        .on_click(move |_| close(id))
        .child(glyph)
}

// the card fades and its space closes first, then the notification is really gone
fn dismiss(id: u32) {
    {
        let mut overlay = Overlay::write();

        if overlay.dismissing.contains(&id) {
            return;
        }

        overlay.dismissing.push(id);
    }

    thread::spawn(move || {
        thread::sleep(motion::paced(DISMISS_DELAY));

        Notifications::dismiss(id);

        Overlay::write().dismissing.retain(|dismissed| *dismissed != id);
    });
}

// bodies may hold simple markup like <b>, which is left out
fn plain_text(body: &str) -> String {
    let mut text = String::new();
    let mut inside_tag = false;

    for character in body.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => text.push(character),
            _ => {}
        }
    }

    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn size_of(text: &Text) -> f32 {
    match text.width() {
        Size::Fixed(pixels) => pixels,
        Size::Parent => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_spaces_in_file_urls() {
        let path = decode_url("/home/me/Screenshot%20from%202026.png");

        assert_eq!(path, "/home/me/Screenshot from 2026.png");

        assert_eq!(decode_url("/100%.png"), "/100%.png");
    }
}

