mod delete_command;
mod disk_verification;
mod launch_plan;
mod manifest_report;
mod programs_command;
mod show_command;
mod watch_command;
mod watch_session;

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use nazar_store::trace_store::TraceStoreError;
use nazar_windows::installed_programs::InstalledProgramsError;

use crate::delete_command::{DeleteArguments, DeleteError, delete_locations};
use crate::programs_command::{LocateArguments, list_leftovers, list_programs, locate_programs};
use crate::show_command::{ShowArguments, list_traces, show_trace};
use crate::watch_command::{WatchArguments, WatchError, watch_program};

#[derive(Parser)]
#[command(
    name = "nazar",
    version,
    about = "Tracks everything a program changes on Windows and puts it back."
)]
struct CommandLine {
    #[arg(
        long,
        global = true,
        value_name = "FILE",
        help = "Also write the reason of a failure to this file"
    )]
    error_file: Option<PathBuf>,
    #[command(subcommand)]
    command: NazarCommand,
}

#[derive(Subcommand)]
enum NazarCommand {
    #[command(about = "Run a program and record every file and registry change it makes")]
    Watch(WatchArguments),
    #[command(about = "List the recorded traces")]
    List,
    #[command(about = "Show what a recorded trace contains")]
    Show(ShowArguments),
    #[command(about = "List the programs installed on this computer")]
    Programs,
    #[command(about = "Show every folder and registry entry that belongs to an installed program")]
    Locate(LocateArguments),
    #[command(
        about = "List folders and registry entries that programs no longer installed left behind"
    )]
    Leftovers,
    #[command(
        about = "Delete locations: files go to the Recycle Bin, registry entries are backed up first"
    )]
    Delete(DeleteArguments),
}

#[derive(Debug, thiserror::Error)]
enum CommandError {
    #[error(transparent)]
    Watch(#[from] WatchError),
    #[error(transparent)]
    TraceStore(#[from] TraceStoreError),
    #[error(transparent)]
    InstalledPrograms(#[from] InstalledProgramsError),
    #[error(transparent)]
    Delete(#[from] DeleteError),
}

fn main() -> ExitCode {
    let command_line = CommandLine::parse();
    match run(command_line.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            report_failure(&error, command_line.error_file.as_ref());
            ExitCode::FAILURE
        }
    }
}

fn report_failure(error: &CommandError, error_file: Option<&PathBuf>) {
    eprintln!("nazar: {error}");
    let Some(error_file) = error_file else {
        return;
    };
    if let Err(write_error) = fs::write(error_file, error.to_string()) {
        eprintln!(
            "nazar: the failure could not be written to {}: {write_error}",
            error_file.display()
        );
    }
}

fn run(command: NazarCommand) -> Result<(), CommandError> {
    match command {
        NazarCommand::Watch(arguments) => watch_program(&arguments)?,
        NazarCommand::List => list_traces()?,
        NazarCommand::Show(arguments) => show_trace(&arguments)?,
        NazarCommand::Programs => list_programs()?,
        NazarCommand::Locate(arguments) => locate_programs(&arguments)?,
        NazarCommand::Leftovers => list_leftovers()?,
        NazarCommand::Delete(arguments) => delete_locations(&arguments)?,
    }
    Ok(())
}
