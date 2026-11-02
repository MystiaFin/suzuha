use std::sync::Mutex;
use std::time::Instant;

/*
 * the most time one look at the spring counts; a window that stalls, like
 * one drawing at a new size for the first time, slows the motion for a
 * moment instead of skipping most of it
 */
const MAX_STEP: f32 = 1.0 / 30.0;

/*
 * a value pulled toward its target by a spring and slowed by damping; unlike
 * a timed animation it has no fixed length, and a new target keeps the speed it
 * already has, so changing direction halfway never jerks
 */
pub struct Spring {
    // how hard it pulls, higher is quicker
    stiffness: f32,

    // how much it resists moving; twice the square root of stiffness stops without overshooting
    damping: f32,

    // how close counts as arrived
    precision: f32,

    from: f32,
    velocity: f32,
    target: f32,

    clock: Mutex<Clock>,
}

// the time since the spring started, counted a capped step at a time
struct Clock {
    elapsed: f32,
    last: Instant,
}

impl Clock {
    fn new() -> Self {
        Self {
            elapsed: 0.0,
            last: Instant::now(),
        }
    }

    fn advance(&mut self) -> f32 {
        let now = Instant::now();

        // the animation speed setting makes time pass faster or slower for the spring
        let step = now.duration_since(self.last).as_secs_f32().min(MAX_STEP) * super::speed();

        self.elapsed += step;
        self.last = now;

        self.elapsed
    }
}

impl Spring {
    // starts at rest on `value`, with a spring that settles in about a third of a second
    pub fn new(value: f32) -> Self {
        Self {
            stiffness: 500.0,
            damping: 44.72,
            precision: 0.001,
            from: value,
            velocity: 0.0,
            target: value,
            clock: Mutex::new(Clock::new()),
        }
    }

    pub fn stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;

        self
    }

    pub fn damping(mut self, damping: f32) -> Self {
        self.damping = damping;

        self
    }

    // 0.001 suits values from 0 to 1, pixel values are fine with 0.1
    pub fn precision(mut self, precision: f32) -> Self {
        self.precision = precision;

        self
    }

    // the damping can change on the way, like a panel that closes firmer than it opens
    pub fn set_damping(&mut self, damping: f32) {
        self.restart();

        self.damping = damping;
    }

    pub fn to(&mut self, target: f32) {
        if target == self.target {
            return;
        }

        self.restart();

        self.target = target;
    }

    // read in the view, and while it is still moving the window keeps drawing frames
    pub fn value(&self) -> f32 {
        if super::reduced() {
            return self.target;
        }

        let (position, velocity) = self.state();

        let near = (position - self.target).abs() <= self.precision;
        let slow = velocity.abs() <= self.precision;

        if near && slow {
            return self.target;
        }

        amane::request_frame();

        position
    }

    // carries on from where it is and how fast it goes right now
    fn restart(&mut self) {
        let (position, velocity) = self.state();

        self.from = position;
        self.velocity = velocity;
        *self.clock.get_mut().expect("failed to lock spring clock") = Clock::new();
    }

    /*
     * where it is and how fast it moves, worked out exactly from the time since
     * it started, so reading it never depends on how often frames are drawn
     */
    fn state(&self) -> (f32, f32) {
        let time = self
            .clock
            .lock()
            .expect("failed to lock spring clock")
            .advance();

        let offset = self.from - self.target;
        let speed = self.velocity;

        // how fast it would swing without damping, and how damped it is
        let natural = self.stiffness.sqrt();
        let ratio = self.damping / (2.0 * natural);

        let (offset, speed) = if (ratio - 1.0).abs() < 0.001 {
            critical(offset, speed, natural, time)
        } else if ratio < 1.0 {
            under(offset, speed, natural, ratio, time)
        } else {
            over(offset, speed, natural, ratio, time)
        };

        (self.target + offset, speed)
    }
}

// the quickest stop without overshooting
fn critical(offset: f32, speed: f32, natural: f32, time: f32) -> (f32, f32) {
    let first = offset;
    let second = speed + natural * offset;

    let decay = (-natural * time).exp();

    let position = (first + second * time) * decay;
    let velocity = (second - natural * (first + second * time)) * decay;

    (position, velocity)
}

// swings past the target a little before settling
fn under(offset: f32, speed: f32, natural: f32, ratio: f32, time: f32) -> (f32, f32) {
    let swing = natural * (1.0 - ratio * ratio).sqrt();
    let fade = ratio * natural;

    let first = offset;
    let second = (speed + fade * offset) / swing;

    let decay = (-fade * time).exp();

    let (sin, cos) = (swing * time).sin_cos();

    let wave = first * cos + second * sin;
    let wave_speed = -first * swing * sin + second * swing * cos;

    let position = decay * wave;
    let velocity = decay * (wave_speed - fade * wave);

    (position, velocity)
}

// creeps in without swinging, slower than critical
fn over(offset: f32, speed: f32, natural: f32, ratio: f32, time: f32) -> (f32, f32) {
    let spread = (ratio * ratio - 1.0).sqrt();

    let fast = -natural * (ratio + spread);
    let slow = -natural * (ratio - spread);

    let second = (speed - slow * offset) / (fast - slow);
    let first = offset - second;

    let slow_part = first * (slow * time).exp();
    let fast_part = second * (fast * time).exp();

    let position = slow_part + fast_part;
    let velocity = slow * slow_part + fast * fast_part;

    (position, velocity)
}
