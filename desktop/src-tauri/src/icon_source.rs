use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use nazar_core::install_folder::icon_file_of;
use nazar_core::installed_program::InstalledProgram;
use nazar_core::program_identity::ProgramIdentity;
use nazar_core::program_locations::{LocationCandidates, LocationRole, find_program_locations};

const EXECUTABLE_EXTENSION: &str = "exe";

pub fn icon_source_of(
    program: &InstalledProgram,
    candidates: &LocationCandidates,
) -> Option<String> {
    let declared_icon = icon_file_of(program)
        .map(PathBuf::from)
        .filter(|icon_file| icon_file.is_file());
    declared_icon
        .or_else(|| main_executable_of(program, candidates))
        .map(|source| source.to_string_lossy().into_owned())
}

fn main_executable_of(
    program: &InstalledProgram,
    candidates: &LocationCandidates,
) -> Option<PathBuf> {
    let identity = ProgramIdentity::of(program);
    let executables: Vec<PathBuf> = program_folders_of(program, candidates)
        .iter()
        .flat_map(|folder| executables_in(folder))
        .collect();
    let named_after_program = executables
        .iter()
        .find(|executable| identity.is_named(&stem_of(executable)));
    named_after_program
        .or_else(|| largest_of(&executables))
        .cloned()
}

fn program_folders_of(program: &InstalledProgram, candidates: &LocationCandidates) -> Vec<PathBuf> {
    find_program_locations(program, candidates)
        .into_iter()
        .filter(|location| {
            matches!(
                location.role,
                LocationRole::InstallFolder | LocationRole::ProgramFolder
            )
        })
        .map(|location| PathBuf::from(location.object.container_path()))
        .collect()
}

fn executables_in(folder: &Path) -> Vec<PathBuf> {
    let top_level_entries = entries_of(folder);
    let nested_entries: Vec<PathBuf> = top_level_entries
        .iter()
        .filter(|entry| entry.is_dir())
        .flat_map(|subfolder| entries_of(subfolder))
        .collect();
    top_level_entries
        .into_iter()
        .chain(nested_entries)
        .filter(|entry| is_executable(entry))
        .collect()
}

fn entries_of(folder: &Path) -> Vec<PathBuf> {
    fs::read_dir(folder)
        .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
        .unwrap_or_default()
}

fn is_executable(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(OsStr::new(EXECUTABLE_EXTENSION)))
}

fn stem_of(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn largest_of(files: &[PathBuf]) -> Option<&PathBuf> {
    files
        .iter()
        .max_by_key(|file| file.metadata().map_or(0, |metadata| metadata.len()))
}
