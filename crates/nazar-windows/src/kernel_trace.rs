use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Duration;

use ferrisetw::native::EvntraceNativeError;
use ferrisetw::trace::{TraceError, TraceProperties, UserTrace, stop_trace_by_name};
use nazar_core::kernel_path::VolumeNames;

use crate::file_events::file_provider;
use crate::process_events::process_provider;
use crate::registry_events::registry_provider;
use crate::trace_signal::TraceSignal;
use crate::volume_names::{VolumeNamesError, read_volume_names};

const SESSION_NAME: &str = "Nazar-Watch";
const BUFFER_SIZE_IN_KILOBYTES: u32 = 256;
const MINIMUM_BUFFER_COUNT: u32 = 64;
const MAXIMUM_BUFFER_COUNT: u32 = 512;
const FLUSH_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, thiserror::Error)]
pub enum KernelTraceError {
    #[error("kernel tracing could not start; run Nazar from an administrator terminal ({0:?})")]
    Start(TraceError),
    #[error("a tracing session left behind by an earlier run could not be stopped ({0:?})")]
    StopLeftoverSession(TraceError),
    #[error("kernel tracing did not stop cleanly ({0:?})")]
    Stop(TraceError),
    #[error(transparent)]
    VolumeNames(#[from] VolumeNamesError),
}

pub struct KernelTraceSession {
    trace: UserTrace,
}

impl KernelTraceSession {
    pub fn start(sink: &Sender<TraceSignal>) -> Result<Self, KernelTraceError> {
        let volume_names = Arc::new(read_volume_names()?);
        let trace = match open_trace(sink, &volume_names) {
            Err(TraceError::EtwNativeError(EvntraceNativeError::AlreadyExist)) => {
                stop_trace_by_name(SESSION_NAME).map_err(KernelTraceError::StopLeftoverSession)?;
                open_trace(sink, &volume_names)
            }
            outcome => outcome,
        };
        trace
            .map(|trace| Self { trace })
            .map_err(KernelTraceError::Start)
    }

    pub fn stop(self) -> Result<(), KernelTraceError> {
        self.trace.stop().map_err(KernelTraceError::Stop)
    }
}

fn open_trace(
    sink: &Sender<TraceSignal>,
    volume_names: &Arc<VolumeNames>,
) -> Result<UserTrace, TraceError> {
    UserTrace::new()
        .named(SESSION_NAME.to_owned())
        .set_trace_properties(session_properties())
        .enable(process_provider(sink.clone(), Arc::clone(volume_names)))
        .enable(file_provider(sink.clone(), Arc::clone(volume_names)))
        .enable(registry_provider(sink.clone()))
        .start_and_process()
}

fn session_properties() -> TraceProperties {
    TraceProperties {
        buffer_size: BUFFER_SIZE_IN_KILOBYTES,
        min_buffer: MINIMUM_BUFFER_COUNT,
        max_buffer: MAXIMUM_BUFFER_COUNT,
        flush_timer: FLUSH_INTERVAL,
        ..TraceProperties::default()
    }
}
