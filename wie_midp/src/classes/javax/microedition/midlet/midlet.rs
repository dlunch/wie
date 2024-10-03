use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::FieldAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class javax.microedition.midlet.MIDlet
pub struct MIDlet;

impl MIDlet {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/midlet/MIDlet",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new(
                    "getAppProperty",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    Self::get_app_property,
                    Default::default(),
                ),
                JavaMethodProto::new_abstract("startApp", "([Ljava/lang/String;)V", Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("currentMIDlet", "Ljavax/microedition/midlet/MIDlet;", FieldAccessFlags::STATIC),
                JavaFieldProto::new("display", "Ljavax/microedition/lcdui/Display;", Default::default()),
            ],
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.midlet.MIDlet::<init>({:?})", &this);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_static_field(
            "javax/microedition/midlet/MIDlet",
            "currentMIDlet",
            "Ljavax/microedition/midlet/MIDlet;",
            this.clone(),
        )
        .await?;

        // TODO we should create display outside
        let display = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?;

        jvm.put_field(&mut this, "display", "Ljavax/microedition/lcdui/Display;", display).await?;

        Ok(())
    }

    async fn get_app_property(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        key: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<String>> {
        tracing::warn!("stub javax.microedition.midlet.MIDlet::getAppProperty({:?}, {:?})", &this, key);

        Ok(None.into())
    }
}
