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

use tauri::{AppHandle, Manager, RunEvent, WindowEvent};
use tracing_subscriber::layer::SubscriberExt;

use runtime::AppState;

#[tauri::command]
async fn initialize_advertising(_app: AppHandle) -> Result<&'static str, String> {
    #[cfg(mobile)]
    {
        wie_app_platform::initialize_advertising(&_app).await.map_err(|error| error.to_string())?;
        Ok(if cfg!(debug_assertions) { "adsense-test" } else { "adsense" })
    }
    #[cfg(desktop)]
    Ok("sponsor")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _guard = sentry::init((
        option_env!("SENTRY_DSN"),
        sentry::ClientOptions::new()
            .environment(option_env!("ENVIRONMENT").unwrap_or("development"))
            .release(option_env!("SENTRY_RELEASE").unwrap_or("local")),
    ));
    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(sentry::integrations::tracing::layer()))
        .expect("error while installing tracing subscriber");

    let builder = tauri::Builder::default().plugin(tauri_plugin_opener::Builder::new().open_js_links_on_click(false).build());
    #[cfg(mobile)]
    let builder = builder.plugin(wie_app_platform::init());
    builder
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
            initialize_advertising,
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

#[cfg(all(test, desktop))]
mod tests {
    use tauri::{
        WebviewUrl, WebviewWindowBuilder,
        ipc::{CallbackFn, InvokeBody},
        test::{INVOKE_KEY, get_ipc_response, mock_builder},
        webview::InvokeRequest,
    };

    use crate::runtime;

    #[test]
    fn hosted_frontend_ipc_is_limited_to_the_main_webview_and_origin() {
        let app = mock_builder()
            .invoke_handler(tauri::generate_handler![runtime::runtime_kind])
            .build(tauri::generate_context!(test = true))
            .unwrap();

        for label in ["main", "other"] {
            let webview = WebviewWindowBuilder::new(&app, label, WebviewUrl::External("https://wie.dlunch.net/".parse().unwrap()))
                .build()
                .unwrap();
            for origin in [
                "https://wie.dlunch.net/",
                "https://wie.dlunch.net/library",
                "http://wie.dlunch.net/",
                "https://wie.dlunch.net.example.com/",
                "https://pagead2.googlesyndication.com/",
            ] {
                let response = get_ipc_response(
                    &webview,
                    InvokeRequest {
                        cmd: "runtime_kind".into(),
                        callback: CallbackFn(0),
                        error: CallbackFn(1),
                        url: origin.parse().unwrap(),
                        body: InvokeBody::default(),
                        headers: Default::default(),
                        invoke_key: INVOKE_KEY.to_string(),
                    },
                );
                if label == "main" && origin.starts_with("https://wie.dlunch.net/") {
                    assert_eq!(response.unwrap().deserialize::<String>().unwrap(), "native");
                } else {
                    assert!(response.unwrap_err().as_str().unwrap().contains("not allowed"), "{label}: {origin}");
                }
            }
        }
    }
}
