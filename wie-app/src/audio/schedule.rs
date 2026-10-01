use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};

use wie_backend::{AudioHandle, AudioSequence};

pub(super) enum Due {
    Event(AudioHandle, Arc<AudioSequence>, usize),
    End(AudioHandle),
}

pub(super) struct Clock {
    pub origin: Instant,
    pub paused_at: Option<Instant>,
}

impl Clock {
    pub fn elapsed(&self, now: Instant) -> Duration {
        self.paused_at.unwrap_or(now).duration_since(self.origin)
    }

    #[cfg(any(mobile, test))]
    pub fn resume(&mut self, now: Instant) {
        if let Some(paused_at) = self.paused_at.take() {
            self.origin += now.duration_since(paused_at);
        }
    }
}

struct Playback {
    sequence: Arc<AudioSequence>,
    repeat: bool,
    start: Duration,
    next_event: usize,
}

#[derive(Default)]
pub(super) struct Schedule {
    playing: BTreeMap<AudioHandle, Playback>,
}

impl Schedule {
    pub fn play(&mut self, handle: AudioHandle, sequence: Arc<AudioSequence>, repeat: bool, now: Duration) {
        self.playing.insert(
            handle,
            Playback {
                sequence,
                repeat,
                start: now,
                next_event: 0,
            },
        );
    }

    pub fn stop(&mut self, handle: AudioHandle) {
        self.playing.remove(&handle);
    }

    pub fn deadline(&self) -> Option<Duration> {
        self.playing
            .values()
            .map(|p| p.start + Duration::from_millis(p.sequence.events.get(p.next_event).map_or(p.sequence.duration, |e| e.time)))
            .min()
    }

    pub fn due(&mut self, now: Duration) -> Vec<Due> {
        let mut due = Vec::new();
        self.playing.retain(|&handle, playback| {
            while let Some(event) = playback.sequence.events.get(playback.next_event) {
                if playback.start + Duration::from_millis(event.time) > now {
                    break;
                }
                due.push(Due::Event(handle, playback.sequence.clone(), playback.next_event));
                playback.next_event += 1;
            }
            let duration = Duration::from_millis(playback.sequence.duration);
            if playback.next_event == playback.sequence.events.len() && playback.start + duration <= now {
                due.push(Due::End(handle));
                if !playback.repeat || duration.is_zero() {
                    return false;
                }
                // Keep the original beat, but do not replay missed cycles after a stalled device.
                let cycles = (now - playback.start).as_nanos() / duration.as_nanos();
                playback.start += duration.mul_f64(cycles as f64);
                playback.next_event = 0;
            }
            true
        });
        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wie_backend::{AudioEventData, TimedAudioEvent};

    #[test]
    fn repeat_stop_and_replay_preserve_other_handles_and_absolute_deadlines() {
        let sequence = Arc::new(AudioSequence {
            duration: 100,
            events: vec![TimedAudioEvent {
                time: 10,
                data: AudioEventData::Midi(vec![0x90, 60, 100]),
            }],
        });
        let mut schedule = Schedule::default();
        schedule.play(1, sequence.clone(), true, Duration::ZERO);
        schedule.play(2, sequence.clone(), false, Duration::from_millis(25));
        assert!(matches!(schedule.due(Duration::from_millis(15)).as_slice(), [Due::Event(1, _, 0)]));
        assert!(matches!(
            schedule.due(Duration::from_millis(105)).as_slice(),
            [Due::End(1), Due::Event(2, _, 0)]
        ));
        assert_eq!(schedule.deadline(), Some(Duration::from_millis(110)));
        schedule.stop(1);
        assert_eq!(schedule.deadline(), Some(Duration::from_millis(125)));
        schedule.play(1, sequence, false, Duration::from_millis(110));
        assert!(matches!(
            schedule.due(Duration::from_millis(125)).as_slice(),
            [Due::Event(1, _, 0), Due::End(2)]
        ));
        assert!(matches!(schedule.due(Duration::from_millis(210)).as_slice(), [Due::End(1)]));
        assert_eq!(schedule.deadline(), None);
    }

    #[test]
    fn stalled_and_zero_duration_repeats_do_not_spin() {
        let mut schedule = Schedule::default();
        schedule.play(
            1,
            Arc::new(AudioSequence {
                duration: 100,
                events: vec![],
            }),
            true,
            Duration::ZERO,
        );
        assert!(matches!(schedule.due(Duration::from_millis(1005)).as_slice(), [Due::End(1)]));
        assert_eq!(schedule.deadline(), Some(Duration::from_millis(1100)));
        schedule.play(2, Arc::new(AudioSequence { duration: 0, events: vec![] }), true, Duration::ZERO);
        assert!(matches!(schedule.due(Duration::from_millis(1005)).as_slice(), [Due::End(2)]));
    }

    #[test]
    fn suspended_time_neither_advances_events_nor_accumulates_repeat_cycles() {
        let origin = Instant::now();
        let mut clock = Clock { origin, paused_at: None };
        let mut schedule = Schedule::default();
        schedule.play(
            1,
            Arc::new(AudioSequence {
                duration: 100,
                events: vec![TimedAudioEvent {
                    time: 50,
                    data: AudioEventData::Midi(vec![0x90, 60, 100]),
                }],
            }),
            true,
            Duration::ZERO,
        );
        clock.paused_at = Some(origin + Duration::from_millis(25));
        let resumed = origin + Duration::from_secs(10);
        assert_eq!(clock.elapsed(resumed), Duration::from_millis(25));
        clock.resume(resumed);
        assert!(schedule.due(clock.elapsed(resumed)).is_empty());
        assert!(matches!(
            schedule.due(clock.elapsed(resumed + Duration::from_millis(25))).as_slice(),
            [Due::Event(1, _, 0)]
        ));
        assert!(matches!(
            schedule.due(clock.elapsed(resumed + Duration::from_millis(75))).as_slice(),
            [Due::End(1)]
        ));
        clock.resume(resumed + Duration::from_millis(80));
        assert_eq!(clock.elapsed(resumed + Duration::from_millis(80)), Duration::from_millis(105));
        assert_eq!(schedule.deadline(), Some(Duration::from_millis(150)));
    }
}
