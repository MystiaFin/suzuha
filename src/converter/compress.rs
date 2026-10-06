#[derive(Clone, Copy, PartialEq)]
pub enum Level {
    Light,
    Medium,
    Strong,
}

// how long the encoder may take; slower finds a smaller file at the same quality
#[derive(Clone, Copy, PartialEq)]
pub enum Speed {
    Fast,
    Medium,
    Slow,
}

pub const LEVELS: [(Level, &str); 3] = [(Level::Light, "Light"), (Level::Medium, "Medium"), (Level::Strong, "Strong")];

pub const SPEEDS: [(Speed, &str); 3] = [(Speed::Fast, "Fast"), (Speed::Medium, "Medium"), (Speed::Slow, "Slow")];

// x264 needs an even width and height
const EVEN_SIZE: &str = "scale=trunc(iw/2)*2:trunc(ih/2)*2";

// fewer frames, a smaller size and fewer colors as the level goes up
const GIF_LIGHT: &str = "scale='min(iw,720)':-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=192:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=3:diff_mode=rectangle";
const GIF_MEDIUM: &str = "fps=15,scale='min(iw,480)':-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=128:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=4:diff_mode=rectangle";
const GIF_STRONG: &str = "fps=10,scale='min(iw,320)':-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=64:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle";

// a png only shrinks much once its colors are cut down to a palette
const PNG_MEDIUM: &str = "split[a][b];[a]palettegen=max_colors=256[p];[b][p]paletteuse=dither=sierra2_4a";
const PNG_STRONG: &str = "split[a][b];[a]palettegen=max_colors=64[p];[b][p]paletteuse=dither=sierra2_4a";

fn by_level(level: Level, values: [&'static str; 3]) -> &'static str {
    match level {
        Level::Light => values[0],
        Level::Medium => values[1],
        Level::Strong => values[2],
    }
}

fn by_speed(speed: Speed, values: [&'static str; 3]) -> &'static str {
    match speed {
        Speed::Fast => values[0],
        Speed::Medium => values[1],
        Speed::Slow => values[2],
    }
}

// what goes between ffmpeg's input and output to shrink a file of this kind, none for kinds it can't shrink
pub fn arguments(extension: &str, level: Level, speed: Speed) -> Option<Vec<&'static str>> {
    let extension = extension.to_lowercase();

    let arguments = match extension.as_str() {
        "mp4" | "mov" | "mkv" | "m4v" => {
            let mut arguments = vec![
                "-vf", EVEN_SIZE,
                "-c:v", "libx264",
                "-crf", by_level(level, ["23", "28", "34"]),
                "-preset", by_speed(speed, ["veryfast", "medium", "slow"]),
                "-pix_fmt", "yuv420p",
                "-c:a", "aac",
                "-b:a", by_level(level, ["160k", "128k", "96k"]),
            ];

            if extension != "mkv" {
                arguments.extend(["-movflags", "+faststart"]);
            }

            arguments
        }

        "webm" => vec![
            "-c:v", "libvpx-vp9",
            "-crf", by_level(level, ["34", "40", "46"]),
            "-b:v", "0",
            "-cpu-used", by_speed(speed, ["6", "3", "1"]),
            "-row-mt", "1",
            "-c:a", "libopus",
            "-b:a", by_level(level, ["128k", "96k", "64k"]),
        ],

        "gif" => vec!["-vf", by_level(level, [GIF_LIGHT, GIF_MEDIUM, GIF_STRONG]), "-loop", "0"],

        "jpg" | "jpeg" => vec!["-frames:v", "1", "-q:v", by_level(level, ["5", "9", "15"])],

        "webp" => vec![
            "-frames:v", "1",
            "-quality", by_level(level, ["80", "65", "45"]),
            "-compression_level", by_speed(speed, ["2", "4", "6"]),
        ],

        "avif" => vec![
            "-frames:v", "1",
            "-c:v", "libaom-av1",
            "-still-picture", "1",
            "-crf", by_level(level, ["28", "36", "44"]),
            "-cpu-used", by_speed(speed, ["8", "4", "2"]),
        ],

        "png" => match level {
            Level::Light => vec!["-frames:v", "1", "-compression_level", "9"],
            Level::Medium => vec!["-frames:v", "1", "-vf", PNG_MEDIUM, "-compression_level", "9"],
            Level::Strong => vec!["-frames:v", "1", "-vf", PNG_STRONG, "-compression_level", "9"],
        },

        _ => return None,
    };

    Some(arguments)
}
