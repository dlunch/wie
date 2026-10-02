use std::collections::BTreeMap;

use wie_backend::{Emulator, Options, Platform, extract_zip};
use wie_j2me::J2MEEmulator;
use wie_ktf::KtfEmulator;
use wie_lgt::LgtEmulator;
use wie_skt::SktEmulator;

pub struct AppMetadata {
    pub id: String,
    pub title: String,
    pub icon: Option<Vec<u8>>,
}

enum ArchivePlatform {
    Ktf,
    Lgt,
    Skt,
}

fn parse_archive(bytes: &[u8]) -> anyhow::Result<(ArchivePlatform, BTreeMap<String, Vec<u8>>)> {
    let files = extract_zip(bytes)?;
    if !files.keys().any(|name| name.to_ascii_lowercase().ends_with(".jar")) {
        anyhow::bail!("Archive does not contain a JAR file");
    }
    let platform = if KtfEmulator::loadable_archive(&files) {
        ArchivePlatform::Ktf
    } else if LgtEmulator::loadable_archive(&files) {
        ArchivePlatform::Lgt
    } else if SktEmulator::loadable_archive(&files) {
        ArchivePlatform::Skt
    } else {
        anyhow::bail!("Unknown archive format");
    };
    Ok((platform, files))
}

pub fn extract_app_metadata(filename: &str, bytes: &[u8]) -> anyhow::Result<AppMetadata> {
    let filename = filename.rsplit(['/', '\\']).next().unwrap();
    let lowercase_filename = filename.to_ascii_lowercase();
    let metadata = if lowercase_filename.ends_with(".zip") {
        let (platform, files) = parse_archive(bytes)?;
        match platform {
            ArchivePlatform::Ktf => KtfEmulator::archive_id(&files)
                .zip(KtfEmulator::archive_title(&files))
                .map(|(id, title)| (id, title, KtfEmulator::archive_icon(&files))),
            ArchivePlatform::Lgt => LgtEmulator::archive_id(&files)
                .zip(LgtEmulator::archive_title(&files))
                .map(|(id, title)| (id, title, LgtEmulator::archive_icon(&files))),
            ArchivePlatform::Skt => SktEmulator::archive_id(&files)
                .zip(SktEmulator::archive_title(&files))
                .map(|(id, title)| (id, title, SktEmulator::archive_icon(&files))),
        }
    } else if lowercase_filename.ends_with(".jar") {
        let id = if KtfEmulator::loadable_jar(bytes) || LgtEmulator::loadable_jar(bytes) || SktEmulator::loadable_jar(bytes) {
            &filename[..filename.len() - 4]
        } else {
            filename
        };
        J2MEEmulator::jar_metadata(bytes)?.map(|(title, icon)| (id.to_owned(), title, icon))
    } else {
        anyhow::bail!("Unknown file format");
    };
    let (id, title, icon) = metadata.ok_or_else(|| anyhow::anyhow!("App metadata does not contain an ID, title or entry point"))?;
    Ok(AppMetadata { id, title, icon })
}

pub fn load_emulator(filename: &str, bytes: Vec<u8>, platform: Box<dyn Platform>, options: Options) -> anyhow::Result<Box<dyn Emulator>> {
    let filename = filename.rsplit(['/', '\\']).next().unwrap();
    let lowercase_filename = filename.to_ascii_lowercase();
    if lowercase_filename.ends_with(".zip") {
        let (archive_platform, files) = parse_archive(&bytes)?;
        Ok(match archive_platform {
            ArchivePlatform::Ktf => Box::new(KtfEmulator::from_archive(platform, files, options)?),
            ArchivePlatform::Lgt => Box::new(LgtEmulator::from_archive(platform, files, options)?),
            ArchivePlatform::Skt => Box::new(SktEmulator::from_archive(platform, files)?),
        })
    } else if lowercase_filename.ends_with(".jar") {
        let id = &filename[..filename.len() - 4];
        Ok(if KtfEmulator::loadable_jar(&bytes) {
            Box::new(KtfEmulator::from_jar(platform, filename, bytes, id, id, None, options)?)
        } else if LgtEmulator::loadable_jar(&bytes) {
            Box::new(LgtEmulator::from_jar(platform, filename, bytes, id, id, None, options)?)
        } else if SktEmulator::loadable_jar(&bytes) {
            Box::new(SktEmulator::from_jar(platform, filename, bytes, id, None)?)
        } else {
            Box::new(J2MEEmulator::from_jar(platform, filename, bytes)?)
        })
    } else {
        anyhow::bail!("Unknown file format");
    }
}
