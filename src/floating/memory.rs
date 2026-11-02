use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use super::placement::Name;

type Spots = HashMap<Name, (f32, f32)>;

const NAMES: [Name; 8] = [
    Name::Clock,
    Name::Weather,
    Name::CpuTemperature,
    Name::CpuUsage,
    Name::GpuTemperature,
    Name::UvIndex,
    Name::Humidity,
    Name::AirQuality,
];

/*
 * where the cards last settled on each monitor, so a new start puts them
 * there at once instead of waiting for the wallpaper to be read again
 */
static REMEMBERED: LazyLock<Mutex<HashMap<String, Spots>>> = LazyLock::new(|| Mutex::new(load()));

// the spots for these cards, when every one of them was remembered
pub fn recall(monitor: &str, names: &[Name]) -> Option<Spots> {
    let remembered = REMEMBERED.lock().expect("failed to lock remembered spots");

    let spots = remembered.get(monitor)?;

    let mut recalled = HashMap::new();

    for name in names {
        recalled.insert(*name, *spots.get(name)?);
    }

    Some(recalled)
}

// only written when the cards moved, the view asks every frame
pub fn remember(monitor: &str, spots: &Spots) {
    let mut remembered = REMEMBERED.lock().expect("failed to lock remembered spots");

    if remembered.get(monitor) == Some(spots) {
        return;
    }

    remembered.insert(String::from(monitor), spots.clone());

    save(&remembered);
}

// "DP-1 Clock 1500 820" on each line
fn load() -> HashMap<String, Spots> {
    let mut remembered: HashMap<String, Spots> = HashMap::new();

    let Ok(text) = fs::read_to_string(path()) else {
        return remembered;
    };

    for line in text.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();

        let [monitor, name, x, y] = fields.as_slice() else {
            continue;
        };

        let Some(name) = NAMES
            .into_iter()
            .find(|known| format!("{known:?}") == *name)
        else {
            continue;
        };

        let (Ok(x), Ok(y)) = (x.parse(), y.parse()) else {
            continue;
        };

        remembered
            .entry(String::from(*monitor))
            .or_default()
            .insert(name, (x, y));
    }

    remembered
}

// losing the file only means the next start reads the wallpaper first
fn save(remembered: &HashMap<String, Spots>) {
    let mut text = String::new();

    for (monitor, spots) in remembered {
        for (name, (x, y)) in spots {
            text.push_str(&format!("{monitor} {name:?} {x} {y}\n"));
        }
    }

    let path = path();

    let folder = path.parent().expect("failed to find the state folder");

    let _ = fs::create_dir_all(folder);
    let _ = fs::write(&path, text);
}

fn path() -> PathBuf {
    let home = env::var("HOME").expect("failed to find home: HOME is not set");

    PathBuf::from(format!("{home}/.local/state/amane/floating"))
}
