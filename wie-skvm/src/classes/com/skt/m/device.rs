use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::midlet::MIDlet;

// class com.skt.m.Device
pub struct Device;

impl Device {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/Device",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "setBacklightEnabled",
                    "(Z)V",
                    Self::set_backlight_enabled,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "isBacklightEnabled",
                    "()Z",
                    Self::is_backlight_enabled,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "setKeyToneEnabled",
                    "(Z)V",
                    Self::set_key_tone_enabled,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "isKeyToneEnabled",
                    "()Z",
                    Self::is_key_tone_enabled,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("beep", "(II)V", Self::beep, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "invokeWapBrowser",
                    "(Ljava/lang/String;)V",
                    Self::invoke_wap_browser,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "enableRestoreLCD",
                    "(Z)V",
                    Self::enable_restore_lcd,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "setKeyRepeatTime",
                    "(II)V",
                    Self::set_key_repeat_time,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "setColorMode",
                    "(I)V",
                    Self::set_color_mode,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "setSISImage",
                    "(ILjava/lang/String;[B)Z",
                    Self::set_sis_image,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "setMelody",
                    "(ILjava/lang/String;[B)Z",
                    Self::set_melody,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("setNAI", "(I)V", Self::set_nai, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "COLOR_MODE_4G",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "COLOR_MODE_256C",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "COLOR_MODE_64KC",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SIS_NORMAL",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SIS_CALL",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SIS_WAP",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SIS_PWR_ON",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "SIS_PWR_OFF",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "MELODY_MYBELL",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "MELODY_MUSICBELL",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

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

    async fn set_color_mode(_jvm: &Jvm, _context: &mut WieJvmContext, mode: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::setColorMode({mode})");

        Ok(())
    }

    async fn is_backlight_enabled(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Device::isBacklightEnabled()");

        Ok(true)
    }

    async fn set_backlight_enabled(_jvm: &Jvm, _context: &mut WieJvmContext, enabled: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::setBacklightEnabled({enabled:?})");

        Ok(())
    }

    async fn set_key_tone_enabled(_jvm: &Jvm, _context: &mut WieJvmContext, enabled: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::setKeyToneEnabled({enabled:?})");

        Ok(())
    }

    async fn is_key_tone_enabled(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Device::isKeyToneEnabled()");

        Ok(true)
    }

    async fn beep(_jvm: &Jvm, _context: &mut WieJvmContext, frequency: i32, duration: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::beep({frequency}, {duration})");

        Ok(())
    }

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

    async fn enable_restore_lcd(_jvm: &Jvm, _context: &mut WieJvmContext, enabled: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::enableRestoreLCD({enabled:?})");

        Ok(())
    }

    async fn set_key_repeat_time(_jvm: &Jvm, _context: &mut WieJvmContext, delay: i32, interval: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::setKeyRepeatTime({delay}, {interval})");

        Ok(())
    }

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

    async fn set_nai(_jvm: &Jvm, _context: &mut WieJvmContext, nai: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Device::setNAI({nai})");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc, vec};
    use core::sync::atomic::{AtomicU32, Ordering};

    use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::JavaMethodProto;
    use jvm_types::{ClassAccessFlags, MethodAccessFlags};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::{TestPlatform, TestPlatformEvent, run_jvm_test_with_system};
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_midp::classes::javax::microedition::midlet::MIDlet;

    use crate::classes::com::skt::m::Call;

    use super::Device;

    async fn destroy_app(_: &Jvm, context: &mut WieJvmContext, _: ClassInstanceRef<MIDlet>, unconditional: bool) -> JvmResult<()> {
        assert!(unconditional);
        context.system().platform().write_stdout(b"destroyed");
        Ok(())
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
        let midlet_proto = WieJavaClassProto {
            name: "test/BrowserMidlet",
            parent_class: Some("javax/microedition/midlet/MIDlet"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("destroyApp", "(Z)V", destroy_app, MethodAccessFlags::PROTECTED)],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        };
        run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), [Device::as_proto(), Call::as_proto(), midlet_proto].into()]),
            Box::new(platform),
            move |jvm, _system| async move {
                let phone_number: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "01012345678").await?.into();
                let connected: bool = jvm
                    .invoke_static("com/skt/m/Call", "connect", "(Ljava/lang/String;)Z", (phone_number,))
                    .await?;
                assert!(!connected);

                let disconnected: bool = jvm.invoke_static("com/skt/m/Call", "disconnect", "()Z", ()).await?;
                assert!(!disconnected);

                let call_supported: bool = jvm.invoke_static("com/skt/m/Call", "isSupported", "()Z", ()).await?;
                assert!(!call_supported);

                let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "standby.sis").await?.into();
                let data: ClassInstanceRef<Array<i8>> = jvm.instantiate_array("B", 3).await?.into();
                let installed: bool = jvm
                    .invoke_static("com/skt/m/Device", "setSISImage", "(ILjava/lang/String;[B)Z", (0, name, data))
                    .await?;
                assert!(!installed);

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
