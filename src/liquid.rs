use std::env;

use amane::{Color, Parent, Rectangle, Stack, Widget};

// the shader has room for this many blobs
const MAX_BLOBS: usize = 8;

const EDGE_OFFSET: f32 = 2.0;
// how far apart two shapes start to flow into each other
pub const CONNECTION: f32 = 24.0;

// a soft black shadow under the blobs, sitting a little lower than them
const SHADOW_OPACITY: f32 = 0.31;
const SHADOW_BLUR: f32 = 10.0;
const SHADOW_OFFSET: f32 = 4.0;

// a rounded rectangle of liquid with content on it, like a panel growing out of an edge
// where the window drawing the liquid sits, since the liquid melts into the screen's edges
pub struct Placement {
    pub x: f32,
    pub y: f32,

    pub screen_width: f32,
    pub screen_height: f32,
}

pub struct Blob {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
    content: Option<Box<dyn Widget>>,
}

impl Blob {
    // placed from the liquid's top-left corner, animate x or y to make it flow
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            radius: 0.0,
            content: None,
        }
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;

        self
    }

    pub fn child(mut self, content: impl Widget + 'static) -> Self {
        self.content = Some(Box::new(content));

        self
    }
}

/*
 * the shader draws every blob melted into the others and into the
 * screen's edges, then each blob's content is laid on top; blobs are
 * placed in screen coordinates
 */
pub fn view(color: Color, blobs: Vec<Blob>, placement: Placement) -> Stack {
    assert!(
        blobs.len() <= MAX_BLOBS,
        "the liquid holds at most {MAX_BLOBS} blobs"
    );

    let surface = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .shader(shader_path())
        .shader_values(values(color, &blobs, &placement));

    let mut layers: Vec<Box<dyn Widget>> = vec![Box::new(surface)];

    for blob in blobs {
        let Some(content) = blob.content else {
            continue;
        };

        // the content is already boxed, and a stack is what takes boxed widgets
        let holder = Rectangle::new()
            .width(blob.width)
            .height(blob.height)
            .translate(blob.x - placement.x, blob.y - placement.y)
            .child(Stack::new(vec![content]).width(Parent).height(Parent));

        layers.push(Box::new(holder));
    }

    Stack::new(layers).width(Parent).height(Parent)
}

// laid out the way liquid.wgsl reads them, one row of four numbers at a time
fn values(color: Color, blobs: &[Blob], placement: &Placement) -> Vec<[f32; 4]> {
    let color = [
        f32::from(color.red()) / 255.0,
        f32::from(color.green()) / 255.0,
        f32::from(color.blue()) / 255.0,
        f32::from(color.alpha()) / 255.0,
    ];

    let count = blobs.len() as f32;

    let mut values = vec![color, [EDGE_OFFSET, CONNECTION, count, 0.0]];

    let mut radii = [[0.0; 4]; 2];

    for index in 0..MAX_BLOBS {
        let Some(blob) = blobs.get(index) else {
            values.push([0.0; 4]);

            continue;
        };

        let x = blob.x - placement.x;
        let y = blob.y - placement.y;

        values.push([x, y, blob.width, blob.height]);

        radii[index / 4][index % 4] = blob.radius;
    }

    values.extend(radii);

    values.push([
        placement.x,
        placement.y,
        placement.screen_width,
        placement.screen_height,
    ]);

    values.push([SHADOW_OPACITY, SHADOW_BLUR, 0.0, SHADOW_OFFSET]);

    values
}

// the shader lives next to the config's src folder
fn shader_path() -> String {
    let home = env::var("HOME").expect("failed to find home: HOME is not set");

    format!("{home}/.config/amane/shaders/liquid.wgsl")
}
