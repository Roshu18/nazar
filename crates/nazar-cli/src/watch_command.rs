use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use nazar_core::change_manifest::{ChangeManifest, MANIFEST_FORMAT_VERSION};
use nazar_windows::kernel_trace::{KernelTraceError, KernelTraceSession};
use nazar_windows::running_processes::RunningProcessesError;

use crate::disk_verification::discard_created_entries_missing_from_disk;
use crate::launch_plan::plan_launch;
use crate::manifest_report::render_summary;
use crate::watch_session::{WatchOutcome, WatchSession};
use nazar_store::trace_store::{TraceStoreError, save_manifest};

#[derive(Debug, clap::Args)]
pub struct WatchArguments {
    #[arg(
        long,
        value_name = "FILE",
        help = "Save the trace to this file instead of the Nazar trace store"
    )]
    pub output: Option<PathBuf>,
    #[arg(
        long,
        value_name = "FILE",
        help = "Stop watching as soon as this file exists"
    )]
    pub stop_file: Option<PathBuf>,
    #[arg(
        value_name = "PROGRAM",
        help = "The installer, executable or script to run"
    )]
    pub program: PathBuf,
    #[arg(
        value_name = "ARGUMENTS",
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "Arguments passed to the program"
    )]
    pub arguments: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum WatchError {
    #[error(transparent)]
    KernelTrace(#[from] KernelTraceError),
    #[error(transparent)]
    RunningProcesses(#[from] RunningProcessesError),
    #[error("`{program}` could not be started: {source}")]
    Launch { program: String, source: io::Error },
    #[error("the state of the launched program could not be read: {0}")]
    ProgramState(io::Error),
    #[error("kernel tracing ended before the program finished")]
    TraceEndedEarly,
    #[error("the system clock is set to a time before 1970")]
    ClockBeforeEpoch,
    #[error(transparent)]
    TraceStore(#[from] TraceStoreError),
}

pub fn watch_program(arguments: &WatchArguments) -> Result<(), WatchError> {
    let (sink, signals) = mpsc::channel();
    let trace_session = KernelTraceSession::start(&sink)?;
    drop(sink);
    let mut watch_session = WatchSession::start(signals)?;
    let started_at = unix_seconds_now()?;
    let mut program = launch(arguments)?;
    println!(
        "Watching {}. Press Enter to stop early.",
        arguments.program.display()
    );
    let stop_was_typed = listen_for_stop_request();
    let is_stop_requested = || {
        let stop_file_exists = arguments.stop_file.as_deref().is_some_and(Path::exists);
        stop_file_exists || stop_was_typed.load(Ordering::Relaxed)
    };
    watch_session.follow_until_settled(&mut program, is_stop_requested)?;
    trace_session.stop()?;
    let outcome = watch_session.finish();
    report_decode_failures(&outcome);
    let manifest = build_manifest(arguments, started_at, unix_seconds_now()?, outcome);
    let saved_path = save_manifest(&manifest, arguments.output.as_deref())?;
    print!("{}", render_summary(&manifest));
    println!("Trace saved to {}", saved_path.display());
    Ok(())
}

fn launch(arguments: &WatchArguments) -> Result<Child, WatchError> {
    let plan = plan_launch(&arguments.program, &arguments.arguments);
    Command::new(&plan.executable)
        .args(&plan.arguments)
        .spawn()
        .map_err(|source| WatchError::Launch {
            program: arguments.program.display().to_string(),
            source,
        })
}

fn listen_for_stop_request() -> Arc<AtomicBool> {
    let stop_requested = Arc::new(AtomicBool::new(false));
    let stop_requested_by_listener = Arc::clone(&stop_requested);
    thread::spawn(move || {
        let mut line = String::new();
        let user_pressed_enter = io::stdin()
            .read_line(&mut line)
            .is_ok_and(|bytes_read| bytes_read > 0);
        if user_pressed_enter {
            stop_requested_by_listener.store(true, Ordering::Relaxed);
        }
    });
    stop_requested
}

fn report_decode_failures(outcome: &WatchOutcome) {
    if outcome.undecoded_event_count == 0 {
        return;
    }
    eprintln!(
        "Warning: {} events from the program could not be decoded, so the trace may be incomplete.",
        outcome.undecoded_event_count
    );
    for example in &outcome.decode_failure_examples {
        eprintln!("  {example}");
    }
}

fn build_manifest(
    arguments: &WatchArguments,
    started_at_unix_seconds: u64,
    finished_at_unix_seconds: u64,
    outcome: WatchOutcome,
) -> ChangeManifest {
    ChangeManifest {
        format_version: MANIFEST_FORMAT_VERSION,
        program: arguments.program.display().to_string(),
        arguments: arguments.arguments.clone(),
        started_at_unix_seconds,
        finished_at_unix_seconds,
        undecoded_event_count: outcome.undecoded_event_count,
        changes: discard_created_entries_missing_from_disk(outcome.changes),
    }
}

fn unix_seconds_now() -> Result<u64, WatchError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since_epoch| since_epoch.as_secs())
        .map_err(|_| WatchError::ClockBeforeEpoch)
}
