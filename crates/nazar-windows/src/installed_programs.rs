use std::io;

use nazar_core::installed_program::InstalledProgram;
use nazar_core::windows_installer::packed_product_code;
use winreg::RegKey;

use crate::registry_access::{Hive, RegistryLocation, number_value, text_value};

const UNINSTALL_LOCATIONS: [RegistryLocation; 3] = [
    RegistryLocation {
        hive: Hive::LocalMachine,
        path: "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
    },
    RegistryLocation {
        hive: Hive::LocalMachine,
        path: "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
    },
    RegistryLocation {
        hive: Hive::CurrentUser,
        path: "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
    },
];
const SYSTEM_COMPONENT_FLAG: u32 = 1;
const PRODUCT_ICON_VALUE: &str = "ProductIcon";
const PRODUCT_LOCATIONS: [RegistryLocation; 2] = [
    RegistryLocation {
        hive: Hive::LocalMachine,
        path: "SOFTWARE\\Classes\\Installer\\Products\\",
    },
    RegistryLocation {
        hive: Hive::CurrentUser,
        path: "Software\\Microsoft\\Installer\\Products\\",
    },
];

#[derive(Debug, Default)]
pub struct InstalledProgramInventory {
    pub programs: Vec<InstalledProgram>,
    pub unreadable_entry_count: usize,
}

#[derive(Debug, thiserror::Error)]
#[error("the installed programs listed under {location} could not be read: {source}")]
pub struct InstalledProgramsError {
    location: String,
    source: io::Error,
}

pub fn read_installed_programs() -> Result<InstalledProgramInventory, InstalledProgramsError> {
    let mut inventory = InstalledProgramInventory::default();
    for location in UNINSTALL_LOCATIONS {
        read_location(&location, &mut inventory).map_err(|source| InstalledProgramsError {
            location: location.display_path(),
            source,
        })?;
    }
    inventory
        .programs
        .sort_by_key(|program| program.display_name.to_lowercase());
    Ok(inventory)
}

fn read_location(
    location: &RegistryLocation,
    inventory: &mut InstalledProgramInventory,
) -> io::Result<()> {
    let Some(uninstall_key) = location.open()? else {
        return Ok(());
    };
    for entry_name in uninstall_key.enum_keys() {
        let entry_name = entry_name?;
        let entry_path = format!("{}\\{entry_name}", location.display_path());
        match read_entry(&uninstall_key, &entry_name, entry_path) {
            Ok(Some(program)) => inventory.programs.push(program),
            Ok(None) => {}
            Err(_) => inventory.unreadable_entry_count += 1,
        }
    }
    Ok(())
}

fn read_entry(
    uninstall_key: &RegKey,
    entry_name: &str,
    entry_path: String,
) -> io::Result<Option<InstalledProgram>> {
    let entry = uninstall_key.open_subkey(entry_name)?;
    let Some(display_name) = text_value(&entry, "DisplayName")? else {
        return Ok(None);
    };
    let is_system_component =
        number_value(&entry, "SystemComponent")? == Some(SYSTEM_COMPONENT_FLAG);
    let is_update_of_another_entry = text_value(&entry, "ParentKeyName")?.is_some();
    if is_system_component || is_update_of_another_entry {
        return Ok(None);
    }
    Ok(Some(InstalledProgram {
        registry_key_path: entry_path,
        display_name,
        publisher: text_value(&entry, "Publisher")?,
        version: text_value(&entry, "DisplayVersion")?,
        install_location: text_value(&entry, "InstallLocation")?,
        display_icon: text_value(&entry, "DisplayIcon")?.or_else(|| product_icon_of(entry_name)),
        uninstall_command: text_value(&entry, "UninstallString")?,
        install_date: text_value(&entry, "InstallDate")?,
        estimated_size_in_kilobytes: number_value(&entry, "EstimatedSize")?.map(u64::from),
    }))
}

fn product_icon_of(entry_name: &str) -> Option<String> {
    let packed_code = packed_product_code(entry_name)?;
    PRODUCT_LOCATIONS.iter().find_map(|location| {
        let product = location
            .hive
            .root()
            .open_subkey(format!("{}{packed_code}", location.path))
            .ok()?;
        text_value(&product, PRODUCT_ICON_VALUE).ok().flatten()
    })
}
