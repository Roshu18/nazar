use crate::manifest_report::{render_changes, render_removal_plan, render_summary};
use nazar_store::trace_store::{TraceStoreError, list_trace_names, load_manifest};

#[derive(Debug, clap::Args)]
pub struct ShowArguments {
    #[arg(
        value_name = "TRACE",
        help = "A trace name from `nazar list`, or the path of a trace file"
    )]
    pub trace: String,
    #[arg(
        long,
        help = "Show the steps that would undo the trace instead of its changes"
    )]
    pub removal_plan: bool,
}

pub fn show_trace(arguments: &ShowArguments) -> Result<(), TraceStoreError> {
    let manifest = load_manifest(&arguments.trace)?;
    let details = if arguments.removal_plan {
        render_removal_plan(&manifest)
    } else {
        render_changes(&manifest)
    };
    println!("{}", render_summary(&manifest));
    print!("{details}");
    Ok(())
}

pub fn list_traces() -> Result<(), TraceStoreError> {
    let names = list_trace_names()?;
    if names.is_empty() {
        println!("No traces recorded yet. Record one with `nazar watch <program>`.");
    }
    for name in names {
        println!("{name}");
    }
    Ok(())
}
