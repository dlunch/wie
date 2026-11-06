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
