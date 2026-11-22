use std::mem;

use nazar_core::observation::ProcessId;
use windows::Win32::Foundation::{CloseHandle, ERROR_NO_MORE_FILES, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};

const EVERY_PROCESS: u32 = 0;
const STRING_TERMINATOR: u16 = 0;

#[derive(Debug, thiserror::Error)]
#[error("the list of running processes could not be read: {0}")]
pub struct RunningProcessesError(#[from] windows::core::Error);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningProcess {
    pub process: ProcessId,
    pub image_name: String,
}

#[allow(unsafe_code)]
pub fn read_running_processes() -> Result<Vec<RunningProcess>, RunningProcessesError> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, EVERY_PROCESS) }?;
    let processes = collect_processes(snapshot);
    unsafe { CloseHandle(snapshot) }?;
    processes
}

#[allow(unsafe_code)]
fn collect_processes(snapshot: HANDLE) -> Result<Vec<RunningProcess>, RunningProcessesError> {
    let mut entry = empty_process_entry();
    let mut processes = Vec::new();
    let mut has_entry = entry_was_read(unsafe { Process32FirstW(snapshot, &raw mut entry) })?;
    while has_entry {
        processes.push(running_process_from(&entry));
        has_entry = entry_was_read(unsafe { Process32NextW(snapshot, &raw mut entry) })?;
    }
    Ok(processes)
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "the process entry structure is a few hundred bytes"
)]
fn empty_process_entry() -> PROCESSENTRY32W {
    PROCESSENTRY32W {
        dwSize: mem::size_of::<PROCESSENTRY32W>() as u32,
        ..PROCESSENTRY32W::default()
    }
}

fn entry_was_read(outcome: windows::core::Result<()>) -> Result<bool, RunningProcessesError> {
    match outcome {
        Ok(()) => Ok(true),
        Err(error) if error.code() == ERROR_NO_MORE_FILES.to_hresult() => Ok(false),
        Err(error) => Err(RunningProcessesError(error)),
    }
}

fn running_process_from(entry: &PROCESSENTRY32W) -> RunningProcess {
    let name_length = entry
        .szExeFile
        .iter()
        .position(|unit| *unit == STRING_TERMINATOR)
        .unwrap_or(entry.szExeFile.len());
    RunningProcess {
        process: ProcessId(entry.th32ProcessID),
        image_name: String::from_utf16_lossy(&entry.szExeFile[..name_length]),
    }
}
