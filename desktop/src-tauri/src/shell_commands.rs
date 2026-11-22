use std::path::Path;
use std::process::Command;

use nazar_windows::disk_usage::measure_readable_size;
use nazar_windows::shell_icon::read_icon_as_png;
use nazar_windows::windows_version::supports_mica;
use tauri::ipc::Response;

use crate::command_failure::CommandFailure;

const ICON_SIDE_LENGTH: u16 = 96;
const FILE_MANAGER: &str = "explorer.exe";
const SELECT_ARGUMENT: &str = "/select,";

#[tauri::command]
pub async fn read_icon(path: String) -> Result<Response, CommandFailure> {
    let reading = tauri::async_runtime::spawn_blocking(move || {
        read_icon_as_png(Path::new(&path), ICON_SIDE_LENGTH)
    });
    let png = reading
        .await
        .map_err(|_| CommandFailure::WorkInterrupted)?
        .map_err(|error| CommandFailure::IconUnavailable(error.to_string()))?;
    Ok(Response::new(png))
}

#[tauri::command]
pub async fn measure_sizes(paths: Vec<String>) -> Result<Vec<u64>, CommandFailure> {
    let measuring = tauri::async_runtime::spawn_blocking(move || {
        paths
            .iter()
            .map(|path| measure_readable_size(Path::new(path)))
            .collect()
    });
    measuring.await.map_err(|_| CommandFailure::WorkInterrupted)
}

#[tauri::command]
pub fn show_in_file_manager(path: &str) -> Result<(), CommandFailure> {
    Command::new(FILE_MANAGER)
        .arg(SELECT_ARGUMENT)
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(CommandFailure::FileManager)
}

#[tauri::command]
pub fn has_mica() -> bool {
    supports_mica()
}
