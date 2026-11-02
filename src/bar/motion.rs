use std::cell::RefCell;
use std::collections::HashMap;

use crate::motion::{self, Glide};

// milliseconds the highlight takes to slide and the star to turn
const DURATION: u64 = 420;

// where the active workspace highlight is, and how far its star has turned
struct Motion {
    active: usize,

    position: Glide,

    rotation: Glide,

    // where the star is turning to, counted apart so a switch mid-turn still lands straight
    turned: f32,
}

thread_local! {
    /*
     * kept outside services on purpose: a service write wakes the window,
     * and writing from inside view() would draw frames forever
     */
    static MOTIONS: RefCell<HashMap<String, Motion>> = RefCell::new(HashMap::new());
}

// the highlight's x offset and the star's rotation in degrees, for one monitor's strip
pub fn highlight(monitor: &str, active: usize, step: f32) -> (f32, f32) {
    MOTIONS.with_borrow_mut(|motions| {
        let motion = motions
            .entry(String::from(monitor))
            .or_insert_with(|| Motion {
                active,
                position: animation(active as f32 * step),
                rotation: animation(0.0),
                turned: 0.0,
            });

        if active != motion.active {
            // the star turns half a turn, the way the highlight moves
            let direction = if active > motion.active { 1.0 } else { -1.0 };

            motion.turned += direction * 180.0;

            motion.position.to(active as f32 * step);
            motion.rotation.to(motion.turned);

            motion.active = active;
        }

        (motion.position.value(), motion.rotation.value())
    })
}

fn animation(value: f32) -> Glide {
    motion::spatial(value, DURATION)
}
