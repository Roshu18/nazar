use serde::{Deserialize, Serialize};

use crate::tracked_object::TrackedObject;

const PATH_SEPARATOR: char = '\\';
const MINIMUM_FOLDER_DEPTH: usize = 1;
const MINIMUM_REGISTRY_KEY_DEPTH: usize = 2;
const PROTECTED_FOLDER_ANCESTORS: [&str; 1] = ["\\windows\\"];
const PROTECTED_PARENT_FOLDER_NAMES: [&str; 1] = ["users"];
const PROTECTED_FOLDER_NAMES: [&str; 18] = [
    "windows",
    "program files",
    "program files (x86)",
    "common files",
    "programdata",
    "users",
    "appdata",
    "roaming",
    "local",
    "locallow",
    "programs",
    "packages",
    "microsoft",
    "temp",
    "desktop",
    "documents",
    "downloads",
    "start menu",
];
const PROTECTED_REGISTRY_KEY_NAMES: [&str; 14] = [
    "software",
    "wow6432node",
    "microsoft",
    "windows",
    "currentversion",
    "classes",
    "policies",
    "system",
    "currentcontrolset",
    "services",
    "uninstall",
    "run",
    "runonce",
    "app paths",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemovalRequest {
    pub objects: Vec<TrackedObject>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RemovalResult {
    Removed,
    AlreadyGone,
    Refused,
    Failed { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemovalOutcome {
    pub object: TrackedObject,
    pub result: RemovalResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemovalReport {
    pub registry_backup_directory: Option<String>,
    pub outcomes: Vec<RemovalOutcome>,
}

pub fn is_protected(object: &TrackedObject) -> bool {
    match object {
        TrackedObject::FileSystemEntry { path } => is_protected_folder(path),
        TrackedObject::RegistryKey { path } => is_protected_registry_key(path),
        TrackedObject::RegistryValue { .. } => false,
    }
}

pub fn in_removal_order(mut objects: Vec<TrackedObject>) -> Vec<TrackedObject> {
    objects.sort_by_key(|object| std::cmp::Reverse(depth_of(object.container_path())));
    objects
}

fn is_protected_folder(path: &str) -> bool {
    let lowercase_path = path.to_lowercase();
    let path_with_separator = format!("{lowercase_path}{PATH_SEPARATOR}");
    let is_inside_windows = PROTECTED_FOLDER_ANCESTORS
        .iter()
        .any(|ancestor| path_with_separator.contains(ancestor));
    let is_user_profile = parent_of(&lowercase_path)
        .is_some_and(|parent| PROTECTED_PARENT_FOLDER_NAMES.contains(&last_component_of(parent)));
    depth_of(&lowercase_path) < MINIMUM_FOLDER_DEPTH
        || is_inside_windows
        || is_user_profile
        || PROTECTED_FOLDER_NAMES.contains(&last_component_of(&lowercase_path))
}

fn is_protected_registry_key(path: &str) -> bool {
    let lowercase_path = path.to_lowercase();
    depth_of(&lowercase_path) < MINIMUM_REGISTRY_KEY_DEPTH
        || PROTECTED_REGISTRY_KEY_NAMES.contains(&last_component_of(&lowercase_path))
}

fn parent_of(path: &str) -> Option<&str> {
    path.trim_end_matches(PATH_SEPARATOR)
        .rsplit_once(PATH_SEPARATOR)
        .map(|(parent, _)| parent)
}

fn depth_of(path: &str) -> usize {
    path.trim_end_matches(PATH_SEPARATOR)
        .matches(PATH_SEPARATOR)
        .count()
}

fn last_component_of(path: &str) -> &str {
    path.trim_end_matches(PATH_SEPARATOR)
        .rsplit(PATH_SEPARATOR)
        .next()
        .unwrap_or(path)
}
