use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::UI::Shell::{FO_DELETE, SHFILEOPSTRUCTW, SHFileOperationW};
use windows::core::PCWSTR;

const ALLOW_UNDO: u16 = 0x0040;
const NO_CONFIRMATION: u16 = 0x0010;
const NO_PROGRESS_WINDOW: u16 = 0x0004;
const NO_ERROR_WINDOW: u16 = 0x0400;
const WARN_BEFORE_PERMANENT_DELETION: u16 = 0x4000;
const RECYCLE_FLAGS: u16 = ALLOW_UNDO
    | NO_CONFIRMATION
    | NO_PROGRESS_WINDOW
    | NO_ERROR_WINDOW
    | WARN_BEFORE_PERMANENT_DELETION;
const PATH_LIST_TERMINATOR: [u16; 2] = [0, 0];
const SHELL_SUCCESS: i32 = 0;

#[derive(Debug, thiserror::Error)]
pub enum RecycleBinError {
    #[error("Windows could not move it to the Recycle Bin (shell error {0})")]
    Shell(i32),
    #[error("Windows stopped before it was moved to the Recycle Bin")]
    Aborted,
}

#[allow(unsafe_code)]
pub fn move_to_recycle_bin(path: &Path) -> Result<(), RecycleBinError> {
    let path_list: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(PATH_LIST_TERMINATOR)
        .collect();
    let mut operation = SHFILEOPSTRUCTW {
        wFunc: FO_DELETE,
        pFrom: PCWSTR(path_list.as_ptr()),
        fFlags: RECYCLE_FLAGS,
        ..SHFILEOPSTRUCTW::default()
    };
    let shell_error = unsafe { SHFileOperationW(&raw mut operation) };
    if shell_error != SHELL_SUCCESS {
        return Err(RecycleBinError::Shell(shell_error));
    }
    if operation.fAnyOperationsAborted.as_bool() {
        return Err(RecycleBinError::Aborted);
    }
    Ok(())
}
