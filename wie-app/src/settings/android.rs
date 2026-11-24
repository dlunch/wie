use anyhow::{Context, Result};
use futures::channel::oneshot;
use jni::{
    JNIEnv, JavaVM,
    objects::{GlobalRef, JClass, JString, JValue},
};
use tauri::{AppHandle, Manager};

use super::Settings;

async fn helper(app: &AppHandle) -> Result<(JavaVM, GlobalRef)> {
    let (tx, rx) = oneshot::channel();
    app.get_webview_window("main")
        .context("Main WebView is unavailable")?
        .with_webview(move |webview| {
            webview.jni_handle().exec(move |env, activity, _| {
                let result = (|| -> Result<_> {
                    let vm = env.get_java_vm()?;
                    let context = env
                        .call_method(activity, "getApplicationContext", "()Landroid/content/Context;", &[])?
                        .l()?;
                    let loader = env.call_method(&context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?.l()?;
                    let name = env.new_string("net.dlunch.wie.settings.NativeSettings")?;
                    let class = env
                        .call_method(loader, "loadClass", "(Ljava/lang/String;)Ljava/lang/Class;", &[JValue::Object(&name)])?
                        .l()?;
                    // The constructor only retains the application context; DataStore starts on the worker.
                    let helper = env.new_object(JClass::from(class), "(Landroid/content/Context;)V", &[JValue::Object(&context)])?;
                    Ok((vm, env.new_global_ref(helper)?))
                })();
                let _ = tx.send(finish_jni(env, result));
            });
        })?;
    rx.await.context("Android settings initialization was interrupted")?
}

pub(super) async fn read(app: &AppHandle) -> Result<Settings> {
    let (vm, helper) = helper(app).await?;
    tauri::async_runtime::spawn_blocking(move || {
        let mut env = vm.attach_current_thread()?;
        let result = (|| -> Result<Settings> {
            let value = env.call_method(helper.as_obj(), "read", "()Ljava/lang/String;", &[])?.l()?;
            let json: String = env.get_string(&JString::from(value))?.into();
            let settings: Settings = serde_json::from_str(&json)?;
            settings.validate()?;
            Ok(settings)
        })();
        finish_jni(&mut env, result).context("Android settings read failed")
    })
    .await?
}

pub(super) async fn write(app: &AppHandle, settings: Settings) -> Result<()> {
    settings.validate()?;
    let (vm, helper) = helper(app).await?;
    tauri::async_runtime::spawn_blocking(move || {
        let mut env = vm.attach_current_thread()?;
        let result = env
            .call_method(
                helper.as_obj(),
                "write",
                "(FFZZ)V",
                &[
                    JValue::Float(settings.midi_volume),
                    JValue::Float(settings.pcm_volume),
                    JValue::Bool(settings.help_dismissed.into()),
                    JValue::Bool(settings.welcome_seen.into()),
                ],
            )
            .map(|_| ())
            .map_err(Into::into);
        finish_jni(&mut env, result).context("Android settings write failed")
    })
    .await?
}

fn finish_jni<T>(env: &mut JNIEnv<'_>, result: Result<T>) -> Result<T> {
    if result.is_err() {
        let _ = env.exception_describe();
        env.exception_clear()?;
    }
    result
}
