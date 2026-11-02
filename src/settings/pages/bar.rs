use super::super::choice;
use super::super::page::Page;
use super::super::slider::{self, Range};
use super::super::switch;

pub fn build(page: &mut Page) {
    choice::add(
        page,
        "bar_position",
        "Position",
        "Place the status bar on the top or bottom edge",
        &[("Top", "top"), ("Bottom", "bottom")],
    );

    slider::add(
        page,
        "bar_height",
        "Height",
        "Adjust status bar height without changing its content scale",
        Range {
            min: 32.0,
            max: 64.0,
            step: 2.0,
            label: slider::pixels,
        },
    );

    switch::add(
        page,
        "bar_auto_hide",
        "Auto-hide",
        "Collapse to a thin screen-edge trigger when the pointer leaves",
    );

    choice::add(
        page,
        "workspace_style",
        "Workspace indicator",
        "Choose the workspace strip presentation",
        &[("Pill", "pill"), ("Dots", "dots"), ("Numbers", "numbers")],
    );

    page.end_group();

    switch::add(
        page,
        "bar_logo",
        "NixOS logo",
        "Show the power-menu launcher",
    );
    switch::add(
        page,
        "bar_workspaces",
        "Workspaces",
        "Show the workspace indicator",
    );
    switch::add(
        page,
        "bar_workspace_name",
        "Workspace name",
        "Show the active workspace label",
    );
    switch::add(
        page,
        "bar_audio",
        "Audio controls",
        "Show output and microphone rings",
    );
    switch::add(page, "bar_media", "Media", "Show current media information");
    switch::add(
        page,
        "bar_clock",
        "Clock and date",
        "Show time and date in the center section",
    );
    switch::add(page, "bar_battery", "Battery", "Show battery status");
    switch::add(page, "bar_memory", "Memory", "Show memory usage");

    switch::add(
        page,
        "bar_tray",
        "System tray group",
        "Show notifications, network, and Bluetooth shortcuts",
    );

    page.end_group();

    switch::add(
        page,
        "clock_24_hour",
        "24-hour clock",
        "Use 24-hour time instead of AM/PM",
    );
    switch::add(
        page,
        "clock_seconds",
        "Show seconds",
        "Include seconds in the status-bar clock",
    );

    page.end_group();
}
