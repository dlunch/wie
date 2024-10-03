use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lcdui::Jlet;

// class wie.WIPIMIDlet
pub struct WIPIMIDlet;

impl WIPIMIDlet {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "wie/WIPIMIDlet",
            parent_class: Some("javax/microedition/midlet/MIDlet"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/lcdui/Jlet;)V", Self::init, Default::default()),
                JavaMethodProto::new("startApp", "()V", Self::start_app, Default::default()),
                JavaMethodProto::new("pauseApp", "()V", Self::pause_app, Default::default()),
                JavaMethodProto::new("destroyApp", "(Z)V", Self::destroy_app, Default::default()),
            ],
            fields: vec![JavaFieldProto::new("jlet", "Lorg/kwis/msp/lcdui/Jlet;", Default::default())],
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, jlet: ClassInstanceRef<Jlet>) -> JvmResult<()> {
        tracing::debug!("wie.WIPIMIDlet::<init>({:?}, {:?})", this, jlet);

        let _: () = jvm.invoke_special(&this, "javax/microedition/midlet/MIDlet", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "jlet", "Lorg/kwis/msp/lcdui/Jlet;", jlet).await?;

        Ok(())
    }

    async fn start_app(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("wie.WIPIMIDlet::startApp({:?})", this);

        let jlet = jvm.get_field(&this, "jlet", "Lorg/kwis/msp/lcdui/Jlet;").await?;
        let _: () = jvm.invoke_virtual(&jlet, "startApp", "([Ljava/lang/String;)V", (None,)).await?;

        Ok(())
    }

    async fn pause_app(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("wie.WIPIMIDlet::pauseApp({:?})", this);

        let jlet = jvm.get_field(&this, "jlet", "Lorg/kwis/msp/lcdui/Jlet;").await?;
        let _: () = jvm.invoke_virtual(&jlet, "pauseApp", "()V", ()).await?;

        Ok(())
    }

    async fn destroy_app(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, unconditional: bool) -> JvmResult<()> {
        tracing::debug!("wie.WIPIMIDlet::destroyApp({:?}, {:?})", this, unconditional);

        let jlet = jvm.get_field(&this, "jlet", "Lorg/kwis/msp/lcdui/Jlet;").await?;
        let _: () = jvm.invoke_virtual(&jlet, "destroyApp", "(Z)V", (unconditional,)).await?;

        Ok(())
    }
}
