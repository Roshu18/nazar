use std::path::Path;

use nazar_core::change_manifest::{Change, ChangeManifest};
use nazar_core::net_effect::NetEffect;
use nazar_core::process_lineage::AttributionRule;
use nazar_core::removal_plan::{RemovalStep, plan_removal};
use nazar_core::tracked_object::TrackedObject;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceOverview {
    pub name: String,
    pub program_name: String,
    pub program_path: String,
    pub started_at_unix_seconds: u64,
    pub change_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ChangeEffect {
    Created,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Attribution {
    Descent,
    DroppedImage,
    InstallerService,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRow {
    pub path: String,
    pub value_name: Option<String>,
    pub effect: ChangeEffect,
    pub attribution: Attribution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RemovalAction {
    Delete,
    Restore,
    Review,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovalRow {
    pub action: RemovalAction,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceDetails {
    pub overview: TraceOverview,
    pub duration_in_seconds: u64,
    pub unresolved_event_count: u64,
    pub file_changes: Vec<ChangeRow>,
    pub registry_changes: Vec<ChangeRow>,
    pub removal_steps: Vec<RemovalRow>,
}

pub fn overview_of(name: &str, manifest: &ChangeManifest) -> TraceOverview {
    let program_name = Path::new(&manifest.program).file_name().map_or_else(
        || manifest.program.clone(),
        |file_name| file_name.to_string_lossy().into_owned(),
    );
    TraceOverview {
        name: name.to_owned(),
        program_name,
        program_path: manifest.program.clone(),
        started_at_unix_seconds: manifest.started_at_unix_seconds,
        change_count: manifest.changes.len(),
    }
}

pub fn details_of(name: &str, manifest: &ChangeManifest) -> TraceDetails {
    let (file_changes, registry_changes): (Vec<&Change>, Vec<&Change>) = manifest
        .changes
        .iter()
        .partition(|change| matches!(change.object, TrackedObject::FileSystemEntry { .. }));
    TraceDetails {
        overview: overview_of(name, manifest),
        duration_in_seconds: manifest
            .finished_at_unix_seconds
            .saturating_sub(manifest.started_at_unix_seconds),
        unresolved_event_count: manifest.undecoded_event_count,
        file_changes: file_changes.into_iter().map(row_of).collect(),
        registry_changes: registry_changes.into_iter().map(row_of).collect(),
        removal_steps: plan_removal(&manifest.changes)
            .iter()
            .map(removal_row_of)
            .collect(),
    }
}

fn row_of(change: &Change) -> ChangeRow {
    let value_name = match &change.object {
        TrackedObject::RegistryValue { name, .. } => Some(name.clone()),
        TrackedObject::FileSystemEntry { .. } | TrackedObject::RegistryKey { .. } => None,
    };
    ChangeRow {
        path: change.object.container_path().to_owned(),
        value_name,
        effect: effect_of(&change.effect),
        attribution: attribution_of(change.attribution),
    }
}

fn effect_of(effect: &NetEffect) -> ChangeEffect {
    match effect {
        NetEffect::Created => ChangeEffect::Created,
        NetEffect::Modified { .. } => ChangeEffect::Modified,
        NetEffect::Deleted { .. } => ChangeEffect::Deleted,
    }
}

fn attribution_of(rule: AttributionRule) -> Attribution {
    match rule {
        AttributionRule::Descent => Attribution::Descent,
        AttributionRule::DroppedImage => Attribution::DroppedImage,
        AttributionRule::InstallerService => Attribution::InstallerService,
    }
}

fn removal_row_of(step: &RemovalStep) -> RemovalRow {
    let (action, target) = match step {
        RemovalStep::DeleteFileSystemEntry { path } | RemovalStep::DeleteRegistryKey { path } => {
            (RemovalAction::Delete, path.clone())
        }
        RemovalStep::DeleteRegistryValue { key_path, name } => {
            (RemovalAction::Delete, value_label(key_path, name))
        }
        RemovalStep::RestoreRegistryValue { key_path, name, .. } => {
            (RemovalAction::Restore, value_label(key_path, name))
        }
        RemovalStep::ReviewManually { object, .. } => (RemovalAction::Review, object.to_string()),
    };
    RemovalRow { action, target }
}

fn value_label(key_path: &str, name: &str) -> String {
    TrackedObject::registry_value(key_path, name).to_string()
}
