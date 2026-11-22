use std::fmt;

use serde::{Deserialize, Serialize};

use crate::object_identity::ObjectIdentity;

const DEFAULT_VALUE_LABEL: &str = "(Default)";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TrackedObject {
    FileSystemEntry { path: String },
    RegistryKey { path: String },
    RegistryValue { key_path: String, name: String },
}

impl TrackedObject {
    pub fn file_system_entry(path: impl Into<String>) -> Self {
        Self::FileSystemEntry { path: path.into() }
    }

    pub fn registry_key(path: impl Into<String>) -> Self {
        Self::RegistryKey { path: path.into() }
    }

    pub fn registry_value(key_path: impl Into<String>, name: impl Into<String>) -> Self {
        Self::RegistryValue {
            key_path: key_path.into(),
            name: name.into(),
        }
    }

    pub fn identity(&self) -> ObjectIdentity {
        match self {
            Self::FileSystemEntry { path } => ObjectIdentity::of_file_system_entry(path),
            Self::RegistryKey { path } => ObjectIdentity::of_registry_key(path),
            Self::RegistryValue { key_path, name } => {
                ObjectIdentity::of_registry_value(key_path, name)
            }
        }
    }

    pub fn container_path(&self) -> &str {
        match self {
            Self::FileSystemEntry { path } | Self::RegistryKey { path } => path,
            Self::RegistryValue { key_path, .. } => key_path,
        }
    }
}

impl fmt::Display for TrackedObject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileSystemEntry { path } | Self::RegistryKey { path } => {
                formatter.write_str(path)
            }
            Self::RegistryValue { key_path, name } if name.is_empty() => {
                write!(formatter, "{key_path} [{DEFAULT_VALUE_LABEL}]")
            }
            Self::RegistryValue { key_path, name } => write!(formatter, "{key_path} [{name}]"),
        }
    }
}
