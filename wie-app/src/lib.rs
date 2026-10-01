mod audio;
mod database;
mod filesystem;
mod library;
mod runtime;
mod screen;

use tauri::{Manager, RunEvent, WindowEvent};

use runtime::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _guard =
        sentry::init(sentry::ClientOptions::new().dsn("https://fa9187d6bd7dd43ae621f26d33641f81@o106536.ingest.us.sentry.io/4512048969678848"));

    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle()
                    .plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())?;
            }

            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_haptics::init())?;
            let window = app.get_webview_window("main").ok_or("Main window is unavailable")?;
            let view = screen::NativeView::new(window)?;
            app.manage(AppState::new(app.path().app_data_dir()?, view));
            #[cfg(target_os = "ios")]
            runtime::ios::register(app.handle())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            runtime::list_apps,
            runtime::import_app,
            runtime::delete_app,
            runtime::read_settings,
            runtime::write_settings,
            runtime::start_game,
            runtime::key_event,
            runtime::release_keys,
            runtime::stop_game,
        ])
        .on_window_event(|window, event| {
            let Some(state) = window.try_state::<AppState>() else {
                return;
            };
            match event {
                WindowEvent::Focused(false) => state.release_keys(),
                #[cfg(target_os = "android")]
                WindowEvent::Suspended => state.suspend(true),
                #[cfg(target_os = "android")]
                WindowEvent::Resumed => state.suspend(false),
                WindowEvent::CloseRequested { api, .. } => {
                    if let Some(completion) = state.stop() {
                        api.prevent_close();
                        let window = window.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = completion.await;
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
                && let Some(completion) = state.stop()
            {
                api.prevent_exit();
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = completion.await;
                    app.exit(code.unwrap_or(0));
                });
            }
        });
}
