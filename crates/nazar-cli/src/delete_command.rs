use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use nazar_core::location_removal::{RemovalReport, RemovalRequest};
use nazar_windows::location_remover::remove_locations;

#[derive(Debug, clap::Args)]
pub struct DeleteArguments {
    #[arg(
        long,
        value_name = "FILE",
        help = "A JSON file listing the locations to delete"
    )]
    pub request: PathBuf,
    #[arg(
        long,
        value_name = "FILE",
        help = "Where to write what happened to each location"
    )]
    pub report: PathBuf,
    #[arg(
        long,
        value_name = "FOLDER",
        help = "Where registry entries are backed up before they are deleted"
    )]
    pub registry_backup: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteError {
    #[error("the request `{path}` could not be read: {source}")]
    ReadRequest { path: String, source: io::Error },
    #[error("the request `{path}` is not a list of locations: {source}")]
    MalformedRequest {
        path: String,
        source: serde_json::Error,
    },
    #[error("the report could not be turned into JSON: {0}")]
    SerialiseReport(serde_json::Error),
    #[error("the report `{path}` could not be written: {source}")]
    WriteReport { path: String, source: io::Error },
}

pub fn delete_locations(arguments: &DeleteArguments) -> Result<(), DeleteError> {
    let request = read_request(&arguments.request)?;
    let report = remove_locations(request, &arguments.registry_backup);
    write_report(&report, &arguments.report)
}

fn read_request(path: &Path) -> Result<RemovalRequest, DeleteError> {
    let json = fs::read_to_string(path).map_err(|source| DeleteError::ReadRequest {
        path: path.display().to_string(),
        source,
    })?;
    serde_json::from_str(&json).map_err(|source| DeleteError::MalformedRequest {
        path: path.display().to_string(),
        source,
    })
}

fn write_report(report: &RemovalReport, path: &Path) -> Result<(), DeleteError> {
    let json = serde_json::to_string_pretty(report).map_err(DeleteError::SerialiseReport)?;
    fs::write(path, json).map_err(|source| DeleteError::WriteReport {
        path: path.display().to_string(),
        source,
    })
}
