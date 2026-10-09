use std::sync::{OnceLock, mpsc};

use anyhow::{Context, Result, anyhow};
use jni::{
    JavaVM,
    objects::{GlobalRef, JClass, JValue},
};
use tauri::{AppHandle, Manager};
use wie_backend::{AudioEventData, AudioHandle, AudioSequence};

use super::{Warning, smf};

// CPAL needs an application context for the lifetime of the process, not an Activity.
static APPLICATION_CONTEXT: OnceLock<GlobalRef> = OnceLock::new();

pub(super) struct Midi {
    vm: JavaVM,
    helper: Option<GlobalRef>,
    volume: f32,
}

impl Midi {
    pub fn new(app: &AppHandle, volume: f32, warning: &Warning) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        app.get_webview_window("main")
            .context("Main WebView is unavailable")?
            .with_webview(move |webview| {
                webview.jni_handle().exec(move |env, activity, _| {
                    let result = (|| -> Result<_> {
                        let vm = env.get_java_vm()?;
                        let context = env
                            .call_method(activity, "getApplicationContext", "()Landroid/content/Context;", &[])?
                            .l()?;
                        let context = env.new_global_ref(context)?;
                        APPLICATION_CONTEXT.get_or_init(|| {
                            unsafe {
                                ndk_context::initialize_android_context(vm.get_java_vm_pointer().cast(), context.as_obj().as_raw().cast());
                            }
                            context
                        });
                        let helper = (|| -> Result<_> {
                            let loader = env.call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?.l()?;
                            let name = env.new_string("net.dlunch.wie.audio.NativeAudio")?;
                            let class = env
                                .call_method(loader, "loadClass", "(Ljava/lang/String;)Ljava/lang/Class;", &[JValue::Object(&name)])?
                                .l()?;
                            let helper = env.new_object(JClass::from(class), "()V", &[])?;
                            Ok(env.new_global_ref(helper)?)
                        })();
                        Ok((vm, helper))
                    })();
                    if result.is_err() || matches!(&result, Ok((_, Err(_)))) {
                        let _ = env.exception_describe();
                        let _ = env.exception_clear();
                    }
                    let _ = tx.send(result);
                });
            })?;
        let (vm, helper) = rx.recv().context("Android audio initialization was interrupted")??;
        let helper = match helper {
            Ok(helper) => Some(helper),
            Err(error) => {
                warning(format!("MIDI is unavailable; app and PCM audio will continue: {error:#}"));
                None
            }
        };
        Ok(Self { vm, helper, volume })
    }

    pub fn start(&mut self, handle: AudioHandle, sequence: &AudioSequence, repeat: bool) -> Result<()> {
        let Some(helper) = &self.helper else {
            return Ok(());
        };
        if !sequence.events.iter().any(|event| matches!(event.data, AudioEventData::Midi(_))) {
            return Ok(());
        }
        let bytes = smf::encode(sequence)?;
        let mut env = self.vm.attach_current_thread()?;
        let result = env.byte_array_from_slice(&bytes).and_then(|bytes| {
            env.call_method(
                helper.as_obj(),
                "play",
                "(J[BZF)V",
                &[
                    JValue::Long(i64::from(handle)),
                    JValue::Object(&bytes),
                    JValue::Bool((repeat && sequence.duration != 0).into()),
                    JValue::Float(self.volume),
                ],
            )
        });
        if let Err(error) = result {
            env.exception_describe()?;
            env.exception_clear()?;
            return Err(anyhow!("Android MIDI preparation failed: {error}"));
        }
        Ok(())
    }

    fn call(&self, method: &str, signature: &str, args: &[JValue<'_, '_>]) -> Result<()> {
        let Some(helper) = &self.helper else {
            return Ok(());
        };
        let mut env = self.vm.attach_current_thread()?;
        if let Err(error) = env.call_method(helper.as_obj(), method, signature, args) {
            env.exception_describe()?;
            env.exception_clear()?;
            return Err(anyhow!("Android audio {method} failed: {error}"));
        }
        Ok(())
    }

    pub fn event(&mut self, _handle: AudioHandle, _data: &[u8]) -> Result<()> {
        Ok(())
    }

    pub fn finish(&mut self, _handle: AudioHandle) -> Result<()> {
        Ok(())
    }

    pub fn reap(&mut self) {}

    pub fn stop(&mut self, handle: AudioHandle) -> Result<()> {
        self.call("stop", "(J)V", &[JValue::Long(i64::from(handle))])
    }

    pub fn set_volume(&mut self, volume: f32) -> Result<()> {
        self.call("volume", "(F)V", &[JValue::Float(volume)])?;
        self.volume = volume;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        self.call("pause", "(Z)V", &[JValue::Bool(1)])
    }

    pub fn resume(&mut self) -> Result<()> {
        self.call("pause", "(Z)V", &[JValue::Bool(0)])
    }
}

impl Drop for Midi {
    fn drop(&mut self) {
        if let Err(error) = self.call("close", "()V", &[]) {
            log::error!("{error:#}");
        }
    }
}
