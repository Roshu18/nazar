use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use winreg::RegKey;
use winreg::RegValue;

use crate::registry_access::Hive;

const FILE_HEADER: &str = "Windows Registry Editor Version 5.00\r\n";
const LINE_END: &str = "\r\n";
const DEFAULT_VALUE_NAME: &str = "@";
const BYTE_ORDER_MARK: u16 = 0xFEFF;
const BACKUP_EXTENSION: &str = "reg";

pub(crate) struct RegistryBackup {
    directory: PathBuf,
    saved_file_count: usize,
}

impl RegistryBackup {
    pub(crate) fn in_directory(directory: &Path) -> Self {
        Self {
            directory: directory.to_owned(),
            saved_file_count: 0,
        }
    }

    pub(crate) fn directory_if_used(&self) -> Option<String> {
        (self.saved_file_count > 0).then(|| self.directory.display().to_string())
    }

    pub(crate) fn save_key_tree(&mut self, hive: Hive, path: &str, key: &RegKey) -> io::Result<()> {
        let mut export = FILE_HEADER.to_owned();
        export_key_tree(hive, path, key, &mut export)?;
        self.save(&export)
    }

    pub(crate) fn save_key_values(
        &mut self,
        hive: Hive,
        path: &str,
        key: &RegKey,
    ) -> io::Result<()> {
        let mut export = FILE_HEADER.to_owned();
        export_key_values(hive, path, key, &mut export)?;
        self.save(&export)
    }

    fn save(&mut self, export: &str) -> io::Result<()> {
        fs::create_dir_all(&self.directory)?;
        let file_name = format!("{}.{BACKUP_EXTENSION}", self.saved_file_count + 1);
        fs::write(
            self.directory.join(file_name),
            as_registry_file_bytes(export),
        )?;
        self.saved_file_count += 1;
        Ok(())
    }
}

fn export_key_tree(hive: Hive, path: &str, key: &RegKey, export: &mut String) -> io::Result<()> {
    export_key_values(hive, path, key, export)?;
    for child_name in key.enum_keys() {
        let child_name = child_name?;
        let child = key.open_subkey(&child_name)?;
        export_key_tree(hive, &format!("{path}\\{child_name}"), &child, export)?;
    }
    Ok(())
}

fn export_key_values(hive: Hive, path: &str, key: &RegKey, export: &mut String) -> io::Result<()> {
    export.push_str(LINE_END);
    for part in ["[", hive.full_name(), "\\", path, "]", LINE_END] {
        export.push_str(part);
    }
    for value in key.enum_values() {
        let (name, value) = value?;
        export.push_str(&value_line(&name, &value));
    }
    Ok(())
}

fn value_line(name: &str, value: &RegValue) -> String {
    let quoted_name = if name.is_empty() {
        DEFAULT_VALUE_NAME.to_owned()
    } else {
        format!("\"{}\"", name.replace('\\', "\\\\").replace('"', "\\\""))
    };
    let type_number = value.vtype.clone() as u32;
    let mut bytes_as_hexadecimal = String::new();
    for (position, byte) in value.bytes.iter().enumerate() {
        let separator = if position == 0 { "" } else { "," };
        write!(bytes_as_hexadecimal, "{separator}{byte:02x}").ok();
    }
    format!("{quoted_name}=hex({type_number:x}):{bytes_as_hexadecimal}{LINE_END}")
}

fn as_registry_file_bytes(export: &str) -> Vec<u8> {
    std::iter::once(BYTE_ORDER_MARK)
        .chain(export.encode_utf16())
        .flat_map(u16::to_le_bytes)
        .collect()
}
