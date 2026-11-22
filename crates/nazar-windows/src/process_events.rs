use std::sync::Arc;

use ferrisetw::provider::Provider;
use ferrisetw::{EventRecord, SchemaLocator};
use nazar_core::kernel_path::VolumeNames;
use nazar_core::observation::{Observation, ProcessId};

use crate::event_fields::{EventFields, decode_with};
use crate::trace_signal::{Decoded, SignalSink, forward};

const PROVIDER_NAME: &str = "Microsoft-Windows-Kernel-Process";
const PROVIDER_GUID: &str = "22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716";
const PROCESS_KEYWORD: u64 = 0x10;
const PROCESS_START_EVENT_ID: u16 = 1;
const PROCESS_STOP_EVENT_ID: u16 = 2;

pub(crate) fn process_provider(sink: SignalSink, volume_names: Arc<VolumeNames>) -> Provider {
    Provider::by_guid(PROVIDER_GUID)
        .any(PROCESS_KEYWORD)
        .add_callback(
            move |record: &EventRecord, schema_locator: &SchemaLocator| {
                let outcome = observe(record, schema_locator, &volume_names);
                forward(&sink, PROVIDER_NAME, record, outcome);
            },
        )
        .build()
}

fn observe(
    record: &EventRecord,
    schema_locator: &SchemaLocator,
    volume_names: &VolumeNames,
) -> Decoded {
    match record.event_id() {
        PROCESS_START_EVENT_ID => decode_with(record, schema_locator, |fields| {
            observe_process_start(fields, volume_names)
        }),
        PROCESS_STOP_EVENT_ID => decode_with(record, schema_locator, observe_process_stop),
        _ => Ok(None),
    }
}

fn observe_process_start(fields: &EventFields<'_, '_>, volume_names: &VolumeNames) -> Decoded {
    let image_path = volume_names.translate_file_path(&fields.text("ImageName")?);
    Ok(Some(Observation::ProcessStarted {
        process: ProcessId(fields.number("ProcessID")?),
        parent: ProcessId(fields.number("ParentProcessID")?),
        image_path,
    }))
}

fn observe_process_stop(fields: &EventFields<'_, '_>) -> Decoded {
    Ok(Some(Observation::ProcessStopped {
        process: ProcessId(fields.number("ProcessID")?),
    }))
}
