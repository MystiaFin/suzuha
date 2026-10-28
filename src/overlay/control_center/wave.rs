use std::f32::consts::TAU;
use std::time::{SystemTime, UNIX_EPOCH};

use amane::{Canvas, Cap, Circle, Color, Line, Path, Shape};

use crate::motion::{self, DEFAULT_SPATIAL};

pub const HEIGHT: f32 = 10.0;

const LINE: f32 = 4.0;
const WAVELENGTH: f32 = 40.0;
const AMPLITUDE: f32 = 3.0;

// the gap between the played part and the rest of the track
const GAP: f32 = 4.0 + LINE;

// the wave is drawn as short straight steps this far apart
const STEP: f32 = 2.0;

// one wavelength scrolls by each second
const PERIOD_MS: u128 = 1000;

/*
 * the played part is a line that waves while playing, the rest a flat
 * track, with a dot marking the end; near either end the wave would
 * crowd the edge, so it lies flat there
 */
pub fn view(width: f32, progress: f32, playing: bool, played: Color, track: Color) -> Canvas {
    let progress = progress.clamp(0.0, 1.0);

    let middle = HEIGHT / 2.0;
    let half_line = LINE / 2.0;

    let played_width = width * progress;

    let end_x = played_width.clamp(half_line, width - half_line);

    let waving = playing && progress > 0.1 && progress < 0.95;

    // 0 for a flat line, 1 for the full wave, so it swells and settles instead of snapping
    let target = if waving { 1.0 } else { 0.0 };

    let swell = motion::follow("media:wave", target, DEFAULT_SPATIAL);

    let mut shapes: Vec<Box<dyn Shape>> = Vec::new();

    let rest_start = (played_width + GAP).max(half_line);

    if rest_start < width - half_line {
        let rest = Line::new()
            .from(rest_start, middle)
            .to(width - half_line, middle)
            .stroke(LINE, track)
            .cap(Cap::Round);

        shapes.push(Box::new(rest));
    }

    if played_width > 0.0 {
        let line = if swell > 0.0 {
            wave(half_line, end_x, middle, AMPLITUDE * swell)
        } else {
            Path::new().move_to(half_line, middle).line_to(end_x, middle)
        };

        shapes.push(Box::new(line.stroke(LINE, played).cap(Cap::Round)));
    }

    shapes.push(Box::new(
        Circle::new()
            .center(width - half_line, middle)
            .radius(half_line)
            .fill(played),
    ));

    Canvas::new().width(width).height(HEIGHT).shapes(shapes)
}

// a sine from start to end, shifted along with the clock so it keeps flowing
fn wave(start: f32, end: f32, middle: f32, amplitude: f32) -> Path {
    // ponytail: a frame every refresh while the wave shows; lower the rate if it costs too much
    amane::request_frame();

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis())
        .unwrap_or(0);

    let shift = (millis % PERIOD_MS) as f32 / PERIOD_MS as f32 * WAVELENGTH;

    let height_at = |x: f32| middle + amplitude * ((x + shift) / WAVELENGTH * TAU).sin();

    let mut path = Path::new().move_to(start, height_at(start));

    let mut x = start + STEP;

    while x < end {
        path = path.line_to(x, height_at(x));

        x += STEP;
    }

    path.line_to(end, height_at(end))
}
