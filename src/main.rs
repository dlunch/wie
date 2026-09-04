extern crate alloc;

mod audio_sink;
mod database;
mod filesystem;
mod window;

use core::str;
use std::{
    collections::{HashMap, hash_map::Entry},
    fs::{self, File},
    io::{LineWriter, Write, stderr},
    path::PathBuf,
    sync::{Mutex, mpsc::Sender, mpsc::channel},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use midir::MidiOutput;
use winit::keyboard::{KeyCode as WinitKeyCode, PhysicalKey};

use wie_backend::{AudioCommand, Emulator, Event, Filesystem, Font, Instant, KeyCode, Options, Platform, ProfileSample, Screen, extract_zip};
use wie_j2me::J2MEEmulator;
use wie_ktf::KtfEmulator;
use wie_lgt::LgtEmulator;
use wie_skt::SktEmulator;

use self::{
    audio_sink::AudioSink,
    database::DatabaseRepository,
    filesystem::CliFilesystem,
    window::{WindowCallbackEvent, WindowHandle, WindowImpl},
};

struct WieCliPlatform {
    audio_tx: Sender<AudioCommand>,
    database_repository: DatabaseRepository,
    filesystem: CliFilesystem,
    font: Font,
    window: WindowHandle,
}

impl WieCliPlatform {
    fn new(window: WindowHandle, font: Font, midi_device: Option<usize>) -> Self {
        let (tx, rx) = channel();
        thread::spawn(move || audio_sink::run(rx, midi_device));

        Self {
            audio_tx: tx,
            database_repository: DatabaseRepository::new(),
            filesystem: CliFilesystem::new(),
            font,
            window,
        }
    }
}

impl Platform for WieCliPlatform {
    fn font(&self) -> &Font {
        &self.font
    }

    fn screen(&self) -> &dyn Screen {
        &self.window
    }

    fn now(&self) -> Instant {
        let now = SystemTime::now();
        let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();

        Instant::from_epoch_millis(since_the_epoch.as_millis() as _)
    }

    fn database_repository(&self) -> &dyn wie_backend::DatabaseRepository {
        &self.database_repository
    }

    fn filesystem(&self) -> &dyn Filesystem {
        &self.filesystem
    }

    fn audio_sink(&self) -> Box<dyn wie_backend::AudioSink> {
        Box::new(AudioSink::new(self.audio_tx.clone()))
    }

    fn write_stdout(&self, buf: &[u8]) {
        let str = str::from_utf8(buf).unwrap();

        print!("{str}")
    }

    fn write_stderr(&self, buf: &[u8]) {
        let str = str::from_utf8(buf).unwrap();

        eprint!("{str}")
    }

    fn exit(&self) {
        self.window.send_quit_event();
    }

    fn vibrate(&self, duration_ms: u64, intensity: u8) {
        tracing::info!("vibrate({duration_ms}ms, {intensity}%) - not supported on this platform");
    }
}

#[derive(Parser)]
struct Args {
    #[arg(required_unless_present = "list_midi_devices")]
    filename: Option<String>,
    #[arg(long, default_value_t = false)]
    debug: bool,
    /// Write a flamegraph-folded sampling profile to this path (one line per
    /// flushed batch; `flamegraph.pl` aggregates duplicates).
    #[arg(long)]
    profile_out: Option<PathBuf>,
    /// Select a MIDI output by zero-based index.
    #[arg(long, value_name = "INDEX")]
    midi_device: Option<usize>,
    /// List available MIDI output devices and exit.
    #[arg(long, conflicts_with_all = ["filename", "debug", "profile_out", "midi_device"])]
    list_midi_devices: bool,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    if args.list_midi_devices {
        return list_midi_devices();
    }

    let profile = args.profile_out.as_ref().map(|path| profile_callback(path)).transpose()?;
    let options = Options {
        enable_gdbserver: args.debug,
        profile,
    };
    let filename = args.filename.as_deref().ok_or_else(|| anyhow::anyhow!("filename is required"))?;

    start_with_midi_device(filename, options, args.midi_device)
}

fn list_midi_devices() -> anyhow::Result<()> {
    let midi_out = MidiOutput::new("wie")?;
    for (index, port) in midi_out.ports().iter().enumerate() {
        let name = midi_out.port_name(port).unwrap_or_else(|_| "<unknown>".to_string());
        println!("{index}: {name}");
    }
    Ok(())
}

fn profile_callback(path: &PathBuf) -> anyhow::Result<wie_backend::ProfileCallback> {
    let writer = Mutex::new(LineWriter::new(File::create(path)?));
    Ok(Box::new(move |batch: Vec<ProfileSample>| {
        let mut writer = writer.lock().unwrap();
        for sample in batch {
            let folded: Vec<String> = sample.stack.iter().rev().map(|pc| format!("0x{pc:x}")).collect();
            let _ = writeln!(writer, "{} {}", folded.join(";"), sample.count);
        }
    }))
}

pub fn start(filename: &str, options: Options) -> anyhow::Result<()> {
    start_with_midi_device(filename, options, None)
}

fn start_with_midi_device(filename: &str, options: Options, midi_device: Option<usize>) -> anyhow::Result<()> {
    let window = WindowImpl::new(240, 320)?;
    let font = Font::try_from_static(include_bytes!("../assets/neodgm.ttf"))?;
    let platform = Box::new(WieCliPlatform::new(window.handle(), font, midi_device));

    let buf = fs::read(filename)?;
    let mut emulator: Box<dyn Emulator> = if filename.ends_with("zip") {
        let files = extract_zip(&buf)?;

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

fn select_midi_output_index(port_count: usize, requested: Option<usize>) -> Option<usize> {
    requested.filter(|&index| index < port_count).or_else(|| port_count.checked_sub(1))
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
        PhysicalKey::Code(WinitKeyCode::ShiftLeft) => Some(KeyCode::LEFT_SOFT_KEY),
        PhysicalKey::Code(WinitKeyCode::ShiftRight) => Some(KeyCode::RIGHT_SOFT_KEY),
        PhysicalKey::Code(WinitKeyCode::Backquote) => Some(KeyCode::VOLUME_UP),
        PhysicalKey::Code(WinitKeyCode::Tab) => Some(KeyCode::VOLUME_DOWN),
        PhysicalKey::Code(WinitKeyCode::F1) => Some(KeyCode::CALL),
        PhysicalKey::Code(WinitKeyCode::F2) => Some(KeyCode::HANGUP),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Args, select_midi_output_index};

    #[test]
    fn defaults_to_last_midi_port() {
        assert_eq!(select_midi_output_index(3, None), Some(2));
    }

    #[test]
    fn selects_midi_port_by_index() {
        for index in 0..3 {
            assert_eq!(select_midi_output_index(3, Some(index)), Some(index));
        }
    }

    #[test]
    fn invalid_midi_device_falls_back_to_last_port() {
        assert_eq!(select_midi_output_index(2, Some(2)), Some(1));
    }

    #[test]
    fn handles_empty_midi_port_list() {
        assert_eq!(select_midi_output_index(0, Some(0)), None);
    }

    #[test]
    fn list_midi_devices_does_not_require_filename() {
        let args = Args::try_parse_from(["wie", "--list-midi-devices"]).unwrap();

        assert!(args.list_midi_devices);
        assert!(args.filename.is_none());
    }

    #[test]
    fn normal_run_requires_filename() {
        assert!(Args::try_parse_from(["wie"]).is_err());
    }

    #[test]
    fn parses_midi_device_with_filename() {
        let args = Args::try_parse_from(["wie", "game.jar", "--midi-device", "1"]).unwrap();

        assert_eq!(args.filename.as_deref(), Some("game.jar"));
        assert_eq!(args.midi_device, Some(1));
    }
}
