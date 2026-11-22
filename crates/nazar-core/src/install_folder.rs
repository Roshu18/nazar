use crate::installed_program::InstalledProgram;

const PATH_SEPARATOR: char = '\\';
const QUOTE: char = '"';
const EXECUTABLE_EXTENSION: &str = ".exe";
const ICON_INDEX_SEPARATOR: char = ',';
const SHARED_FOLDER_NAMES: [&str; 12] = [
    "windows",
    "system32",
    "syswow64",
    "program files",
    "program files (x86)",
    "common files",
    "programdata",
    "users",
    "desktop",
    "downloads",
    "documents",
    "temp",
];
const SHARED_FOLDER_ANCESTORS: [&str; 2] = ["\\windows\\", "\\package cache\\"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallFolder {
    pub path: String,
    pub is_declared: bool,
}

pub fn install_folder_of(program: &InstalledProgram) -> Option<InstallFolder> {
    let declared_folder = program
        .install_location
        .as_deref()
        .map(clean_path)
        .filter(|path| !path.is_empty());
    if let Some(path) = declared_folder {
        return owned_folder(path, true);
    }
    referenced_files_of(program)
        .into_iter()
        .filter_map(|file| parent_folder_of(&file))
        .find_map(|folder| owned_folder(folder, false))
}

pub fn referenced_files_of(program: &InstalledProgram) -> Vec<String> {
    let icon_file = program.display_icon.as_deref().map(icon_file_in);
    let uninstaller = program.uninstall_command.as_deref().and_then(executable_in);
    [icon_file, uninstaller].into_iter().flatten().collect()
}

pub fn icon_file_of(program: &InstalledProgram) -> Option<String> {
    program.display_icon.as_deref().map(icon_file_in)
}

pub fn last_component_of(path: &str) -> &str {
    path.rsplit(PATH_SEPARATOR).next().unwrap_or(path)
}

fn owned_folder(path: String, is_declared: bool) -> Option<InstallFolder> {
    (!is_shared_folder(&path)).then_some(InstallFolder { path, is_declared })
}

fn is_shared_folder(path: &str) -> bool {
    let lowercase_path = path.to_lowercase();
    let is_drive_root = !lowercase_path.contains(PATH_SEPARATOR);
    let has_shared_name = SHARED_FOLDER_NAMES.contains(&last_component_of(&lowercase_path));
    let path_with_separator = format!("{lowercase_path}{PATH_SEPARATOR}");
    let is_under_shared_ancestor = SHARED_FOLDER_ANCESTORS
        .iter()
        .any(|ancestor| path_with_separator.contains(ancestor));
    is_drive_root || has_shared_name || is_under_shared_ancestor
}

fn icon_file_in(display_icon: &str) -> String {
    let without_index = match display_icon.rsplit_once(ICON_INDEX_SEPARATOR) {
        Some((file, index)) if index.trim().parse::<i64>().is_ok() => file,
        _ => display_icon,
    };
    clean_path(without_index)
}

fn executable_in(command: &str) -> Option<String> {
    let command = command.trim();
    if let Some(quoted) = command.strip_prefix(QUOTE) {
        return quoted.split(QUOTE).next().map(clean_path);
    }
    let extension_start = command.to_lowercase().find(EXECUTABLE_EXTENSION)?;
    let executable = command.get(..extension_start + EXECUTABLE_EXTENSION.len())?;
    Some(clean_path(executable))
}

fn parent_folder_of(file: &str) -> Option<String> {
    file.rsplit_once(PATH_SEPARATOR)
        .map(|(folder, _)| folder.to_owned())
}

fn clean_path(path: &str) -> String {
    path.trim()
        .trim_matches(QUOTE)
        .trim_end_matches(PATH_SEPARATOR)
        .to_owned()
}
