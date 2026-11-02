use amane::{Padding, Rectangle, Service, Start, TextInput};

use super::page::Page;
use super::{Settings, high_surface, row};

const MOST_WIDTH: f32 = 280.0;

// a typed setting; the text input is named after the key
pub fn add(page: &mut Page, key: &'static str, title: &str, detail: &str) {
    let input = TextInput::new(key)
        .size(14.0)
        .color(page.theme.text)
        .on_change(move |text| Settings::stage(key, text));

    let row = view(page, title, detail, input);

    page.row(row::TALL_HEIGHT, row);
}

// any text input in a field, like the display name that isn't a setting
pub fn view(page: &Page, title: &str, detail: &str, input: TextInput) -> Rectangle {
    let width = MOST_WIDTH.min(page.width * 0.42);

    let field = Rectangle::new()
        .width(width)
        .height(row::CONTROL_HEIGHT)
        .radius(12.0)
        .fill(high_surface(page.theme))
        .padding(Padding {
            top: 0.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        })
        .align_child(Start, amane::Center)
        .child(input);

    row::view(
        page.theme,
        page.width,
        row::TALL_HEIGHT,
        title,
        detail,
        field,
        width,
    )
}

// every setting that is typed, which the fields are named after
const KEYS: [&str; 4] = [
    "wallpaper_folder",
    "weather_place",
    "weather_latitude",
    "weather_longitude",
];

// the fields show the draft, not what was typed before it was thrown away
pub fn fill() {
    let settings = Settings::read();

    for key in KEYS {
        TextInput::set_text(key, settings.staged(key));
    }
}
