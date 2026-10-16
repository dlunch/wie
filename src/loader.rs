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

pub enum FileType {
    Ktf,
    Lgt,
    Skt,
    J2me,
}

pub fn determine_file_type(filename: &str, bytes: &[u8]) -> anyhow::Result<FileType> {
    let filename = filename.to_ascii_lowercase();
    if filename.ends_with(".zip") {
        let files = extract_zip(bytes)?;
        if !files.keys().any(|name| name.to_ascii_lowercase().ends_with(".jar")) {
            anyhow::bail!("Archive does not contain a JAR file");
        }
        if KtfEmulator::loadable_archive(&files) {
            KtfEmulator::validate_archive(&files)?;
            Ok(FileType::Ktf)
        } else if LgtEmulator::loadable_archive(&files) {
            Ok(FileType::Lgt)
        } else if SktEmulator::loadable_archive(&files) {
            Ok(FileType::Skt)
        } else {
            anyhow::bail!("Unknown archive format");
        }
    } else if filename.ends_with(".jar") {
        if KtfEmulator::loadable_jar(bytes) {
            KtfEmulator::validate_jar(bytes)?;
            Ok(FileType::Ktf)
        } else if LgtEmulator::loadable_jar(bytes) {
            Ok(FileType::Lgt)
        } else if SktEmulator::loadable_jar(bytes) {
            Ok(FileType::Skt)
        } else {
            extract_zip(bytes)?;
            Ok(FileType::J2me)
        }
    } else {
        anyhow::bail!("Unknown file format");
    }
}

pub fn extract_app_metadata(filename: &str, bytes: &[u8], file_type: FileType) -> anyhow::Result<AppMetadata> {
    let filename = filename.rsplit(['/', '\\']).next().unwrap();
    let lowercase_filename = filename.to_ascii_lowercase();
    let metadata = match file_type {
        FileType::Ktf if lowercase_filename.ends_with(".zip") => {
            let files = extract_zip(bytes)?;
            KtfEmulator::archive_id(&files)
                .zip(KtfEmulator::archive_title(&files))
                .map(|(id, title)| (id, title, KtfEmulator::archive_icon(&files)))
        }
        FileType::Lgt if lowercase_filename.ends_with(".zip") => {
            let files = extract_zip(bytes)?;
            LgtEmulator::archive_id(&files)
                .zip(LgtEmulator::archive_title(&files))
                .map(|(id, title)| (id, title, LgtEmulator::archive_icon(&files)))
        }
        FileType::Skt if lowercase_filename.ends_with(".zip") => {
            let files = extract_zip(bytes)?;
            SktEmulator::archive_id(&files)
                .zip(SktEmulator::archive_title(&files))
                .map(|(id, title)| (id, title, SktEmulator::archive_icon(&files)))
        }
        file_type => {
            let id = if matches!(file_type, FileType::J2me) {
                filename
            } else {
                &filename[..filename.len() - 4]
            };
            J2MEEmulator::jar_metadata(bytes)?.map(|(title, icon)| (id.to_owned(), title, icon))
        }
    };
    let (id, title, icon) = metadata.ok_or_else(|| anyhow::anyhow!("App metadata does not contain an ID, title or entry point"))?;
    Ok(AppMetadata { id, title, icon })
}

pub fn load_emulator(filename: &str, bytes: Vec<u8>, platform: Box<dyn Platform>, options: Options) -> anyhow::Result<Box<dyn Emulator>> {
    let filename = filename.rsplit(['/', '\\']).next().unwrap();
    let lowercase_filename = filename.to_ascii_lowercase();
    let file_type = determine_file_type(filename, &bytes)?;
    Ok(match file_type {
        FileType::Ktf if lowercase_filename.ends_with(".zip") => Box::new(KtfEmulator::from_archive(platform, extract_zip(&bytes)?, options)?),
        FileType::Lgt if lowercase_filename.ends_with(".zip") => Box::new(LgtEmulator::from_archive(platform, extract_zip(&bytes)?, options)?),
        FileType::Skt if lowercase_filename.ends_with(".zip") => Box::new(SktEmulator::from_archive(platform, extract_zip(&bytes)?)?),
        FileType::Ktf => {
            let id = &filename[..filename.len() - 4];
            Box::new(KtfEmulator::from_jar(platform, filename, bytes, id, id, None, options)?)
        }
        FileType::Lgt => {
            let id = &filename[..filename.len() - 4];
            Box::new(LgtEmulator::from_jar(platform, filename, bytes, id, id, None, options)?)
        }
        FileType::Skt => {
            let id = &filename[..filename.len() - 4];
            Box::new(SktEmulator::from_jar(platform, filename, bytes, id, None)?)
        }
        FileType::J2me => Box::new(J2MEEmulator::from_jar(platform, filename, bytes)?),
    })
}
