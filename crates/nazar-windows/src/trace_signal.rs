use std::sync::mpsc::Sender;

use ferrisetw::EventRecord;
use nazar_core::observation::{Observation, ProcessId};

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("the event schema is unavailable ({0})")]
    SchemaUnavailable(String),
    #[error("field `{field}` could not be read ({reason})")]
    FieldUnreadable { field: &'static str, reason: String },
    #[error("the registry key behind handle {key_object:#x} was opened before tracing started")]
    UnknownRegistryKey { key_object: usize },
}

#[derive(Debug, thiserror::Error)]
#[error("{provider} event {event_id} could not be decoded: {error}")]
pub struct DecodeFailure {
    pub actor: ProcessId,
    pub provider: &'static str,
    pub event_id: u16,
    pub error: DecodeError,
}

#[derive(Debug)]
pub enum TraceSignal {
    Observed(Observation),
    DecodeFailed(DecodeFailure),
}

pub(crate) type SignalSink = Sender<TraceSignal>;
pub(crate) type Decoded = Result<Option<Observation>, DecodeError>;

pub(crate) fn forward(
    sink: &SignalSink,
    provider: &'static str,
    record: &EventRecord,
    decoded: Decoded,
) {
    let signal = match decoded {
        Ok(Some(observation)) => TraceSignal::Observed(observation),
        Ok(None) => return,
        Err(error) => TraceSignal::DecodeFailed(DecodeFailure {
            actor: ProcessId(record.process_id()),
            provider,
            event_id: record.event_id(),
            error,
        }),
    };
    send_unless_session_ended(sink, signal);
}

fn send_unless_session_ended(sink: &SignalSink, signal: TraceSignal) {
    sink.send(signal).ok();
}
