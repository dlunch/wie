use std::{
    cell::{Cell, RefCell},
    num::NonZeroU32,
    rc::Rc,
};

use anyhow::{Result, anyhow};
use jni::{
    JNIEnv, JavaVM,
    objects::{GlobalRef, JObject, JValue},
    sys::jint,
};
use ndk::native_window::NativeWindow;
use raw_window_handle::DisplayHandle;
use softbuffer::{Context, Surface};
use tauri::wry::prelude::find_class;

use super::{Frame, VIEW, report_error};

pub(super) struct View {
    vm: JavaVM,
    helper: GlobalRef,
    surface: RefCell<Option<Surface<DisplayHandle<'static>, NativeWindow>>>,
    size: Cell<(u32, u32)>,
    frame: RefCell<Option<Frame>>,
    playing: Cell<bool>,
}

impl View {
    pub(super) fn new(env: &mut JNIEnv<'_>, activity: &JObject<'_>, webview: &JObject<'_>) -> Result<Rc<Self>> {
        let class = find_class(env, activity, "net/dlunch/wie/screen/NativeScreenView".into())?;
        let helper = env.new_object(
            class,
            "(Landroid/app/Activity;Landroid/webkit/WebView;)V",
            &[activity.into(), webview.into()],
        )?;
        Ok(Rc::new(Self {
            vm: env.get_java_vm()?,
            helper: env.new_global_ref(helper)?,
            surface: RefCell::new(None),
            size: Cell::new((0, 0)),
            frame: RefCell::new(None),
            playing: Cell::new(false),
        }))
    }

    pub(super) fn set_playing(&self, playing: bool) -> Result<()> {
        self.playing.set(playing);
        if !playing {
            self.surface.take();
            self.frame.take();
        }
        let mut env = self.vm.attach_current_thread()?;
        if let Err(error) = env.call_method(&self.helper, "setPlaying", "(Z)V", &[JValue::Bool(playing.into())]) {
            env.exception_clear()?;
            return Err(error.into());
        }
        Ok(())
    }

    pub(super) fn present(&self, frame: Frame) -> Result<()> {
        self.frame.replace(Some(frame));
        self.resize()
    }

    pub(super) fn resize(&self) -> Result<()> {
        let (width, height) = self.size.get();
        let (Some(width), Some(height)) = (NonZeroU32::new(width), NonZeroU32::new(height)) else {
            return Ok(());
        };
        let mut surface = self.surface.borrow_mut();
        let Some(surface) = surface.as_mut() else { return Ok(()) };
        surface.resize(width, height).map_err(|error| anyhow!(error.to_string()))?;
        let mut buffer = surface.buffer_mut().map_err(|error| anyhow!(error.to_string()))?;
        if let Some(frame) = self.frame.borrow().as_ref() {
            frame.draw_into(&mut buffer, width.get(), height.get());
        } else {
            buffer.fill(0);
        }
        buffer.present().map_err(|error| anyhow!(error.to_string()))
    }

    fn surface_changed(&self, env: &JNIEnv<'_>, surface: &JObject<'_>, width: u32, height: u32) -> Result<()> {
        self.surface.take();
        if !self.playing.get() {
            return Ok(());
        }
        let window =
            unsafe { NativeWindow::from_surface(env.get_raw(), surface.as_raw()) }.ok_or_else(|| anyhow!("Android surface is unavailable"))?;
        let context = Context::new(DisplayHandle::android()).map_err(|error| anyhow!(error.to_string()))?;
        let surface = Surface::new(&context, window).map_err(|error| anyhow!(error.to_string()))?;
        self.surface.replace(Some(surface));
        self.size.set((width, height));
        self.resize()
    }

    pub(super) fn close(&self) {
        self.playing.set(false);
        self.surface.take();
        self.frame.take();
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
        view.native.surface.take();
    }
}
