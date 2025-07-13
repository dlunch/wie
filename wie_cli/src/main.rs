extern crate alloc;

mod audio_sink;
mod database;
mod window;

use core::str;
use std::{
    collections::{HashMap, hash_map::Entry},
    error::Error,
    fs,
    io::stderr,
    sync::mpsc::{Receiver, Sender, channel},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use midir::MidiOutput;
use rodio::{OutputStream, Sink, buffer::SamplesBuffer};
use winit::keyboard::{KeyCode as WinitKeyCode, PhysicalKey};

use wie_backend::{Emulator, Event, Instant, KeyCode, Options, Platform, Screen, extract_zip};
use wie_j2me::J2MEEmulator;
use wie_ktf::KtfEmulator;
use wie_lgt::LgtEmulator;
use wie_skt::SktEmulator;

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
    fn new(window: Box<dyn Screen>) -> Self {
        let (tx, rx) = channel();
        thread::spawn(|| Self::audio_thread(rx));

        Self {
            audio_thread_tx: tx,
            database_repository: DatabaseRepository::new(),
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
            let result = rx.recv();
            if result.is_err() {
                break;
            }
            let (channel, sampling_rate, wave_data) = result.unwrap();

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

    fn write_stdout(&self, buf: &[u8]) {
        let str = str::from_utf8(buf).unwrap();

        println!("{str}")
    }

    fn write_stderr(&self, buf: &[u8]) {
        let str = str::from_utf8(buf).unwrap();

        eprintln!("{str}")
    }
}

#[derive(Parser)]
struct Args {
    filename: String,
    #[arg(long, default_value_t = false)]
    debug: bool,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let options = Options {
        enable_gdbserver: args.debug,
    };

    start(&args.filename, options)
}

pub fn start(filename: &str, options: Options) -> anyhow::Result<()> {
    let window = WindowImpl::new(240, 320).unwrap(); // TODO hardcoded size
    let platform = Box::new(WieCliPlatform::new(Box::new(window.handle())));

    let buf = fs::read(filename)?;
    let mut emulator: Box<dyn Emulator> = if filename.ends_with("zip") {
        let files = extract_zip(&buf).unwrap();

        if KtfEmulator::loadable_archive(&files) {
            Box::new(KtfEmulator::from_archive(platform, files, options)?)
        } else if LgtEmulator::loadable_archive(&files) {
            Box::new(LgtEmulator::from_archive(platform, files, options)?)
        } else if SktEmulator::loadable_archive(&files) {
            Box::new(SktEmulator::from_archive(platform, files)?)
        } else {
            anyhow::bail!("Unknown archive format");
        }
    } else if filename.ends_with("jad") {
        let jar_filename = filename.replace(".jad", ".jar");
        let jar = fs::read(&jar_filename)?;

        let jar_filename = jar_filename[jar_filename.rfind('/').unwrap_or(0) + 1..].to_owned();

        Box::new(J2MEEmulator::from_jad_jar(platform, buf, jar_filename, jar)?)
    } else if filename.ends_with("jar") {
        let filename_without_path = filename[filename.rfind('/').unwrap_or(0) + 1..].to_owned();
        let filename_without_ext = filename_without_path.trim_end_matches(".jar");

        if KtfEmulator::loadable_jar(&buf) {
            Box::new(KtfEmulator::from_jar(
                platform,
                &filename_without_path,
                buf,
                filename_without_ext,
                None,
                options,
            )?)
        } else if LgtEmulator::loadable_jar(&buf) {
            Box::new(LgtEmulator::from_jar(
                platform,
                &filename_without_path,
                buf,
                filename_without_ext,
                None,
                options,
            )?)
        } else if SktEmulator::loadable_jar(&buf) {
            Box::new(SktEmulator::from_jar(platform, &filename_without_path, buf, filename_without_ext, None)?)
        } else {
            Box::new(J2MEEmulator::from_jar(platform, &filename_without_path, buf)?)
        }
    } else {
        anyhow::bail!("Unknown file format");
    };

    let mut key_events = HashMap::new();
    window.run(move |event| {
        match event {
            WindowCallbackEvent::Update => {
                let now = SystemTime::now();

                for entry in key_events.iter_mut() {
                    let (keycode, time) = entry;

                    // TODO const
                    if now.duration_since(*time).unwrap().as_millis() > 100 {
                        emulator.handle_event(Event::Keyrepeat(*keycode));
                        *time = now;
                    }
                }

                emulator.tick()?
            }
            WindowCallbackEvent::Redraw => emulator.handle_event(Event::Redraw),
            WindowCallbackEvent::Keydown(x) => {
                if let Some(keycode) = convert_key(x) {
                    let entry = key_events.entry(keycode);
                    if let Entry::Vacant(entry) = entry {
                        emulator.handle_event(Event::Keydown(keycode));

                        let now = SystemTime::now();

                        entry.insert(now);
                    }
                }
            }
            WindowCallbackEvent::Keyup(x) => {
                if let Some(keycode) = convert_key(x)
                    && key_events.contains_key(&keycode)
                {
                    key_events.remove(&keycode);
                    emulator.handle_event(Event::Keyup(keycode));
                }
            }
        }

        Ok(())
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
        PhysicalKey::Code(WinitKeyCode::Backspace) => Some(KeyCode::CLEAR),
        _ => None,
    }
}
