use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::io::InputStream;
use jvm::{ClassInstanceRef, Jvm, Result, runtime::JavaIoInputStream};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class net.wie.SmafPlayer
pub struct SmafPlayer;

impl SmafPlayer {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/SmafPlayer",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/io/InputStream;)V", Self::init, Default::default()),
                JavaMethodProto::new("start", "()V", Self::start, Default::default()),
            ],
            fields: vec![JavaFieldProto::new("audioHandle", "I", Default::default())],
        }
    }

    async fn init(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, stream: ClassInstanceRef<InputStream>) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::<init>({:?})", &stream);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let data = JavaIoInputStream::read_until_end(jvm, &stream).await?;
        let audio_handle = context.system().audio().load_smaf(&data).unwrap();

        jvm.put_field(&mut this, "audioHandle", "I", audio_handle as i32).await?;

        Ok(())
    }

    async fn start(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::start({:?})", &this);

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;

        let system = context.system();

        system.audio().play(system, audio_handle as u32).unwrap();

        Ok(())
    }
}
