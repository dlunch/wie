extern crate alloc;

mod audio_sink;
mod database;
mod window;

use std::{
    collections::HashSet,
    error::Error,
    fs,
    io::stderr,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use midir::MidiOutput;
use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use winit::keyboard::{KeyCode as WinitKeyCode, PhysicalKey};

use wie_backend::{extract_zip, Archive, Event, Instant, KeyCode, Platform, Screen};
use wie_j2me::J2MEArchive;
use wie_ktf::KtfArchive;
use wie_lgt::LgtArchive;
use wie_skt::SktArchive;

use self::{
    audio_sink::AudioSink,
    database::DatabaseRepository,
    window::{WindowCallbackEvent, WindowImpl},
};

struct WieCliPlatform {
    audio_thread_tx: Sender<(u8, u32, Vec<i16>)>,
    database_repository: DatabaseRepository,
    window: Box<dyn Screen>,
}

impl WieCliPlatform {
    fn new(app_id: &str, window: Box<dyn Screen>) -> Self {
        let (tx, rx) = channel();
        thread::spawn(|| Self::audio_thread(rx));

        Self {
            audio_thread_tx: tx,
            database_repository: DatabaseRepository::new(app_id),
            window,
        }
    }

    fn audio_thread(rx: Receiver<(u8, u32, Vec<i16>)>) {
        let default_output = OutputStream::try_default();
        if default_output.is_err() {
            // do nothing if we can't open output
            loop {
                rx.recv().unwrap();
            }
        }

        let (_output_stream, stream_handle) = default_output.unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        loop {
            let (channel, sampling_rate, wave_data) = rx.recv().unwrap();
            let buffer = SamplesBuffer::new(channel as _, sampling_rate as _, wave_data);

            // TODO we should be able to play multiple audio at once
            sink.append(buffer);
        }
    }
}

impl Platform for WieCliPlatform {
    fn screen(&mut self) -> &mut dyn Screen {
        self.window.as_mut()
    }

    fn now(&self) -> Instant {
        let now = SystemTime::now();
        let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();

        Instant::from_epoch_millis(since_the_epoch.as_millis() as _)
    }

    fn database_repository(&self) -> &dyn wie_backend::DatabaseRepository {
        &self.database_repository
    }

    fn audio_sink(&self) -> Box<dyn wie_backend::AudioSink> {
        let midi_out = (|| {
            let midi_out = MidiOutput::new("wie_cli")?;
            let midi_ports = midi_out.ports();
            let out_port = midi_ports.last().ok_or_else(|| anyhow::anyhow!("No MIDI output port"))?;

            Ok::<_, Box<dyn Error>>(midi_out.connect(out_port, "wie_cli")?)
        })()
        .ok();

        Box::new(AudioSink::new(midi_out, self.audio_thread_tx.clone()))
    }
}

#[derive(Parser)]
struct Args {
    filename: String,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    start(&Args::parse().filename)
}

pub fn start(filename: &str) -> anyhow::Result<()> {
    let buf = fs::read(filename)?;
    let archive: Box<dyn Archive> = if filename.ends_with("zip") {
        let files = extract_zip(&buf).unwrap();

        if KtfArchive::is_ktf_archive(&files) {
            Box::new(KtfArchive::from_zip(files)?)
        } else if LgtArchive::is_lgt_archive(&files) {
            Box::new(LgtArchive::from_zip(files)?)
        } else if SktArchive::is_skt_archive(&files) {
            Box::new(SktArchive::from_zip(files)?)
        } else {
            anyhow::bail!("Unknown archive format");
        }
    } else if filename.ends_with("jad") {
        let jar_filename = filename.replace(".jad", ".jar");
        let jar = fs::read(jar_filename)?;

        Box::new(J2MEArchive::from_jad_jar(buf, jar))
    } else if filename.ends_with("jar") {
        let filename_without_ext = filename.trim_end_matches(".jar");

        if KtfArchive::is_ktf_jar(&buf) {
            Box::new(KtfArchive::from_jar(buf, filename_without_ext.into(), None, Default::default()))
        } else if LgtArchive::is_lgt_jar(&buf) {
            Box::new(LgtArchive::from_jar(buf, filename_without_ext, None))
        } else if SktArchive::is_skt_jar(&buf) {
            Box::new(SktArchive::from_jar(buf, filename_without_ext, None, Default::default()))
        } else {
            Box::new(J2MEArchive::from_jar(filename_without_ext.into(), buf))
        }
    } else {
        anyhow::bail!("Unknown file format");
    };

    let window = WindowImpl::new(240, 320).unwrap(); // TODO hardcoded size
    let platform = WieCliPlatform::new(&archive.id(), Box::new(window.handle()));

    let mut app = archive.load_app(Box::new(platform))?;

    app.start()?;

    let mut key_events = HashSet::new();
    window.run(move |event| {
        match event {
            WindowCallbackEvent::Update => app.tick()?,
            WindowCallbackEvent::Redraw => app.on_event(Event::Redraw),
            WindowCallbackEvent::Keydown(x) => {
                if let Some(keycode) = convert_key(x) {
                    if !key_events.contains(&keycode) {
                        app.on_event(Event::Keydown(keycode));
                        key_events.insert(keycode);
                    }
                }
            }
            WindowCallbackEvent::Keyup(x) => {
                if let Some(keycode) = convert_key(x) {
                    if key_events.contains(&keycode) {
                        key_events.remove(&keycode);
                    }
                    app.on_event(Event::Keyup(keycode));
                }
            }
        }

        anyhow::Ok(())
    })
}

fn convert_key(key: PhysicalKey) -> Option<KeyCode> {
    match key {
        PhysicalKey::Code(WinitKeyCode::Digit1) => Some(KeyCode::NUM1),
        PhysicalKey::Code(WinitKeyCode::Digit2) => Some(KeyCode::NUM2),
        PhysicalKey::Code(WinitKeyCode::Digit3) => Some(KeyCode::NUM3),
        PhysicalKey::Code(WinitKeyCode::KeyQ) => Some(KeyCode::NUM4),
        PhysicalKey::Code(WinitKeyCode::KeyW) => Some(KeyCode::NUM5),
        PhysicalKey::Code(WinitKeyCode::KeyE) => Some(KeyCode::NUM6),
        PhysicalKey::Code(WinitKeyCode::KeyA) => Some(KeyCode::NUM7),
        PhysicalKey::Code(WinitKeyCode::KeyS) => Some(KeyCode::NUM8),
        PhysicalKey::Code(WinitKeyCode::KeyD) => Some(KeyCode::NUM9),
        PhysicalKey::Code(WinitKeyCode::KeyZ) => Some(KeyCode::STAR),
        PhysicalKey::Code(WinitKeyCode::KeyX) => Some(KeyCode::NUM0),
        PhysicalKey::Code(WinitKeyCode::KeyC) => Some(KeyCode::HASH),
        PhysicalKey::Code(WinitKeyCode::Space) => Some(KeyCode::OK),
        PhysicalKey::Code(WinitKeyCode::ArrowUp) => Some(KeyCode::UP),
        PhysicalKey::Code(WinitKeyCode::ArrowDown) => Some(KeyCode::DOWN),
        PhysicalKey::Code(WinitKeyCode::ArrowLeft) => Some(KeyCode::LEFT),
        PhysicalKey::Code(WinitKeyCode::ArrowRight) => Some(KeyCode::RIGHT),
        _ => None,
    }
}
