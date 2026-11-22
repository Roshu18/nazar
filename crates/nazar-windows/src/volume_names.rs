use std::io;
use std::iter;

use nazar_core::kernel_path::{VolumeName, VolumeNames};
use windows::Win32::Storage::FileSystem::{GetLogicalDrives, QueryDosDeviceW};
use windows::core::PCWSTR;

const DRIVE_LETTERS: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const DEVICE_PATH_CAPACITY: usize = 1024;
const STRING_TERMINATOR: u16 = 0;

#[derive(Debug, thiserror::Error)]
#[error("the device behind drive {drive} could not be read: {source}")]
pub struct VolumeNamesError {
    drive: String,
    source: io::Error,
}

pub(crate) fn read_volume_names() -> Result<VolumeNames, VolumeNamesError> {
    let present_drives = read_present_drive_mask();
    let volumes = DRIVE_LETTERS
        .iter()
        .enumerate()
        .filter(|(position, _)| present_drives & (1 << position) != 0)
        .map(|(_, letter)| read_volume_name(*letter))
        .collect::<Result<Vec<VolumeName>, VolumeNamesError>>()?;
    Ok(VolumeNames::new(volumes))
}

#[allow(unsafe_code)]
fn read_present_drive_mask() -> u32 {
    unsafe { GetLogicalDrives() }
}

#[allow(unsafe_code)]
fn read_volume_name(letter: char) -> Result<VolumeName, VolumeNamesError> {
    let drive = format!("{letter}:");
    let drive_as_wide_text: Vec<u16> = drive
        .encode_utf16()
        .chain(iter::once(STRING_TERMINATOR))
        .collect();
    let mut device_path = [STRING_TERMINATOR; DEVICE_PATH_CAPACITY];
    let written_length =
        unsafe { QueryDosDeviceW(PCWSTR(drive_as_wide_text.as_ptr()), Some(&mut device_path)) };
    if written_length == 0 {
        let source = io::Error::last_os_error();
        return Err(VolumeNamesError { drive, source });
    }
    Ok(VolumeName {
        device_path: text_before_terminator(&device_path),
        drive,
    })
}

fn text_before_terminator(wide_text: &[u16]) -> String {
    let text_length = wide_text
        .iter()
        .position(|unit| *unit == STRING_TERMINATOR)
        .unwrap_or(wide_text.len());
    String::from_utf16_lossy(&wide_text[..text_length])
}
