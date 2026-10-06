#[derive(Clone, Copy, PartialEq)]
pub enum Group {
    Image,
    Video,
    Audio,
}

pub struct Format {
    pub extension: &'static str,
    pub group: Group,

    // what goes between ffmpeg's input and output
    pub arguments: &'static [&'static str],
}

// x264 needs an even width and height
const EVEN_SIZE: &str = "scale=trunc(iw/2)*2:trunc(ih/2)*2";

// a palette made from the clip itself keeps gifs from banding
const GIF_FILTER: &str = "fps=15,scale=480:-1:flags=lanczos,split[a][b];[a]palettegen[p];[b][p]paletteuse";

// images take the first frame, so a video turns into its opening still
pub const FORMATS: &[Format] = &[
    Format { extension: "png", group: Group::Image, arguments: &["-frames:v", "1"] },
    Format { extension: "jpg", group: Group::Image, arguments: &["-frames:v", "1", "-q:v", "2"] },
    Format { extension: "webp", group: Group::Image, arguments: &["-frames:v", "1", "-quality", "90"] },
    Format {
        extension: "avif",
        group: Group::Image,
        arguments: &["-frames:v", "1", "-c:v", "libaom-av1", "-still-picture", "1"],
    },
    Format { extension: "bmp", group: Group::Image, arguments: &["-frames:v", "1"] },
    Format {
        extension: "mp4",
        group: Group::Video,
        arguments: &[
            "-vf", EVEN_SIZE, "-c:v", "libx264", "-crf", "20", "-pix_fmt", "yuv420p", "-c:a", "aac",
            "-movflags", "+faststart",
        ],
    },
    Format {
        extension: "mov",
        group: Group::Video,
        arguments: &["-vf", EVEN_SIZE, "-c:v", "libx264", "-crf", "20", "-pix_fmt", "yuv420p", "-c:a", "aac"],
    },
    Format { extension: "mkv", group: Group::Video, arguments: &[] },
    Format {
        extension: "webm",
        group: Group::Video,
        arguments: &["-c:v", "libvpx-vp9", "-crf", "32", "-b:v", "0", "-row-mt", "1", "-c:a", "libopus"],
    },
    Format { extension: "gif", group: Group::Video, arguments: &["-vf", GIF_FILTER, "-loop", "0"] },
    Format { extension: "mp3", group: Group::Audio, arguments: &["-vn", "-c:a", "libmp3lame", "-q:a", "2"] },
    Format { extension: "m4a", group: Group::Audio, arguments: &["-vn", "-c:a", "aac", "-b:a", "256k"] },
    Format { extension: "opus", group: Group::Audio, arguments: &["-vn", "-c:a", "libopus", "-b:a", "160k"] },
    Format { extension: "ogg", group: Group::Audio, arguments: &["-vn", "-c:a", "libvorbis", "-q:a", "6"] },
    Format { extension: "flac", group: Group::Audio, arguments: &["-vn"] },
    Format { extension: "wav", group: Group::Audio, arguments: &["-vn"] },
];
