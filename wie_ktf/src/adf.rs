use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};

use wie_backend::extract_zip;
use wie_util::{Result, WieError};

pub struct KtfAdf {
    pub aid: String,
    pub pid: String,
    pub mclass: String,
}

impl KtfAdf {
    pub fn parse(data: &[u8]) -> Self {
        let mut aid = String::new();
        let mut pid = String::new();
        let mut mclass = String::new();

        let mut lines = data.split(|x| *x == b'\n');

        for line in &mut lines {
            if line.starts_with(b"AID:") {
                aid = String::from_utf8_lossy(&line[4..]).into();
            } else if line.starts_with(b"PID:") {
                pid = String::from_utf8_lossy(&line[4..]).into();
            } else if line.starts_with(b"MClass:") {
                mclass = String::from_utf8_lossy(&line[7..]).into();
            }
            // TODO load name, it's in euc-kr..
        }

        Self { aid, pid, mclass }
    }
}

pub fn find_client_bin(jar: &[u8]) -> Result<(String, Vec<u8>)> {
    let files: BTreeMap<String, Vec<u8>> = extract_zip(jar)?;

    files
        .into_iter()
        .find(|(name, _)| name.starts_with("client.bin"))
        .ok_or_else(|| WieError::FatalError("client.bin* not found in jar".to_string()))
}

pub fn parse_bss_size(filename: &str) -> Result<u32> {
    filename
        .strip_prefix("client.bin")
        .ok_or_else(|| WieError::FatalError(format!("Filename does not start with 'client.bin': {filename}")))?
        .parse::<u32>()
        .map_err(|e| WieError::FatalError(format!("Invalid bss_size in filename {filename}: {e}")))
}

#[cfg(test)]
mod tests {
    use super::{KtfAdf, parse_bss_size};

    #[test]
    fn parse_adf_full() {
        let data = b"AID:foo\nPID:bar\nMClass:baz\n";
        let adf = KtfAdf::parse(data);
        assert_eq!(adf.aid, "foo");
        assert_eq!(adf.pid, "bar");
        assert_eq!(adf.mclass, "baz");
    }

    #[test]
    fn parse_adf_empty() {
        let adf = KtfAdf::parse(b"");
        assert!(adf.aid.is_empty());
        assert!(adf.pid.is_empty());
        assert!(adf.mclass.is_empty());
    }

    #[test]
    fn parse_adf_partial() {
        let data = b"AID:only\n";
        let adf = KtfAdf::parse(data);
        assert_eq!(adf.aid, "only");
        assert!(adf.pid.is_empty());
        assert!(adf.mclass.is_empty());
    }

    #[test]
    fn parse_bss_size_ok() {
        assert_eq!(parse_bss_size("client.bin12345").unwrap(), 12345);
        assert_eq!(parse_bss_size("client.bin0").unwrap(), 0);
    }

    #[test]
    fn parse_bss_size_missing_marker() {
        assert!(parse_bss_size("not_a_client_bin_name").is_err());
    }

    #[test]
    fn parse_bss_size_no_digits() {
        assert!(parse_bss_size("client.bin").is_err());
    }

    #[test]
    fn parse_bss_size_non_numeric() {
        assert!(parse_bss_size("client.binABC").is_err());
    }
}
