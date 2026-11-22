use std::io;
use std::path::Path;

use nazar_core::location_removal::{
    RemovalOutcome, RemovalReport, RemovalRequest, RemovalResult, in_removal_order, is_protected,
};
use nazar_core::tracked_object::TrackedObject;
use winreg::enums::{KEY_READ, KEY_SET_VALUE};

use crate::recycle_bin::{RecycleBinError, move_to_recycle_bin};
use crate::registry_access::{found, split_display_path};
use crate::registry_backup::RegistryBackup;

#[derive(Debug, thiserror::Error)]
enum LocationRemovalError {
    #[error(transparent)]
    RecycleBin(#[from] RecycleBinError),
    #[error("{0}")]
    Access(#[from] io::Error),
    #[error("`{0}` is not a registry path Nazar understands")]
    UnknownRegistryPath(String),
}

type Removal = Result<RemovalResult, LocationRemovalError>;

pub fn remove_locations(
    request: RemovalRequest,
    registry_backup_directory: &Path,
) -> RemovalReport {
    let mut backup = RegistryBackup::in_directory(registry_backup_directory);
    let outcomes = in_removal_order(request.objects)
        .into_iter()
        .map(|object| RemovalOutcome {
            result: remove_location(&object, &mut backup),
            object,
        })
        .collect();
    RemovalReport {
        registry_backup_directory: backup.directory_if_used(),
        outcomes,
    }
}

fn remove_location(object: &TrackedObject, backup: &mut RegistryBackup) -> RemovalResult {
    if is_protected(object) {
        return RemovalResult::Refused;
    }
    let removal = match object {
        TrackedObject::FileSystemEntry { path } => remove_file_system_entry(Path::new(path)),
        TrackedObject::RegistryKey { path } => remove_registry_key(path, backup),
        TrackedObject::RegistryValue { key_path, name } => {
            remove_registry_value(key_path, name, backup)
        }
    };
    removal.unwrap_or_else(|error| RemovalResult::Failed {
        reason: error.to_string(),
    })
}

fn remove_file_system_entry(path: &Path) -> Removal {
    if found(path.symlink_metadata())?.is_none() {
        return Ok(RemovalResult::AlreadyGone);
    }
    move_to_recycle_bin(path)?;
    Ok(RemovalResult::Removed)
}

fn remove_registry_key(display_path: &str, backup: &mut RegistryBackup) -> Removal {
    let (hive, path) = split_display_path(display_path)
        .ok_or_else(|| LocationRemovalError::UnknownRegistryPath(display_path.to_owned()))?;
    let Some(key) = found(hive.root().open_subkey(path))? else {
        return Ok(RemovalResult::AlreadyGone);
    };
    backup.save_key_tree(hive, path, &key)?;
    hive.root().delete_subkey_all(path)?;
    Ok(RemovalResult::Removed)
}

fn remove_registry_value(
    key_display_path: &str,
    name: &str,
    backup: &mut RegistryBackup,
) -> Removal {
    let (hive, path) = split_display_path(key_display_path)
        .ok_or_else(|| LocationRemovalError::UnknownRegistryPath(key_display_path.to_owned()))?;
    let access = KEY_READ | KEY_SET_VALUE;
    let Some(key) = found(hive.root().open_subkey_with_flags(path, access))? else {
        return Ok(RemovalResult::AlreadyGone);
    };
    if found(key.get_raw_value(name))?.is_none() {
        return Ok(RemovalResult::AlreadyGone);
    }
    backup.save_key_values(hive, path, &key)?;
    key.delete_value(name)?;
    Ok(RemovalResult::Removed)
}
