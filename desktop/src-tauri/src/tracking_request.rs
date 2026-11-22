use std::path::{Path, PathBuf};

use nazar_store::trace_store::store_path_for;

use crate::command_failure::CommandFailure;
use crate::engine::EngineSession;

pub struct TrackingRequest {
    session: EngineSession,
    program_path: String,
    trace_path: PathBuf,
    stop_file: PathBuf,
}

impl TrackingRequest {
    pub fn prepare(program_path: String) -> Result<Self, CommandFailure> {
        let session = EngineSession::begin()?;
        Ok(Self {
            trace_path: store_path_for(&program_path, session.started_at_unix_seconds())?,
            stop_file: session.file("stop"),
            session,
            program_path,
        })
    }

    pub fn stop_file(&self) -> &Path {
        &self.stop_file
    }

    pub fn run(self) -> Result<String, CommandFailure> {
        self.session.run_engine(&[
            "watch".to_owned(),
            "--output".to_owned(),
            self.trace_path.to_string_lossy().into_owned(),
            "--stop-file".to_owned(),
            self.stop_file.to_string_lossy().into_owned(),
            self.program_path.clone(),
        ])?;
        Ok(self
            .trace_path
            .file_stem()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default())
    }
}
