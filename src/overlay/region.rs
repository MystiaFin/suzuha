// a rectangle on the screen, below the bar
#[derive(Clone, Copy)]
pub struct Region {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Region {
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    // the smallest region holding both
    pub fn join(&self, other: Region) -> Region {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);

        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());

        Region {
            x,
            y,
            width: right - x,
            height: bottom - y,
        }
    }

    // grown out to whole pixels, the only steps a window is placed and sized in
    pub fn snapped(&self) -> Region {
        let x = self.x.floor();
        let y = self.y.floor();

        Region {
            x,
            y,
            width: self.right().ceil() - x,
            height: self.bottom().ceil() - y,
        }
    }

    // the part inside other, empty when they don't touch
    pub fn within(&self, other: Region) -> Region {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);

        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        Region {
            x,
            y,
            width: (right - x).max(0.0),
            height: (bottom - y).max(0.0),
        }
    }
}
