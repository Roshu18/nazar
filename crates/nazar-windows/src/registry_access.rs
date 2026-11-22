use std::io;

use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS, RegType};
use winreg::types::FromRegValue;

const PATH_SEPARATOR: char = '\\';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hive {
    LocalMachine,
    CurrentUser,
    Users,
}

const HIVES: [Hive; 3] = [Hive::LocalMachine, Hive::CurrentUser, Hive::Users];

impl Hive {
    pub(crate) fn familiar_name(self) -> &'static str {
        match self {
            Self::LocalMachine => "HKLM",
            Self::CurrentUser => "HKCU",
            Self::Users => "HKU",
        }
    }

    pub(crate) fn full_name(self) -> &'static str {
        match self {
            Self::LocalMachine => "HKEY_LOCAL_MACHINE",
            Self::CurrentUser => "HKEY_CURRENT_USER",
            Self::Users => "HKEY_USERS",
        }
    }

    pub(crate) fn root(self) -> RegKey {
        RegKey::predef(match self {
            Self::LocalMachine => HKEY_LOCAL_MACHINE,
            Self::CurrentUser => HKEY_CURRENT_USER,
            Self::Users => HKEY_USERS,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RegistryLocation {
    pub hive: Hive,
    pub path: &'static str,
}

impl RegistryLocation {
    pub(crate) fn display_path(&self) -> String {
        format!("{}{PATH_SEPARATOR}{}", self.hive.familiar_name(), self.path)
    }

    pub(crate) fn open(&self) -> io::Result<Option<RegKey>> {
        found(self.hive.root().open_subkey(self.path))
    }
}

pub(crate) fn found<Found>(outcome: io::Result<Found>) -> io::Result<Option<Found>> {
    match outcome {
        Ok(found) => Ok(Some(found)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub(crate) fn text_value(key: &RegKey, name: &str) -> io::Result<Option<String>> {
    let Some(raw_value) = found(key.get_raw_value(name))? else {
        return Ok(None);
    };
    if !matches!(raw_value.vtype, RegType::REG_SZ | RegType::REG_EXPAND_SZ) {
        return Ok(None);
    }
    let text = String::from_reg_value(&raw_value)?;
    Ok(Some(text).filter(|text| !text.trim().is_empty()))
}

pub(crate) fn number_value(key: &RegKey, name: &str) -> io::Result<Option<u32>> {
    let Some(raw_value) = found(key.get_raw_value(name))? else {
        return Ok(None);
    };
    if raw_value.vtype != RegType::REG_DWORD {
        return Ok(None);
    }
    Ok(Some(u32::from_reg_value(&raw_value)?))
}

pub(crate) fn split_display_path(display_path: &str) -> Option<(Hive, &str)> {
    let (hive_name, path) = display_path.split_once(PATH_SEPARATOR)?;
    let hive = HIVES
        .into_iter()
        .find(|hive| hive.familiar_name().eq_ignore_ascii_case(hive_name))?;
    Some((hive, path))
}
