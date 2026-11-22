use nazar_core::installed_program::InstalledProgram;
use nazar_core::leftover_groups::LeftoverGroup;
use nazar_core::location_removal::{RemovalReport, RemovalResult};
use nazar_core::program_locations::{
    Confidence, LocationCandidates, LocationRole, ProgramLocation, find_program_locations,
};
use nazar_core::tracked_object::TrackedObject;
use nazar_core::uninstall_launch::uninstall_launch_of;
use nazar_windows::location_presence::is_present;
use serde::{Deserialize, Serialize};

use crate::icon_source::icon_source_of;

pub const LEFTOVER_ID_PREFIX: &str = "leftover:";
const LEFTOVER_PUBLISHER_NOTE: &str = "No longer installed";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramOverview {
    pub id: String,
    pub name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub install_date: Option<String>,
    pub size_in_kilobytes: Option<u64>,
    pub icon_source: Option<String>,
    pub has_uninstaller: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LocationKind {
    FileSystemEntry,
    RegistryKey,
    RegistryValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationReference {
    pub kind: LocationKind,
    pub path: String,
    pub value_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationRow {
    pub location: LocationReference,
    pub role: Role,
    pub is_certain: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramDetails {
    pub overview: ProgramOverview,
    pub is_installed: bool,
    pub file_locations: Vec<LocationRow>,
    pub registry_locations: Vec<LocationRow>,
    pub unreadable_location_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeletionStatus {
    Removed,
    AlreadyGone,
    Refused,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletionOutcome {
    pub location: LocationReference,
    pub status: DeletionStatus,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletionReport {
    pub registry_backup_directory: Option<String>,
    pub outcomes: Vec<DeletionOutcome>,
}

impl LocationReference {
    pub fn of(object: &TrackedObject) -> Self {
        let (kind, value_name) = match object {
            TrackedObject::FileSystemEntry { .. } => (LocationKind::FileSystemEntry, None),
            TrackedObject::RegistryKey { .. } => (LocationKind::RegistryKey, None),
            TrackedObject::RegistryValue { name, .. } => {
                (LocationKind::RegistryValue, Some(name.clone()))
            }
        };
        Self {
            kind,
            path: object.container_path().to_owned(),
            value_name,
        }
    }

    pub fn into_object(self) -> TrackedObject {
        match self.kind {
            LocationKind::FileSystemEntry => TrackedObject::file_system_entry(self.path),
            LocationKind::RegistryKey => TrackedObject::registry_key(self.path),
            LocationKind::RegistryValue => {
                TrackedObject::registry_value(self.path, self.value_name.unwrap_or_default())
            }
        }
    }
}

pub fn overview_of(program: &InstalledProgram, candidates: &LocationCandidates) -> ProgramOverview {
    ProgramOverview {
        id: program.registry_key_path.clone(),
        name: program.display_name.clone(),
        publisher: program.publisher.clone(),
        version: program.version.clone(),
        install_date: program.install_date.clone(),
        size_in_kilobytes: program.estimated_size_in_kilobytes,
        icon_source: icon_source_of(program, candidates),
        has_uninstaller: uninstall_launch_of(program).is_some(),
    }
}

pub fn details_of(
    program: &InstalledProgram,
    candidates: &LocationCandidates,
    is_installed: bool,
) -> ProgramDetails {
    let (file_locations, registry_locations): (Vec<ProgramLocation>, Vec<ProgramLocation>) =
        find_program_locations(program, candidates)
            .into_iter()
            .filter(|location| is_present(&location.object))
            .partition(|location| matches!(location.object, TrackedObject::FileSystemEntry { .. }));
    ProgramDetails {
        overview: overview_of(program, candidates),
        is_installed,
        file_locations: file_locations.iter().map(row_of).collect(),
        registry_locations: registry_locations.iter().map(row_of).collect(),
        unreadable_location_count: candidates.unreadable_location_count,
    }
}

pub fn leftover_id_of(group: &LeftoverGroup) -> String {
    format!("{LEFTOVER_ID_PREFIX}{}", group.key)
}

pub fn leftover_overview_of(group: &LeftoverGroup) -> ProgramOverview {
    let first_folder = group
        .locations
        .iter()
        .find_map(|location| match &location.object {
            TrackedObject::FileSystemEntry { path } => Some(path.clone()),
            TrackedObject::RegistryKey { .. } | TrackedObject::RegistryValue { .. } => None,
        });
    ProgramOverview {
        id: leftover_id_of(group),
        name: group.name.clone(),
        publisher: Some(LEFTOVER_PUBLISHER_NOTE.to_owned()),
        version: None,
        install_date: None,
        size_in_kilobytes: None,
        icon_source: first_folder,
        has_uninstaller: false,
    }
}

pub fn leftover_details_of(
    group: &LeftoverGroup,
    candidates: &LocationCandidates,
) -> ProgramDetails {
    let (file_locations, registry_locations): (Vec<&ProgramLocation>, Vec<&ProgramLocation>) =
        group
            .locations
            .iter()
            .filter(|location| is_present(&location.object))
            .partition(|location| matches!(location.object, TrackedObject::FileSystemEntry { .. }));
    ProgramDetails {
        overview: leftover_overview_of(group),
        is_installed: false,
        file_locations: file_locations.into_iter().map(row_of).collect(),
        registry_locations: registry_locations.into_iter().map(row_of).collect(),
        unreadable_location_count: candidates.unreadable_location_count,
    }
}

pub fn deletion_report_of(report: RemovalReport) -> DeletionReport {
    let outcomes = report
        .outcomes
        .into_iter()
        .map(|outcome| {
            let (status, reason) = match outcome.result {
                RemovalResult::Removed => (DeletionStatus::Removed, None),
                RemovalResult::AlreadyGone => (DeletionStatus::AlreadyGone, None),
                RemovalResult::Refused => (DeletionStatus::Refused, None),
                RemovalResult::Failed { reason } => (DeletionStatus::Failed, Some(reason)),
            };
            DeletionOutcome {
                location: LocationReference::of(&outcome.object),
                status,
                reason,
            }
        })
        .collect();
    DeletionReport {
        registry_backup_directory: report.registry_backup_directory,
        outcomes,
    }
}

fn row_of(location: &ProgramLocation) -> LocationRow {
    LocationRow {
        location: LocationReference::of(&location.object),
        role: role_of(location.role),
        is_certain: location.confidence == Confidence::Certain,
    }
}

fn role_of(role: LocationRole) -> Role {
    match role {
        LocationRole::UninstallEntry => Role::UninstallEntry,
        LocationRole::InstallFolder => Role::InstallFolder,
        LocationRole::ProgramFolder => Role::ProgramFolder,
        LocationRole::DataFolder => Role::DataFolder,
        LocationRole::Shortcut => Role::Shortcut,
        LocationRole::SettingsKey => Role::SettingsKey,
        LocationRole::Autostart => Role::Autostart,
        LocationRole::Service => Role::Service,
        LocationRole::AppPath => Role::AppPath,
    }
}
