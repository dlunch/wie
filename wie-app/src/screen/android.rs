use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use anyhow::{Result, anyhow};
use jni::{
    JNIEnv, JavaVM,
    objects::{GlobalRef, JObject, JValue},
    sys::jint,
};
use ndk::native_window::NativeWindow;
use raw_window_handle::{AndroidDisplayHandle, AndroidNdkWindowHandle, RawDisplayHandle, RawWindowHandle};
use tauri::wry::prelude::find_class;
use wgpu::{Instance, Surface, SurfaceTargetUnsafe};

use super::{VIEW, release_surface, report_error, request_redraw};

pub struct View {
    vm: JavaVM,
    helper: GlobalRef,
    window: RefCell<Option<NativeWindow>>,
    size: Cell<(u32, u32)>,
    playing: Cell<bool>,
}

impl View {
    pub fn new(env: &mut JNIEnv<'_>, activity: &JObject<'_>, webview: &JObject<'_>) -> Result<Rc<Self>> {
        let class = find_class(env, activity, "net/dlunch/wie/screen/NativeScreenView".into())?;
        let helper = env.new_object(
            class,
            "(Landroid/app/Activity;Landroid/webkit/WebView;)V",
            &[activity.into(), webview.into()],
        )?;
        Ok(Rc::new(Self {
            vm: env.get_java_vm()?,
            helper: env.new_global_ref(helper)?,
            window: RefCell::new(None),
            size: Cell::new((0, 0)),
            playing: Cell::new(false),
        }))
    }

    pub fn set_playing(&self, playing: bool) -> Result<()> {
        self.playing.set(playing);
        if !playing {
            release_surface();
            self.window.take();
            self.size.set((0, 0));
        }
        let mut env = self.vm.attach_current_thread()?;
        if let Err(error) = env.call_method(&self.helper, "setPlaying", "(Z)V", &[JValue::Bool(playing.into())]) {
            env.exception_clear()?;
            return Err(error.into());
        }
        Ok(())
    }

    pub fn resize(&self) -> Result<()> {
        request_redraw();
        Ok(())
    }

    fn surface_changed(&self, env: &JNIEnv<'_>, surface: &JObject<'_>, width: u32, height: u32) -> Result<()> {
        release_surface();
        self.window.take();
        self.size.set((0, 0));
        if !self.playing.get() {
            return Ok(());
        }
        let window =
            unsafe { NativeWindow::from_surface(env.get_raw(), surface.as_raw()) }.ok_or_else(|| anyhow!("Android surface is unavailable"))?;
        self.window.replace(Some(window));
        self.size.set((width, height));
        self.resize()
    }

    pub fn surface(&self, instance: &Instance) -> Result<Option<Surface<'static>>> {
        let window = self.window.borrow();
        let Some(window) = window.as_ref() else { return Ok(None) };
        let raw_window = RawWindowHandle::AndroidNdk(AndroidNdkWindowHandle::new(window.ptr().cast()));
        // Surface destruction releases the presenter before this UI-owned ANativeWindow.
        Ok(Some(unsafe {
            instance.create_surface_unsafe(SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(RawDisplayHandle::Android(AndroidDisplayHandle::new())),
                raw_window_handle: raw_window,
            })
        }?))
    }

    pub fn output_size(&self) -> Result<(u32, u32)> {
        Ok(self.size.get())
    }

    pub fn close(&self) {
        self.playing.set(false);
        release_surface();
        self.window.take();
        self.size.set((0, 0));
        let result = (|| -> Result<()> {
            let mut env = self.vm.attach_current_thread()?;
            if let Err(error) = env.call_method(&self.helper, "close", "()V", &[]) {
                env.exception_clear()?;
                return Err(error.into());
            }
            Ok(())
        })();
        if let Err(error) = result {
            log::error!("Native screen cleanup failed: {error:#}");
        }
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_net_dlunch_wie_screen_NativeScreenView_nativeSurfaceChanged(
    env: JNIEnv<'_>,
    _helper: JObject<'_>,
    surface: JObject<'_>,
    width: jint,
    height: jint,
) {
    let view = VIEW.with_borrow(Clone::clone);
    if let Some(view) = view
        && let Err(error) = view.native.surface_changed(&env, &surface, width as u32, height as u32)
    {
        report_error(error);
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_net_dlunch_wie_screen_NativeScreenView_nativeSurfaceDestroyed(_env: JNIEnv<'_>, _helper: JObject<'_>) {
    // This callback must release every ANativeWindow reference before returning to Android.
    let view = VIEW.with_borrow(Clone::clone);
    if let Some(view) = view {
        release_surface();
        view.native.window.take();
        view.native.size.set((0, 0));
    }
}
