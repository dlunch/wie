use alloc::vec;

use jvm::{Jvm, Result, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// interface javax.microedition.media.PlayerListener
pub struct PlayerListener;

impl PlayerListener {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/media/PlayerListener",
            parent_class: None,
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new_abstract(
                    "playerUpdate",
                    "(Ljavax/microedition/media/Player;Ljava/lang/String;Ljava/lang/Object;)V",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "STARTED",
                    "Ljava/lang/String;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "STOPPED",
                    "Ljava/lang/String;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "END_OF_MEDIA",
                    "Ljava/lang/String;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "DURATION_UPDATED",
                    "Ljava/lang/String;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "DEVICE_UNAVAILABLE",
                    "Ljava/lang/String;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "DEVICE_AVAILABLE",
                    "Ljava/lang/String;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "VOLUME_CHANGED",
                    "Ljava/lang/String;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "ERROR",
                    "Ljava/lang/String;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "CLOSED",
                    "Ljava/lang/String;",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> Result<()> {
        tracing::debug!("javax.microedition.media.PlayerListener::<clinit>");

        let started = JavaLangString::from_rust_string(jvm, "started").await?;
        jvm.put_static_field("javax/microedition/media/PlayerListener", "STARTED", "Ljava/lang/String;", started)
            .await?;
        let stopped = JavaLangString::from_rust_string(jvm, "stopped").await?;
        jvm.put_static_field("javax/microedition/media/PlayerListener", "STOPPED", "Ljava/lang/String;", stopped)
            .await?;
        let end_of_media = JavaLangString::from_rust_string(jvm, "endOfMedia").await?;
        jvm.put_static_field(
            "javax/microedition/media/PlayerListener",
            "END_OF_MEDIA",
            "Ljava/lang/String;",
            end_of_media,
        )
        .await?;
        let duration_updated = JavaLangString::from_rust_string(jvm, "durationUpdated").await?;
        jvm.put_static_field(
            "javax/microedition/media/PlayerListener",
            "DURATION_UPDATED",
            "Ljava/lang/String;",
            duration_updated,
        )
        .await?;
        let device_unavailable = JavaLangString::from_rust_string(jvm, "deviceUnavailable").await?;
        jvm.put_static_field(
            "javax/microedition/media/PlayerListener",
            "DEVICE_UNAVAILABLE",
            "Ljava/lang/String;",
            device_unavailable,
        )
        .await?;
        let device_available = JavaLangString::from_rust_string(jvm, "deviceAvailable").await?;
        jvm.put_static_field(
            "javax/microedition/media/PlayerListener",
            "DEVICE_AVAILABLE",
            "Ljava/lang/String;",
            device_available,
        )
        .await?;
        let volume_changed = JavaLangString::from_rust_string(jvm, "volumeChanged").await?;
        jvm.put_static_field(
            "javax/microedition/media/PlayerListener",
            "VOLUME_CHANGED",
            "Ljava/lang/String;",
            volume_changed,
        )
        .await?;
        let error = JavaLangString::from_rust_string(jvm, "error").await?;
        jvm.put_static_field("javax/microedition/media/PlayerListener", "ERROR", "Ljava/lang/String;", error)
            .await?;
        let closed = JavaLangString::from_rust_string(jvm, "closed").await?;
        jvm.put_static_field("javax/microedition/media/PlayerListener", "CLOSED", "Ljava/lang/String;", closed)
            .await?;

        Ok(())
    }
}
