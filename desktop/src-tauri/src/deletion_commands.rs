use std::env;
use std::fs;
use std::path::PathBuf;

use nazar_core::location_removal::{RemovalReport, RemovalRequest};

use crate::command_failure::CommandFailure;
use crate::engine::EngineSession;
use crate::program_view::{DeletionReport, LocationReference, deletion_report_of};

const PROGRAM_DATA_VARIABLE: &str = "ProgramData";
const DEFAULT_PROGRAM_DATA: &str = "C:\\ProgramData";
const BACKUP_FOLDERS: [&str; 2] = ["Nazar", "registry-backups"];

#[tauri::command]
pub async fn delete_locations(
    locations: Vec<LocationReference>,
) -> Result<DeletionReport, CommandFailure> {
    let deletion = tauri::async_runtime::spawn_blocking(move || delete_with_engine(locations));
    deletion
        .await
        .map_err(|_| CommandFailure::WorkInterrupted)?
}

fn delete_with_engine(locations: Vec<LocationReference>) -> Result<DeletionReport, CommandFailure> {
    let session = EngineSession::begin()?;
    let request_file = session.file("delete-request.json");
    let report_file = session.file("delete-report.json");
    let request = RemovalRequest {
        objects: locations
            .into_iter()
            .map(LocationReference::into_object)
            .collect(),
    };
    let request_json = serde_json::to_string(&request)
        .map_err(|error| CommandFailure::DeletionRequest(error.to_string()))?;
    fs::write(&request_file, request_json)
        .map_err(|error| CommandFailure::DeletionRequest(error.to_string()))?;
    session.run_engine(&[
        "delete".to_owned(),
        "--request".to_owned(),
        request_file.to_string_lossy().into_owned(),
        "--report".to_owned(),
        report_file.to_string_lossy().into_owned(),
        "--registry-backup".to_owned(),
        registry_backup_directory(&session),
    ])?;
    let report_json = fs::read_to_string(&report_file)
        .map_err(|error| CommandFailure::DeletionReport(error.to_string()))?;
    let report: RemovalReport = serde_json::from_str(&report_json)
        .map_err(|error| CommandFailure::DeletionReport(error.to_string()))?;
    Ok(deletion_report_of(report))
}

fn registry_backup_directory(session: &EngineSession) -> String {
    let program_data =
        env::var_os(PROGRAM_DATA_VARIABLE).unwrap_or_else(|| DEFAULT_PROGRAM_DATA.into());
    let mut directory = PathBuf::from(program_data);
    directory.extend(BACKUP_FOLDERS);
    directory.push(session.started_at_unix_seconds().to_string());
    directory.to_string_lossy().into_owned()
}
