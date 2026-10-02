use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
#[cfg(target_os = "linux")]
use tauri::Manager;

#[cfg_attr(target_os = "android", path = "settings/android.rs")]
#[cfg_attr(target_vendor = "apple", path = "settings/apple.rs")]
#[cfg_attr(target_os = "linux", path = "settings/linux.rs")]
#[cfg_attr(target_os = "windows", path = "settings/windows.rs")]
mod platform;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub midi_volume: f32,
    pub pcm_volume: f32,
    pub help_dismissed: bool,
    pub welcome_seen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            midi_volume: 0.5,
            pcm_volume: 0.5,
            help_dismissed: false,
            welcome_seen: false,
        }
    }
}

impl Settings {
    fn validate(&self) -> Result<()> {
        ensure!(
            (0.0..=1.0).contains(&self.midi_volume) && (0.0..=1.0).contains(&self.pcm_volume),
            "Volume must be between 0 and 1"
        );
        Ok(())
    }
}

pub struct SettingsStore {
    app: AppHandle,
}

impl SettingsStore {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub async fn read(&self) -> Result<Settings> {
        #[cfg(target_os = "android")]
        {
            platform::read(&self.app).await
        }
        #[cfg(not(target_os = "android"))]
        {
            let context = self.app.clone();
            tauri::async_runtime::spawn_blocking(move || {
                #[cfg(target_os = "linux")]
                let context = context.path().app_config_dir()?;
                platform::read(&context)
            })
            .await?
        }
    }

    pub async fn write(&self, settings: Settings) -> Result<()> {
        #[cfg(target_os = "android")]
        {
            platform::write(&self.app, settings).await
        }
        #[cfg(not(target_os = "android"))]
        {
            let context = self.app.clone();
            tauri::async_runtime::spawn_blocking(move || {
                #[cfg(target_os = "linux")]
                let context = context.path().app_config_dir()?;
                platform::write(&context, &settings)
            })
            .await?
        }
    }
}
