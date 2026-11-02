use super::super::page::Page;
use super::super::slider::{self, Range};
use super::super::{choice, switch};

pub fn build(page: &mut Page) {
    choice::add(
        page,
        "floating_visibility",
        "Visibility",
        "Show widgets only on an empty desktop, always, or never",
        &[
            ("Desktop only", "desktop"),
            ("Always", "always"),
            ("Hidden", "hidden"),
        ],
    );

    slider::add(
        page,
        "floating_scale",
        "Widget scale",
        "Scale desktop cards while keeping automatic placement",
        Range {
            min: 0.7,
            max: 1.35,
            step: 0.05,
            label: slider::times,
        },
    );

    slider::add(
        page,
        "floating_opacity",
        "Opacity",
        "Adjust the opacity of the complete desktop widget layer",
        Range {
            min: 0.35,
            max: 1.0,
            step: 0.05,
            label: slider::plain,
        },
    );

    switch::add(
        page,
        "floating_lock_placement",
        "Lock placement",
        "Keep the current automatic placement instead of recomputing it",
    );

    page.end_group();

    switch::add(page, "widget_clock", "Clock", "Large desktop clock");
    switch::add(page, "widget_weather", "Weather", "Current weather summary");
    switch::add(
        page,
        "widget_cpu_temperature",
        "CPU temperature",
        "Processor thermal card",
    );
    switch::add(
        page,
        "widget_cpu_usage",
        "CPU usage",
        "Processor utilization card",
    );

    switch::add(
        page,
        "widget_gpu_temperature",
        "GPU temperature",
        "Shown only when a supported GPU is available",
    );

    switch::add(
        page,
        "widget_uv",
        "UV index",
        "Requires configured weather data",
    );
    switch::add(
        page,
        "widget_humidity",
        "Humidity",
        "Requires configured weather data",
    );

    switch::add(
        page,
        "widget_air_quality",
        "Air quality",
        "Shown only when air-quality data is available",
    );

    page.end_group();
}
