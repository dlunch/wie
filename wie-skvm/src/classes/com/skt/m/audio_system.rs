use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::com::skt::m::audio_clip::AudioClip;

// class com.skt.m.AudioSystem
pub struct AudioSystem;

impl AudioSystem {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/AudioSystem",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "getAudioClip",
                    "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                    Self::get_audio_clip,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getClipFormats",
                    "()[Ljava/lang/String;",
                    Self::get_clip_formats,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getMaxVolume",
                    "(Ljava/lang/String;)I",
                    Self::get_max_volume,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getVolume",
                    "(Ljava/lang/String;)I",
                    Self::get_volume,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "setVolume",
                    "(Ljava/lang/String;I)V",
                    Self::set_volume,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    async fn get_audio_clip(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<AudioClip>> {
        tracing::debug!("com.skt.m.AudioSystem::getAudioClip({name:?})");

        if name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "format is null").await);
        }
        if JavaLangString::to_rust_string(jvm, &name).await? != "mmf" {
            return Err(jvm.exception("com/skt/m/UnsupportedFormatException", "Unsupported audio format").await);
        }

        let audio_clip = jvm.new_class("net/wie/WieAudioClip", "(Ljava/lang/String;)V", (name,)).await?;

        Ok(audio_clip.into())
    }

    async fn get_clip_formats(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Array<String>>> {
        tracing::debug!("com.skt.m.AudioSystem::getClipFormats()");

        let mut formats = jvm.instantiate_array("Ljava/lang/String;", 1).await?;
        let format = JavaLangString::from_rust_string(jvm, "mmf").await?;
        jvm.store_array(&mut formats, 0, [format]).await?;
        Ok(formats.into())
    }

    async fn get_max_volume(jvm: &Jvm, _context: &mut WieJvmContext, format: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.AudioSystem::getMaxVolume({format:?})");

        if format.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "format is null").await);
        }

        Ok(0)
    }

    async fn get_volume(jvm: &Jvm, _context: &mut WieJvmContext, format: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.AudioSystem::getVolume({format:?})");

        if format.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "format is null").await);
        }

        Ok(0)
    }

    async fn set_volume(jvm: &Jvm, _context: &mut WieJvmContext, format: ClassInstanceRef<String>, level: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.AudioSystem::setVolume({format:?}, {level})");

        if format.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "format is null").await);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;

    use crate::{classes::com::skt::m::AudioClip, get_protos};

    #[test]
    fn audio_system_rejects_null_format_arguments() {
        let result = run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let null_format = ClassInstanceRef::<String>::new(None);

            let clip_result: JvmResult<ClassInstanceRef<AudioClip>> = jvm
                .invoke_static(
                    "com/skt/m/AudioSystem",
                    "getAudioClip",
                    "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                    (null_format.clone(),),
                )
                .await;
            let Err(JavaError::JavaException(exception)) = clip_result else {
                panic!("AudioSystem.getAudioClip accepted a null format");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

            let unsupported = JavaLangString::from_rust_string(&jvm, "audio/mpeg").await?;
            let clip_result: JvmResult<ClassInstanceRef<AudioClip>> = jvm
                .invoke_static(
                    "com/skt/m/AudioSystem",
                    "getAudioClip",
                    "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                    (unsupported,),
                )
                .await;
            let Err(JavaError::JavaException(exception)) = clip_result else {
                panic!("AudioSystem.getAudioClip accepted an unsupported format");
            };
            assert!(jvm.is_instance(&*exception, "com/skt/m/UnsupportedFormatException"));

            for method in ["getMaxVolume", "getVolume"] {
                let volume_result: JvmResult<i32> = jvm
                    .invoke_static("com/skt/m/AudioSystem", method, "(Ljava/lang/String;)I", (null_format.clone(),))
                    .await;
                let Err(JavaError::JavaException(exception)) = volume_result else {
                    panic!("AudioSystem.{method} accepted a null format");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));
            }

            let set_result: JvmResult<()> = jvm
                .invoke_static("com/skt/m/AudioSystem", "setVolume", "(Ljava/lang/String;I)V", (null_format, 0))
                .await;
            let Err(JavaError::JavaException(exception)) = set_result else {
                panic!("AudioSystem.setVolume accepted a null format");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

            Ok(())
        });

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
