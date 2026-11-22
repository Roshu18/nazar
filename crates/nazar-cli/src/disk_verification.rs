use std::fs;
use std::io;

use nazar_core::change_manifest::Change;
use nazar_core::net_effect::NetEffect;
use nazar_core::tracked_object::TrackedObject;

pub fn discard_created_entries_missing_from_disk(changes: Vec<Change>) -> Vec<Change> {
    changes
        .into_iter()
        .filter(|change| !is_created_entry_missing_from_disk(change))
        .collect()
}

fn is_created_entry_missing_from_disk(change: &Change) -> bool {
    let (TrackedObject::FileSystemEntry { path }, NetEffect::Created) =
        (&change.object, &change.effect)
    else {
        return false;
    };
    matches!(
        fs::symlink_metadata(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound
    )
}
