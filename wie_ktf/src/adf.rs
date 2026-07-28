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
    pub display_size: Option<(u32, u32)>,
}

impl KtfAdf {
    pub fn parse(data: &[u8]) -> Self {
        let mut aid = String::new();
        let mut pid = String::new();
        let mut mclass = String::new();
        let mut display_size = None;

        let mut lines = data.split(|x| *x == b'\n');

        for line in &mut lines {
            if line.starts_with(b"AID:") {
                aid = String::from_utf8_lossy(&line[4..]).trim().into();
            } else if line.starts_with(b"PID:") {
                pid = String::from_utf8_lossy(&line[4..]).trim().into();
            } else if line.starts_with(b"MClass:") {
                mclass = String::from_utf8_lossy(&line[7..]).trim().into();
            } else if line.starts_with(b"DisplaySize:") {
                display_size = parse_display_size(&line[12..]);
            }
            // TODO load name, it's in euc-kr..
        }

        Self {
            aid,
            pid,
            mclass,
            display_size,
        }
    }
}

fn parse_display_size(data: &[u8]) -> Option<(u32, u32)> {
    let value = core::str::from_utf8(data).ok()?.trim();
    let separator = value.find(['*', 'x', 'X'])?;
    let width: u32 = value[..separator].trim().parse().ok()?;
    let height: u32 = value[separator + 1..].trim().parse().ok()?;

    (width > 0 && height > 0).then_some((width, height))
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
        let data = b"AID:foo\nPID:bar\nMClass:baz\nDisplaySize:176*220\n";
        let adf = KtfAdf::parse(data);
        assert_eq!(adf.aid, "foo");
        assert_eq!(adf.pid, "bar");
        assert_eq!(adf.mclass, "baz");
        assert_eq!(adf.display_size, Some((176, 220)));
    }

    #[test]
    fn parse_adf_crlf() {
        let data = b"AID:foo\r\nPID:bar\r\nMClass:baz\r\nDisplaySize:176*220\r\n";
        let adf = KtfAdf::parse(data);
        assert_eq!(adf.aid, "foo");
        assert_eq!(adf.pid, "bar");
        assert_eq!(adf.mclass, "baz");
        assert_eq!(adf.display_size, Some((176, 220)));
    }

    #[test]
    fn parse_adf_empty() {
        let adf = KtfAdf::parse(b"");
        assert!(adf.aid.is_empty());
        assert!(adf.pid.is_empty());
        assert!(adf.mclass.is_empty());
        assert_eq!(adf.display_size, None);
    }

    #[test]
    fn parse_adf_partial() {
        let data = b"AID:only\n";
        let adf = KtfAdf::parse(data);
        assert_eq!(adf.aid, "only");
        assert!(adf.pid.is_empty());
        assert!(adf.mclass.is_empty());
        assert_eq!(adf.display_size, None);
    }

    #[test]
    fn parse_adf_display_size_variants() {
        assert_eq!(KtfAdf::parse(b"DisplaySize: 176 x 220\r\n").display_size, Some((176, 220)));
        assert_eq!(KtfAdf::parse(b"DisplaySize:176X220\n").display_size, Some((176, 220)));
    }

    #[test]
    fn parse_adf_invalid_display_size() {
        assert_eq!(KtfAdf::parse(b"DisplaySize:invalid\n").display_size, None);
        assert_eq!(KtfAdf::parse(b"DisplaySize:0*220\n").display_size, None);
        assert_eq!(KtfAdf::parse(b"DisplaySize:176*0\n").display_size, None);
        assert_eq!(KtfAdf::parse(b"DisplaySize:4294967296*220\n").display_size, None);
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
