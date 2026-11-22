use std::io;

use nazar_core::program_locations::{
    LocationCandidates, LocationRole, NamedLocation, PathReference,
};
use nazar_core::tracked_object::TrackedObject;
use winreg::RegKey;
use winreg::enums::RegType;
use winreg::types::FromRegValue;

use crate::registry_access::{Hive, RegistryLocation, text_value};

const fn at(hive: Hive, path: &'static str) -> RegistryLocation {
    RegistryLocation { hive, path }
}

const SETTINGS_LOCATIONS: [RegistryLocation; 3] = [
    at(Hive::LocalMachine, "SOFTWARE"),
    at(Hive::LocalMachine, "SOFTWARE\\WOW6432Node"),
    at(Hive::CurrentUser, "Software"),
];
const KEYS_WITHOUT_PROGRAM_CHILDREN: [&str; 3] = ["classes", "wow6432node", "policies"];
const AUTOSTART_LOCATIONS: [RegistryLocation; 6] = [
    at(
        Hive::LocalMachine,
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
    ),
    at(
        Hive::LocalMachine,
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
    ),
    at(
        Hive::LocalMachine,
        "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run",
    ),
    at(
        Hive::LocalMachine,
        "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
    ),
    at(
        Hive::CurrentUser,
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
    ),
    at(
        Hive::CurrentUser,
        "Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
    ),
];
const SERVICES_LOCATION: RegistryLocation =
    at(Hive::LocalMachine, "SYSTEM\\CurrentControlSet\\Services");
const APP_PATHS_LOCATION: RegistryLocation = at(
    Hive::LocalMachine,
    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\App Paths",
);
const SERVICE_IMAGE_VALUE: &str = "ImagePath";
const DEFAULT_VALUE: &str = "";

pub(crate) fn collect_registry_candidates(candidates: &mut LocationCandidates) {
    for location in &SETTINGS_LOCATIONS {
        collect_settings_keys(location, candidates);
    }
    for location in &AUTOSTART_LOCATIONS {
        collect_autostart_values(location, candidates);
    }
    let service = LocationRole::Service;
    collect_keys_referencing_files(&SERVICES_LOCATION, SERVICE_IMAGE_VALUE, service, candidates);
    let app_path = LocationRole::AppPath;
    collect_keys_referencing_files(&APP_PATHS_LOCATION, DEFAULT_VALUE, app_path, candidates);
}

fn collect_settings_keys(location: &RegistryLocation, candidates: &mut LocationCandidates) {
    let Some(root) = readable(location.open(), candidates).flatten() else {
        return;
    };
    for name in root.enum_keys() {
        let Some(name) = readable(name, candidates) else {
            continue;
        };
        let path = format!("{}\\{name}", location.display_path());
        candidates
            .named_locations
            .push(settings_key(path.clone(), name.clone(), None));
        if !KEYS_WITHOUT_PROGRAM_CHILDREN.contains(&name.to_lowercase().as_str()) {
            collect_publisher_children(&root, &name, &path, candidates);
        }
    }
}

fn collect_publisher_children(
    root: &RegKey,
    publisher_name: &str,
    publisher_path: &str,
    candidates: &mut LocationCandidates,
) {
    let Some(publisher_key) = readable(root.open_subkey(publisher_name), candidates) else {
        return;
    };
    for child_name in publisher_key.enum_keys() {
        let Some(child_name) = readable(child_name, candidates) else {
            continue;
        };
        let path = format!("{publisher_path}\\{child_name}");
        let publisher_name = Some(publisher_name.to_owned());
        candidates
            .named_locations
            .push(settings_key(path, child_name, publisher_name));
    }
}

fn collect_autostart_values(location: &RegistryLocation, candidates: &mut LocationCandidates) {
    let Some(key) = readable(location.open(), candidates).flatten() else {
        return;
    };
    for value in key.enum_values() {
        let Some((name, value)) = readable(value, candidates) else {
            continue;
        };
        if !matches!(value.vtype, RegType::REG_SZ | RegType::REG_EXPAND_SZ) {
            continue;
        }
        let Some(referenced_text) = readable(String::from_reg_value(&value), candidates) else {
            continue;
        };
        candidates.path_references.push(PathReference {
            object: TrackedObject::registry_value(location.display_path(), name),
            referenced_text,
            role: LocationRole::Autostart,
        });
    }
}

fn collect_keys_referencing_files(
    location: &RegistryLocation,
    value_name: &str,
    role: LocationRole,
    candidates: &mut LocationCandidates,
) {
    let Some(root) = readable(location.open(), candidates).flatten() else {
        return;
    };
    for key_name in root.enum_keys() {
        let Some(key_name) = readable(key_name, candidates) else {
            continue;
        };
        let referenced_text = root
            .open_subkey(&key_name)
            .and_then(|key| text_value(&key, value_name));
        let Some(referenced_text) = readable(referenced_text, candidates).flatten() else {
            continue;
        };
        candidates.path_references.push(PathReference {
            object: TrackedObject::registry_key(format!("{}\\{key_name}", location.display_path())),
            referenced_text,
            role,
        });
    }
}

fn settings_key(path: String, name: String, publisher_name: Option<String>) -> NamedLocation {
    NamedLocation {
        object: TrackedObject::registry_key(path),
        name,
        publisher_name,
        role: LocationRole::SettingsKey,
    }
}

fn readable<Read>(outcome: io::Result<Read>, candidates: &mut LocationCandidates) -> Option<Read> {
    if outcome.is_err() {
        candidates.unreadable_location_count += 1;
    }
    outcome.ok()
}
