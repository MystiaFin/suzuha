use std::cell::RefCell;
use std::collections::HashSet;
use std::env;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

thread_local! {
    // links already being downloaded, so each is only fetched once
    static STARTED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

/*
 * where the cover for a player's art link is on disk; a web link is
 * downloaded into the cache first and gives none until it has arrived
 */
pub fn path(url: &str) -> Option<PathBuf> {
    if url.is_empty() {
        return None;
    }

    // a player can name a file that is already gone, and reading it would fail
    if let Some(local) = url.strip_prefix("file://") {
        let local = PathBuf::from(local);

        return local.exists().then_some(local);
    }

    // any player can set the link, and it goes into a shell command inside single quotes
    let web = url.starts_with("https://") || url.starts_with("http://");

    if !web || url.contains('\'') {
        return None;
    }

    let cached = cached(url);

    if cached.exists() {
        return Some(cached);
    }

    download(url, &cached);

    None
}

fn cached(url: &str) -> PathBuf {
    let home = env::var("HOME").expect("failed to find home: HOME is not set");

    let mut hasher = DefaultHasher::new();

    url.hash(&mut hasher);

    PathBuf::from(format!("{home}/.cache/amane/art/{:x}", hasher.finish()))
}

// written under a temporary name and moved in once complete, so a half file is never read
fn download(url: &str, cached: &Path) {
    let first_time = STARTED.with_borrow_mut(|started| started.insert(String::from(url)));

    if !first_time {
        return;
    }

    let file = cached.display();

    let folder = cached
        .parent()
        .expect("failed to find the art folder")
        .display();

    // ponytail: the cache is never cleaned, prune it if it grows
    amane::spawn(&format!(
        "mkdir -p '{folder}' && curl -sfL -o '{file}.part' '{url}' && mv '{file}.part' '{file}'"
    ));
}
