use amane::{Media, MediaPlayer, Service};

use crate::motion::{self, FAST_SPATIAL};
use crate::overlay::Overlay;

// the chosen player while it is still open, otherwise the one amane picks
pub fn shown<'a>(media: &'a Media, overlay: &Overlay) -> Option<&'a MediaPlayer> {
    let chosen = overlay.player.as_deref();

    let found = media
        .players()
        .iter()
        .find(|player| Some(player.name()) == chosen);

    found.or(media.active())
}

// 1 for the next player, -1 for the one before, wrapping around
pub fn switch(direction: i32) {
    let media = Media::read();

    let players = media.players();

    if players.len() < 2 {
        return;
    }

    let mut overlay = Overlay::write();

    let current = shown(&media, &overlay).map(MediaPlayer::name);

    let index = players
        .iter()
        .position(|player| Some(player.name()) == current)
        .unwrap_or(0);

    let count = players.len() as i32;

    let next = (index as i32 + direction).rem_euclid(count) as usize;

    overlay.player = Some(String::from(players[next].name()));

    // the new player's card starts off to that side and slides into place
    overlay.player_switch = motion::spatial(direction as f32, FAST_SPATIAL);
    overlay.player_switch.to(0.0);
}

// runs a playback command on the player shown when it was clicked, if it is still open
pub fn control(name: &str, action: fn(&MediaPlayer)) {
    let media = Media::read();

    if let Some(player) = media.players().iter().find(|player| player.name() == name) {
        action(player);
    }
}
