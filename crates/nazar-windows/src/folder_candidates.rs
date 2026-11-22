use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use nazar_core::program_locations::{LocationCandidates, LocationRole, NamedLocation};
use nazar_core::tracked_object::TrackedObject;

const SHORTCUT_EXTENSION: &str = "lnk";
const FOLDER_HOLDING_ONLY_SHORTCUTS: &str = "Desktop";
const START_MENU_PROGRAMS: [&str; 4] = ["Microsoft", "Windows", "Start Menu", "Programs"];

struct FolderRoot {
    role: LocationRole,
    environment_variable: &'static str,
    folders_below: &'static [&'static str],
}

const fn root(
    role: LocationRole,
    environment_variable: &'static str,
    folders_below: &'static [&'static str],
) -> FolderRoot {
    FolderRoot {
        role,
        environment_variable,
        folders_below,
    }
}

const FOLDER_ROOTS: [FolderRoot; 13] = [
    root(LocationRole::ProgramFolder, "ProgramFiles", &[]),
    root(LocationRole::ProgramFolder, "ProgramFiles(x86)", &[]),
    root(LocationRole::ProgramFolder, "CommonProgramFiles", &[]),
    root(LocationRole::ProgramFolder, "CommonProgramFiles(x86)", &[]),
    root(LocationRole::ProgramFolder, "LOCALAPPDATA", &["Programs"]),
    root(LocationRole::DataFolder, "ProgramData", &[]),
    root(LocationRole::DataFolder, "APPDATA", &[]),
    root(LocationRole::DataFolder, "LOCALAPPDATA", &[]),
    root(
        LocationRole::DataFolder,
        "USERPROFILE",
        &["AppData", "LocalLow"],
    ),
    root(LocationRole::Shortcut, "ProgramData", &START_MENU_PROGRAMS),
    root(LocationRole::Shortcut, "APPDATA", &START_MENU_PROGRAMS),
    root(LocationRole::Shortcut, "USERPROFILE", &["Desktop"]),
    root(LocationRole::Shortcut, "PUBLIC", &["Desktop"]),
];

pub(crate) fn collect_folder_candidates(candidates: &mut LocationCandidates) {
    for folder_root in &FOLDER_ROOTS {
        let Some(path) = path_of(folder_root) else {
            continue;
        };
        collect_named_children(&path, None, folder_root.role, candidates);
    }
}

fn path_of(folder_root: &FolderRoot) -> Option<PathBuf> {
    let mut path = PathBuf::from(env::var_os(folder_root.environment_variable)?);
    path.extend(folder_root.folders_below);
    path.is_dir().then_some(path)
}

fn collect_named_children(
    folder: &Path,
    publisher_name: Option<&str>,
    role: LocationRole,
    candidates: &mut LocationCandidates,
) {
    let Ok(entries) = fs::read_dir(folder) else {
        candidates.unreadable_location_count += 1;
        return;
    };
    for entry in entries {
        let Ok(entry) = entry else {
            candidates.unreadable_location_count += 1;
            continue;
        };
        let path = entry.path();
        if holds_only_shortcuts(folder) && is_real_folder(&path) {
            continue;
        }
        let Some(name) = candidate_name(&path, role) else {
            continue;
        };
        candidates.named_locations.push(NamedLocation {
            object: TrackedObject::file_system_entry(path.to_string_lossy()),
            name: name.clone(),
            publisher_name: publisher_name.map(str::to_owned),
            role,
        });
        if publisher_name.is_none() && is_real_folder(&path) {
            collect_named_children(&path, Some(&name), role, candidates);
        }
    }
}

fn holds_only_shortcuts(folder: &Path) -> bool {
    folder.file_name() == Some(OsStr::new(FOLDER_HOLDING_ONLY_SHORTCUTS))
}

fn candidate_name(path: &Path, role: LocationRole) -> Option<String> {
    let is_shortcut_file =
        role == LocationRole::Shortcut && path.extension() == Some(OsStr::new(SHORTCUT_EXTENSION));
    let name = if is_shortcut_file {
        path.file_stem()
    } else if is_real_folder(path) {
        path.file_name()
    } else {
        None
    };
    name.map(|name| name.to_string_lossy().into_owned())
}

fn is_real_folder(path: &Path) -> bool {
    path.symlink_metadata()
        .is_ok_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
}
