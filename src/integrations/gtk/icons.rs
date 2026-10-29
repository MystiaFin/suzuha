use std::fs;
use std::path::{Path, PathBuf};

use super::super::{hex, home, write};
use super::glyphs::GLYPHS;
use crate::theme::{self, Theme};

// where the material design glyphs go, at any size
const GLYPH_FOLDER: &str = "scalable/amane";

// the blues papirus draws its folders with: the front, the back and the emblem on it
const FRONT: &str = "#5294e2";
const BACK: &str = "#4877b1";
const EMBLEM: &str = "#1d344f";

/*
 * an icon theme for one mode: the places and the toolbar drawn with the
 * glyphs the settings window uses, papirus' folders tinted with the accent,
 * and papirus itself for everything else
 */
pub fn export(folder: &str, name: &str, theme: &Theme) {
    let mut folders = vec![Folder::scalable(GLYPH_FOLDER, 8, 512)];

    // symbolic icons are recolored by gtk to the text around them, the rest keep this
    let plain = hex(theme.secondary_text);

    for (names, outline) in GLYPHS {
        for icon in names {
            write(&format!("{folder}/{GLYPH_FOLDER}/{icon}.svg"), &glyph(outline, &plain));
            write(&format!("{folder}/{GLYPH_FOLDER}/{icon}-symbolic.svg"), &glyph(outline, "#bebebe"));
        }
    }

    if let Some(papirus) = papirus() {
        folders.extend(tint_folders(&papirus, folder, theme));
    }

    let inherits = if theme.light { "Papirus" } else { "Papirus-Dark" };

    write(&format!("{folder}/index.theme"), &index(name, inherits, &folders));
}

fn glyph(outline: &str, fill: &str) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 512 512\">\
         <path fill=\"{fill}\" d=\"{outline}\"/></svg>\n"
    )
}

// one folder of the theme, as index.theme lists it
struct Folder {
    path: String,
    size: u32,

    // the sizes a scalable folder serves, none for a fixed one
    range: Option<(u32, u32)>,
}

impl Folder {
    fn scalable(path: &str, min: u32, max: u32) -> Self {
        Self {
            path: String::from(path),
            size: min.max(16),
            range: Some((min, max)),
        }
    }

    fn fixed(path: &str, size: u32) -> Self {
        Self {
            path: String::from(path),
            size,
            range: None,
        }
    }
}

// the first papirus found where icon themes are installed
fn papirus() -> Option<PathBuf> {
    let data_dirs = std::env::var("XDG_DATA_DIRS").unwrap_or_default();

    let profile = format!("{}/.nix-profile/share", home());

    data_dirs
        .split(':')
        .chain([profile.as_str(), "/run/current-system/sw/share", "/usr/share"])
        .map(|data| Path::new(data).join("icons/Papirus"))
        .find(|papirus| papirus.join("index.theme").exists())
}

// every size of papirus' blue folders, recolored; returns the folders written
fn tint_folders(papirus: &Path, folder: &str, theme: &Theme) -> Vec<Folder> {
    // the accent at papirus' own lightnesses, so folders weigh the same in either mode
    let colors = [
        (FRONT, hex(theme::retone(theme.accent, 0.68, 0.30))),
        (BACK, hex(theme::retone(theme.accent, 0.58, 0.30))),
        (EMBLEM, hex(theme::retone(theme.accent, 0.30, 0.20))),
    ];

    let mut written = Vec::new();

    let Ok(sizes) = fs::read_dir(papirus) else {
        return written;
    };

    for entry in sizes.flatten() {
        let size_name = entry.file_name().to_string_lossy().into_owned();

        // 48x48, but not the 48x48@2x links back to it
        let Some((width, _)) = size_name.split_once('x') else {
            continue;
        };

        let Ok(size) = width.parse::<u32>() else {
            continue;
        };

        if size_name.contains('@') {
            continue;
        }

        let Ok(places) = fs::read_dir(entry.path().join("places")) else {
            continue;
        };

        let mut any = false;

        for icon in places.flatten() {
            let file_name = icon.file_name().to_string_lossy().into_owned();

            // folder-blue.svg and folder-blue-music.svg, not folder-bluegrey
            let Some(rest) = file_name.strip_prefix("folder-blue") else {
                continue;
            };

            if !(rest.starts_with('-') || rest == ".svg") {
                continue;
            }

            let Ok(mut svg) = fs::read_to_string(icon.path()) else {
                continue;
            };

            for (from, to) in &colors {
                svg = svg.replace(from, to);
            }

            write(&format!("{folder}/{size_name}/places/folder{rest}"), &svg);

            any = true;
        }

        if any {
            written.push(Folder::fixed(&format!("{size_name}/places"), size));
        }
    }

    written
}

fn index(name: &str, inherits: &str, folders: &[Folder]) -> String {
    let listed: Vec<&str> = folders.iter().map(|folder| folder.path.as_str()).collect();

    let mut text = format!(
        "[Icon Theme]\nName={name}\nComment=Wallpaper-derived amane icons\n\
         Inherits={inherits},hicolor\nDirectories={}\n",
        listed.join(",")
    );

    for folder in folders {
        text.push_str(&format!("\n[{}]\nSize={}\nContext=Places\n", folder.path, folder.size));

        match folder.range {
            Some((min, max)) => text.push_str(&format!("Type=Scalable\nMinSize={min}\nMaxSize={max}\n")),
            None => text.push_str("Type=Fixed\n"),
        }
    }

    text
}
