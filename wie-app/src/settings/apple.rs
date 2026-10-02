use anyhow::{Context, Result};
use objc2::rc::autoreleasepool;
use objc2_foundation::{NSData, NSUserDefaults, ns_string};
use tauri::AppHandle;

use super::Settings;

pub(super) fn read(_app: &AppHandle) -> Result<Settings> {
    autoreleasepool(|_| {
        let defaults = NSUserDefaults::standardUserDefaults();
        let Some(value) = defaults.objectForKey(ns_string!("settings")) else {
            return Ok(Settings::default());
        };
        let data = value.downcast_ref::<NSData>().context("Settings preference is not NSData")?;
        let settings: Settings = serde_json::from_slice(&data.to_vec())?;
        settings.validate()?;
        Ok(settings)
    })
}

pub(super) fn write(_app: &AppHandle, settings: &Settings) -> Result<()> {
    settings.validate()?;
    let bytes = serde_json::to_vec(settings)?;
    autoreleasepool(|_| {
        let defaults = NSUserDefaults::standardUserDefaults();
        let data = NSData::with_bytes(&bytes);
        // NSData is an immutable property-list value, as required by UserDefaults.
        unsafe { defaults.setObject_forKey(Some(&data), ns_string!("settings")) };
    });
    Ok(())
}
