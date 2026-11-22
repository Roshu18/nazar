use std::collections::HashMap;

use ferrisetw::provider::Provider;
use ferrisetw::{EventRecord, SchemaLocator};
use nazar_core::kernel_path::translate_registry_path;
use nazar_core::observation::{Observation, Original, ProcessId, RegistryValueContent, Transition};
use nazar_core::tracked_object::TrackedObject;

use crate::event_fields::{EventFields, decode_with};
use crate::trace_signal::{DecodeError, Decoded, SignalSink, forward};

const PROVIDER_NAME: &str = "Microsoft-Windows-Kernel-Registry";
const PROVIDER_GUID: &str = "70eb4f03-c1de-4f73-a051-33d13d5413bd";
const CLOSE_KEY_KEYWORD: u64 = 0x1;
const SET_VALUE_KEYWORD: u64 = 0x100;
const DELETE_VALUE_KEYWORD: u64 = 0x200;
const CREATE_KEY_KEYWORD: u64 = 0x1000;
const OPEN_KEY_KEYWORD: u64 = 0x2000;
const DELETE_KEY_KEYWORD: u64 = 0x4000;
const WATCHED_KEYWORDS: u64 = CLOSE_KEY_KEYWORD
    | SET_VALUE_KEYWORD
    | DELETE_VALUE_KEYWORD
    | CREATE_KEY_KEYWORD
    | OPEN_KEY_KEYWORD
    | DELETE_KEY_KEYWORD;
const CREATE_KEY_EVENT_ID: u16 = 1;
const OPEN_KEY_EVENT_ID: u16 = 2;
const DELETE_KEY_EVENT_ID: u16 = 3;
const SET_VALUE_EVENT_ID: u16 = 5;
const DELETE_VALUE_EVENT_ID: u16 = 6;
const CLOSE_KEY_EVENT_ID: u16 = 13;
const STATUS_SUCCESS: u32 = 0;
const DISPOSITION_CREATED_NEW_KEY: u32 = 1;
const KERNEL_REGISTRY_ROOT: &str = "\\REGISTRY";
const PATH_SEPARATOR: char = '\\';

struct RegistryEventDecoder {
    open_keys: HashMap<usize, String>,
}

pub(crate) fn registry_provider(sink: SignalSink) -> Provider {
    let mut decoder = RegistryEventDecoder {
        open_keys: HashMap::new(),
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

impl RegistryEventDecoder {
    fn observe(&mut self, record: &EventRecord, schema_locator: &SchemaLocator) -> Decoded {
        let actor = ProcessId(record.process_id());
        let decode = |fields: &EventFields<'_, '_>| {
            if fields.number("Status")? != STATUS_SUCCESS {
                return Ok(None);
            }
            match record.event_id() {
                CREATE_KEY_EVENT_ID | OPEN_KEY_EVENT_ID => self.observe_key_handle(actor, fields),
                CLOSE_KEY_EVENT_ID => self.forget_key_handle(fields),
                DELETE_KEY_EVENT_ID => self.observe_key_deletion(actor, fields),
                SET_VALUE_EVENT_ID => self.observe_value_set(actor, fields),
                DELETE_VALUE_EVENT_ID => self.observe_value_deletion(actor, fields),
                _ => Ok(None),
            }
        };
        decode_with(record, schema_locator, decode)
    }

    fn observe_key_handle(&mut self, actor: ProcessId, fields: &EventFields<'_, '_>) -> Decoded {
        let key_object = fields.address("KeyObject")?;
        let Some(path) = self.path_of_opened_key(fields)? else {
            self.open_keys.remove(&key_object);
            return Ok(None);
        };
        self.open_keys.insert(key_object, path.clone());
        let is_new_key = fields.number("Disposition")? == DISPOSITION_CREATED_NEW_KEY;
        let key = TrackedObject::registry_key(path);
        Ok(is_new_key.then(|| object_changed(actor, key, Transition::Appeared)))
    }

    fn forget_key_handle(&mut self, fields: &EventFields<'_, '_>) -> Decoded {
        self.open_keys.remove(&fields.address("KeyObject")?);
        Ok(None)
    }

    fn observe_key_deletion(&self, actor: ProcessId, fields: &EventFields<'_, '_>) -> Decoded {
        let key = TrackedObject::registry_key(self.path_of_used_key(fields)?);
        Ok(Some(object_changed(actor, key, vanished())))
    }

    fn observe_value_set(&self, actor: ProcessId, fields: &EventFields<'_, '_>) -> Decoded {
        let key_path = self.path_of_used_key(fields)?;
        let value = TrackedObject::registry_value(key_path, fields.text("ValueName")?);
        let transition = transition_of_value_set(fields)?;
        Ok(Some(object_changed(actor, value, transition)))
    }

    fn observe_value_deletion(&self, actor: ProcessId, fields: &EventFields<'_, '_>) -> Decoded {
        let key_path = self.path_of_used_key(fields)?;
        let value = TrackedObject::registry_value(key_path, fields.text("ValueName")?);
        Ok(Some(object_changed(actor, value, vanished())))
    }

    fn path_of_opened_key(
        &self,
        fields: &EventFields<'_, '_>,
    ) -> Result<Option<String>, DecodeError> {
        let relative_name = fields.text("RelativeName")?;
        if is_kernel_registry_path(&relative_name) {
            return Ok(Some(translate_registry_path(&relative_name)));
        }
        let base_name = fields.text("BaseName")?;
        let base_path = if base_name.is_empty() {
            self.open_keys.get(&fields.address("BaseObject")?).cloned()
        } else {
            Some(translate_registry_path(&base_name))
        };
        Ok(base_path.map(|base_path| join_key_path(base_path, &relative_name)))
    }

    fn path_of_used_key(&self, fields: &EventFields<'_, '_>) -> Result<String, DecodeError> {
        let key_name = fields.text("KeyName")?;
        if is_kernel_registry_path(&key_name) {
            return Ok(translate_registry_path(&key_name));
        }
        let key_object = fields.address("KeyObject")?;
        self.open_keys
            .get(&key_object)
            .cloned()
            .ok_or(DecodeError::UnknownRegistryKey { key_object })
    }
}

fn transition_of_value_set(fields: &EventFields<'_, '_>) -> Result<Transition, DecodeError> {
    let previous_size = fields.number("PreviousDataSize")?;
    let captured_size = u32::from(fields.small_number("PreviousDataCapturedSize")?);
    let was_fully_captured = previous_size > 0 && captured_size == previous_size;
    let original = if was_fully_captured {
        Original::RegistryValue(RegistryValueContent {
            value_type: fields.number("PreviousDataType")?,
            bytes: fields.bytes("PreviousData")?,
        })
    } else {
        Original::NotCaptured
    };
    Ok(Transition::Changed { original })
}

fn is_kernel_registry_path(name: &str) -> bool {
    name.get(..KERNEL_REGISTRY_ROOT.len())
        .is_some_and(|root| root.eq_ignore_ascii_case(KERNEL_REGISTRY_ROOT))
}

fn join_key_path(base_path: String, relative_name: &str) -> String {
    if relative_name.is_empty() {
        return base_path;
    }
    format!("{base_path}{PATH_SEPARATOR}{relative_name}")
}

fn object_changed(actor: ProcessId, object: TrackedObject, transition: Transition) -> Observation {
    Observation::ObjectChanged {
        actor,
        object,
        transition,
    }
}

fn vanished() -> Transition {
    Transition::Vanished {
        original: Original::NotCaptured,
    }
}
