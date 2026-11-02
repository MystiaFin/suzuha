use std::collections::HashMap;

use super::analysis::{Analysis, Area};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Name {
    Clock,
    Weather,
    CpuTemperature,
    CpuUsage,
    GpuTemperature,
    UvIndex,
    Humidity,
    AirQuality,
}

// the cards to place and the screen they go on, sizes in screen pixels
pub struct Request<'a> {
    pub screen: (f32, f32),

    // where cards may go, the screen minus its margins: x, y, width, height
    pub usable: (f32, f32, f32, f32),

    pub clock: (f32, f32),

    // shown with the clock, like the weather
    pub primary: Vec<(Name, f32, f32)>,

    pub resources: Vec<(Name, f32, f32)>,
    pub environment: Vec<(Name, f32, f32)>,

    // the clock's text brightness, 0 to 1
    pub text_luminance: f32,

    pub analysis: &'a Analysis,

    // the part of the small copy this screen shows, the wallpaper being cropped to cover it
    pub crop: Area,
}

// a card in the small copy's pixels, inside the crop
#[derive(Clone)]
struct Card {
    name: Name,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

// cards in a row or a column, measured from its own corner
struct Layout {
    width: i32,
    height: i32,
    cards: Vec<Card>,
    vertical: bool,
}

// a layout put somewhere, and how good a spot that is
#[derive(Clone)]
struct Placement {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    cards: Vec<Card>,
    score: f32,
}

// how far apart cards stay, in screen pixels
const CARD_GAP: f32 = 12.0;

// the edge spots a group may take, as shares of the free space; few, so the result is predictable
const ANCHORS: [(f32, f32); 8] = [
    (0.0, 0.0),
    (0.5, 0.0),
    (1.0, 0.0),
    (0.0, 0.5),
    (1.0, 0.5),
    (0.0, 1.0),
    (0.5, 1.0),
    (1.0, 1.0),
];

// the screen's middle, which cards should leave to the wallpaper
const CENTER: (f32, f32, f32, f32) = (0.25, 0.16, 0.5, 0.68);

// the context every step needs
struct Search<'a> {
    request: &'a Request<'a>,

    // the crop's size, and the usable area inside it
    width: i32,
    height: i32,
    bounds: (i32, i32, i32, i32),

    gap: i32,
}

/*
 * tries the clock with the main group, or apart from it, at every anchor;
 * then the resource and environment groups around them; keeps the calmest
 * set, or none when nothing fits
 */
pub fn arrange(request: &Request) -> Option<HashMap<Name, (f32, f32)>> {
    let search = Search::new(request);

    let clock = search.cards(&[(Name::Clock, request.clock.0, request.clock.1)]);
    let primary = search.cards(&request.primary);
    let resources = search.cards(&request.resources);
    let environment = search.cards(&request.environment);

    let mut candidates: Vec<(f32, Vec<Placement>)> = Vec::new();

    let mut joined = clock.clone();
    joined.extend(primary.iter().cloned());

    for layout in [search.row(&joined), search.column(&joined)] {
        for placement in search.placements(&layout, &[], false) {
            candidates.push((placement.score + 0.2, vec![placement]));
        }
    }

    let clock_spots = search.placements(&search.row(&clock), &[], true);

    for layout in [search.row(&primary), search.column(&primary)] {
        for group in search.placements(&layout, &[], false) {
            for spot in &clock_spots {
                if overlaps(&group, spot, search.gap * 2) {
                    continue;
                }

                let score = (group.score * 2.0 + spot.score) / 3.0;

                candidates.push((score, vec![group.clone(), spot.clone()]));
            }
        }
    }

    candidates.sort_by(|first, second| second.0.total_cmp(&first.0));
    candidates.truncate(16);

    let mut best: Option<(f32, Vec<Placement>)> = None;

    for (score, mut occupied) in candidates {
        let Some(resource_group) = search.best_group(&resources, &occupied) else {
            continue;
        };

        occupied.push(resource_group);

        let Some(environment_group) = search.best_group(&environment, &occupied) else {
            continue;
        };

        occupied.push(environment_group);

        let groups_score: f32 = occupied[occupied.len() - 2..]
            .iter()
            .map(|group| group.score)
            .sum();

        let total = score + groups_score - search.balance_penalty(&occupied);

        let better = match &best {
            Some((best_score, _)) => total > *best_score,
            None => true,
        };

        if better {
            best = Some((total, occupied));
        }
    }

    let (_, groups) = best?;

    let mut positions = HashMap::new();

    for group in groups {
        for card in group.cards {
            let x = card.x as f32 / search.width as f32 * request.screen.0;
            let y = card.y as f32 / search.height as f32 * request.screen.1;

            positions.insert(card.name, (x, y));
        }
    }

    Some(positions)
}

impl<'a> Search<'a> {
    fn new(request: &'a Request<'a>) -> Self {
        let width = request.crop.width as i32;
        let height = request.crop.height as i32;

        let (left, top, right, bottom) = {
            let (x, y, usable_width, usable_height) = request.usable;

            (
                search_x(request, x, width),
                search_y(request, y, height),
                search_x(request, x + usable_width, width),
                search_y(request, y + usable_height, height),
            )
        };

        let gap = (CARD_GAP / request.screen.0 * width as f32)
            .round()
            .max(1.0) as i32;

        Self {
            request,
            width,
            height,
            bounds: (left, top, (right - left).max(0), (bottom - top).max(0)),
            gap,
        }
    }

    // screen sizes turned into the small copy's pixels, at least 2 so every card counts
    fn cards(&self, sizes: &[(Name, f32, f32)]) -> Vec<Card> {
        let mut cards = Vec::new();

        for (name, width, height) in sizes {
            cards.push(Card {
                name: *name,
                x: 0,
                y: 0,
                width: search_x(self.request, *width, self.width).max(2),
                height: search_y(self.request, *height, self.height).max(2),
            });
        }

        cards
    }

    // side by side, centered on the tallest
    fn row(&self, cards: &[Card]) -> Layout {
        let height = cards.iter().map(|card| card.height).max().unwrap_or(0);

        let mut x = 0;
        let mut placed = Vec::new();

        for card in cards {
            placed.push(Card {
                x,
                y: (height - card.height) / 2,
                ..card.clone()
            });

            x += card.width + self.gap;
        }

        let width = (x - self.gap).max(0);

        Layout {
            width,
            height,
            cards: placed,
            vertical: false,
        }
    }

    // stacked, lined up on the left for now
    fn column(&self, cards: &[Card]) -> Layout {
        let width = cards.iter().map(|card| card.width).max().unwrap_or(0);

        let mut y = 0;
        let mut placed = Vec::new();

        for card in cards {
            placed.push(Card { y, ..card.clone() });

            y += card.height + self.gap;
        }

        let height = (y - self.gap).max(0);

        Layout {
            width,
            height,
            cards: placed,
            vertical: true,
        }
    }

    // the layout at every anchor that fits and stays clear of what is taken, best first
    fn placements(
        &self,
        layout: &Layout,
        occupied: &[Placement],
        text_only: bool,
    ) -> Vec<Placement> {
        let (left, top, free_width, free_height) = self.bounds;

        let mut placements = Vec::new();

        if layout.width > free_width || layout.height > free_height {
            return placements;
        }

        for (along, down) in ANCHORS {
            let x = left + ((free_width - layout.width) as f32 * along).round() as i32;
            let y = top + ((free_height - layout.height) as f32 * down).round() as i32;

            let mut placement = self.put(layout, x, y);

            if occupied
                .iter()
                .any(|taken| overlaps(&placement, taken, self.gap * 2))
            {
                continue;
            }

            placement.score = self.score(&placement, text_only) - self.center_penalty(&placement);

            placements.push(placement);
        }

        placements.sort_by(|first, second| {
            second
                .score
                .total_cmp(&first.score)
                .then(first.y.cmp(&second.y))
                .then(first.x.cmp(&second.x))
        });

        placements
    }

    // a column on the right half lines its cards up on the right, toward the edge
    fn put(&self, layout: &Layout, x: i32, y: i32) -> Placement {
        let align_right = layout.vertical && x + layout.width / 2 >= self.width / 2;

        let mut cards = Vec::new();

        for card in &layout.cards {
            let offset = if align_right {
                layout.width - card.width
            } else {
                card.x
            };

            cards.push(Card {
                x: x + offset,
                y: y + card.y,
                ..card.clone()
            });
        }

        Placement {
            x,
            y,
            width: layout.width,
            height: layout.height,
            cards,
            score: 0.0,
        }
    }

    fn best_group(&self, cards: &[Card], occupied: &[Placement]) -> Option<Placement> {
        let mut best: Option<Placement> = None;

        for layout in [self.row(cards), self.column(cards)] {
            let Some(first) = self.placements(&layout, occupied, false).into_iter().next() else {
                continue;
            };

            if best.as_ref().is_none_or(|best| first.score > best.score) {
                best = Some(first);
            }
        }

        best
    }

    // the average over the group's cards, looked up inside the crop
    fn score(&self, placement: &Placement, text_only: bool) -> f32 {
        let crop = self.request.crop;

        let mut total = 0.0;

        for card in &placement.cards {
            let area = Area {
                x: crop.x + card.x.max(0) as usize,
                y: crop.y + card.y.max(0) as usize,
                width: card.width as usize,
                height: card.height as usize,
            };

            total += self
                .request
                .analysis
                .score(area, self.request.text_luminance, text_only);
        }

        total / placement.cards.len().max(1) as f32
    }

    fn center_penalty(&self, placement: &Placement) -> f32 {
        let (x, y, width, height) = CENTER;

        let center = Placement {
            x: (self.width as f32 * x).round() as i32,
            y: (self.height as f32 * y).round() as i32,
            width: (self.width as f32 * width).round() as i32,
            height: (self.height as f32 * height).round() as i32,
            cards: Vec::new(),
            score: 0.0,
        };

        let size = (placement.width * placement.height).max(1) as f32;

        shared_area(placement, &center) as f32 / size * 5.0
    }

    // everything on one side of the screen looks lopsided
    fn balance_penalty(&self, groups: &[Placement]) -> f32 {
        let mut left = 0;

        for group in groups {
            if (group.x + group.width / 2) < self.width / 2 {
                left += 1;
            }
        }

        let right = groups.len() as i32 - left;

        (left - right).abs() as f32 * 0.35
    }
}

// screen pixels across or down turned into the crop's pixels
fn search_x(request: &Request, value: f32, width: i32) -> i32 {
    (value / request.screen.0 * width as f32).round() as i32
}

fn search_y(request: &Request, value: f32, height: i32) -> i32 {
    (value / request.screen.1 * height as f32).round() as i32
}

fn overlaps(first: &Placement, second: &Placement, margin: i32) -> bool {
    first.x < second.x + second.width + margin
        && first.x + first.width + margin > second.x
        && first.y < second.y + second.height + margin
        && first.y + first.height + margin > second.y
}

fn shared_area(first: &Placement, second: &Placement) -> i32 {
    let width = (first.x + first.width).min(second.x + second.width) - first.x.max(second.x);
    let height = (first.y + first.height).min(second.y + second.height) - first.y.max(second.y);

    width.max(0) * height.max(0)
}
