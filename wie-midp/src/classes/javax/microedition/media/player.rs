use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::media::PlayerListener;

// interface javax.microedition.media.Player
#[java_class(
    name = "javax/microedition/media/Player",
    implements("javax/microedition/media/Controllable"),
    public,
    interface,
    abstract
)]
pub struct Player {
    #[field(public, static, final)]
    UNREALIZED: i32,
    #[field(public, static, final)]
    REALIZED: i32,
    #[field(public, static, final)]
    PREFETCHED: i32,
    #[field(public, static, final)]
    STARTED: i32,
    #[field(public, static, final)]
    CLOSED: i32,
    #[field(public, static, final)]
    TIME_UNKNOWN: i64,
}

#[java_methods]
impl Player {
    #[method(public)]
    async fn realize(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()>;

    #[method(public)]
    async fn prefetch(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()>;

    #[method(public)]
    async fn start(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()>;

    #[method(public)]
    async fn stop(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()>;

    #[method(public)]
    async fn deallocate(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()>;

    #[method(public)]
    async fn close(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()>;

    #[method(public)]
    async fn set_media_time(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, media_time: i64) -> Result<i64>;

    #[method(public)]
    async fn get_media_time(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i64>;

    #[method(public)]
    async fn get_state(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i32>;

    #[method(public)]
    async fn get_duration(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i64>;

    #[method(public)]
    async fn get_content_type(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<ClassInstanceRef<String>>;

    #[method(public)]
    async fn set_loop_count(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, count: i32) -> Result<()>;

    #[method(public)]
    async fn add_player_listener(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<PlayerListener>,
    ) -> Result<()>;

    #[method(public)]
    async fn remove_player_listener(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<PlayerListener>,
    ) -> Result<()>;

    #[method(static_initializer)]
    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> Result<()> {
        tracing::debug!("javax.microedition.media.Player::<clinit>");

        jvm.put_static_field("javax/microedition/media/Player", "UNREALIZED", "I", 100).await?;
        jvm.put_static_field("javax/microedition/media/Player", "REALIZED", "I", 200).await?;
        jvm.put_static_field("javax/microedition/media/Player", "PREFETCHED", "I", 300).await?;
        jvm.put_static_field("javax/microedition/media/Player", "STARTED", "I", 400).await?;
        jvm.put_static_field("javax/microedition/media/Player", "CLOSED", "I", 0).await?;
        jvm.put_static_field("javax/microedition/media/Player", "TIME_UNKNOWN", "J", -1i64)
            .await?;

        Ok(())
    }
}
