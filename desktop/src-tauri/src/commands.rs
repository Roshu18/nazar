use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use nazar_store::trace_store::{list_trace_names, load_manifest};
use tauri::State;

use crate::command_failure::CommandFailure;
use crate::trace_view::{TraceDetails, TraceOverview, details_of, overview_of};
use crate::tracking_request::TrackingRequest;

#[derive(Default)]
pub struct TrackingState {
    stop_file: Mutex<Option<PathBuf>>,
}

impl TrackingState {
    fn remember_stop_file(&self, stop_file: Option<PathBuf>) {
        let mut remembered = self
            .stop_file
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        *remembered = stop_file;
    }

    fn stop_file(&self) -> Option<PathBuf> {
        self.stop_file
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

#[tauri::command]
pub fn list_traces() -> Result<Vec<TraceOverview>, CommandFailure> {
    let mut overviews = Vec::new();
    for name in list_trace_names()? {
        let manifest = load_manifest(&name)?;
        overviews.push(overview_of(&name, &manifest));
    }
    Ok(overviews)
}

#[tauri::command]
pub fn read_trace(name: &str) -> Result<TraceDetails, CommandFailure> {
    let manifest = load_manifest(name)?;
    Ok(details_of(name, &manifest))
}

#[tauri::command]
pub async fn track_program(
    program_path: String,
    tracking: State<'_, TrackingState>,
) -> Result<String, CommandFailure> {
    let request = TrackingRequest::prepare(program_path)?;
    tracking.remember_stop_file(Some(request.stop_file().to_owned()));
    let outcome = tauri::async_runtime::spawn_blocking(move || request.run()).await;
    tracking.remember_stop_file(None);
    outcome.map_err(|_| CommandFailure::WorkInterrupted)?
}

#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri hands managed state to commands by value"
)]
pub fn stop_tracking(tracking: State<'_, TrackingState>) -> Result<(), CommandFailure> {
    let stop_file = tracking
        .stop_file()
        .ok_or(CommandFailure::NothingBeingTracked)?;
    fs::write(stop_file, []).map_err(CommandFailure::StopRequest)
}
