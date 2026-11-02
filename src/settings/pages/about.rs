use std::sync::OnceLock;

use amane::Service;

use super::super::button::{self, Style};
use super::super::page::Page;
use super::super::{Confirm, Shown, row};

// asked once, a commit made while running shows after a restart
static COMMIT: OnceLock<String> = OnceLock::new();

pub fn build(page: &mut Page) {
    let commit = COMMIT.get_or_init(|| {
        let commit = amane::output("git -C ~/.config/amane rev-parse --short HEAD");

        String::from(commit.trim())
    });

    let detail = format!("Git commit {commit}");

    action(
        page,
        "Config checkout",
        &detail,
        "Open folder",
        Style::Plain,
        || {
            amane::spawn("xdg-open ~/.config/amane");
        },
    );

    action(
        page,
        "Reset settings",
        "Restore every setting to its built-in default",
        "Reset",
        Style::Danger,
        || Shown::write().confirm = Some(Confirm::Reset),
    );

    page.end_group();
}

fn action(
    page: &mut Page,
    title: &str,
    detail: &str,
    label: &str,
    style: Style,
    on_click: impl Fn() + 'static,
) {
    let control = button::view(page.theme, label, style, true, on_click);

    let row = row::view(
        page.theme,
        page.width,
        row::HEIGHT,
        title,
        detail,
        control,
        button::width(label),
    );

    page.row(row::HEIGHT, row);
}
