use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use ferrisetw::provider::Provider;
use ferrisetw::{EventRecord, SchemaLocator};
use nazar_core::kernel_path::VolumeNames;
use nazar_core::observation::{Observation, Original, ProcessId, Transition};
use nazar_core::tracked_object::TrackedObject;

use crate::event_fields::{EventFields, decode_with};
use crate::trace_signal::{DecodeError, Decoded, SignalSink, forward};

const PROVIDER_NAME: &str = "Microsoft-Windows-Kernel-File";
const PROVIDER_GUID: &str = "edd08927-9cc4-4e65-b970-c2560fb5c289";
const OPEN_KEYWORD: u64 = 0x80;
const WRITE_KEYWORD: u64 = 0x200;
const DELETE_PATH_KEYWORD: u64 = 0x400;
const RENAME_AND_LINK_PATH_KEYWORD: u64 = 0x800;
const NEW_FILE_KEYWORD: u64 = 0x1000;
const WATCHED_KEYWORDS: u64 = OPEN_KEYWORD
    | WRITE_KEYWORD
    | DELETE_PATH_KEYWORD
    | RENAME_AND_LINK_PATH_KEYWORD
    | NEW_FILE_KEYWORD;
const OPEN_EVENT_ID: u16 = 12;
const WRITE_EVENT_ID: u16 = 16;
const DELETE_PATH_EVENT_ID: u16 = 26;
const RENAME_PATH_EVENT_ID: u16 = 27;
const LINK_PATH_EVENT_ID: u16 = 28;
const NEW_FILE_EVENT_ID: u16 = 30;

struct OpenFile {
    path: String,
    writers: Vec<ProcessId>,
}

struct FileEventDecoder {
    volume_names: Arc<VolumeNames>,
    open_files: HashMap<usize, OpenFile>,
}

pub(crate) fn file_provider(sink: SignalSink, volume_names: Arc<VolumeNames>) -> Provider {
    let mut decoder = FileEventDecoder {
        volume_names,
        open_files: HashMap::new(),
    };
    Provider::by_guid(PROVIDER_GUID)
        .any(WATCHED_KEYWORDS)
        .add_callback(
            move |record: &EventRecord, schema_locator: &SchemaLocator| {
                let outcome = decoder.observe(record, schema_locator);
                forward(&sink, PROVIDER_NAME, record, outcome);
            },
        )
        .build()
}

impl FileEventDecoder {
    fn observe(&mut self, record: &EventRecord, schema_locator: &SchemaLocator) -> Decoded {
        let actor = ProcessId(record.process_id());
        let decode = |fields: &EventFields<'_, '_>| match record.event_id() {
            OPEN_EVENT_ID => self.remember_open_file(fields).map(|_| None),
            NEW_FILE_EVENT_ID => self.observe_new_file(actor, fields),
            WRITE_EVENT_ID => self.observe_write(actor, fields),
            DELETE_PATH_EVENT_ID => self.observe_deletion(actor, fields),
            RENAME_PATH_EVENT_ID => self.observe_rename(actor, fields),
            LINK_PATH_EVENT_ID => self.observe_link(actor, fields),
            _ => Ok(None),
        };
        decode_with(record, schema_locator, decode)
    }

    fn remember_open_file(&mut self, fields: &EventFields<'_, '_>) -> Result<String, DecodeError> {
        let file_object = fields.address("FileObject")?;
        let path = self.translate(&fields.text("FileName")?);
        let open_file = OpenFile {
            path: path.clone(),
            writers: Vec::new(),
        };
        self.open_files.insert(file_object, open_file);
        Ok(path)
    }

    fn observe_new_file(&mut self, actor: ProcessId, fields: &EventFields<'_, '_>) -> Decoded {
        let path = self.remember_open_file(fields)?;
        Ok(Some(entry_changed(actor, path, Transition::Appeared)))
    }

    fn observe_write(&mut self, actor: ProcessId, fields: &EventFields<'_, '_>) -> Decoded {
        let file_object = fields.address("FileObject")?;
        let Some(open_file) = self.open_files.get_mut(&file_object) else {
            return Ok(None);
        };
        if open_file.writers.contains(&actor) {
            return Ok(None);
        }
        open_file.writers.push(actor);
        let transition = Transition::Changed {
            original: Original::NotCaptured,
        };
        Ok(Some(entry_changed(
            actor,
            open_file.path.clone(),
            transition,
        )))
    }

    fn observe_deletion(&self, actor: ProcessId, fields: &EventFields<'_, '_>) -> Decoded {
        let path = self.translate(&fields.text("FilePath")?);
        let transition = Transition::Vanished {
            original: Original::NotCaptured,
        };
        Ok(Some(entry_changed(actor, path, transition)))
    }

    fn observe_rename(&mut self, actor: ProcessId, fields: &EventFields<'_, '_>) -> Decoded {
        let file_object = fields.address("FileObject")?;
        let path = self.translate(&fields.text("FilePath")?);
        let previous_path = self
            .open_files
            .get_mut(&file_object)
            .map(|open_file| mem::replace(&mut open_file.path, path.clone()));
        Ok(Some(match previous_path {
            Some(previous_path) => Observation::FileSystemEntryRenamed {
                actor,
                previous_path,
                path,
            },
            None => entry_changed(actor, path, Transition::Appeared),
        }))
    }

    fn observe_link(&self, actor: ProcessId, fields: &EventFields<'_, '_>) -> Decoded {
        let path = self.translate(&fields.text("FilePath")?);
        Ok(Some(entry_changed(actor, path, Transition::Appeared)))
    }

    fn translate(&self, kernel_path: &str) -> String {
        self.volume_names.translate_file_path(kernel_path)
    }
}

fn entry_changed(actor: ProcessId, path: String, transition: Transition) -> Observation {
    Observation::ObjectChanged {
        actor,
        object: TrackedObject::file_system_entry(path),
        transition,
    }
}
