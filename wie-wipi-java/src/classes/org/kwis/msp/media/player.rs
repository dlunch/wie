use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;
use wie_midp::classes::javax::microedition::media::Player as MidpPlayer;

use crate::classes::org::kwis::msp::media::{BaseClip, Clip};

// class org.kwis.msp.media.Player
#[java_class(name = "org/kwis/msp/media/Player", public)]
pub struct Player;

#[java_methods]
impl Player {
    #[method(public, static)]
    async fn pause(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::pause({clip:?})");

        Ok(false)
    }

    #[method(public, static)]
    async fn stop(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::stop({clip:?})");

        Ok(false)
    }

    #[method(public, static)]
    async fn resume(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::resume({clip:?})");

        Ok(false)
    }

    #[method(public, static)]
    async fn play(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>, repeat: bool) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::play({clip:?}, {repeat})");

        Ok(false)
    }

    #[method(public, static)]
    async fn record(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<BaseClip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::record({clip:?})");

        Ok(false)
    }

    #[method(name = "play", public, static)]
    async fn play_clip(jvm: &Jvm, _context: &mut WieJvmContext, clip: ClassInstanceRef<Clip>, repeat: bool) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::play({clip:?}, {repeat})");

        let player: ClassInstanceRef<MidpPlayer> = jvm
            .invoke_virtual(
                &clip,
                "org/kwis/msp/media/Clip",
                "getOrCreatePlayer",
                "()Ljavax/microedition/media/Player;",
                (),
            )
            .await?;
        if player.is_null() {
            return Ok(false);
        }
        let _: () = jvm.invoke_virtual(&player, "net/wie/SmafPlayer", "start", "(Z)V", (repeat,)).await?;

        Ok(true)
    }

    #[method(name = "stop", public, static)]
    async fn stop_clip(jvm: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<Clip>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::stop({clip:?})");

        if clip.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "clip is null").await);
        }

        let player: ClassInstanceRef<MidpPlayer> = jvm
            .get_field(&clip, "org/kwis/msp/media/BaseClip", "player", "Ljavax/microedition/media/Player;")
            .await?;

        if !player.is_null() {
            let _: () = jvm.invoke_virtual(&player, "javax/microedition/media/Player", "stop", "()V", ()).await?;

            return Ok(true);
        }

        Ok(false)
    }

    #[method(name = "resume", public, static)]
    async fn resume_clip(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<Clip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::resume({clip:?})");

        Ok(false)
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use alloc::{boxed::Box, sync::Arc, vec::Vec};
    use std::sync::Mutex;

    use bytemuck::cast_slice;
    use jvm::{ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::{TestPlatform, run_jvm_test_with_system};
    use wie_backend::{AudioCommand, AudioEventData};
    use wie_util::Result;

    use crate::{classes::org::kwis::msp::media::Clip, get_protos};

    #[test]
    fn test_clip_playback_reuses_player_until_buffer_replacement() -> Result<()> {
        let commands = Arc::new(Mutex::new(Vec::new()));
        let recorded = commands.clone();
        let platform = TestPlatform::new().with_audio_handler(move |command| recorded.lock().unwrap().push(command));
        run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            Box::new(platform),
            |jvm, _| async move {
                let result: JvmResult<bool> = jvm
                    .invoke_static(
                        "org/kwis/msp/media/Player",
                        "stop",
                        "(Lorg/kwis/msp/media/Clip;)Z",
                        (ClassInstanceRef::<Clip>::new(None),),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = result else {
                    panic!("expected NullPointerException");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

                // A single SMAF note with a 20 ms gate time.
                let mut smaf = *b"MMMD\0\0\0\x0dSEQU\0\0\0\x03\0\x21\x01\0\0";
                let r#type: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "audio/test").await?.into();
                let mut data = jvm.instantiate_array("B", smaf.len()).await?;
                let clip: ClassInstanceRef<Clip> = jvm
                    .new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;[B)V", (r#type, data.clone()))
                    .await?
                    .into();

                let stopped_before_play: bool = jvm
                    .invoke_static("org/kwis/msp/media/Player", "stop", "(Lorg/kwis/msp/media/Clip;)Z", (clip.clone(),))
                    .await?;
                assert!(!stopped_before_play);

                jvm.store_array(&mut data, 0, cast_slice::<u8, i8>(&smaf).iter().copied()).await?;
                let played: bool = jvm
                    .invoke_static("org/kwis/msp/media/Player", "play", "(Lorg/kwis/msp/media/Clip;Z)Z", (clip.clone(), true))
                    .await?;
                let stopped: bool = jvm
                    .invoke_static("org/kwis/msp/media/Player", "stop", "(Lorg/kwis/msp/media/Clip;)Z", (clip.clone(),))
                    .await?;

                assert!(played);
                assert!(stopped);

                let replayed: bool = jvm
                    .invoke_static(
                        "org/kwis/msp/media/Player",
                        "play",
                        "(Lorg/kwis/msp/media/Clip;Z)Z",
                        (clip.clone(), false),
                    )
                    .await?;
                assert!(replayed);

                smaf[17] = 0x22;
                jvm.store_array(&mut data, 0, cast_slice::<u8, i8>(&smaf).iter().copied()).await?;
                let _: () = jvm
                    .invoke_virtual(&clip, "org/kwis/msp/media/Clip", "setBuffer", "([BI)V", (data, smaf.len() as i32))
                    .await?;
                let played_replacement: bool = jvm
                    .invoke_static(
                        "org/kwis/msp/media/Player",
                        "play",
                        "(Lorg/kwis/msp/media/Clip;Z)Z",
                        (clip.clone(), false),
                    )
                    .await?;
                assert!(played_replacement);

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
        assert_eq!(commands.len(), 8);
        let AudioCommand::Play { handle: first_handle, .. } = &commands[0] else {
            panic!("expected playback");
        };
        assert!(matches!(&commands[2], AudioCommand::Play { handle, .. } if handle == first_handle));
        assert!(matches!(&commands[4], AudioCommand::Play { handle, .. } if handle != first_handle));
        for (index, note, repeated) in [(0, 61, true), (2, 61, false), (4, 62, false), (6, 63, false)] {
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
