use std::{
    mem::size_of,
    ptr,
    sync::mpsc::{self, Receiver, Sender},
};

use anyhow::{Result, bail, ensure};
use windows::Win32::Media::{
    Audio::{
        CALLBACK_FUNCTION, HMIDIOUT, MIDIHDR, MIDIOUTCAPSW, MOD_SWSYNTH, midiOutClose, midiOutGetDevCapsW, midiOutGetNumDevs, midiOutLongMsg,
        midiOutOpen, midiOutPrepareHeader, midiOutReset, midiOutShortMsg, midiOutUnprepareHeader,
    },
    MM_MOM_DONE,
};

pub(super) struct Connection {
    device: HMIDIOUT,
    pending: Vec<(Box<MIDIHDR>, Vec<u8>)>,
    returned: Receiver<usize>,
    _callback: Box<Sender<usize>>,
}

unsafe extern "system" fn buffer_returned(_device: HMIDIOUT, message: u32, context: usize, header: usize, _unused: usize) {
    if message == MM_MOM_DONE {
        let returned = unsafe { &*(context as *const Sender<usize>) };
        let _ = returned.send(header);
    }
}

impl Connection {
    pub fn new() -> Result<Self> {
        unsafe {
            for index in 0..midiOutGetNumDevs() {
                let mut caps = MIDIOUTCAPSW::default();
                if midiOutGetDevCapsW(index as usize, &mut caps, size_of::<MIDIOUTCAPSW>() as u32) != 0 {
                    continue;
                }
                let name = caps.szPname;
                let end = name.iter().position(|c| *c == 0).unwrap_or(name.len());
                if u32::from(caps.wTechnology) != MOD_SWSYNTH || String::from_utf16_lossy(&name[..end]) != "Microsoft GS Wavetable Synth" {
                    continue;
                }
                let mut device = HMIDIOUT::default();
                let (tx, returned) = mpsc::channel();
                let callback = Box::new(tx);
                let status = midiOutOpen(
                    &mut device,
                    index,
                    Some(buffer_returned as *const () as usize),
                    Some(ptr::from_ref(&*callback) as usize),
                    CALLBACK_FUNCTION,
                );
                ensure!(status == 0, "Could not open Microsoft GS Wavetable Synth: WinMM {status}");
                return Ok(Self {
                    device,
                    pending: Vec::new(),
                    returned,
                    _callback: callback,
                });
            }
        }
        bail!("Microsoft GS Wavetable Synth is unavailable")
    }

    pub fn send(&mut self, data: &[u8]) -> Result<()> {
        unsafe {
            if data.first() == Some(&0xf0) {
                let mut bytes = data.to_vec();
                let mut header = Box::new(MIDIHDR::default());
                header.lpData = windows::core::PSTR(bytes.as_mut_ptr());
                header.dwBufferLength = bytes.len() as u32;
                let size = size_of::<MIDIHDR>() as u32;
                let status = midiOutPrepareHeader(self.device, &mut *header, size);
                ensure!(status == 0, "MIDI SysEx preparation failed: WinMM {status}");
                let status = midiOutLongMsg(self.device, &*header, size);
                if status != 0 {
                    midiOutUnprepareHeader(self.device, &mut *header, size);
                    bail!("MIDI SysEx output failed: WinMM {status}");
                }
                // Both allocations remain stable until the driver returns this buffer.
                self.pending.push((header, bytes));
            } else {
                let mut message = [0; 4];
                for (target, source) in message.iter_mut().zip(data) {
                    *target = *source;
                }
                let status = midiOutShortMsg(self.device, u32::from_le_bytes(message));
                ensure!(status == 0, "MIDI output failed: WinMM {status}");
            }
        }
        self.reap();
        Ok(())
    }

    pub fn reap(&mut self) {
        while let Ok(address) = self.returned.try_recv() {
            let index = self
                .pending
                .iter()
                .position(|(header, _)| (&raw const **header) as usize == address)
                .unwrap();
            let status = unsafe { midiOutUnprepareHeader(self.device, &mut *self.pending[index].0, size_of::<MIDIHDR>() as u32) };
            if status == 0 {
                self.pending.swap_remove(index);
            } else {
                log::error!("Could not release MIDI SysEx buffer: WinMM {status}");
            }
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            midiOutReset(self.device);
            for (header, _) in &mut self.pending {
                midiOutUnprepareHeader(self.device, &mut **header, size_of::<MIDIHDR>() as u32);
            }
            midiOutClose(self.device);
        }
    }
}
