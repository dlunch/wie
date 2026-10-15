use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::io::InputStream;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::media::Player as MidpPlayer;

use crate::classes::org::kwis::msp::media::{BaseClip, Clip};

// class org.kwis.msp.media.Player
pub struct Player;

impl Player {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/media/Player",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "pause",
                    "(Lorg/kwis/msp/media/BaseClip;)Z",
                    Self::pause,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "stop",
                    "(Lorg/kwis/msp/media/BaseClip;)Z",
                    Self::stop,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "resume",
                    "(Lorg/kwis/msp/media/BaseClip;)Z",
                    Self::resume,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "play",
                    "(Lorg/kwis/msp/media/BaseClip;Z)Z",
                    Self::play,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "record",
                    "(Lorg/kwis/msp/media/BaseClip;)Z",
                    Self::record,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "play",
                    "(Lorg/kwis/msp/media/Clip;Z)Z",
                    Self::play_clip,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "stop",
                    "(Lorg/kwis/msp/media/Clip;)Z",
                    Self::stop_clip,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn pause(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::pause({clip:?})");

        Ok(false)
    }

    async fn stop(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::stop({clip:?})");

        Ok(false)
    }

    async fn resume(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::resume({clip:?})");

        Ok(false)
    }

    async fn play(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>, repeat: bool) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::play({clip:?}, {repeat})");

        Ok(false)
    }

    async fn record(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::record({clip:?})");

        Ok(false)
    }

    async fn play_clip(jvm: &Jvm, _context: &mut WieJvmContext, mut clip: ClassInstanceRef<Clip>, repeat: bool) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::play({clip:?}, {repeat})");

        let stream: ClassInstanceRef<InputStream> = jvm
            .get_field(&clip, "org/kwis/msp/media/BaseClip", "stream", "Ljava/io/InputStream;")
            .await?;
        if stream.is_null() {
            return Ok(false);
        }

        let player = Clip::player(jvm, &clip).await?;
        if !player.is_null() {
            let _: () = jvm.invoke_virtual(&player, "javax/microedition/media/Player", "close", "()V", ()).await?;
            jvm.put_field(
                &mut clip,
                "org/kwis/msp/media/BaseClip",
                "player",
                "Ljavax/microedition/media/Player;",
                None,
            )
            .await?;
        }

        // The application can refill the shared buffer between playbacks.
        let _: () = jvm.invoke_virtual(&stream, "java/io/InputStream", "reset", "()V", ()).await?;
        let r#type = JavaLangString::from_rust_string(jvm, "application/vnd.smaf").await?;
        let player: ClassInstanceRef<MidpPlayer> = jvm
            .invoke_static(
                "javax/microedition/media/Manager",
                "createPlayer",
                "(Ljava/io/InputStream;Ljava/lang/String;)Ljavax/microedition/media/Player;",
                (stream, r#type),
            )
            .await?;
        jvm.put_field(
            &mut clip,
            "org/kwis/msp/media/BaseClip",
            "player",
            "Ljavax/microedition/media/Player;",
            player.clone(),
        )
        .await?;
        let _: () = jvm.invoke_virtual(&player, "net/wie/SmafPlayer", "start", "(Z)V", (repeat,)).await?;

        Ok(true)
    }

    async fn stop_clip(jvm: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<Clip>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::stop({clip:?})");

        let player = Clip::player(jvm, &clip).await?;

        if !player.is_null() {
            let _: () = jvm.invoke_virtual(&player, "javax/microedition/media/Player", "stop", "()V", ()).await?;

            return Ok(true);
        }

        Ok(false)
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use alloc::{boxed::Box, sync::Arc, vec::Vec};
    use std::sync::Mutex;

    use bytemuck::cast_slice;
    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::{TestPlatform, run_jvm_test, run_jvm_test_with_system};
    use wie_backend::{AudioCommand, AudioEventData};
    use wie_util::Result;

    use crate::{
        classes::org::kwis::msp::media::{BaseClip, Clip},
        get_protos,
    };

    #[test]
    fn test_base_clip_overloads_return_false() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let clip: ClassInstanceRef<BaseClip> = jvm.new_class("org/kwis/msp/media/BaseClip", "()V", ()).await?.into();

            let paused: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "pause", "(Lorg/kwis/msp/media/BaseClip;)Z", (clip.clone(),))
                .await?;
            let stopped: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "stop", "(Lorg/kwis/msp/media/BaseClip;)Z", (clip.clone(),))
                .await?;
            let resumed: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "resume", "(Lorg/kwis/msp/media/BaseClip;)Z", (clip.clone(),))
                .await?;
            let played: bool = jvm
                .invoke_static(
                    "org/kwis/msp/media/Player",
                    "play",
                    "(Lorg/kwis/msp/media/BaseClip;Z)Z",
                    (clip.clone(), true),
                )
                .await?;
            let recorded: bool = jvm
                .invoke_static("org/kwis/msp/media/Player", "record", "(Lorg/kwis/msp/media/BaseClip;)Z", (clip,))
                .await?;

            assert!(!paused);
            assert!(!stopped);
            assert!(!resumed);
            assert!(!played);
            assert!(!recorded);

            Ok(())
        })
    }

    #[test]
    fn test_clip_playback_reads_current_buffer() -> Result<()> {
        let commands = Arc::new(Mutex::new(Vec::new()));
        let recorded = commands.clone();
        let platform = TestPlatform::with_audio_handler(move |command| recorded.lock().unwrap().push(command));
        run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            Box::new(platform),
            |jvm, _| async move {
                // A single SMAF note with a 20 ms gate time.
                let mut smaf = *b"MMMD\0\0\0\x0dSEQU\0\0\0\x03\0\x21\x01\0\0";
                let r#type: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "audio/test").await?.into();
                let mut data = jvm.instantiate_array("B", smaf.len()).await?;
                let clip: ClassInstanceRef<Clip> = jvm
                    .new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;[B)V", (r#type, data.clone()))
                    .await?
                    .into();

                jvm.store_array(&mut data, 0, cast_slice::<u8, i8>(&smaf).iter().copied()).await?;
                let played: bool = jvm
                    .invoke_static("org/kwis/msp/media/Player", "play", "(Lorg/kwis/msp/media/Clip;Z)Z", (clip.clone(), true))
                    .await?;
                let stopped: bool = jvm
                    .invoke_static("org/kwis/msp/media/Player", "stop", "(Lorg/kwis/msp/media/Clip;)Z", (clip.clone(),))
                    .await?;

                assert!(played);
                assert!(stopped);

                smaf[17] = 0x22;
                jvm.store_array(&mut data, 0, cast_slice::<u8, i8>(&smaf).iter().copied()).await?;
                let replayed: bool = jvm
                    .invoke_static(
                        "org/kwis/msp/media/Player",
                        "play",
                        "(Lorg/kwis/msp/media/Clip;Z)Z",
                        (clip.clone(), false),
                    )
                    .await?;
                assert!(replayed);

                smaf[17] = 0x23;
                let mut source = jvm.instantiate_array("B", smaf.len() + 2).await?;
                jvm.store_array(&mut source, 1, cast_slice::<u8, i8>(&smaf).iter().copied()).await?;
                let written: i32 = jvm
                    .invoke_virtual(
                        &clip,
                        "org/kwis/msp/media/Clip",
                        "putData",
                        "([BII)I",
                        (source.clone(), 1, smaf.len() as i32),
                    )
                    .await?;
                assert_eq!(written, smaf.len() as i32);
                jvm.store_array(&mut source, 18, [0x24i8]).await?;
                let played_copy: bool = jvm
                    .invoke_static(
                        "org/kwis/msp/media/Player",
                        "play",
                        "(Lorg/kwis/msp/media/Clip;Z)Z",
                        (clip.clone(), false),
                    )
                    .await?;
                assert!(played_copy);

                let _: () = jvm.invoke_virtual(&clip, "org/kwis/msp/media/Clip", "clearData", "()V", ()).await?;
                let played_empty: bool = jvm
                    .invoke_static("org/kwis/msp/media/Player", "play", "(Lorg/kwis/msp/media/Clip;Z)Z", (clip, false))
                    .await?;
                assert!(!played_empty);

                Ok(())
            },
        )?;

        let commands = commands.lock().unwrap();
        assert_eq!(commands.len(), 6);
        for (index, note, repeated) in [(0, 61, true), (2, 62, false), (4, 63, false)] {
            let AudioCommand::Play { handle, sequence, repeat } = &commands[index] else {
                panic!("expected playback");
            };
            assert_eq!(*repeat, repeated);
            assert!(
                sequence
                    .events
                    .iter()
                    .any(|event| matches!(&event.data, AudioEventData::Midi(data) if data[0] == 0x90 && data[1] == note))
            );
            assert!(matches!(commands[index + 1], AudioCommand::Stop { handle: stopped } if stopped == *handle));
        }
        Ok(())
    }
}
