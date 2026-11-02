use std::cell::RefCell;
use std::collections::HashMap;

use super::{FAST_EFFECTS, Glide};

thread_local! {
    /*
     * glides the view reads by name; kept outside the services because the
     * view sets their targets, and a service write from the view would draw
     * frames forever
     */
    static GLIDES: RefCell<HashMap<String, Glide>> = RefCell::new(HashMap::new());
}

// slides toward whatever the view wants this frame, starting where it was first asked for
pub fn follow(name: &str, target: f32, milliseconds: u64) -> f32 {
    glide(name, target, |value| super::spatial(value, milliseconds))
}

// the same, but a new one starts from 0, so its first target is played in
pub fn appear(name: &str, target: f32, milliseconds: u64) -> f32 {
    glide(name, target, |_| super::spatial(0.0, milliseconds))
}

// the same for a color change, 0 for the first color and 1 for the second
pub fn fade(name: &str, target: f32) -> f32 {
    glide(name, target, |value| super::effects(value, FAST_EFFECTS))
}

fn glide(name: &str, target: f32, start: impl FnOnce(f32) -> Glide) -> f32 {
    GLIDES.with_borrow_mut(|glides| {
        if !glides.contains_key(name) {
            glides.insert(String::from(name), start(target));
        }

        let glide = glides
            .get_mut(name)
            .expect("failed to find the glide just added");

        glide.to(target);

        glide.value()
    })
}
