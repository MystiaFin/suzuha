use std::f32::consts::TAU;
use std::time::{SystemTime, UNIX_EPOCH};

use amane::{Arc, Canvas, Cap, Color, Path, Shape};

const LINE: f32 = 10.0;

// how far the wave swings off the circle, and how many waves fit in one turn
const AMPLITUDE: f32 = 4.0;
const WAVES: f32 = 14.0;

// the space between the played part and the flat track, in pixels along the circle
const GAP: f32 = 14.0;

// the wave is drawn as short straight steps this many degrees apart
const STEP: f32 = 1.5;

// one wavelength flows by each second
const PERIOD_MS: u128 = 1000;

/*
 * the part that has gone by runs around the circle from the top, the
 * rest is a flat track; the played part waves only while running, and
 * near either end it would crowd the gap, so it lies flat there
 */
pub fn view(size: f32, progress: f32, running: bool, played: Color, track: Color) -> Canvas {
    let progress = progress.clamp(0.0, 1.0);

    let middle = size / 2.0;

    let radius = middle - LINE / 2.0 - AMPLITUDE;

    let sweep = 360.0 * progress;

    let gap = (GAP + LINE) / radius * 360.0 / TAU;

    let waving = running && progress > 0.1 && progress < 0.95;

    let mut shapes: Vec<Box<dyn Shape>> = Vec::new();

    // the gap is on both ends, so the track needs room for two
    let rest = 360.0 - sweep - gap * 2.0;

    if sweep == 0.0 {
        shapes.push(Box::new(Arc::new().center(middle, middle).radius(radius).stroke(LINE, track)));
    } else if rest > 0.0 {
        let arc = Arc::new()
            .center(middle, middle)
            .radius(radius)
            .start(sweep + gap)
            .sweep(rest)
            .stroke(LINE, track)
            .cap(Cap::Round);

        shapes.push(Box::new(arc));
    }

    if sweep > 0.0 && waving {
        let wave = wave(middle, radius, sweep).stroke(LINE, played).cap(Cap::Round);

        shapes.push(Box::new(wave));
    } else if sweep > 0.0 {
        let arc = Arc::new()
            .center(middle, middle)
            .radius(radius)
            .sweep(sweep)
            .stroke(LINE, played)
            .cap(Cap::Round);

        shapes.push(Box::new(arc));
    }

    Canvas::new().width(size).height(size).shapes(shapes)
}

/*
 * a sine around the circle, shifted along with the clock so it keeps
 * flowing; it swings less near its ends so they meet the circle
 */
fn wave(middle: f32, radius: f32, sweep: f32) -> Path {
    amane::request_frame();

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis())
        .unwrap_or(0);

    let shift = (millis % PERIOD_MS) as f32 / PERIOD_MS as f32 * TAU;

    // degrees over which the swing grows from nothing at each end
    let taper = 360.0 / WAVES;

    let point = |degrees: f32| {
        let angle = degrees.to_radians();

        let room = degrees.min(sweep - degrees) / taper;

        let swing = AMPLITUDE * room.min(1.0) * (angle * WAVES - shift).sin();

        let distance = radius + swing;

        (middle + distance * angle.sin(), middle - distance * angle.cos())
    };

    let (x, y) = point(0.0);

    let mut path = Path::new().move_to(x, y);

    let mut degrees = STEP;

    while degrees < sweep {
        let (x, y) = point(degrees);

        path = path.line_to(x, y);

        degrees += STEP;
    }

    let (x, y) = point(sweep);

    path.line_to(x, y)
}
