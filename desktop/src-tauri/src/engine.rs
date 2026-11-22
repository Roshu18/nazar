use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use nazar_windows::shell_launch::run_elevated_and_wait;

use crate::command_failure::CommandFailure;

const ENGINE_FILE_NAME: &str = "nazar.exe";
const SUCCESS_EXIT_CODE: u32 = 0;
const UNEXPLAINED_ENGINE_FAILURE: &str = "the engine stopped without saying why";

pub struct EngineSession {
    started_at_unix_seconds: u64,
}

impl EngineSession {
    pub fn begin() -> Result<Self, CommandFailure> {
        let started_at_unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|since_epoch| since_epoch.as_secs())
            .map_err(|_| CommandFailure::ClockBeforeEpoch)?;
        Ok(Self {
            started_at_unix_seconds,
        })
    }

    pub fn started_at_unix_seconds(&self) -> u64 {
        self.started_at_unix_seconds
    }

    pub fn file(&self, purpose: &str) -> PathBuf {
        let file_name = format!("nazar-{}-{purpose}", self.started_at_unix_seconds);
        env::temp_dir().join(file_name)
    }

    pub fn run_engine(&self, arguments: &[String]) -> Result<(), CommandFailure> {
        let error_file = self.file("error");
        let mut engine_arguments = vec![
            "--error-file".to_owned(),
            error_file.to_string_lossy().into_owned(),
        ];
        engine_arguments.extend_from_slice(arguments);
        let exit_code = run_elevated_and_wait(&find_engine()?, &engine_arguments)?;
        if exit_code == SUCCESS_EXIT_CODE {
            Ok(())
        } else {
            Err(CommandFailure::EngineFailed(read_engine_failure(
                &error_file,
            )))
        }
    }
}

fn find_engine() -> Result<PathBuf, CommandFailure> {
    let app_path = env::current_exe().map_err(CommandFailure::AppLocationUnknown)?;
    let engine_path = app_path.with_file_name(ENGINE_FILE_NAME);
    if engine_path.is_file() {
        Ok(engine_path)
    } else {
        Err(CommandFailure::EngineMissing)
    }
}

fn read_engine_failure(error_file: &PathBuf) -> String {
    match fs::read_to_string(error_file) {
        Ok(reason) if !reason.is_empty() => reason,
        Ok(_) => UNEXPLAINED_ENGINE_FAILURE.to_owned(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            UNEXPLAINED_ENGINE_FAILURE.to_owned()
        }
        Err(error) => format!("{UNEXPLAINED_ENGINE_FAILURE} ({error})"),
    }
}
