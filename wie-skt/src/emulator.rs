use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use encoding_rs::EUC_KR;
use jvm::{Result as JvmResult, runtime::JavaLangString};

use wie_backend::{
    DEFAULT_PHONE_NUMBER, DefaultTaskRunner, Emulator, Event, Options, Platform, System,
    canvas::{decode_res, encode_png},
    extract_zip,
};
use wie_jvm_support::{JvmSupport, RustJavaJvmImplementation};
use wie_util::{Result, WieError};

pub struct SktEmulator {
    system: System,
}

impl SktEmulator {
    pub fn from_archive(platform: Box<dyn Platform>, files: BTreeMap<String, Vec<u8>>, options: Options) -> Result<Self> {
        let (msd_filename, msd) = SktMsd::from_archive(&files).unwrap();

        tracing::info!("Loading app {}, mclass {}", msd.id, msd.main_class);

        let jar_filename = msd_filename.replace(".msd", ".jar");

        Self::load(
            platform,
            &jar_filename,
            &msd.id,
            Some(msd.main_class),
            msd.properties,
            &files,
            options.phone_number,
        )
    }

    pub fn from_jar(platform: Box<dyn Platform>, jar_filename: &str, jar: Vec<u8>, id: &str, options: Options) -> Result<Self> {
        let (_, manifest) = extract_zip(&jar)?
            .into_iter()
            .find(|(path, _)| path.eq_ignore_ascii_case("META-INF/MANIFEST.MF"))
            .ok_or_else(|| WieError::FatalError("JAR manifest not found".into()))?;
        // Manifest values wrap onto continuation lines that start with a space.
        let manifest = String::from_utf8_lossy(&manifest).replace("\r\n ", "").replace("\n ", "");
        let msd = SktMsd::parse(jar_filename, &manifest);
        let main_class_name = (!msd.main_class.is_empty()).then_some(msd.main_class);
        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();

        Self::load(platform, jar_filename, id, main_class_name, msd.properties, &files, options.phone_number)
    }

    pub fn loadable_archive(files: &BTreeMap<String, Vec<u8>>) -> bool {
        files.iter().any(|x| x.0.ends_with(".msd"))
    }

    pub fn archive_title(files: &BTreeMap<String, Vec<u8>>) -> Option<String> {
        let (_, msd) = SktMsd::from_archive(files)?;
        (!msd.name.is_empty()).then_some(msd.name)
    }

    pub fn archive_id(files: &BTreeMap<String, Vec<u8>>) -> Option<String> {
        let (_, msd) = SktMsd::from_archive(files)?;
        (!msd.id.is_empty()).then_some(msd.id)
    }

    pub fn archive_icon(files: &BTreeMap<String, Vec<u8>>) -> Option<Vec<u8>> {
        if let Some(icon) = files.iter().find(|(filename, _)| filename.ends_with(".wmr")).and_then(|(_, wmr)| {
            if !wmr.starts_with(b"\xad\xde\xce\xfa") {
                return None;
            }

            let icon_size = u32::from_le_bytes(wmr.get(12..16)?.try_into().ok()?) as usize;
            let icon = wmr.get(16..16usize.checked_add(icon_size)?)?;
            icon.starts_with(b"BM").then(|| icon.to_vec())
        }) {
            return Some(icon);
        }

        let (_, resource) = files.iter().find(|(filename, _)| filename.ends_with(".res"))?;
        let image = decode_res(resource).ok()?;
        encode_png(&*image).ok()
    }

    pub fn loadable_jar(jar: &[u8]) -> bool {
        // A plain JAR is an SKVM app when its class constant pools reference SKVM-only packages.
        jar.starts_with(b"\x20\x00\x00\x00\x00\x00\x00\x00")
            || extract_zip(jar).is_ok_and(|files| {
                files.iter().any(|(filename, data)| {
                    filename.ends_with(".class")
                        && [b"com/skt/m/".as_slice(), b"com/xce/"]
                            .iter()
                            .any(|package| data.windows(package.len()).any(|x| x == *package))
                })
            })
    }

    fn load(
        platform: Box<dyn Platform>,
        jar_filename: &str,
        id: &str,
        main_class_name: Option<String>,
        properties: BTreeMap<String, String>,
        files: &BTreeMap<String, Vec<u8>>,
        phone_number: Option<String>,
    ) -> Result<Self> {
        let system = System::new(platform, id, id, DefaultTaskRunner);

        for (filename, data) in files {
            system.filesystem().add_virtual(filename, data.clone())
        }

        let mut system_clone = system.clone();
        let jar_filename_clone = jar_filename.to_owned();
        let phone_number = phone_number.unwrap_or_else(|| DEFAULT_PHONE_NUMBER.into());

        system.spawn(async move || Self::do_start(&mut system_clone, jar_filename_clone, properties, main_class_name, &phone_number).await);

        Ok(Self { system })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(
        system: &mut System,
        jar_filename: String,
        properties: BTreeMap<String, String>,
        main_class_name: Option<String>,
        phone_number: &str,
    ) -> Result<()> {
        let system_properties = [
            ("wie.phoneNumber", phone_number),
            ("MIN", phone_number),
            // The subscriber number without the three-digit carrier prefix; a handset may have no number.
            ("m.MIN", phone_number.get(3..).unwrap_or_default()),
            ("m.COLOR", "7"),
            ("m.VENDER", "vender"),
            ("m.MODEL", "0"), // Stub model identifier.
            ("m.CARRIER", "SKT"),
            ("m.SK_VM", "10"),
            ("com.xce.wipi.version", ""),
            // SK-VM resolves a relative Class resource name from the JAR root, whatever the receiver class.
            ("rustjava.class.resource.mode", "root"),
        ];
        let properties = properties
            .into_iter()
            .map(|(k, v)| (format!("wie.appProperty.{k}"), v))
            .collect::<Vec<_>>();
        let properties = system_properties
            .into_iter()
            .chain(properties.iter().map(|(k, v)| (k.as_ref(), v.as_ref())))
            .collect::<Vec<_>>();

        let protos = [
            wie_midp::get_protos().into(),
            wie_skvm::get_protos().into(),
            wie_wipi_java::get_protos().into(),
        ];
        let jvm = JvmSupport::new_jvm(system, Some(&jar_filename), Box::new(protos), &properties, RustJavaJvmImplementation).await?;

        let main_class_name = if let Some(x) = main_class_name {
            x.replace('.', "/")
        } else {
            return Err(WieError::FatalError("Main class not found".into()))?;
        };

        let main_class = jvm.resolve_class(&main_class_name).await.unwrap();
        let main_class_java = JavaLangString::from_rust_string(&jvm, &main_class_name).await.unwrap();

        let result: JvmResult<()> = if jvm.is_inherited_from(&*main_class.definition, "javax/microedition/midlet/MIDlet") {
            jvm.invoke_static("net/wie/Launcher", "start", "(Ljava/lang/String;)V", (main_class_java,))
                .await
        } else {
            let mut args = jvm.instantiate_array("Ljava/lang/String;", 1).await.unwrap();
            jvm.store_array(&mut args, 0, vec![main_class_java]).await.unwrap();
            jvm.invoke_static("org/kwis/msp/lcdui/Main", "main", "([Ljava/lang/String;)V", (args,))
                .await
        };

        if let Err(x) = result {
            return Err(JvmSupport::to_wie_err(&jvm, x).await);
        }

        Ok(())
    }
}

impl Emulator for SktEmulator {
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

struct SktMsd {
    name: String,
    id: String,
    main_class: String,
    properties: BTreeMap<String, String>,
}

impl SktMsd {
    fn from_archive(files: &BTreeMap<String, Vec<u8>>) -> Option<(&str, Self)> {
        let (filename, data) = files.iter().find(|(filename, _)| filename.ends_with(".msd"))?;
        let (text, _) = EUC_KR.decode_without_bom_handling(data);
        Some((filename, Self::parse(filename, &text)))
    }

    fn parse(filename: &str, text: &str) -> Self {
        let mut name = String::new();
        let mut main_class = String::new();
        let mut id = filename[..filename.find('.').unwrap()].into();
        let mut properties = BTreeMap::new();

        for line in text.lines() {
            let Some((key, value)) = line.split_once(':') else { continue };
            let key = key.trim();
            let value = value.trim();

            match key {
                "MIDlet-Name" => name = value.to_string(),
                "MIDlet-1" => {
                    if let Some(class) = value.split(',').nth(2) {
                        main_class = class.trim().to_string();
                    }
                }
                "DD-ProgName" => id = value.to_string(),
                _ => {}
            }

            tracing::info!("Adding property {key}={value}");
            properties.insert(key.to_string(), value.to_string());
        }

        Self {
            name,
            id,
            main_class,
            properties,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use alloc::{collections::BTreeMap, vec, vec::Vec};
    use std::io::{Cursor, Write};

    use zip::{ZipWriter, write::SimpleFileOptions};

    use super::{SktEmulator, SktMsd};

    #[test]
    fn detects_plain_jar_referencing_skvm_classes() {
        for (class, expected) in [
            (b"\x01\x00\x13com/skt/m/AudioClip".as_slice(), true),
            (b"\x01\x00\x10java/lang/Object", false),
        ] {
            let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
            writer.start_file("Main.class", SimpleFileOptions::default()).unwrap();
            writer.write_all(class).unwrap();
            let jar = writer.finish().unwrap().into_inner();

            assert_eq!(SktEmulator::loadable_jar(&jar), expected);
        }
    }

    #[test]
    fn parse_msd_name() {
        let files = BTreeMap::from([(
            "0051758461.msd".into(),
            b"MIDlet-Name: \xbf\xb5\xbf\xf5\xbc\xad\xb1\xe2_\xbc\xd6\xc6\xbc\xbe\xc6\xc0\xc7\xb9\xd9\xb6\xf7\r\nMIDlet-1: title,,rpg.GameMIDlet\r\nDD-ProgName: 0051758461\r\n".to_vec(),
        )]);
        let (_, msd) = SktMsd::from_archive(&files).unwrap();

        assert_eq!(msd.name, "영웅서기_솔티아의바람");
        assert_eq!(msd.id, "0051758461");
        assert_eq!(msd.main_class, "rpg.GameMIDlet");
        assert_eq!(msd.properties["MIDlet-Name"], msd.name);
    }

    #[test]
    fn extracts_static_icon_from_wmr() {
        let mut wmr = vec![0; 16];
        wmr[..4].copy_from_slice(b"\xad\xde\xce\xfa");
        wmr[12..16].copy_from_slice(&6u32.to_le_bytes());
        wmr.extend_from_slice(b"BMicon");
        let files = BTreeMap::from([("sample.wmr".into(), wmr)]);

        assert_eq!(SktEmulator::archive_icon(&files), Some(b"BMicon".to_vec()));
    }

    #[test]
    fn extracts_static_icon_from_res_after_animation() {
        let mut resource = vec![0; 15];
        resource[..4].copy_from_slice(b"\xce\xfa\xad\xde");
        resource[6..8].copy_from_slice(&15u16.to_le_bytes());
        resource.extend_from_slice(&[3, 1, 1, 8, 0, 0, 0, 0, 2, 0, 0]);
        resource.extend_from_slice(&[2, 2, 1, 8, 0, 0, 0, 0, 0xe0, 0x03]);
        let files = BTreeMap::from([("sample.res".into(), resource)]);

        let png = SktEmulator::archive_icon(&files).unwrap();
        let image = wie_backend::canvas::decode_image(&png).unwrap();

        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 1);
        let red = image.get_pixel(0, 0);
        let blue = image.get_pixel(1, 0);
        assert_eq!((red.r, red.g, red.b, red.a), (252, 0, 0, 255));
        assert_eq!((blue.r, blue.g, blue.b, blue.a), (0, 0, 255, 255));
    }
}
