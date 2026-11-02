use std::path::Path;

use amane::DesktopApp;

// the app's icon file, only when it's an image the drawing code can read
pub fn path(app: &DesktopApp) -> Option<&Path> {
    let path = app.icon_path()?;

    let readable = matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("png" | "jpg" | "jpeg" | "svg")
    );

    readable.then_some(path)
}
