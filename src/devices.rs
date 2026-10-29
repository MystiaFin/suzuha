use std::collections::HashMap;

use amane::Service;

// udev only reports whole usb devices here, not each interface a keyboard or receiver splits into
const MONITOR: &str = "udevadm monitor --udev --property --subsystem-match=usb/usb_device";

/*
 * sends a notification when a usb device like a mouse or keyboard is
 * plugged in or pulled out; devices already there at startup are not
 * announced, since udev only reports changes
 */
pub struct Devices;

impl Service for Devices {
    fn new() -> Self {
        Self
    }

    fn listen() {
        let mut event = HashMap::new();

        // each event comes as KEY=VALUE lines, ended by a blank line
        for line in amane::lines(MONITOR) {
            if let Some((key, value)) = line.split_once('=') {
                event.insert(String::from(key), String::from(value));

                continue;
            }

            if line.is_empty() {
                notify(&event);

                event.clear();
            }
        }
    }
}

fn notify(event: &HashMap<String, String>) {
    let action = match event.get("ACTION").map(String::as_str) {
        Some("add") => "Connected",
        Some("remove") => "Disconnected",
        // the header udevadm prints first, and changes like a device waking from suspend
        _ => return,
    };

    // a quote in the name would end the shell string early
    let name = name(event).replace('\'', "'\\''");

    amane::spawn(&format!("notify-send -a Devices '{name}' '{action}'"));
}

// what the notification calls the device, like "AJAZZ 2.4G 4K"
fn name(event: &HashMap<String, String>) -> String {
    // udev swaps the spaces in the name the device reports for underscores
    event
        .get("ID_MODEL")
        .map(|model| model.replace('_', " "))
        .or_else(|| event.get("ID_MODEL_FROM_DATABASE").cloned())
        .unwrap_or_else(|| String::from("USB device"))
}
