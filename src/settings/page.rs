use amane::{Column, Rectangle, Stack, Widget};

use crate::theme::{self, Theme};

const GROUP_GAP: f32 = 12.0;
const GROUP_RADIUS: f32 = 20.0;

// the line between two rows of a group, kept off the card's rounded sides
const SEPARATOR_INSET: f32 = 20.0;

/*
 * one settings page being put together: rows go into the open group, and a
 * finished group becomes a card; it counts how far down each row starts, so
 * dropdown menus can be laid over the rows below them
 */
pub struct Page<'a> {
    pub theme: &'a Theme,
    pub width: f32,

    // where the next row starts, from the page's top
    pub y: f32,

    cards: Vec<Box<dyn Widget>>,
    rows: Vec<Box<dyn Widget>>,
    group_height: f32,

    menus: Vec<Box<dyn Widget>>,
    bottom: f32,
}

impl<'a> Page<'a> {
    pub fn new(theme: &'a Theme, width: f32) -> Self {
        Self {
            theme,
            width,
            y: 0.0,
            cards: Vec::new(),
            rows: Vec::new(),
            group_height: 0.0,
            menus: Vec::new(),
            bottom: 0.0,
        }
    }

    // a row in the open group, under a thin line when it isn't the first
    pub fn row(&mut self, height: f32, row: impl Widget + 'static) {
        // a new group starts a gap below the card before it
        if self.rows.is_empty() && !self.cards.is_empty() {
            self.y += GROUP_GAP;
        }

        if !self.rows.is_empty() {
            self.rows.push(Box::new(separator(self.theme, self.width)));

            self.group_height += 1.0;
            self.y += 1.0;
        }

        self.rows.push(Box::new(row));

        self.group_height += height;
        self.y += height;
    }

    // closes the open group into a card
    pub fn end_group(&mut self) {
        if self.rows.is_empty() {
            return;
        }

        let rows = std::mem::take(&mut self.rows);

        let card = Rectangle::new()
            .width(self.width)
            .height(self.group_height)
            .radius(GROUP_RADIUS)
            .fill(self.theme.surface)
            .clip()
            .child(Column::new(rows));

        self.cards.push(Box::new(card));

        self.group_height = 0.0;
    }

    // a card of its own, like the palette preview
    pub fn card(&mut self, height: f32, card: impl Widget + 'static) {
        self.end_group();

        if !self.cards.is_empty() {
            self.y += GROUP_GAP;
        }

        self.cards.push(Box::new(card));

        self.y += height;
    }

    // laid over everything, already moved to where it belongs
    pub fn menu(&mut self, menu: impl Widget + 'static, bottom: f32) {
        self.menus.push(Box::new(menu));

        self.bottom = self.bottom.max(bottom);
    }

    pub fn finish(mut self) -> Stack {
        self.end_group();

        let height = self.y.max(self.bottom);

        let mut layers: Vec<Box<dyn Widget>> =
            vec![Box::new(Column::new(self.cards).gap(GROUP_GAP))];

        layers.append(&mut self.menus);

        Stack::new(layers).width(self.width).height(height)
    }
}

fn separator(theme: &Theme, width: f32) -> Rectangle {
    let line = Rectangle::new()
        .width(width - SEPARATOR_INSET * 2.0)
        .height(1.0)
        .fill(theme::with_opacity(theme.border, 0.65))
        .translate(SEPARATOR_INSET, 0.0);

    Rectangle::new().width(width).height(1.0).child(line)
}
