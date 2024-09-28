use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::{lcdui::Displayable, midlet::MIDlet};

// class javax.microedition.lcdui.Display
pub struct Display;

impl Display {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Display",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new(
                    "setCurrent",
                    "(Ljavax/microedition/lcdui/Displayable;)V",
                    Self::set_current,
                    Default::default(),
                ),
                JavaMethodProto::new(
                    "getDisplay",
                    "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                    Self::get_display,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Display::<init>({:?})", &this);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn set_current(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        displayable: ClassInstanceRef<Displayable>,
    ) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Display::setCurrent({:?}, {:?})", &this, displayable);

        Ok(())
    }

    async fn get_display(jvm: &Jvm, _context: &mut WieJvmContext, midlet: ClassInstanceRef<MIDlet>) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::warn!("stub javax.microedition.lcdui.Display::getDisplay({:?})", midlet);

        let instance = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?;

        Ok(instance.into())
    }
}
