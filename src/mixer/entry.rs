use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;

use amane::{Apps, AudioApp, DesktopApp};

// how many parents up a stream's process is followed, enough to get from an electron helper to the app
const ANCESTORS: usize = 6;

// programs that start other programs, so their desktop entries say nothing about who owns a process
const WRAPPERS: [&str; 4] = ["env", "sh", "bash", "flatpak"];

thread_local! {
    /*
     * the desktop entry name that owns a process, looked up once per process;
     * none when no entry was found, and forgotten once the app stops playing
     */
    static OWNERS: RefCell<HashMap<u32, Option<String>>> = RefCell::new(HashMap::new());
}

/*
 * the app's desktop entry: the one owning its process first, then
 * one matched by icon name, program or name
 */
pub fn find<'a>(desktop: &'a Apps, app: &AudioApp) -> Option<&'a DesktopApp> {
    let list = desktop.list();

    let owner = app.process().and_then(|process| owner(desktop, process));

    if let Some(owner) = owner {
        return list.iter().find(|entry| entry.name() == owner);
    }

    let by_icon = list
        .iter()
        .find(|entry| entry.icon().is_some() && entry.icon() == app.icon());

    if by_icon.is_some() {
        return by_icon;
    }

    let by_program = list
        .iter()
        .find(|entry| app.binary() == Some(program(entry.exec())));

    if by_program.is_some() {
        return by_program;
    }

    list.iter()
        .find(|entry| entry.name().eq_ignore_ascii_case(app.name()))
}

// /proc lives in memory, so reading it from the view doesn't hold up a frame
fn owner(desktop: &Apps, process: u32) -> Option<String> {
    OWNERS.with_borrow_mut(|owners| {
        owners
            .entry(process)
            .or_insert_with(|| find_owner(desktop, process))
            .clone()
    })
}

/*
 * walks up from the process to its parents, until one's command line
 * names an app's program, like the teams-for-linux folder electron
 * is started with
 */
fn find_owner(desktop: &Apps, process: u32) -> Option<String> {
    let mut current = process;

    for _ in 0..ANCESTORS {
        let command = fs::read(format!("/proc/{current}/cmdline")).ok()?;

        let command = String::from_utf8_lossy(&command);

        let parts = command.split(['\0', '/', ' ']);

        let found = desktop.list().iter().find(|entry| {
            let program = program(entry.exec());

            !WRAPPERS.contains(&program) && parts.clone().any(|part| part == program)
        });

        if let Some(entry) = found {
            return Some(String::from(entry.name()));
        }

        current = read_parent(current)?;
    }

    None
}

// an app that stopped playing is looked up again, since its process id may belong to another program by then
pub fn forget(apps: &[AudioApp]) {
    OWNERS.with_borrow_mut(|owners| {
        owners.retain(|process, _| apps.iter().any(|app| app.process() == Some(*process)));
    });
}

// none once the parent is init, which every process ends at
fn read_parent(process: u32) -> Option<u32> {
    let stat = fs::read_to_string(format!("/proc/{process}/stat")).ok()?;

    // the name in brackets can hold spaces, so the fields are counted from after it
    let (_, fields) = stat.rsplit_once(')')?;

    let parent = fields.split_whitespace().nth(1)?.parse().ok()?;

    (parent > 1).then_some(parent)
}

// the file name of the program a command line starts, like "firefox"
fn program(exec: &str) -> &str {
    let first = exec.split_whitespace().next().unwrap_or_default();

    first.rsplit('/').next().unwrap_or_default()
}
