use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::btree_map::BTreeMap,
    format, str,
    string::{String, ToString},
    vec::Vec,
};

use jvm::{Result as JvmResult, runtime::JavaLangString};

use wie_backend::{DefaultTaskRunner, Emulator, Event, Platform, System, extract_zip};
use wie_jvm_support::{JvmSupport, RustJavaJvmImplementation};
use wie_util::{Result, WieError};

pub struct J2MEEmulator {
    system: System,
}

impl J2MEEmulator {
    pub fn jar_metadata(jar: &[u8]) -> Result<Option<(String, Option<Vec<u8>>)>> {
        let files = extract_zip(jar)?;
        let Some(manifest) = files.get("META-INF/MANIFEST.MF") else {
            return Ok(None);
        };
        let descriptor = J2MEDescriptor::parse(manifest);
        if descriptor.name.is_empty() || descriptor.main_class_name.is_empty() {
            return Ok(None);
        }

        let icon = if descriptor.icon.is_empty() {
            None
        } else {
            files.get(descriptor.icon.trim_start_matches('/')).cloned()
        };

        Ok(Some((descriptor.name, icon)))
    }

    pub fn from_jad_jar(platform: Box<dyn Platform>, jad: Vec<u8>, jar_filename: String, jar: Vec<u8>) -> Result<Self> {
        let descriptor = J2MEDescriptor::parse(&jad);

        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();
        Self::load(platform, &jar_filename, descriptor, &files)
    }

    pub fn from_jar(platform: Box<dyn Platform>, jar_filename: &str, jar: Vec<u8>) -> Result<Self> {
        let contents = extract_zip(&jar)?;
        let manifest = contents
            .get("META-INF/MANIFEST.MF")
            .ok_or_else(|| WieError::FatalError("JAR manifest not found".into()))?;
        let descriptor = J2MEDescriptor::parse(manifest);
        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();

        Self::load(platform, jar_filename, descriptor, &files)
    }

    fn load(platform: Box<dyn Platform>, jar_filename: &str, descriptor: J2MEDescriptor, files: &BTreeMap<String, Vec<u8>>) -> Result<Self> {
        if descriptor.name.is_empty() || descriptor.main_class_name.is_empty() {
            return Err(WieError::FatalError("MIDlet-1 name or main class not found".into()));
        }
        let system = System::new(platform, &descriptor.name, &descriptor.name, DefaultTaskRunner);

        for (path, data) in files {
            system.filesystem().add_virtual(path, data.clone());
        }

        let mut system_clone = system.clone();
        let jar_filename = jar_filename.to_owned();

        system.spawn(async move || Self::do_start(&mut system_clone, jar_filename, descriptor.properties, descriptor.main_class_name).await);

        Ok(J2MEEmulator { system })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(system: &mut System, jar_filename: String, properties: BTreeMap<String, String>, main_class_name: String) -> Result<()> {
        let properties = properties
            .into_iter()
            .map(|(k, v)| (format!("wie.appProperty.{k}"), v))
            .collect::<Vec<_>>();
        let properties = properties.iter().map(|(k, v)| (k.as_ref(), v.as_ref())).collect::<Vec<_>>();

        let protos = [wie_midp::get_protos().into()];
        let jvm = JvmSupport::new_jvm(system, Some(&jar_filename), Box::new(protos), &properties, RustJavaJvmImplementation).await?;

        let main_class_name = main_class_name.replace('.', "/");

        // Resolve with the system loader before entering the rustjar-loaded Launcher.
        if let Err(error) = jvm.resolve_class(&main_class_name).await {
            return Err(JvmSupport::to_wie_err(&jvm, error).await);
        }
        let main_class_java = JavaLangString::from_rust_string(&jvm, &main_class_name).await.unwrap();

        let result: JvmResult<()> = jvm
            .invoke_static("net/wie/Launcher", "start", "(Ljava/lang/String;)V", (main_class_java,))
            .await;

        if let Err(x) = result {
            return Err(JvmSupport::to_wie_err(&jvm, x).await);
        }

        Ok(())
    }
}

impl Emulator for J2MEEmulator {
    fn is_preparing(&self) -> bool {
        false
    }

    fn handle_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> Result<()> {
        self.system.tick()
    }
}

struct J2MEDescriptor {
    name: String,
    main_class_name: String,
    icon: String,
    properties: BTreeMap<String, String>,
}

impl J2MEDescriptor {
    pub fn parse(data: &[u8]) -> Self {
        let lines = data.split(|x| *x == b'\n');

        let mut properties = BTreeMap::new();
        let mut logical_lines: Vec<String> = Vec::new();

        for line in lines {
            let Ok(line) = str::from_utf8(line) else {
                continue;
            };
            let line = line.trim_end_matches('\r');

            if let Some(continuation) = line.strip_prefix(' ') {
                if let Some(previous) = logical_lines.last_mut() {
                    previous.push_str(continuation);
                }
            } else {
                logical_lines.push(line.to_string());
            }
        }

        for line in logical_lines {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            properties.insert(key.to_string(), value.to_string());
        }

        Self::from_properties(properties)
    }

    fn from_properties(properties: BTreeMap<String, String>) -> Self {
        let mut name = String::new();
        let mut main_class_name = String::new();
        let mut icon = String::new();
        let mut midlet_icon = String::new();

        for (key, value) in &properties {
            match key.as_str() {
                "MIDlet-Icon" => icon = value.to_string(),
                "MIDlet-1" => {
                    let mut values = value.split(',');
                    if let Some(value) = values.next() {
                        name = value.trim().to_string();
                    }
                    if let Some(value) = values.next() {
                        midlet_icon = value.trim().to_string();
                    }
                    if let Some(value) = values.next() {
                        main_class_name = value.trim().to_string();
                    }
                }
                _ => {}
            }
        }

        if icon.is_empty() {
            icon = midlet_icon;
        }

        Self {
            name,
            main_class_name,
            icon,
            properties,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use alloc::{boxed::Box, string::ToString, vec::Vec};
    use std::io::{Cursor, Write};

    use test_utils::TestPlatform;
    use zip::{ZipWriter, write::SimpleFileOptions};

    use super::{J2MEDescriptor, J2MEEmulator};

    #[test]
    fn jar_and_jad_use_midlet_name_for_storage() {
        let manifest = b"MIDlet-Name: My MIDlet Suite\nMIDlet-1: App title, , example.Main\n";
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer.start_file("META-INF/MANIFEST.MF", SimpleFileOptions::default()).unwrap();
        writer.write_all(manifest).unwrap();
        let jar = writer.finish().unwrap().into_inner();

        for filename in ["original.jar", "renamed.jar"] {
            let emulator = J2MEEmulator::from_jar(Box::new(TestPlatform::new()), filename, jar.clone()).unwrap();
            assert_eq!(emulator.system.pid(), "App title");
            assert_eq!(emulator.system.aid(), "App title");
            let emulator = J2MEEmulator::from_jad_jar(Box::new(TestPlatform::new()), manifest.to_vec(), filename.to_string(), jar.clone()).unwrap();
            assert_eq!(emulator.system.pid(), "App title");
            assert_eq!(emulator.system.aid(), "App title");
        }
    }

    #[test]
    fn parses_utf8_midlet_metadata() {
        let descriptor = J2MEDescriptor::parse(
            "Manifest-Version: 1.0\r\nMIDlet-Name: 모바일 앱\r\nMIDlet-Icon: /app.png\r\nMIDlet-1: 모바일 앱, /midlet.png, example.Main\r\n"
                .as_bytes(),
        );

        assert_eq!(descriptor.name, "모바일 앱");
        assert_eq!(descriptor.main_class_name, "example.Main");
        assert_eq!(descriptor.icon, "/app.png");
    }

    #[test]
    fn ignores_malformed_manifest_lines() {
        let descriptor = J2MEDescriptor::parse(b"invalid line\nMIDlet-1: incomplete\n\xff\nMIDlet-Name: Valid App\n");

        assert_eq!(descriptor.name, "incomplete");
        assert!(descriptor.main_class_name.is_empty());
    }

    #[test]
    fn parses_midlet_icon_and_continued_manifest_value() {
        let descriptor =
            J2MEDescriptor::parse(b"MIDlet-Name: Boulder Dash\nMIDlet-1: Boulder Dash, icon.png, net.instantcom.boulderdash.BoulderDa\n shMIDlet\n");

        assert_eq!(descriptor.main_class_name, "net.instantcom.boulderdash.BoulderDashMIDlet");
        assert_eq!(descriptor.icon, "icon.png");
    }
}
