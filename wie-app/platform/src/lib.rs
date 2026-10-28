#![cfg(any(target_os = "android", target_os = "ios"))]

use tauri::{
    AppHandle, Manager, Runtime,
    plugin::{Builder, PluginHandle, TauriPlugin, mobile::PluginInvokeError},
};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_wie_platform);

struct Platform<R: Runtime>(PluginHandle<R>);

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("wie-platform")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            let handle = api.register_android_plugin("net.dlunch.wie.platform", "PlatformPlugin")?;
            #[cfg(target_os = "ios")]
            let handle = api.register_ios_plugin(init_plugin_wie_platform)?;
            app.manage(Platform(handle));
            Ok(())
        })
        .build()
}

pub async fn initialize_advertising<R: Runtime>(app: &AppHandle<R>) -> Result<(), PluginInvokeError> {
    app.state::<Platform<R>>().0.run_mobile_plugin_async("initializeAdvertising", ()).await
}
