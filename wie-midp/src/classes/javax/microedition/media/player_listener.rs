use jvm::{ClassInstanceRef, Jvm, Result, runtime::JavaLangString};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::{Object, String};

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::media::Player;

// interface javax.microedition.media.PlayerListener
#[java_class(name = "javax/microedition/media/PlayerListener", public, interface, abstract)]
pub struct PlayerListener {
    #[field(public, static, final)]
    STARTED: ClassInstanceRef<String>,
    #[field(public, static, final)]
    STOPPED: ClassInstanceRef<String>,
    #[field(public, static, final)]
    END_OF_MEDIA: ClassInstanceRef<String>,
    #[field(public, static, final)]
    DURATION_UPDATED: ClassInstanceRef<String>,
    #[field(public, static, final)]
    DEVICE_UNAVAILABLE: ClassInstanceRef<String>,
    #[field(public, static, final)]
    DEVICE_AVAILABLE: ClassInstanceRef<String>,
    #[field(public, static, final)]
    VOLUME_CHANGED: ClassInstanceRef<String>,
    #[field(public, static, final)]
    ERROR: ClassInstanceRef<String>,
    #[field(public, static, final)]
    CLOSED: ClassInstanceRef<String>,
}

#[java_methods]
impl PlayerListener {
    #[method(public)]
    async fn player_update(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        player: ClassInstanceRef<Player>,
        event: ClassInstanceRef<String>,
        event_data: ClassInstanceRef<Object>,
    ) -> Result<()>;

    #[method(static_initializer)]
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
