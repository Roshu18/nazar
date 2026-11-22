use std::io;
use std::path::Path;

use nazar_core::tracked_object::TrackedObject;

use crate::registry_access::{found, split_display_path};

pub fn is_present(object: &TrackedObject) -> bool {
    let presence = match object {
        TrackedObject::FileSystemEntry { path } => {
            found(Path::new(path).symlink_metadata()).map(|metadata| metadata.is_some())
        }
        TrackedObject::RegistryKey { path } => has_registry_key(path, None),
        TrackedObject::RegistryValue { key_path, name } => has_registry_key(key_path, Some(name)),
    };
    presence.unwrap_or(true)
}

fn has_registry_key(display_path: &str, value_name: Option<&str>) -> io::Result<bool> {
    let Some((hive, path)) = split_display_path(display_path) else {
        return Ok(false);
    };
    let Some(key) = found(hive.root().open_subkey(path))? else {
        return Ok(false);
    };
    match value_name {
        Some(value_name) => Ok(found(key.get_raw_value(value_name))?.is_some()),
        None => Ok(true),
    }
}
