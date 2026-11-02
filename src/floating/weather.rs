mod csv;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use amane::Service;

use crate::settings::Settings;

const FORECAST: &str = "https://api.open-meteo.com/v1/forecast";
const AIR_QUALITY: &str = "https://air-quality-api.open-meteo.com/v1/air-quality";

// how often the loop looks at the settings and the refresh button between polls
const CHECK: Duration = Duration::from_secs(1);

// set by the refresh button, the next check asks at once
static REFRESH: AtomicBool = AtomicBool::new(false);

// the latest reading for the saved location, empty until the first answer
#[derive(Default, PartialEq)]
pub struct Weather {
    pub place: String,

    pub temperature: Option<f32>,
    pub high: Option<f32>,
    pub low: Option<f32>,

    // a wmo weather code, like 3 for cloudy
    pub code: Option<u32>,

    pub humidity: Option<f32>,
    pub uv_index: Option<f32>,

    // the us air quality index, 0 to 500
    pub air_quality: Option<f32>,
}

/*
 * polled, and asked outside the lock: the answer comes over the network,
 * and nothing is written when it didn't change
 */
impl Service for Weather {
    fn new() -> Self {
        Self::default()
    }

    // asks again once the interval is up, the location or unit changes, or refresh is pressed
    fn listen() {
        loop {
            let asked = request();

            let fresh = fetch(&asked);

            if *Self::read() != fresh {
                *Self::write() = fresh;
            }

            let started = Instant::now();

            loop {
                thread::sleep(CHECK);

                let minutes = Settings::read().number("weather_minutes").max(1.0);

                let due = started.elapsed().as_secs_f32() >= minutes * 60.0;

                if due || request() != asked || REFRESH.swap(false, Ordering::Relaxed) {
                    break;
                }
            }
        }
    }
}

impl Weather {
    // from the settings' refresh button
    pub fn refresh() {
        REFRESH.store(true, Ordering::Relaxed);
    }

    pub fn condition(&self) -> &'static str {
        match self.code {
            Some(0) => "Clear sky",
            Some(1) => "Mainly clear",
            Some(2) => "Partly cloudy",
            Some(3) => "Cloudy",
            Some(45 | 48) => "Fog",
            Some(51 | 53 | 55) => "Drizzle",
            Some(56 | 57) => "Freezing drizzle",
            Some(61) => "Light rain",
            Some(63) => "Rain",
            Some(65) => "Heavy rain",
            Some(66 | 67) => "Freezing rain",
            Some(71 | 73 | 75 | 77) => "Snow",
            Some(80 | 81 | 82) => "Rain showers",
            Some(85 | 86) => "Snow showers",
            Some(95 | 96 | 99) => "Thunderstorm",
            _ => "Weather",
        }
    }

    // material design weather glyphs from the nerd font
    pub fn icon(&self) -> &'static str {
        match self.code.unwrap_or(3) {
            0 => "\u{f0599}",
            1 | 2 => "\u{f0595}",
            3 => "\u{f0590}",
            45 | 48 => "\u{f0591}",
            51..=57 | 80..=82 => "\u{f0596}",
            61..=67 => "\u{f0597}",
            71..=77 => "\u{f0598}",
            85 | 86 => "\u{f0f36}",
            95.. => "\u{f067e}",
            _ => "\u{f0590}",
        }
    }
}

// where to ask about and in which unit, as the settings say
#[derive(PartialEq)]
struct Request {
    location: Option<Location>,
    fahrenheit: bool,
}

#[derive(PartialEq)]
struct Location {
    latitude: String,
    longitude: String,
    name: String,
}

fn fetch(request: &Request) -> Weather {
    let Some(location) = &request.location else {
        return Weather {
            place: String::from("Set a location"),
            ..Weather::default()
        };
    };

    let coordinates = format!(
        "latitude={}&longitude={}",
        location.latitude, location.longitude
    );

    let unit = if request.fahrenheit {
        "fahrenheit"
    } else {
        "celsius"
    };

    let forecast = get(&format!(
        "{FORECAST}?{coordinates}&current=temperature_2m,relative_humidity_2m,weather_code,uv_index\
         &daily=temperature_2m_max,temperature_2m_min&temperature_unit={unit}\
         &forecast_days=1&timezone=auto&format=csv"
    ));

    let air = get(&format!(
        "{AIR_QUALITY}?{coordinates}&current=us_aqi&format=csv"
    ));

    let number = |values: &HashMap<String, String>, name: &str| values.get(name)?.parse().ok();

    Weather {
        place: location.name.clone(),
        temperature: number(&forecast, "temperature_2m"),
        high: number(&forecast, "temperature_2m_max"),
        low: number(&forecast, "temperature_2m_min"),
        code: forecast
            .get("weather_code")
            .and_then(|code| code.parse().ok()),
        humidity: number(&forecast, "relative_humidity_2m"),
        uv_index: number(&forecast, "uv_index"),
        air_quality: number(&air, "us_aqi"),
    }
}

// empty when offline; the next poll tries again
fn get(url: &str) -> HashMap<String, String> {
    let text = amane::output(&format!("curl -sf --max-time 20 '{url}'"));

    csv::values(&text)
}

fn request() -> Request {
    let settings = Settings::read();

    // the coordinates go into a shell command, so only numbers are let through
    let number = |key: &str| {
        let value = settings.text(key).trim();

        value.parse::<f64>().ok()?;

        Some(String::from(value))
    };

    let location = match (number("weather_latitude"), number("weather_longitude")) {
        (Some(latitude), Some(longitude)) => {
            let name = settings.text("weather_place").trim();

            let name = if name.is_empty() { "Weather" } else { name };

            Some(Location {
                latitude,
                longitude,
                name: String::from(name),
            })
        }

        _ => None,
    };

    Request {
        location,
        fahrenheit: settings.text("weather_unit") == "fahrenheit",
    }
}
