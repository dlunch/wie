use anyhow::Result;
use tauri::AppHandle;
use windows::{
    Win32::{
        Foundation::ERROR_FILE_NOT_FOUND,
        System::Registry::{HKEY_CURRENT_USER, REG_BINARY, RRF_RT_REG_BINARY, RegGetValueW, RegSetKeyValueW},
    },
    core::w,
};

use super::Settings;

pub(super) fn read(_app: &AppHandle) -> Result<Settings> {
    let mut size = 0;
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\net.dlunch.wie"),
            w!("Settings"),
            RRF_RT_REG_BINARY,
            None,
            None,
            Some(&mut size),
        )
    };
    if result == ERROR_FILE_NOT_FOUND {
        return Ok(Settings::default());
    }
    result.ok()?;
    let mut bytes = vec![0u8; size as usize];
    unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\net.dlunch.wie"),
            w!("Settings"),
            RRF_RT_REG_BINARY,
            None,
            Some(bytes.as_mut_ptr().cast()),
            Some(&mut size),
        )
        .ok()?;
    }
    let settings: Settings = serde_json::from_slice(&bytes[..size as usize])?;
    settings.validate()?;
    Ok(settings)
}

pub(super) fn write(_app: &AppHandle, settings: &Settings) -> Result<()> {
    settings.validate()?;
    let bytes = serde_json::to_vec(settings)?;
    unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            w!("Software\\net.dlunch.wie"),
            w!("Settings"),
            REG_BINARY.0,
            Some(bytes.as_ptr().cast()),
            bytes.len().try_into()?,
        )
        .ok()?;
    }
    Ok(())
}
