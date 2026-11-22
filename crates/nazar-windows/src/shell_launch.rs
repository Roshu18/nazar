use std::iter;
use std::mem;
use std::path::Path;

use nazar_core::command_line::join_arguments;
use windows::Win32::Foundation::{CloseHandle, ERROR_CANCELLED, HANDLE};
use windows::Win32::System::Threading::{GetExitCodeProcess, INFINITE, WaitForSingleObject};
use windows::Win32::UI::Shell::{SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOWNORMAL};
use windows::core::PCWSTR;

const RUN_AS_ADMINISTRATOR_VERB: &str = "runas";
const RUN_AS_THE_USER_VERB: &str = "open";
const STRING_TERMINATOR: u16 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elevation {
    AsTheUser,
    AsAdministrator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowVisibility {
    Hidden,
    Visible,
}

#[derive(Debug, Clone, Copy)]
pub struct ShellLaunch<'launch> {
    pub executable: &'launch Path,
    pub parameters: &'launch str,
    pub elevation: Elevation,
    pub window: WindowVisibility,
}

#[derive(Debug, thiserror::Error)]
pub enum ShellLaunchError {
    #[error("permission to run it was declined")]
    DeclinedByUser,
    #[error("it could not be started: {0}")]
    Launch(windows::core::Error),
    #[error("its result could not be read: {0}")]
    ExitCode(windows::core::Error),
}

pub fn run_elevated_and_wait(
    executable: &Path,
    arguments: &[String],
) -> Result<u32, ShellLaunchError> {
    run_and_wait(&ShellLaunch {
        executable,
        parameters: &join_arguments(arguments),
        elevation: Elevation::AsAdministrator,
        window: WindowVisibility::Hidden,
    })
}

pub fn run_and_wait(launch: &ShellLaunch<'_>) -> Result<u32, ShellLaunchError> {
    let process = start(launch)?;
    let exit_code = wait_for_exit_code(process);
    close_process_handle(process)?;
    exit_code
}

#[allow(unsafe_code)]
fn start(launch: &ShellLaunch<'_>) -> Result<HANDLE, ShellLaunchError> {
    let verb = as_wide_text(match launch.elevation {
        Elevation::AsTheUser => RUN_AS_THE_USER_VERB,
        Elevation::AsAdministrator => RUN_AS_ADMINISTRATOR_VERB,
    });
    let file = as_wide_text(&launch.executable.to_string_lossy());
    let parameters = as_wide_text(launch.parameters);
    let window_state = match launch.window {
        WindowVisibility::Hidden => SW_HIDE,
        WindowVisibility::Visible => SW_SHOWNORMAL,
    };
    let mut execution = SHELLEXECUTEINFOW {
        cbSize: size_of_execution_structure(),
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(parameters.as_ptr()),
        nShow: window_state.0,
        ..SHELLEXECUTEINFOW::default()
    };
    unsafe { ShellExecuteExW(&raw mut execution) }.map_err(launch_error)?;
    Ok(execution.hProcess)
}

#[allow(unsafe_code)]
fn wait_for_exit_code(process: HANDLE) -> Result<u32, ShellLaunchError> {
    let mut exit_code = 0;
    unsafe {
        WaitForSingleObject(process, INFINITE);
        GetExitCodeProcess(process, &raw mut exit_code)
    }
    .map_err(ShellLaunchError::ExitCode)?;
    Ok(exit_code)
}

#[allow(unsafe_code)]
fn close_process_handle(process: HANDLE) -> Result<(), ShellLaunchError> {
    unsafe { CloseHandle(process) }.map_err(ShellLaunchError::ExitCode)
}

fn launch_error(error: windows::core::Error) -> ShellLaunchError {
    if error.code() == ERROR_CANCELLED.to_hresult() {
        return ShellLaunchError::DeclinedByUser;
    }
    ShellLaunchError::Launch(error)
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "the execution structure is about a hundred bytes"
)]
fn size_of_execution_structure() -> u32 {
    mem::size_of::<SHELLEXECUTEINFOW>() as u32
}

fn as_wide_text(text: &str) -> Vec<u16> {
    text.encode_utf16()
        .chain(iter::once(STRING_TERMINATOR))
        .collect()
}
