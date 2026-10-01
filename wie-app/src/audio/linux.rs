use alsa::{
    Direction, Seq,
    seq::{Addr, ClientIter, MidiEvent, PortCap, PortIter, PortType},
};
use anyhow::{Context, Result};

pub(super) struct Connection {
    seq: Seq,
    encoder: MidiEvent,
    source: i32,
    destination: Addr,
}

impl Connection {
    pub fn new() -> Result<Self> {
        let seq = Seq::open(None, Some(Direction::Playback), false)?;
        seq.set_client_name(c"wie")?;
        let destination = ClientIter::new(&seq)
            .flat_map(|client| PortIter::new(&seq, client.get_client()))
            .find(|port| {
                let kind = port.get_type();
                port.get_capability().contains(PortCap::WRITE | PortCap::SUBS_WRITE)
                    && kind.intersects(PortType::SYNTH | PortType::SYNTHESIZER)
                    && !kind.contains(PortType::HARDWARE)
            })
            .map(|port| port.addr())
            .context("No installed software MIDI synthesizer is available")?;
        let source = seq.create_simple_port(c"wie", PortCap::READ, PortType::MIDI_GENERIC | PortType::APPLICATION)?;
        let encoder = MidiEvent::new(4096)?;
        Ok(Self {
            seq,
            encoder,
            source,
            destination,
        })
    }

    pub fn send(&mut self, data: &[u8]) -> Result<()> {
        self.encoder.resize_buffer(data.len() as u32)?;
        self.encoder.reset_encode();
        let (_, event) = self.encoder.encode(data)?;
        if let Some(mut event) = event {
            event.set_source(self.source);
            event.set_dest(self.destination);
            event.set_direct();
            self.seq.event_output_direct(&mut event)?;
        }
        Ok(())
    }

    pub fn reap(&mut self) {}
}
