use std::cmp::Reverse;

use crate::change_manifest::Change;
use crate::net_effect::NetEffect;
use crate::observation::{Original, RegistryValueContent};
use crate::tracked_object::TrackedObject;

const PATH_SEPARATOR: char = '\\';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewReason {
    OriginalNotCaptured,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemovalStep {
    DeleteRegistryValue {
        key_path: String,
        name: String,
    },
    DeleteFileSystemEntry {
        path: String,
    },
    DeleteRegistryKey {
        path: String,
    },
    RestoreRegistryValue {
        key_path: String,
        name: String,
        original: RegistryValueContent,
    },
    ReviewManually {
        object: TrackedObject,
        reason: ReviewReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ExecutionPhase {
    DeleteRegistryValues,
    DeleteFileSystemEntries,
    DeleteRegistryKeys,
    RestoreOriginals,
    ManualReview,
}

pub fn plan_removal(changes: &[Change]) -> Vec<RemovalStep> {
    let mut steps: Vec<RemovalStep> = changes.iter().map(step_reversing).collect();
    steps.sort_by_key(execution_order);
    steps
}

fn step_reversing(change: &Change) -> RemovalStep {
    match &change.effect {
        NetEffect::Created => deletion_of(&change.object),
        NetEffect::Modified { original } | NetEffect::Deleted { original } => {
            restoration_of(&change.object, original)
        }
    }
}

fn deletion_of(object: &TrackedObject) -> RemovalStep {
    match object.clone() {
        TrackedObject::FileSystemEntry { path } => RemovalStep::DeleteFileSystemEntry { path },
        TrackedObject::RegistryKey { path } => RemovalStep::DeleteRegistryKey { path },
        TrackedObject::RegistryValue { key_path, name } => {
            RemovalStep::DeleteRegistryValue { key_path, name }
        }
    }
}

fn restoration_of(object: &TrackedObject, original: &Original) -> RemovalStep {
    match (object.clone(), original.clone()) {
        (TrackedObject::RegistryValue { key_path, name }, Original::RegistryValue(original)) => {
            RemovalStep::RestoreRegistryValue {
                key_path,
                name,
                original,
            }
        }
        (object, _) => RemovalStep::ReviewManually {
            object,
            reason: ReviewReason::OriginalNotCaptured,
        },
    }
}

fn execution_order(step: &RemovalStep) -> (ExecutionPhase, Reverse<usize>) {
    match step {
        RemovalStep::DeleteRegistryValue { .. } => {
            (ExecutionPhase::DeleteRegistryValues, Reverse(0))
        }
        RemovalStep::DeleteFileSystemEntry { path } => (
            ExecutionPhase::DeleteFileSystemEntries,
            Reverse(depth_of(path)),
        ),
        RemovalStep::DeleteRegistryKey { path } => {
            (ExecutionPhase::DeleteRegistryKeys, Reverse(depth_of(path)))
        }
        RemovalStep::RestoreRegistryValue { .. } => (ExecutionPhase::RestoreOriginals, Reverse(0)),
        RemovalStep::ReviewManually { .. } => (ExecutionPhase::ManualReview, Reverse(0)),
    }
}

fn depth_of(path: &str) -> usize {
    path.matches(PATH_SEPARATOR).count()
}
