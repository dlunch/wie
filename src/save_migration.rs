// Temporary save migration identity parsing added 2026-10-19.
use std::path::Path;

use anyhow::Result;
use wie_backend::extract_zip;

pub fn save_migration_identity(filename: &str, bytes: &[u8], pid: &str) -> Result<(String, bool)> {
    let mut aid = pid.to_owned();
    let mut clet = false;
    if Path::new(filename)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
    {
        let files = extract_zip(bytes)?;
        let descriptor = files
            .get("__adf__")
            .map(|data| (data, true))
            .or_else(|| files.get("app_info").map(|data| (data, false)));
        if let Some((descriptor, trim)) = descriptor {
            aid.clear();
            for line in descriptor.split(|byte| *byte == b'\n') {
                if let Some(value) = line.strip_prefix(b"AID:") {
                    let value = String::from_utf8_lossy(value);
                    aid = if trim { value.trim().to_owned() } else { value.into_owned() };
                } else if let Some(value) = line.strip_prefix(b"MClass:") {
                    let value = String::from_utf8_lossy(value);
                    clet = if trim { value.trim() } else { &value } == "Clet";
                }
            }
        }
    }
    Ok((aid, clet))
}
