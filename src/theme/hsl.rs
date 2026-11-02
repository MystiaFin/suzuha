use amane::Color;

// hue in degrees 0 to 360, saturation and lightness 0 to 1
pub struct Hsl {
    pub hue: f32,
    pub saturation: f32,
    pub lightness: f32,
}

pub fn from_color(color: Color) -> Hsl {
    let red = f32::from(color.red()) / 255.0;
    let green = f32::from(color.green()) / 255.0;
    let blue = f32::from(color.blue()) / 255.0;

    let highest = red.max(green).max(blue);
    let lowest = red.min(green).min(blue);

    let lightness = (highest + lowest) / 2.0;

    let spread = highest - lowest;

    // grey has no hue and no saturation
    if spread == 0.0 {
        return Hsl {
            hue: 0.0,
            saturation: 0.0,
            lightness,
        };
    }

    let saturation = spread / (1.0 - (2.0 * lightness - 1.0).abs());

    let sector = if highest == red {
        ((green - blue) / spread).rem_euclid(6.0)
    } else if highest == green {
        (blue - red) / spread + 2.0
    } else {
        (red - green) / spread + 4.0
    };

    Hsl {
        hue: sector * 60.0,
        saturation,
        lightness,
    }
}

pub fn to_color(hsl: Hsl) -> Color {
    let chroma = (1.0 - (2.0 * hsl.lightness - 1.0).abs()) * hsl.saturation;

    let sector = hsl.hue / 60.0;

    let second = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());

    let (red, green, blue) = match sector as u32 {
        0 => (chroma, second, 0.0),
        1 => (second, chroma, 0.0),
        2 => (0.0, chroma, second),
        3 => (0.0, second, chroma),
        4 => (second, 0.0, chroma),
        _ => (chroma, 0.0, second),
    };

    let lift = hsl.lightness - chroma / 2.0;

    Color::rgb(
        channel(red + lift),
        channel(green + lift),
        channel(blue + lift),
    )
}

fn channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
