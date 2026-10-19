#[cfg(not(target_os = "ios"))]
mod aot_cache;
mod audio;
mod database;
mod filesystem;
mod library;
mod runtime;
mod save_migration;
#[cfg(not(target_os = "ios"))]
mod screen;
mod settings;
mod store;

use tauri::{Manager, RunEvent, WindowEvent};

use runtime::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _guard = sentry::init(
        sentry::ClientOptions::new()
            .dsn("https://218d5a11f3e753fe4b60f92e5c1e64b0@o106536.ingest.us.sentry.io/4512049004412928")
            .environment(option_env!("ENVIRONMENT").unwrap_or("development"))
            .release(option_env!("SENTRY_RELEASE").unwrap_or("local")),
    );

    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle()
                    .plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())?;
            }

            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_haptics::init())?;
            #[cfg(not(target_os = "ios"))]
            let view = screen::NativeView::new(app.get_webview_window("main").ok_or("Main window is unavailable")?)?;
            app.manage(AppState::new(
                app.handle().clone(),
                app.path().app_data_dir()?,
                #[cfg(not(target_os = "ios"))]
                view,
            )?);
            #[cfg(target_os = "ios")]
            runtime::platform::register(app.handle())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            runtime::runtime_kind,
            runtime::list_apps,
            runtime::import_app,
            runtime::delete_app,
            runtime::read_settings,
            runtime::write_settings,
            runtime::start_app,
            #[cfg(not(target_os = "ios"))]
            runtime::platform::key_event,
            #[cfg(not(target_os = "ios"))]
            runtime::platform::release_keys,
            #[cfg(target_os = "ios")]
            runtime::platform::guest_storage,
            #[cfg(target_os = "ios")]
            runtime::platform::guest_audio,
            #[cfg(target_os = "ios")]
            runtime::platform::guest_vibrate,
            runtime::stop_app,
        ])
        .on_window_event(|window, event| {
            let Some(state) = window.try_state::<AppState>() else {
                return;
            };
            match event {
                #[cfg(not(target_os = "ios"))]
                WindowEvent::Focused(false) => state.release_keys(),
                #[cfg(target_os = "android")]
                WindowEvent::Suspended => state.suspend(true),
                #[cfg(target_os = "android")]
                WindowEvent::Resumed => state.suspend(false),
                WindowEvent::CloseRequested { api, .. } => {
                    if let Some(completion) = state.stop_app() {
                        api.prevent_close();
                        let window = window.clone();
                        tauri::async_runtime::spawn(async move {
                            completion.await;
                            let _ = window.close();
                        });
                    }
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let RunEvent::ExitRequested { api, code, .. } = event
                && let Some(state) = app.try_state::<AppState>()
                && let Some(completion) = state.stop_app()
            {
                api.prevent_exit();
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    completion.await;
                    app.exit(code.unwrap_or(0));
                });
            }
        });
}
