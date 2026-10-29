use amane::Color;

/*
 * lightness 0 to 1, chroma (how colorful, about 0 to 0.37) and hue in degrees;
 * unlike hsl, two colors with the same lightness here also look as bright,
 * whatever their hue, so a palette made in it reads evenly
 */
pub struct Oklch {
    pub lightness: f32,
    pub chroma: f32,
    pub hue: f32,
}

pub fn from_color(color: Color) -> Oklch {
    let red = to_linear(color.red());
    let green = to_linear(color.green());
    let blue = to_linear(color.blue());

    let l = (0.412_221_47 * red + 0.536_332_55 * green + 0.051_445_99 * blue).cbrt();
    let m = (0.211_903_5 * red + 0.680_699_5 * green + 0.107_396_96 * blue).cbrt();
    let s = (0.088_302_46 * red + 0.281_718_85 * green + 0.629_978_7 * blue).cbrt();

    let lightness = 0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s;
    let a = 1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s;
    let b = 0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s;

    Oklch {
        lightness,
        chroma: a.hypot(b),
        hue: b.atan2(a).to_degrees().rem_euclid(360.0),
    }
}

/*
 * a color too colorful for the screen to show loses chroma until it fits,
 * keeping its lightness and hue, so it stays as bright and the same color
 */
pub fn to_color(oklch: Oklch) -> Color {
    let mut chroma = oklch.chroma;

    for _ in 0..24 {
        if let Some(color) = to_screen(oklch.lightness, chroma, oklch.hue) {
            return color;
        }

        chroma *= 0.92;
    }

    let grey = to_screen(oklch.lightness, 0.0, 0.0);

    grey.unwrap_or(Color::rgb(0, 0, 0))
}

// none when the screen can't show it
fn to_screen(lightness: f32, chroma: f32, hue: f32) -> Option<Color> {
    let (sin, cos) = hue.to_radians().sin_cos();

    let a = chroma * cos;
    let b = chroma * sin;

    let l = (lightness + 0.396_337_78 * a + 0.215_803_76 * b).powi(3);
    let m = (lightness - 0.105_561_346 * a - 0.063_854_17 * b).powi(3);
    let s = (lightness - 0.089_484_18 * a - 1.291_485_5 * b).powi(3);

    let red = 4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s;
    let green = -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s;
    let blue = -0.004_196_086 * l - 0.703_418_6 * m + 1.707_614_7 * s;

    // a hair of slack, so rounding doesn't push an edge color out
    let inside = |channel: f32| (-0.0001..=1.0001).contains(&channel);

    if !(inside(red) && inside(green) && inside(blue)) {
        return None;
    }

    Some(Color::rgb(to_srgb(red), to_srgb(green), to_srgb(blue)))
}

fn to_linear(channel: u8) -> f32 {
    let channel = f32::from(channel) / 255.0;

    if channel <= 0.040_45 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn to_srgb(channel: f32) -> u8 {
    let channel = channel.clamp(0.0, 1.0);

    let encoded = if channel <= 0.003_130_8 {
        channel * 12.92
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    };

    (encoded * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_color_comes_back_the_same() {
        for color in [
            Color::rgb(0x84, 0x23, 0x4c),
            Color::rgb(0x1e, 0x66, 0xf5),
            Color::rgb(0xdf, 0x8e, 0x1d),
            Color::rgb(0xff, 0xff, 0xff),
        ] {
            let back = to_color(from_color(color));

            assert_eq!(
                (back.red(), back.green(), back.blue()),
                (color.red(), color.green(), color.blue())
            );
        }
    }
}
