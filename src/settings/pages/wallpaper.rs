use super::super::page::Page;
use super::super::slider::{self, Range};
use super::super::{choice, field, switch};

pub fn build(page: &mut Page) {
    field::add(
        page,
        "wallpaper_folder",
        "Wallpaper directory",
        "Folder scanned by the wallpaper picker and shuffle timer",
    );

    choice::add(
        page,
        "wallpaper_transition",
        "Transition",
        "Choose how the new wallpaper replaces the current one",
        &[
            ("Circle", "circle"),
            ("Fade", "fade"),
            ("Instant", "instant"),
        ],
    );

    slider::add(
        page,
        "wallpaper_duration",
        "Transition duration",
        "Duration used by wallpaper reveal and fade animations",
        Range {
            min: 100.0,
            max: 3000.0,
            step: 100.0,
            label: slider::milliseconds,
        },
    );

    page.end_group();

    switch::add(
        page,
        "wallpaper_shuffle",
        "Random wallpaper",
        "Automatically choose another image from the wallpaper directory",
    );

    slider::add(
        page,
        "wallpaper_shuffle_minutes",
        "Shuffle interval",
        "How often a random wallpaper is selected",
        Range {
            min: 1.0,
            max: 180.0,
            step: 1.0,
            label: slider::minutes,
        },
    );

    switch::add(
        page,
        "follow_wallpaper",
        "Update colors with wallpaper",
        "Rebuild the dynamic palette after a wallpaper change",
    );

    page.end_group();
}
