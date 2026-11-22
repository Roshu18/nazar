use tauri::image::Image;
use tauri::{App, Manager};

const ICON_SIZE_AT_NORMAL_SCALE: f64 = 32.0;
const WINDOW_ICONS: [(u32, &[u8]); 6] = [
    (32, include_bytes!("../icons/window-32.png")),
    (48, include_bytes!("../icons/window-48.png")),
    (64, include_bytes!("../icons/window-64.png")),
    (96, include_bytes!("../icons/window-96.png")),
    (128, include_bytes!("../icons/window-128.png")),
    (256, include_bytes!("../icons/window-256.png")),
];

pub fn sharpen_window_icons(app: &App) -> tauri::Result<()> {
    for window in app.webview_windows().values() {
        let wanted_size = ICON_SIZE_AT_NORMAL_SCALE * window.scale_factor()?;
        window.set_icon(Image::from_bytes(icon_at_least(wanted_size))?)?;
    }
    Ok(())
}

fn icon_at_least(wanted_size: f64) -> &'static [u8] {
    let [.., (_, largest_icon)] = WINDOW_ICONS;
    WINDOW_ICONS
        .iter()
        .find(|(size, _)| f64::from(*size) >= wanted_size)
        .map_or(largest_icon, |(_, icon)| *icon)
}
