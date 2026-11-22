use serde::{Deserialize, Serialize};

use crate::net_effect::NetEffect;
use crate::process_lineage::AttributionRule;
use crate::tracked_object::TrackedObject;

pub const MANIFEST_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub object: TrackedObject,
    pub effect: NetEffect,
    pub attribution: AttributionRule,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeManifest {
    pub format_version: u32,
    pub program: String,
    pub arguments: Vec<String>,
    pub started_at_unix_seconds: u64,
    pub finished_at_unix_seconds: u64,
    pub undecoded_event_count: u64,
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EffectCounts {
    pub created: usize,
    pub modified: usize,
    pub deleted: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ManifestSummary {
    pub file_system_entries: EffectCounts,
    pub registry_keys: EffectCounts,
    pub registry_values: EffectCounts,
}

impl ChangeManifest {
    pub fn summarize(&self) -> ManifestSummary {
        let mut summary = ManifestSummary::default();
        for change in &self.changes {
            summary.counts_for(&change.object).count(&change.effect);
        }
        summary
    }
}

impl ManifestSummary {
    fn counts_for(&mut self, object: &TrackedObject) -> &mut EffectCounts {
        match object {
            TrackedObject::FileSystemEntry { .. } => &mut self.file_system_entries,
            TrackedObject::RegistryKey { .. } => &mut self.registry_keys,
            TrackedObject::RegistryValue { .. } => &mut self.registry_values,
        }
    }
}

impl EffectCounts {
    fn count(&mut self, effect: &NetEffect) {
        match effect {
            NetEffect::Created => self.created += 1,
            NetEffect::Modified { .. } => self.modified += 1,
            NetEffect::Deleted { .. } => self.deleted += 1,
        }
    }
}
