use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result, runtime::JavaLangString};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{io::InputStream, lang::String};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::media::Player;

// class javax.microedition.media.Manager
pub struct Manager;

impl Manager {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/media/Manager",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "createPlayer",
                    "(Ljava/io/InputStream;Ljava/lang/String;)Ljavax/microedition/media/Player;",
                    Self::create_player,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getSupportedContentTypes",
                    "(Ljava/lang/String;)[Ljava/lang/String;",
                    Self::get_supported_content_types,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    async fn create_player(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        stream: ClassInstanceRef<InputStream>,
        r#type: ClassInstanceRef<String>,
    ) -> Result<ClassInstanceRef<Player>> {
        tracing::debug!("javax.microedition.media.Manager::createPlayer({stream:?}, {type:?})");

        let type_string = JavaLangString::to_rust_string(jvm, &r#type).await?;
        if type_string == "application/vnd.smaf" {
            Ok(jvm.new_class("net/wie/SmafPlayer", "(Ljava/io/InputStream;)V", (stream,)).await?.into())
        } else {
            Err(jvm.exception("javax/microedition/media/MediaException", "Unsupported media type").await)
        }
    }

    async fn get_supported_content_types(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        protocol: ClassInstanceRef<String>,
    ) -> Result<ClassInstanceRef<Array<String>>> {
        tracing::debug!("javax.microedition.media.Manager::getSupportedContentTypes({protocol:?})");

        let mut types = jvm.instantiate_array("Ljava/lang/String;", usize::from(protocol.is_null())).await?;
        if protocol.is_null() {
            let content_type = JavaLangString::from_rust_string(jvm, "application/vnd.smaf").await?;
            jvm.store_array(&mut types, 0, [content_type]).await?;
        }
        Ok(types.into())
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{Array, ClassInstanceRef, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{classes::javax::microedition::media::Player, get_protos};

    #[test]
    fn supported_content_types_match_player_creation() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let content_types: ClassInstanceRef<Array<String>> = jvm
                .invoke_static(
                    "javax/microedition/media/Manager",
                    "getSupportedContentTypes",
                    "(Ljava/lang/String;)[Ljava/lang/String;",
                    (None,),
                )
                .await?;
            assert_eq!(jvm.array_length(&content_types).await?, 1);
            let content_type: ClassInstanceRef<String> = jvm.load_array(&content_types, 0, 1).await?.remove(0);
            assert_eq!(JavaLangString::to_rust_string(&jvm, &content_type).await?, "application/vnd.smaf");
            for protocol in ["http", "file", "", "unknown"] {
                let protocol = JavaLangString::from_rust_string(&jvm, protocol).await?;
                let types: ClassInstanceRef<Array<String>> = jvm
                    .invoke_static(
                        "javax/microedition/media/Manager",
                        "getSupportedContentTypes",
                        "(Ljava/lang/String;)[Ljava/lang/String;",
                        (protocol,),
                    )
                    .await?;
                assert_eq!(jvm.array_length(&types).await?, 0);
            }
            let data = jvm.instantiate_array("B", 0).await?;
            let stream = jvm.new_class("java/io/ByteArrayInputStream", "([B)V", (data,)).await?;
            let player: ClassInstanceRef<Player> = jvm
                .invoke_static(
                    "javax/microedition/media/Manager",
                    "createPlayer",
                    "(Ljava/io/InputStream;Ljava/lang/String;)Ljavax/microedition/media/Player;",
                    (stream, content_type),
                )
                .await?;
            assert!(jvm.is_instance(&**player, "javax/microedition/media/Player"));
            Ok(())
        })
    }
}
