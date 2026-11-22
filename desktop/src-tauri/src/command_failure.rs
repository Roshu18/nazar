use nazar_store::trace_store::TraceStoreError;
use nazar_windows::installed_programs::InstalledProgramsError;
use nazar_windows::shell_launch::ShellLaunchError;
use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum CommandFailure {
    #[error(transparent)]
    TraceStore(#[from] TraceStoreError),
    #[error(transparent)]
    ShellLaunch(#[from] ShellLaunchError),
    #[error(transparent)]
    InstalledPrograms(#[from] InstalledProgramsError),
    #[error("The Nazar engine (nazar.exe) is missing from the folder the app runs from.")]
    EngineMissing,
    #[error("The folder the app runs from could not be found: {0}")]
    AppLocationUnknown(std::io::Error),
    #[error("The engine could not finish: {0}")]
    EngineFailed(String),
    #[error("The work stopped unexpectedly before it could finish.")]
    WorkInterrupted,
    #[error("Nazar could not ask the tracker to stop: {0}")]
    StopRequest(std::io::Error),
    #[error("No program is being tracked right now.")]
    NothingBeingTracked,
    #[error("The system clock is set to a time before 1970.")]
    ClockBeforeEpoch,
    #[error("Windows could not provide the icon: {0}")]
    IconUnavailable(String),
    #[error("File Explorer could not be opened: {0}")]
    FileManager(std::io::Error),
    #[error("This program does not say how to uninstall it. Delete its locations instead.")]
    NoUninstaller,
    #[error("This program is no longer in the list of installed programs.")]
    ProgramNotListed,
    #[error("The list of locations to delete could not be handed to the engine: {0}")]
    DeletionRequest(String),
    #[error("The engine's account of what it deleted could not be read: {0}")]
    DeletionReport(String),
}

impl Serialize for CommandFailure {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
