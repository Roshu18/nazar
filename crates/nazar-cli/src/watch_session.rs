use std::process::{self, Child};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use nazar_core::change_manifest::Change;
use nazar_core::change_recorder::ChangeRecorder;
use nazar_core::observation::{Observation, ProcessId};
use nazar_windows::running_processes::{RunningProcessesError, read_running_processes};
use nazar_windows::trace_signal::{DecodeFailure, TraceSignal};

use crate::watch_command::WatchError;

const SIGNAL_POLL_INTERVAL: Duration = Duration::from_millis(200);
const SETTLE_DURATION: Duration = Duration::from_secs(3);
const DECODE_FAILURE_EXAMPLE_LIMIT: usize = 5;

pub struct WatchOutcome {
    pub changes: Vec<Change>,
    pub undecoded_event_count: u64,
    pub decode_failure_examples: Vec<String>,
}

pub struct WatchSession {
    signals: Receiver<TraceSignal>,
    recorder: ChangeRecorder,
    undecoded_event_count: u64,
    decode_failure_examples: Vec<String>,
}

impl WatchSession {
    pub fn start(signals: Receiver<TraceSignal>) -> Result<Self, RunningProcessesError> {
        let mut recorder = ChangeRecorder::new(ProcessId(process::id()));
        for running_process in read_running_processes()? {
            recorder.record(Observation::ProcessAlreadyRunning {
                process: running_process.process,
                image_name: running_process.image_name,
            });
        }
        Ok(Self {
            signals,
            recorder,
            undecoded_event_count: 0,
            decode_failure_examples: Vec::new(),
        })
    }

    pub fn follow_until_settled(
        &mut self,
        program: &mut Child,
        is_stop_requested: impl Fn() -> bool,
    ) -> Result<(), WatchError> {
        let mut quiet_since: Option<Instant> = None;
        loop {
            self.absorb_signals_for(SIGNAL_POLL_INTERVAL)?;
            if is_stop_requested() {
                return Ok(());
            }
            let program_exited = program
                .try_wait()
                .map_err(WatchError::ProgramState)?
                .is_some();
            let is_quiet = program_exited && !self.recorder.has_live_tracked_process();
            quiet_since = is_quiet.then(|| quiet_since.unwrap_or_else(Instant::now));
            if quiet_since.is_some_and(|since| since.elapsed() >= SETTLE_DURATION) {
                return Ok(());
            }
        }
    }

    pub fn finish(mut self) -> WatchOutcome {
        while let Ok(signal) = self.signals.try_recv() {
            self.absorb(signal);
        }
        WatchOutcome {
            changes: self.recorder.into_changes(),
            undecoded_event_count: self.undecoded_event_count,
            decode_failure_examples: self.decode_failure_examples,
        }
    }

    fn absorb_signals_for(&mut self, duration: Duration) -> Result<(), WatchError> {
        let deadline = Instant::now() + duration;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.signals.recv_timeout(remaining) {
                Ok(signal) => self.absorb(signal),
                Err(RecvTimeoutError::Timeout) => return Ok(()),
                Err(RecvTimeoutError::Disconnected) => return Err(WatchError::TraceEndedEarly),
            }
        }
    }

    fn absorb(&mut self, signal: TraceSignal) {
        match signal {
            TraceSignal::Observed(observation) => self.recorder.record(observation),
            TraceSignal::DecodeFailed(failure) => self.note_decode_failure(&failure),
        }
    }

    fn note_decode_failure(&mut self, failure: &DecodeFailure) {
        if !self.recorder.is_tracking(failure.actor) {
            return;
        }
        self.undecoded_event_count += 1;
        if self.decode_failure_examples.len() < DECODE_FAILURE_EXAMPLE_LIMIT {
            self.decode_failure_examples.push(failure.to_string());
        }
    }
}
