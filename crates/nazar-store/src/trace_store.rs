use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use nazar_core::change_manifest::ChangeManifest;

const PROGRAM_DATA_VARIABLE: &str = "ProgramData";
const STORE_DIRECTORY: &str = "Nazar";
const TRACES_DIRECTORY: &str = "traces";
const MANIFEST_EXTENSION: &str = "json";
const UNNAMED_PROGRAM: &str = "program";

#[derive(Debug, thiserror::Error)]
pub enum TraceStoreError {
    #[error("the ProgramData folder is unknown, so the trace store cannot be located")]
    StoreLocationUnknown,
    #[error("`{path}` could not be written: {source}")]
    Write { path: String, source: io::Error },
    #[error("`{path}` could not be read: {source}")]
    Read { path: String, source: io::Error },
    #[error("`{path}` is not a Nazar trace: {source}")]
    Malformed {
        path: String,
        source: serde_json::Error,
    },
    #[error("the trace could not be turned into JSON: {0}")]
    Serialise(serde_json::Error),
}

pub fn save_manifest(
    manifest: &ChangeManifest,
    destination: Option<&Path>,
) -> Result<PathBuf, TraceStoreError> {
    let path = match destination {
        Some(destination) => destination.to_owned(),
        None => store_path_for(&manifest.program, manifest.started_at_unix_seconds)?,
    };
    let json = serde_json::to_string_pretty(manifest).map_err(TraceStoreError::Serialise)?;
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory).map_err(|source| write_error(&path, source))?;
    }
    fs::write(&path, json).map_err(|source| write_error(&path, source))?;
    Ok(path)
}

pub fn load_manifest(reference: &str) -> Result<ChangeManifest, TraceStoreError> {
    let path = resolve_reference(reference)?;
    let json = fs::read_to_string(&path).map_err(|source| read_error(&path, source))?;
    serde_json::from_str(&json).map_err(|source| TraceStoreError::Malformed {
        path: path.display().to_string(),
        source,
    })
}

pub fn list_trace_names() -> Result<Vec<String>, TraceStoreError> {
    let directory = traces_directory()?;
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(&directory).map_err(|source| read_error(&directory, source))? {
        let path = entry
            .map_err(|source| read_error(&directory, source))?
            .path();
        names.extend(trace_name_of(&path));
    }
    names.sort();
    Ok(names)
}

fn trace_name_of(path: &Path) -> Option<String> {
    let is_manifest = path.extension() == Some(OsStr::new(MANIFEST_EXTENSION));
    let name = path.file_stem().and_then(OsStr::to_str)?;
    is_manifest.then(|| name.to_owned())
}

fn resolve_reference(reference: &str) -> Result<PathBuf, TraceStoreError> {
    let direct_path = PathBuf::from(reference);
    if direct_path.is_file() {
        return Ok(direct_path);
    }
    Ok(traces_directory()?.join(format!("{reference}.{MANIFEST_EXTENSION}")))
}

pub fn store_path_for(
    program: &str,
    started_at_unix_seconds: u64,
) -> Result<PathBuf, TraceStoreError> {
    let program_name = Path::new(program)
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or(UNNAMED_PROGRAM);
    let file_name = format!("{started_at_unix_seconds}-{program_name}.{MANIFEST_EXTENSION}");
    Ok(traces_directory()?.join(file_name))
}

fn traces_directory() -> Result<PathBuf, TraceStoreError> {
    let program_data =
        env::var_os(PROGRAM_DATA_VARIABLE).ok_or(TraceStoreError::StoreLocationUnknown)?;
    Ok(PathBuf::from(program_data)
        .join(STORE_DIRECTORY)
        .join(TRACES_DIRECTORY))
}

fn write_error(path: &Path, source: io::Error) -> TraceStoreError {
    TraceStoreError::Write {
        path: path.display().to_string(),
        source,
    }
}

fn read_error(path: &Path, source: io::Error) -> TraceStoreError {
    TraceStoreError::Read {
        path: path.display().to_string(),
        source,
    }
}
