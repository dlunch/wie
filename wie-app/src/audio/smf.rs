use anyhow::{Context, Result, ensure};
use midly::{Arena, Format, Header, MetaMessage, Smf, Timing, TrackEvent, TrackEventKind, live::LiveEvent};
use wie_backend::{AudioEventData, AudioSequence};

pub(super) fn encode(sequence: &AudioSequence) -> Result<Vec<u8>> {
    let arena = Arena::new();
    let mut track = vec![TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::Tempo(1_000_000.into())),
    }];
    let mut previous = 0;
    for event in &sequence.events {
        if let AudioEventData::Midi(data) = &event.data {
            let delta = event.time - previous;
            ensure!(delta <= 0x0fff_ffff, "MIDI event delay exceeds SMF limits");
            track.push(TrackEvent {
                delta: (delta as u32).into(),
                kind: LiveEvent::parse(data).context("Invalid MIDI message")?.as_track_event(&arena),
            });
            previous = event.time;
        }
    }
    let tail = sequence.duration - previous;
    ensure!(tail <= 0x0fff_ffff, "MIDI sequence duration exceeds SMF limits");
    track.push(TrackEvent {
        delta: (tail as u32).into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });
    let smf = Smf {
        header: Header::new(Format::SingleTrack, Timing::Metrical(1000.into())),
        tracks: vec![track],
    };
    let mut bytes = Vec::new();
    smf.write_std(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wie_backend::TimedAudioEvent;

    #[test]
    fn memory_smf_preserves_order_sysex_and_silent_tail_without_pcm() {
        let bytes = encode(&AudioSequence {
            duration: 1250,
            events: vec![
                TimedAudioEvent {
                    time: 0,
                    data: AudioEventData::Midi(vec![0xf0, 0x7e, 0x7f, 9, 1, 0xf7]),
                },
                TimedAudioEvent {
                    time: 25,
                    data: AudioEventData::Wave {
                        channels: 1,
                        sampling_rate: 8000,
                        samples: vec![0],
                    },
                },
                TimedAudioEvent {
                    time: 100,
                    data: AudioEventData::Midi(vec![0xc2, 3]),
                },
                TimedAudioEvent {
                    time: 100,
                    data: AudioEventData::Midi(vec![0x92, 60, 100]),
                },
                TimedAudioEvent {
                    time: 1000,
                    data: AudioEventData::Midi(vec![0x82, 60, 0]),
                },
            ],
        })
        .unwrap();
        let smf = Smf::parse(&bytes).unwrap();
        assert_eq!(smf.header.timing, Timing::Metrical(1000.into()));
        let track = &smf.tracks[0];
        assert_eq!(track.iter().map(|e| e.delta.as_int()).collect::<Vec<_>>(), [0, 0, 100, 0, 900, 250]);
        assert!(matches!(track[0].kind, TrackEventKind::Meta(MetaMessage::Tempo(t)) if t.as_int() == 1_000_000));
        assert!(matches!(track[1].kind, TrackEventKind::SysEx([0x7e, 0x7f, 9, 1, 0xf7])));
        assert!(
            matches!(track[2].kind, TrackEventKind::Midi { channel, message: midly::MidiMessage::ProgramChange { program } } if channel.as_int() == 2 && program.as_int() == 3)
        );
        assert!(matches!(track[5].kind, TrackEventKind::Meta(MetaMessage::EndOfTrack)));
    }
}
