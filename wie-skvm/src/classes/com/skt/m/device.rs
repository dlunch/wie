use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;
use wie_midp::classes::javax::microedition::midlet::MIDlet;

// class com.skt.m.Device
#[java_class(name = "com/skt/m/Device", public)]
pub struct Device {
    #[field(public, static, final)]
    COLOR_MODE_4G: i32,
    #[field(public, static, final)]
    COLOR_MODE_256C: i32,
    #[field(public, static, final)]
    COLOR_MODE_64KC: i32,
    #[field(public, static, final)]
    SIS_NORMAL: i32,
    #[field(public, static, final)]
    SIS_CALL: i32,
    #[field(public, static, final)]
    SIS_WAP: i32,
    #[field(public, static, final)]
    SIS_PWR_ON: i32,
    #[field(public, static, final)]
    SIS_PWR_OFF: i32,
    #[field(public, static, final)]
    MELODY_MYBELL: i32,
    #[field(public, static, final)]
    MELODY_MUSICBELL: i32,
}

#[java_methods]
impl Device {
    #[method(static_initializer)]
    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Device::<clinit>()");

        jvm.put_static_field("com/skt/m/Device", "COLOR_MODE_4G", "I", 0).await?;
        jvm.put_static_field("com/skt/m/Device", "COLOR_MODE_256C", "I", 1).await?;
        jvm.put_static_field("com/skt/m/Device", "COLOR_MODE_64KC", "I", 2).await?;
        jvm.put_static_field("com/skt/m/Device", "SIS_NORMAL", "I", 3).await?;
        jvm.put_static_field("com/skt/m/Device", "SIS_CALL", "I", 4).await?;
        jvm.put_static_field("com/skt/m/Device", "SIS_WAP", "I", 5).await?;
        jvm.put_static_field("com/skt/m/Device", "SIS_PWR_ON", "I", 6).await?;
        jvm.put_static_field("com/skt/m/Device", "SIS_PWR_OFF", "I", 7).await?;
        jvm.put_static_field("com/skt/m/Device", "MELODY_MYBELL", "I", 8).await?;
        jvm.put_static_field("com/skt/m/Device", "MELODY_MUSICBELL", "I", 9).await?;

        Ok(())
    }

    #[method(public, static)]
    async fn set_color_mode(_jvm: &Jvm, _context: &mut WieJvmContext, mode: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::setColorMode({mode})");

        Ok(())
    }

    #[method(public, static)]
    async fn is_backlight_enabled(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Device::isBacklightEnabled()");

        Ok(true)
    }

    #[method(public, static)]
    async fn set_backlight_enabled(_jvm: &Jvm, _context: &mut WieJvmContext, enabled: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::setBacklightEnabled({enabled:?})");

        Ok(())
    }

    #[method(public, static)]
    async fn set_key_tone_enabled(_jvm: &Jvm, _context: &mut WieJvmContext, enabled: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::setKeyToneEnabled({enabled:?})");

        Ok(())
    }

    #[method(public, static)]
    async fn is_key_tone_enabled(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Device::isKeyToneEnabled()");

        Ok(true)
    }

    #[method(public, static)]
    async fn beep(_jvm: &Jvm, _context: &mut WieJvmContext, frequency: i32, duration: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::beep({frequency}, {duration})");

        Ok(())
    }

    #[method(public, static)]
    async fn invoke_wap_browser(jvm: &Jvm, context: &mut WieJvmContext, url: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::invokeWapBrowser({url:?})");

        if url.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "url is null").await);
        }
        let midlet: ClassInstanceRef<MIDlet> = jvm
            .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
            .await?;
        let _: () = jvm
            .invoke_virtual(&midlet, "javax/microedition/midlet/MIDlet", "destroyApp", "(Z)V", (true,))
            .await?;
        context.system().platform().exit();

        Ok(())
    }

    #[method(name = "enableRestoreLCD", public, static)]
    async fn enable_restore_lcd(_jvm: &Jvm, _context: &mut WieJvmContext, enabled: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::enableRestoreLCD({enabled:?})");

        Ok(())
    }

    #[method(public, static)]
    async fn set_key_repeat_time(_jvm: &Jvm, _context: &mut WieJvmContext, delay: i32, interval: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::setKeyRepeatTime({delay}, {interval})");

        Ok(())
    }

    #[method(name = "setSISImage", public, static)]
    async fn set_sis_image(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        image_type: i32,
        name: ClassInstanceRef<String>,
        data: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Device::setSISImage({image_type}, {name:?}, {data:?})");

        Ok(false)
    }

    #[method(public, static)]
    async fn set_melody(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        melody_type: i32,
        name: ClassInstanceRef<String>,
        data: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Device::setMelody({melody_type}, {name:?}, {data:?})");

        Ok(false)
    }

    #[method(name = "setNAI", public, static)]
    async fn set_nai(_jvm: &Jvm, _context: &mut WieJvmContext, nai: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::setNAI({nai})");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc};
    use core::sync::atomic::{AtomicU32, Ordering};

    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::{java_class, java_methods};
    use test_utils::{TestPlatform, TestPlatformEvent, run_jvm_test_with_system};
    use wie_jvm_support::WieJvmContext;
    use wie_midp::classes::javax::microedition::midlet::MIDlet;

    use super::Device;

    #[java_class(name = "test/BrowserMidlet", extends = "javax/microedition/midlet/MIDlet", public)]
    struct BrowserMidlet;

    #[java_methods]
    impl BrowserMidlet {
        #[method(protected)]
        async fn destroy_app(_: &Jvm, context: &mut WieJvmContext, _: ClassInstanceRef<MIDlet>, unconditional: bool) -> JvmResult<()> {
            assert!(unconditional);
            context.system().platform().write_stdout(b"destroyed");
            Ok(())
        }
    }

    #[test]
    fn device_operations_report_failures_and_browser_destroys_midlet_before_exit() {
        let events = Arc::new(AtomicU32::new(0));
        let recorded = events.clone();
        let platform = TestPlatform::with_event_handler(move |event| match event {
            TestPlatformEvent::Stdout(bytes) => {
                assert_eq!(bytes, b"destroyed");
                assert_eq!(recorded.fetch_add(1, Ordering::SeqCst), 0);
            }
            TestPlatformEvent::Exit => assert_eq!(recorded.fetch_add(1, Ordering::SeqCst), 1),
        });
        run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), [Device::as_proto(), BrowserMidlet::as_proto()].into()]),
            Box::new(platform),
            move |jvm, _system| async move {
                let midlet = jvm.instantiate_class("test/BrowserMidlet").await?;
                let _: () = jvm
                    .invoke_special(&midlet, "javax/microedition/midlet/MIDlet", "<init>", "()V", ())
                    .await?;
                let url = JavaLangString::from_rust_string(&jvm, "http://example.invalid/").await?;
                let _: () = jvm
                    .invoke_static("com/skt/m/Device", "invokeWapBrowser", "(Ljava/lang/String;)V", (url,))
                    .await?;
                assert_eq!(events.load(Ordering::SeqCst), 2);

                Ok(())
            },
        )
        .unwrap();
    }
}
