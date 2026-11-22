use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::installed_program::InstalledProgram;
use crate::object_identity::ObjectIdentity;
use crate::program_identity::ProgramIdentity;
use crate::tracked_object::TrackedObject;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationRole {
    UninstallEntry,
    InstallFolder,
    ProgramFolder,
    DataFolder,
    Shortcut,
    SettingsKey,
    Autostart,
    Service,
    AppPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Certain,
    Likely,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramLocation {
    pub object: TrackedObject,
    pub role: LocationRole,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedLocation {
    pub object: TrackedObject,
    pub name: String,
    pub publisher_name: Option<String>,
    pub role: LocationRole,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathReference {
    pub object: TrackedObject,
    pub referenced_text: String,
    pub role: LocationRole,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocationCandidates {
    pub named_locations: Vec<NamedLocation>,
    pub path_references: Vec<PathReference>,
    pub unreadable_location_count: usize,
}

pub fn find_program_locations(
    program: &InstalledProgram,
    candidates: &LocationCandidates,
) -> Vec<ProgramLocation> {
    let identity = ProgramIdentity::of(program);
    let named_matches = candidates
        .named_locations
        .iter()
        .filter(|location| is_named_after(&identity, location))
        .map(|location| likely(location.object.clone(), location.role));
    let reference_matches = candidates
        .path_references
        .iter()
        .filter(|reference| identity.has_inside_install_folder(&reference.referenced_text))
        .map(|reference| certain(reference.object.clone(), reference.role));
    let mut locations: BTreeMap<ObjectIdentity, ProgramLocation> = BTreeMap::new();
    let all_matches = declared_locations(program, &identity)
        .into_iter()
        .chain(reference_matches)
        .chain(named_matches);
    for location in all_matches {
        locations
            .entry(location.object.identity())
            .or_insert(location);
    }
    without_locations_inside_others(locations)
}

fn without_locations_inside_others(
    mut locations: BTreeMap<ObjectIdentity, ProgramLocation>,
) -> Vec<ProgramLocation> {
    let identities: Vec<ObjectIdentity> = locations.keys().cloned().collect();
    locations.retain(|identity, _| !identities.iter().any(|other| identity.is_inside(other)));
    locations.into_values().collect()
}

fn declared_locations(
    program: &InstalledProgram,
    identity: &ProgramIdentity,
) -> Vec<ProgramLocation> {
    let uninstall_key = TrackedObject::registry_key(&program.registry_key_path);
    let uninstall_entry = certain(uninstall_key, LocationRole::UninstallEntry);
    let install_folder = identity.install_folder().map(|folder| ProgramLocation {
        object: TrackedObject::file_system_entry(&folder.path),
        role: LocationRole::InstallFolder,
        confidence: if folder.is_declared {
            Confidence::Certain
        } else {
            Confidence::Likely
        },
    });
    [Some(uninstall_entry), install_folder]
        .into_iter()
        .flatten()
        .collect()
}

fn is_named_after(identity: &ProgramIdentity, location: &NamedLocation) -> bool {
    let has_matching_publisher = location
        .publisher_name
        .as_deref()
        .is_none_or(|publisher_name| identity.is_published_by(publisher_name));
    has_matching_publisher && identity.is_named(&location.name)
}

fn certain(object: TrackedObject, role: LocationRole) -> ProgramLocation {
    ProgramLocation {
        object,
        role,
        confidence: Confidence::Certain,
    }
}

fn likely(object: TrackedObject, role: LocationRole) -> ProgramLocation {
    ProgramLocation {
        object,
        role,
        confidence: Confidence::Likely,
    }
}
