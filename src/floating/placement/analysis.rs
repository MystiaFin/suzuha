/*
 * a small copy of the wallpaper as sums over every rectangle from the top
 * left corner, so any area's brightness, spread and detail is four lookups
 */
#[derive(Default)]
pub struct Analysis {
    pub width: usize,
    pub height: usize,

    // width + 1 by height + 1 each, the first row and column all 0
    luminance: Vec<f32>,
    squared: Vec<f32>,
    detail: Vec<f32>,
}

// an area of the small copy, in its pixels
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Area {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Analysis {
    // rgb bytes, three per pixel, row by row
    pub fn new(rgb: &[u8], width: usize, height: usize) -> Option<Self> {
        if width == 0 || height == 0 || rgb.len() != width * height * 3 {
            return None;
        }

        let mut luminance = vec![0.0; width * height];
        let mut squared = vec![0.0; width * height];

        // how much each pixel differs from the one left of it and the one above it
        let mut detail = vec![0.0; width * height];

        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;

                let red = f32::from(rgb[index * 3]);
                let green = f32::from(rgb[index * 3 + 1]);
                let blue = f32::from(rgb[index * 3 + 2]);

                let value = (red * 0.2126 + green * 0.7152 + blue * 0.0722) / 255.0;

                luminance[index] = value;
                squared[index] = value * value;

                let left = if x > 0 {
                    (value - luminance[index - 1]).abs()
                } else {
                    0.0
                };
                let above = if y > 0 {
                    (value - luminance[index - width]).abs()
                } else {
                    0.0
                };

                detail[index] = left + above;
            }
        }

        Some(Self {
            width,
            height,
            luminance: summed(&luminance, width, height),
            squared: summed(&squared, width, height),
            detail: summed(&detail, width, height),
        })
    }

    /*
     * higher is a better place: cards want calm, even areas; the clock has
     * no card behind it, so it also wants its text color to stand out
     */
    pub fn score(&self, area: Area, text_luminance: f32, text_only: bool) -> f32 {
        let size = (area.width * area.height).max(1) as f32;

        let mean = self.sum(&self.luminance, area) / size;
        let squared_mean = self.sum(&self.squared, area) / size;

        let spread = (squared_mean - mean * mean).max(0.0).sqrt();
        let detail = self.sum(&self.detail, area) / size;

        if !text_only {
            return -spread * 12.0 - detail * 10.0;
        }

        let contrast = (text_luminance.max(mean) + 0.05) / (text_luminance.min(mean) + 0.05);

        contrast.min(4.0) * 0.25 - spread * 6.0 - detail * 6.0
    }

    fn sum(&self, table: &[f32], area: Area) -> f32 {
        let stride = self.width + 1;

        let right = area.x + area.width;
        let bottom = area.y + area.height;

        table[bottom * stride + right]
            - table[area.y * stride + right]
            - table[bottom * stride + area.x]
            + table[area.y * stride + area.x]
    }
}

// each entry is the total of everything above and to the left of it
fn summed(values: &[f32], width: usize, height: usize) -> Vec<f32> {
    let stride = width + 1;

    let mut table = vec![0.0; stride * (height + 1)];

    for y in 1..=height {
        let mut row = 0.0;

        for x in 1..=width {
            row += values[(y - 1) * width + x - 1];

            table[y * stride + x] = table[(y - 1) * stride + x] + row;
        }
    }

    table
}

#[cfg(test)]
mod tests {
    use super::*;

    // a flat half scores better than a striped half
    #[test]
    fn calm_beats_busy() {
        let (width, height) = (8, 4);

        let mut rgb = Vec::new();

        for _ in 0..height {
            for x in 0..width {
                let value = if x < 4 {
                    128
                } else if x % 2 == 0 {
                    0
                } else {
                    255
                };

                rgb.extend([value, value, value]);
            }
        }

        let analysis = Analysis::new(&rgb, width, height).expect("failed to analyse");

        let calm = Area {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
        };
        let busy = Area {
            x: 4,
            y: 0,
            width: 4,
            height: 4,
        };

        assert!(analysis.score(calm, 1.0, false) > analysis.score(busy, 1.0, false));

        assert!(Analysis::new(&rgb[1..], width, height).is_none());
    }
}
