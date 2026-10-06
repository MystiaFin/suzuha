mod art;
mod cava;
mod fader;
mod media;
mod player;
mod visualizer;
pub mod wave;

use amane::{Audio, Center, Padding, Rectangle, Row, Service, children};

use super::{Overlay, PanelView, Region};
use crate::liquid::{self, Blob};
use crate::theme::Theme;

const MAX_WIDTH: f32 = 780.0;
// 276 tall on screen, just enough for the media card's details, plus what hides past the top edge
const HEIGHT: f32 = 276.0 + EDGE_OVERLAP;
const RADIUS: f32 = 30.0;

/*
 * the panel reaches past the top edge by more than its corner radius, so
 * its sides meet the edge straight; with the round corners on screen the
 * melt narrows once it stops moving
 */
const EDGE_OVERLAP: f32 = 40.0;

// how far past its own height it slides to let go of the edge
const HIDDEN_MARGIN: f32 = 32.0;

const PADDING: Padding = Padding {
    top: EDGE_OVERLAP + 18.0,
    right: 18.0,
    bottom: 18.0,
    left: 18.0,
};

const INNER_HEIGHT: f32 = HEIGHT - PADDING.top - PADDING.bottom;

const GAP: f32 = 14.0;

const FADERS_WIDTH: f32 = 120.0;
const CARD_RADIUS: f32 = 20.0;

const VOLUME_ICON: &str = "󰕾";
const MICROPHONE_ICON: &str = "󰍬";

// none while fully hidden
pub fn view(overlay: &Overlay, theme: &Theme, screen: Region) -> Option<PanelView> {
    let progress = overlay.control_center.progress.value();

    if progress <= 0.001 {
        return None;
    }

    let width = MAX_WIDTH.min(screen.width - 64.0);

    let x = (screen.width - width) / 2.0;

    let hidden = -(HEIGHT + HIDDEN_MARGIN);

    let y = -EDGE_OVERLAP + hidden * (1.0 - progress);

    let blob = Blob::new(x, y, width, HEIGHT)
        .radius(RADIUS)
        .child(content(overlay, theme, width));

    let input = Region {
        x,
        y,
        width,
        height: HEIGHT,
    };

    // fully out it hangs from the top edge, and its melted corners spread a little past its sides
    let reach = Region {
        x: x - liquid::CONNECTION,
        y: 0.0,
        width: width + liquid::CONNECTION * 2.0,
        height: -EDGE_OVERLAP + HEIGHT + liquid::CONNECTION,
    };

    Some(PanelView { blob, input, reach })
}

fn content(overlay: &Overlay, theme: &Theme, width: f32) -> Rectangle {
    let card_width = width - PADDING.left - PADDING.right - FADERS_WIDTH - GAP;

    let row = Row::new(children![faders(theme), media::view(overlay, theme, card_width, INNER_HEIGHT)]).gap(GAP);

    Rectangle::new()
        .width(width)
        .height(HEIGHT)
        .padding(PADDING)
        .on_hover(Overlay::hover_control_center)
        .child(row)
}

fn faders(theme: &Theme) -> Rectangle {
    let audio = Audio::read();

    let volume = fader::Fader {
        icon: VOLUME_ICON,
        level: audio.volume(),
        muted: audio.muted(),
        set: Audio::set_volume,
        toggle_mute: Audio::toggle_mute,
    };

    let microphone = fader::Fader {
        icon: MICROPHONE_ICON,
        level: audio.microphone_volume(),
        muted: audio.microphone_muted(),
        set: Audio::set_microphone_volume,
        toggle_mute: Audio::toggle_microphone_mute,
    };

    let row = Row::new(children![
        fader::view(volume, theme, INNER_HEIGHT - 12.0),
        fader::view(microphone, theme, INNER_HEIGHT - 12.0),
    ])
    .gap(4.0);

    Rectangle::new()
        .width(FADERS_WIDTH)
        .height(INNER_HEIGHT)
        .radius(CARD_RADIUS)
        .fill(theme.surface)
        .align_child(Center, Center)
        .child(row)
}

// "toggle", "show" or "hide"
pub fn ipc(arguments: &[String]) -> String {
    let command = arguments.first().map(String::as_str).unwrap_or("toggle");

    let shown = Overlay::read().control_center.shown;

    let wanted = match command {
        "show" => true,
        "hide" => false,
        _ => !shown,
    };

    if wanted != shown {
        Overlay::toggle_control_center();
    }

    String::from("ok")
}
