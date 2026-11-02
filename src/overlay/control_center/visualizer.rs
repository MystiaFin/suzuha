use amane::{End, Rectangle, Row, Service, Widget};

use super::cava::{BARS, Cava};
use crate::motion::{self, FAST_EFFECTS};
use crate::theme::Theme;

pub const HEIGHT: f32 = 64.0;

const GAP: f32 = 3.0;

// a silent band still shows as a dot
const MIN_HEIGHT: f32 = 3.0;

// one rounded bar per cava band, rising from the bottom
pub fn view(theme: &Theme, width: f32) -> Row {
    let levels = *Cava::read().levels();

    let bar_width = (width - GAP * (BARS - 1) as f32) / BARS as f32;

    let mut bars: Vec<Box<dyn Widget>> = Vec::new();

    for (index, level) in levels.iter().enumerate() {
        let target = (HEIGHT * level).max(MIN_HEIGHT);

        let height = motion::follow(&format!("cava:{index}"), target, FAST_EFFECTS);

        let bar = Rectangle::new()
            .width(bar_width)
            .height(height)
            .radius(bar_width / 2.0)
            .fill(theme.accent);

        bars.push(Box::new(bar));
    }

    Row::new(bars)
        .width(width)
        .height(HEIGHT)
        .gap(GAP)
        .align(End)
}
