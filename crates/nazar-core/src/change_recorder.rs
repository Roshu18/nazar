use std::collections::BTreeMap;
use std::iter;
use std::ops::Bound;

use crate::change_manifest::Change;
use crate::net_effect::{NetEffect, combine};
use crate::object_identity::ObjectIdentity;
use crate::observation::{Observation, Original, ProcessId, Transition};
use crate::process_lineage::{AttributionRule, ProcessLineage};
use crate::tracked_object::TrackedObject;

#[derive(Debug)]
pub struct ChangeRecorder {
    lineage: ProcessLineage,
    changes: BTreeMap<ObjectIdentity, Change>,
}

impl ChangeRecorder {
    pub fn new(launcher: ProcessId) -> Self {
        Self {
            lineage: ProcessLineage::new(launcher),
            changes: BTreeMap::new(),
        }
    }

    pub fn record(&mut self, observation: Observation) {
        match observation {
            Observation::ProcessAlreadyRunning {
                process,
                image_name,
            } => self.lineage.note_running_process(process, &image_name),
            Observation::ProcessStarted {
                process,
                parent,
                image_path,
            } => self
                .lineage
                .note_process_start(process, parent, &image_path),
            Observation::ProcessStopped { process } => self.lineage.note_process_stop(process),
            Observation::ObjectChanged {
                actor,
                object,
                transition,
            } => self.record_object_change(actor, object, transition),
            Observation::FileSystemEntryRenamed {
                actor,
                previous_path,
                path,
            } => self.record_rename(actor, &previous_path, &path),
        }
    }

    pub fn has_live_tracked_process(&self) -> bool {
        self.lineage.has_live_tracked_process()
    }

    pub fn is_tracking(&self, process: ProcessId) -> bool {
        self.lineage.attribution_of(process).is_some()
    }

    pub fn into_changes(self) -> Vec<Change> {
        self.changes.into_values().collect()
    }

    fn record_object_change(
        &mut self,
        actor: ProcessId,
        object: TrackedObject,
        transition: Transition,
    ) {
        let Some(rule) = self.lineage.attribution_of(actor) else {
            return;
        };
        let transition = self.transition_knowing_created_keys(&object, transition);
        self.note_possible_image(&object, &transition);
        let identity = object.identity();
        let vanishes = matches!(transition, Transition::Vanished { .. });
        self.apply(object, transition, rule);
        if vanishes {
            self.vanish_values_of(&identity, rule);
        }
    }

    fn record_rename(&mut self, actor: ProcessId, previous_path: &str, path: &str) {
        let Some(rule) = self.lineage.attribution_of(actor) else {
            return;
        };
        self.lineage.note_dropped_image(path);
        for (previous_entry, entry) in self.entries_moved_by_rename(previous_path, path) {
            self.apply(previous_entry, vanished_without_original(), rule);
            self.apply(entry, Transition::Appeared, rule);
        }
    }

    fn apply(&mut self, object: TrackedObject, transition: Transition, rule: AttributionRule) {
        let identity = object.identity();
        let previous_change = self.changes.remove(&identity);
        let attribution = previous_change
            .as_ref()
            .map_or(rule, |change| change.attribution);
        let previous_effect = previous_change.map(|change| change.effect);
        if let Some(effect) = combine(previous_effect, transition) {
            let change = Change {
                object,
                effect,
                attribution,
            };
            self.changes.insert(identity, change);
        }
    }

    fn transition_knowing_created_keys(
        &self,
        object: &TrackedObject,
        transition: Transition,
    ) -> Transition {
        let TrackedObject::RegistryValue { key_path, .. } = object else {
            return transition;
        };
        let key_was_created = self
            .changes
            .get(&ObjectIdentity::of_registry_key(key_path))
            .is_some_and(|key_change| key_change.effect == NetEffect::Created);
        match transition {
            Transition::Changed { .. } if key_was_created => Transition::Appeared,
            unchanged => unchanged,
        }
    }

    fn note_possible_image(&mut self, object: &TrackedObject, transition: &Transition) {
        let still_exists = !matches!(transition, Transition::Vanished { .. });
        if let (TrackedObject::FileSystemEntry { path }, true) = (object, still_exists) {
            self.lineage.note_dropped_image(path);
        }
    }

    fn vanish_values_of(&mut self, key: &ObjectIdentity, rule: AttributionRule) {
        let values_after_key = (Bound::Excluded(key), Bound::Unbounded);
        let values: Vec<TrackedObject> = self
            .changes
            .range::<ObjectIdentity, _>(values_after_key)
            .take_while(|(identity, _)| identity.is_value_of(key))
            .map(|(_, change)| change.object.clone())
            .collect();
        for value in values {
            self.apply(value, vanished_without_original(), rule);
        }
    }

    fn entries_moved_by_rename(
        &self,
        previous_path: &str,
        path: &str,
    ) -> Vec<(TrackedObject, TrackedObject)> {
        let previous_identity = ObjectIdentity::of_file_system_entry(previous_path);
        let moved_descendants = self
            .changes
            .iter()
            .filter(|(identity, change)| {
                identity.is_inside(&previous_identity) && !change.effect.is_deleted()
            })
            .filter_map(|(_, change)| moved_descendant(&change.object, previous_path, path));
        let renamed_entry = (
            TrackedObject::file_system_entry(previous_path),
            TrackedObject::file_system_entry(path),
        );
        iter::once(renamed_entry).chain(moved_descendants).collect()
    }
}

fn vanished_without_original() -> Transition {
    Transition::Vanished {
        original: Original::NotCaptured,
    }
}

fn moved_descendant(
    descendant: &TrackedObject,
    previous_ancestor_path: &str,
    ancestor_path: &str,
) -> Option<(TrackedObject, TrackedObject)> {
    let relative_path = path_after_ancestor(descendant.container_path(), previous_ancestor_path)?;
    let moved = TrackedObject::file_system_entry(format!("{ancestor_path}{relative_path}"));
    Some((descendant.clone(), moved))
}

fn path_after_ancestor<'path>(path: &'path str, ancestor_path: &str) -> Option<&'path str> {
    let mut remaining = path.chars();
    for expected in ancestor_path.chars() {
        let actual = remaining.next()?;
        if !actual.to_lowercase().eq(expected.to_lowercase()) {
            return None;
        }
    }
    Some(remaining.as_str())
}
