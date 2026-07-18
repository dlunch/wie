use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::{ClassAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class net.wie.WieAudioClip
pub struct WieAudioClip;

impl WieAudioClip {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/WieAudioClip",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["com/skt/m/AudioClip"],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("open", "([BII)V", Self::open, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("close", "()V", Self::close, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("loop", "()V", Self::r#loop, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("pause", "()V", Self::pause, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("play", "()V", Self::play, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("resume", "()V", Self::resume, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("stop", "()V", Self::stop, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::<init>({this:?}, {name:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn open(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        buffer_size: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub net.wie.WieAudioClip::open({this:?}, {data:?}, {offset}, {buffer_size})");

        if data.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "data is null").await);
        }

        let array_length = jvm.array_length(&data).await?;
        if offset < 0 || buffer_size < 0 || (offset as usize).checked_add(buffer_size as usize).is_none_or(|end| end > array_length) {
            return Err(jvm
                .exception("java/lang/ArrayIndexOutOfBoundsException", "Invalid offset or length")
                .await);
        }

        Ok(())
    }

    async fn play(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub net.wie.WieAudioClip::play({this:?})");

        Ok(())
    }

    async fn r#loop(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub net.wie.WieAudioClip::loop({this:?})");

        Ok(())
    }

    async fn pause(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub net.wie.WieAudioClip::pause({this:?})");

        Ok(())
    }

    async fn resume(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub net.wie.WieAudioClip::resume({this:?})");

        Ok(())
    }

    async fn stop(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub net.wie.WieAudioClip::stop({this:?})");

        Ok(())
    }

    async fn close(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub net.wie.WieAudioClip::close({this:?})");

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;
    use jvm::{Array, ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use test_utils::run_jvm_test;

    use crate::{classes::com::skt::m::AudioClip, get_protos};

    #[test]
    fn audio_clip_accepts_valid_slices_and_rejects_invalid_ranges() {
        let result = run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let formats: ClassInstanceRef<Array<String>> = jvm
                .invoke_static("com/skt/m/AudioSystem", "getClipFormats", "()[Ljava/lang/String;", ())
                .await?;
            assert_eq!(jvm.array_length(&formats).await?, 0);

            let name = JavaLangString::from_rust_string(&jvm, "audio/mpeg").await?;
            let clip: ClassInstanceRef<AudioClip> = jvm
                .invoke_static(
                    "com/skt/m/AudioSystem",
                    "getAudioClip",
                    "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                    (name,),
                )
                .await?;
            let mut data = jvm.instantiate_array("B", 3).await?;
            jvm.store_array(&mut data, 0, [1i8, 2, 3]).await?;
            let empty_data = jvm.instantiate_array("B", 0).await?;

            for (array, offset, length) in [
                (empty_data, 0, 0),
                (data.clone(), 0, 0),
                (data.clone(), 3, 0),
                (data.clone(), 0, 3),
                (data.clone(), 1, 2),
            ] {
                let _: () = jvm.invoke_virtual(&clip, "open", "([BII)V", (array, offset, length)).await?;
            }
            let _: () = jvm.invoke_virtual(&clip, "play", "()V", ()).await?;
            let _: () = jvm.invoke_virtual(&clip, "loop", "()V", ()).await?;
            let _: () = jvm.invoke_virtual(&clip, "pause", "()V", ()).await?;
            let _: () = jvm.invoke_virtual(&clip, "resume", "()V", ()).await?;
            let _: () = jvm.invoke_virtual(&clip, "stop", "()V", ()).await?;
            let _: () = jvm.invoke_virtual(&clip, "close", "()V", ()).await?;

            for (offset, length) in [(-1, 1), (0, -1), (3, 1), (2, 2)] {
                let invalid: JvmResult<()> = jvm.invoke_virtual(&clip, "open", "([BII)V", (data.clone(), offset, length)).await;
                let Err(JavaError::JavaException(exception)) = invalid else {
                    panic!("AudioClip.open accepted invalid range ({offset}, {length})");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/ArrayIndexOutOfBoundsException"));
            }

            let null_data = ClassInstanceRef::<Array<i8>>::new(None);
            let null_result: JvmResult<()> = jvm.invoke_virtual(&clip, "open", "([BII)V", (null_data, 0, 0)).await;
            let Err(JavaError::JavaException(exception)) = null_result else {
                panic!("AudioClip.open accepted a null byte array");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

            Ok(())
        });

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
