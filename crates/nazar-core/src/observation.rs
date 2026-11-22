use serde::{Deserialize, Serialize};

use crate::tracked_object::TrackedObject;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProcessId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryValueContent {
    pub value_type: u32,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum Original {
    NotCaptured,
    RegistryValue(RegistryValueContent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transition {
    Appeared,
    Changed { original: Original },
    Vanished { original: Original },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Observation {
    ProcessAlreadyRunning {
        process: ProcessId,
        image_name: String,
    },
    ProcessStarted {
        process: ProcessId,
        parent: ProcessId,
        image_path: String,
    },
    ProcessStopped {
        process: ProcessId,
    },
    ObjectChanged {
        actor: ProcessId,
        object: TrackedObject,
        transition: Transition,
    },
    FileSystemEntryRenamed {
        actor: ProcessId,
        previous_path: String,
        path: String,
    },
}
