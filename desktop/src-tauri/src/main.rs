#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod command_failure;
mod commands;
mod deletion_commands;
mod engine;
mod icon_source;
mod program_commands;
mod program_view;
mod shell_commands;
mod trace_view;
mod tracking_request;
mod window_icon;

use std::process::ExitCode;

use crate::commands::{TrackingState, list_traces, read_trace, stop_tracking, track_program};
use crate::deletion_commands::delete_locations;
use crate::program_commands::{
    ProgramCatalogue, list_installed_programs, list_leftovers, read_program, uninstall_program,
};

use crate::shell_commands::{has_mica, measure_sizes, read_icon, show_in_file_manager};

fn main() -> ExitCode {
    let outcome = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| Ok(window_icon::sharpen_window_icons(app)?))
        .manage(TrackingState::default())
        .manage(ProgramCatalogue::default())
        .invoke_handler(tauri::generate_handler![
            list_traces,
            read_trace,
            track_program,
            stop_tracking,
            list_installed_programs,
            read_program,
            delete_locations,
            read_icon,
            measure_sizes,
            show_in_file_manager,
            has_mica,
            uninstall_program,
            list_leftovers
        ])
        .run(tauri::generate_context!());
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nazar: the app could not start: {error}");
            ExitCode::FAILURE
        }
    }
}
