use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use midir::MidiOutputConnection;
use wie_backend::{MidiEvent, ScheduledMidiEvent};

const MAX_EVENT_LATENESS: Duration = Duration::from_millis(50);

pub type MidiConnection = Arc<Mutex<MidiOutputConnection>>;

pub struct MidiWorker {
    tx: Sender<MidiCommand>,
}

impl MidiWorker {
    pub fn new(connection: MidiConnection) -> Option<Self> {
        let (tx, rx) = mpsc::channel();
        if let Err(error) = thread::Builder::new()
            .name("wie-midi".to_string())
            .spawn(move || run_worker(connection, rx))
        {
            tracing::warn!(%error, "Failed to start MIDI worker");
            return None;
        }

        Some(Self { tx })
    }

    pub fn schedule(&self, playback_id: u64, events: &[ScheduledMidiEvent], duration_ms: u64, repeat: bool) -> bool {
        self.tx
            .send(MidiCommand::Play {
                playback_id,
                events: events.to_vec(),
                duration: Duration::from_millis(duration_ms),
                repeat,
            })
            .is_ok()
    }

    pub fn stop(&self, playback_id: u64) -> bool {
        self.tx.send(MidiCommand::Stop { playback_id }).is_ok()
    }

    pub fn send(&self, event: MidiEvent) -> bool {
        self.tx.send(MidiCommand::Send(event)).is_ok()
    }
}

enum MidiCommand {
    Play {
        playback_id: u64,
        events: Vec<ScheduledMidiEvent>,
        duration: Duration,
        repeat: bool,
    },
    Stop {
        playback_id: u64,
    },
    Send(MidiEvent),
}

struct Playback {
    events: Vec<ScheduledMidiEvent>,
    duration: Duration,
    repeat: bool,
    cycle_start: Instant,
    next_event: usize,
    active_notes: BTreeSet<(u8, u8)>,
    used_channels: BTreeSet<u8>,
    late_event_count: u64,
    max_lateness: Duration,
}

impl Playback {
    fn new(events: Vec<ScheduledMidiEvent>, duration: Duration, repeat: bool, now: Instant) -> Self {
        Self {
            events,
            duration,
            repeat,
            cycle_start: now,
            next_event: 0,
            active_notes: BTreeSet::new(),
            used_channels: BTreeSet::new(),
            late_event_count: 0,
            max_lateness: Duration::ZERO,
        }
    }

    fn next_deadline(&self) -> Instant {
        if let Some(event) = self.events.get(self.next_event) {
            self.cycle_start + Duration::from_millis(event.at_ms)
        } else {
            self.cycle_start + self.duration
        }
    }

    fn take_due_event(&mut self, now: Instant) -> Option<(Instant, MidiEvent)> {
        let event = self.events.get(self.next_event)?;
        let deadline = self.cycle_start + Duration::from_millis(event.at_ms);
        if deadline > now {
            return None;
        }

        self.next_event += 1;
        let lateness = now.saturating_duration_since(deadline);
        if lateness > Duration::from_millis(10) {
            self.late_event_count += 1;
            self.max_lateness = self.max_lateness.max(lateness);
        }

        Some((deadline, event.event.clone()))
    }

    fn finish_cycle(&mut self, now: Instant) -> bool {
        if self.next_event < self.events.len() || self.next_deadline() > now {
            return false;
        }

        if !self.repeat || self.duration.is_zero() {
            return true;
        }

        let elapsed = now.saturating_duration_since(self.cycle_start);
        let completed_cycles = (elapsed.as_nanos() / self.duration.as_nanos()).max(1);
        let advance = self.duration.saturating_mul(completed_cycles.min(u32::MAX as u128) as u32);
        self.cycle_start += advance;
        self.next_event = 0;
        self.active_notes.clear();
        self.used_channels.clear();
        self.late_event_count = 0;
        self.max_lateness = Duration::ZERO;

        false
    }

    fn recover_from_late_wakeup(&mut self, now: Instant) -> LateRecovery {
        if !self.repeat || self.duration.is_zero() || self.next_event >= self.events.len() {
            return LateRecovery::None;
        }

        let elapsed = now.saturating_duration_since(self.cycle_start);
        let completed_cycles = elapsed.as_nanos() / self.duration.as_nanos();
        let cycle_remainder = elapsed.as_nanos() % self.duration.as_nanos();
        if completed_cycles == 0 && now.saturating_duration_since(self.next_deadline()) <= MAX_EVENT_LATENESS {
            return LateRecovery::None;
        }

        if completed_cycles > 0 && cycle_remainder == 0 {
            let first_boundary_event = self.events[self.next_event..]
                .iter()
                .position(|event| Duration::from_millis(event.at_ms) == self.duration)
                .map(|index| self.next_event + index);
            if let Some(first_boundary_event) = first_boundary_event {
                let previous_cycles = completed_cycles - 1;
                let advance = self.duration.saturating_mul(previous_cycles.min(u32::MAX as u128) as u32);
                self.cycle_start += advance;
                self.next_event = first_boundary_event;
                return LateRecovery::CycleBoundary;
            }
        }

        self.cycle_start = now;
        self.next_event = 0;

        LateRecovery::RestartCycle
    }

    fn track(&mut self, event: &MidiEvent) {
        match event {
            MidiEvent::NoteOn { channel, note, velocity } => {
                self.used_channels.insert(*channel);
                if *velocity == 0 {
                    self.active_notes.remove(&(*channel, *note));
                } else {
                    self.active_notes.insert((*channel, *note));
                }
            }
            MidiEvent::NoteOff { channel, note, .. } => {
                self.used_channels.insert(*channel);
                self.active_notes.remove(&(*channel, *note));
            }
            MidiEvent::ProgramChange { channel, .. } | MidiEvent::ControlChange { channel, .. } | MidiEvent::PitchBend { channel, .. } => {
                self.used_channels.insert(*channel);
            }
            MidiEvent::SysEx(_) => {}
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LateRecovery {
    None,
    CycleBoundary,
    RestartCycle,
}

fn run_worker(connection: MidiConnection, rx: Receiver<MidiCommand>) {
    // Scheduling and deadline handling are shared by every native target. Some
    // platforms need a process-level hint to make timed waits precise enough;
    // that optional hint must not affect playback semantics.
    let mut playbacks = BTreeMap::new();
    let mut timer_precision = None;

    loop {
        let now = Instant::now();
        dispatch_due_events(&connection, &mut playbacks, now);

        if playbacks.is_empty() {
            timer_precision = None;
        } else if timer_precision.is_none() {
            timer_precision = Some(platform_timer::TimerPrecision::acquire());
        }

        let command = match next_deadline(&playbacks) {
            Some(deadline) => match receive_until(&rx, deadline) {
                WorkerWake::Command(command) => Some(command),
                WorkerWake::Deadline => continue,
                WorkerWake::Disconnected => None,
            },
            None => rx.recv().ok(),
        };

        let Some(command) = command else {
            for (_, playback) in playbacks {
                cleanup_playback(&connection, &playback);
            }
            break;
        };

        match command {
            MidiCommand::Play {
                playback_id,
                events,
                duration,
                repeat,
            } => {
                if let Some(previous) = playbacks.remove(&playback_id) {
                    cleanup_playback(&connection, &previous);
                }
                tracing::debug!(
                    playback_id,
                    event_count = events.len(),
                    duration_ms = duration.as_millis(),
                    repeat,
                    "Scheduled MIDI playback"
                );
                playbacks.insert(playback_id, Playback::new(events, duration, repeat, Instant::now()));
            }
            MidiCommand::Stop { playback_id } => {
                if let Some(playback) = playbacks.remove(&playback_id) {
                    cleanup_playback(&connection, &playback);
                }
            }
            MidiCommand::Send(event) => send_event(&connection, &event),
        }
    }
}

enum WorkerWake {
    Command(MidiCommand),
    Deadline,
    Disconnected,
}

fn receive_until(rx: &Receiver<MidiCommand>, deadline: Instant) -> WorkerWake {
    loop {
        let now = Instant::now();
        if now >= deadline {
            return WorkerWake::Deadline;
        }

        match rx.recv_timeout(deadline - now) {
            Ok(command) => return WorkerWake::Command(command),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return WorkerWake::Disconnected,
        }
    }
}

fn next_deadline(playbacks: &BTreeMap<u64, Playback>) -> Option<Instant> {
    playbacks.values().map(Playback::next_deadline).min()
}

fn dispatch_due_events(connection: &MidiConnection, playbacks: &mut BTreeMap<u64, Playback>, now: Instant) {
    loop {
        let next_playback_id = playbacks
            .iter()
            .filter(|(_, playback)| playback.next_deadline() <= now)
            .min_by_key(|(playback_id, playback)| (playback.next_deadline(), **playback_id))
            .map(|(playback_id, _)| *playback_id);
        let Some(playback_id) = next_playback_id else {
            break;
        };

        let playback = playbacks.get_mut(&playback_id).unwrap();
        match playback.recover_from_late_wakeup(now) {
            LateRecovery::None | LateRecovery::CycleBoundary => {}
            LateRecovery::RestartCycle => {
                cleanup_playback(connection, playback);
                tracing::debug!(
                    playback_id,
                    late_event_count = playback.late_event_count,
                    max_lateness_ms = playback.max_lateness.as_millis(),
                    "Restarted a late MIDI playback cycle"
                );
                playback.active_notes.clear();
                playback.used_channels.clear();
                playback.late_event_count = 0;
                playback.max_lateness = Duration::ZERO;
                continue;
            }
        }
        if let Some((_deadline, event)) = playback.take_due_event(now) {
            send_event(connection, &event);
            playback.track(&event);
            continue;
        }

        cleanup_playback(connection, playback);
        tracing::debug!(
            playback_id,
            late_event_count = playback.late_event_count,
            max_lateness_ms = playback.max_lateness.as_millis(),
            "MIDI playback timing"
        );
        if playback.finish_cycle(now) {
            playbacks.remove(&playback_id);
        }
    }
}

fn cleanup_playback(connection: &MidiConnection, playback: &Playback) {
    for event in cleanup_events(playback) {
        send_event(connection, &event);
    }
}

fn cleanup_events(playback: &Playback) -> Vec<MidiEvent> {
    let mut result = Vec::new();
    for (channel, note) in &playback.active_notes {
        result.push(MidiEvent::NoteOff {
            channel: *channel,
            note: *note,
            velocity: 0,
        });
    }

    for channel in &playback.used_channels {
        result.push(MidiEvent::ControlChange {
            channel: *channel,
            control: 64,
            value: 0,
        });
        result.push(MidiEvent::ControlChange {
            channel: *channel,
            control: 120,
            value: 0,
        });
        result.push(MidiEvent::ControlChange {
            channel: *channel,
            control: 123,
            value: 0,
        });
    }

    result
}

pub fn send_event(connection: &MidiConnection, event: &MidiEvent) {
    match event {
        MidiEvent::NoteOn { channel, note, velocity } => send_bytes(connection, &[0x90 | channel, *note, *velocity]),
        MidiEvent::NoteOff { channel, note, velocity } => send_bytes(connection, &[0x80 | channel, *note, *velocity]),
        MidiEvent::ProgramChange { channel, program } => send_bytes(connection, &[0xC0 | channel, *program]),
        MidiEvent::ControlChange { channel, control, value } => send_bytes(connection, &[0xB0 | channel, *control, *value]),
        MidiEvent::PitchBend { channel, value } => send_bytes(connection, &[0xE0 | channel, (value & 0x7f) as u8, ((value >> 7) & 0x7f) as u8]),
        MidiEvent::SysEx(data) => send_bytes(connection, data),
    }
}

fn send_bytes(connection: &MidiConnection, data: &[u8]) {
    let result = match connection.lock() {
        Ok(mut connection) => connection.send(data),
        Err(poisoned) => poisoned.into_inner().send(data),
    };
    if let Err(error) = result {
        tracing::warn!(%error, "Failed to send MIDI message");
    }
}

mod platform_timer {
    #[cfg(windows)]
    mod implementation {
        use windows_sys::Win32::Media::{TIMERR_NOERROR, timeBeginPeriod, timeEndPeriod};

        pub struct TimerPrecision {
            enabled: bool,
        }

        impl TimerPrecision {
            pub fn acquire() -> Self {
                let enabled = unsafe { timeBeginPeriod(1) } == TIMERR_NOERROR;
                if !enabled {
                    tracing::warn!("Failed to request 1 ms Windows timer resolution");
                }

                Self { enabled }
            }
        }

        impl Drop for TimerPrecision {
            fn drop(&mut self) {
                if self.enabled {
                    unsafe {
                        timeEndPeriod(1);
                    }
                }
            }
        }
    }

    #[cfg(not(windows))]
    mod implementation {
        pub struct TimerPrecision;

        impl TimerPrecision {
            pub fn acquire() -> Self {
                Self
            }
        }
    }

    pub use implementation::TimerPrecision;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note_on(at_ms: u64, note: u8) -> ScheduledMidiEvent {
        ScheduledMidiEvent {
            at_ms,
            event: MidiEvent::NoteOn {
                channel: 0,
                note,
                velocity: 100,
            },
        }
    }

    fn program_change(at_ms: u64, program: u8) -> ScheduledMidiEvent {
        ScheduledMidiEvent {
            at_ms,
            event: MidiEvent::ProgramChange { channel: 0, program },
        }
    }

    #[test]
    fn uses_absolute_event_deadlines() {
        let start = Instant::now();
        let mut playback = Playback::new(vec![note_on(10, 60), note_on(20, 62)], Duration::from_millis(30), false, start);

        assert!(playback.take_due_event(start + Duration::from_millis(9)).is_none());
        assert_eq!(
            playback.take_due_event(start + Duration::from_millis(10)),
            Some((start + Duration::from_millis(10), note_on(10, 60).event))
        );
        assert_eq!(playback.next_deadline(), start + Duration::from_millis(20));
        assert_eq!(
            playback.take_due_event(start + Duration::from_millis(25)),
            Some((start + Duration::from_millis(20), note_on(20, 62).event))
        );
        assert_eq!(playback.next_deadline(), start + Duration::from_millis(30));
    }

    #[test]
    fn repeating_playback_restarts_a_late_cycle_without_drift() {
        let start = Instant::now();
        let mut playback = Playback::new(vec![note_on(5, 60)], Duration::from_millis(30), true, start);

        assert_eq!(
            playback.recover_from_late_wakeup(start + Duration::from_millis(95)),
            LateRecovery::RestartCycle
        );
        assert_eq!(playback.cycle_start, start + Duration::from_millis(95));
        assert_eq!(playback.next_deadline(), start + Duration::from_millis(100));
    }

    #[test]
    fn repeating_playback_keeps_events_within_the_lateness_limit() {
        let start = Instant::now();
        let mut playback = Playback::new(vec![note_on(5, 60)], Duration::from_millis(1000), true, start);

        assert_eq!(playback.recover_from_late_wakeup(start + Duration::from_millis(55)), LateRecovery::None);
        assert_eq!(playback.cycle_start, start);
    }

    #[test]
    fn repeating_playback_does_not_catch_up_previous_cycles() {
        let start = Instant::now();
        let mut playback = Playback::new(vec![note_on(5, 60)], Duration::from_millis(30), true, start);

        assert_eq!(
            playback.recover_from_late_wakeup(start + Duration::from_millis(80)),
            LateRecovery::RestartCycle
        );
        assert_eq!(playback.cycle_start, start + Duration::from_millis(80));
        assert_eq!(playback.next_deadline(), start + Duration::from_millis(85));
    }

    #[test]
    fn repeating_playback_keeps_an_event_at_the_cycle_boundary() {
        let start = Instant::now();
        let mut playback = Playback::new(vec![note_on(30, 60)], Duration::from_millis(30), true, start);

        assert_eq!(
            playback.recover_from_late_wakeup(start + Duration::from_millis(30)),
            LateRecovery::CycleBoundary
        );
        assert_eq!(
            playback.take_due_event(start + Duration::from_millis(30)),
            Some((start + Duration::from_millis(30), note_on(30, 60).event))
        );
    }

    #[test]
    fn repeating_playback_restarts_state_immediately_after_a_late_wakeup() {
        let start = Instant::now();
        let mut playback = Playback::new(vec![program_change(0, 4), note_on(100, 62)], Duration::from_secs(180), true, start);

        assert_eq!(
            playback.recover_from_late_wakeup(start + Duration::from_millis(51)),
            LateRecovery::RestartCycle
        );
        assert_eq!(playback.cycle_start, start + Duration::from_millis(51));
        assert_eq!(playback.next_deadline(), start + Duration::from_millis(51));
        assert_eq!(
            playback.take_due_event(start + Duration::from_millis(51)),
            Some((start + Duration::from_millis(51), program_change(0, 4).event))
        );
    }

    #[test]
    fn repeating_playback_keeps_only_boundary_events_from_the_previous_cycle() {
        let start = Instant::now();
        let mut playback = Playback::new(
            vec![note_on(5, 60), note_on(10, 62), note_on(30, 64)],
            Duration::from_millis(30),
            true,
            start,
        );

        assert_eq!(
            playback.recover_from_late_wakeup(start + Duration::from_millis(60)),
            LateRecovery::CycleBoundary
        );
        assert_eq!(playback.cycle_start, start + Duration::from_millis(30));
        assert_eq!(playback.next_deadline(), start + Duration::from_millis(60));
        assert_eq!(
            playback.take_due_event(start + Duration::from_millis(60)),
            Some((start + Duration::from_millis(60), note_on(30, 64).event))
        );
    }

    #[test]
    fn repeating_playback_starts_a_new_cycle_at_an_exact_boundary() {
        let start = Instant::now();
        let mut playback = Playback::new(vec![note_on(5, 60), note_on(10, 62)], Duration::from_millis(30), true, start);

        assert_eq!(
            playback.recover_from_late_wakeup(start + Duration::from_millis(60)),
            LateRecovery::RestartCycle
        );
        assert_eq!(playback.cycle_start, start + Duration::from_millis(60));
        assert_eq!(playback.next_deadline(), start + Duration::from_millis(65));
    }

    #[test]
    fn repeating_playback_finishes_normally_after_a_late_cycle_end_wakeup() {
        let start = Instant::now();
        let mut playback = Playback::new(vec![note_on(0, 60)], Duration::from_millis(30), true, start);

        assert_eq!(playback.take_due_event(start), Some((start, note_on(0, 60).event)));
        let late_wakeup = start + Duration::from_millis(31);
        assert_eq!(playback.recover_from_late_wakeup(late_wakeup), LateRecovery::None);
        assert!(!playback.finish_cycle(late_wakeup));
        assert_eq!(playback.cycle_start, start + Duration::from_millis(30));
        assert_eq!(playback.next_deadline(), start + Duration::from_millis(30));
    }

    #[test]
    fn disconnected_worker_reports_fallback_for_all_commands() {
        let (tx, rx) = mpsc::channel();
        drop(rx);
        let worker = MidiWorker { tx };

        assert!(!worker.send(note_on(0, 60).event));
        assert!(!worker.schedule(1, &[note_on(0, 60)], 30, false));
        assert!(!worker.stop(1));
    }

    #[test]
    fn tracks_active_notes_and_used_channels() {
        let start = Instant::now();
        let mut playback = Playback::new(vec![], Duration::from_millis(30), false, start);

        playback.track(&note_on(0, 60).event);
        playback.track(&MidiEvent::ProgramChange { channel: 1, program: 4 });
        assert!(playback.active_notes.contains(&(0, 60)));
        assert_eq!(playback.used_channels, BTreeSet::from([0, 1]));

        playback.track(&MidiEvent::NoteOff {
            channel: 0,
            note: 60,
            velocity: 0,
        });
        assert!(playback.active_notes.is_empty());
    }

    #[test]
    fn full_cleanup_stops_notes_and_resets_used_channels() {
        let start = Instant::now();
        let mut playback = Playback::new(vec![], Duration::from_millis(30), false, start);
        playback.track(&note_on(0, 60).event);
        playback.track(&MidiEvent::ProgramChange { channel: 1, program: 4 });

        assert_eq!(
            cleanup_events(&playback),
            vec![
                MidiEvent::NoteOff {
                    channel: 0,
                    note: 60,
                    velocity: 0,
                },
                MidiEvent::ControlChange {
                    channel: 0,
                    control: 64,
                    value: 0,
                },
                MidiEvent::ControlChange {
                    channel: 0,
                    control: 120,
                    value: 0,
                },
                MidiEvent::ControlChange {
                    channel: 0,
                    control: 123,
                    value: 0,
                },
                MidiEvent::ControlChange {
                    channel: 1,
                    control: 64,
                    value: 0,
                },
                MidiEvent::ControlChange {
                    channel: 1,
                    control: 120,
                    value: 0,
                },
                MidiEvent::ControlChange {
                    channel: 1,
                    control: 123,
                    value: 0,
                },
            ]
        );
    }
}
