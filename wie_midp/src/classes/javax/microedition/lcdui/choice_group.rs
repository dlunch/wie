use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::Image;

// class javax.microedition.lcdui.ChoiceGroup
pub struct ChoiceGroup;

impl ChoiceGroup {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/ChoiceGroup",
            parent_class: Some("javax/microedition/lcdui/Item"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, Default::default()),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;I[Ljava/lang/String;[Ljavax/microedition/lcdui/Image;)V",
                    Self::init_with_elements,
                    Default::default(),
                ),
            ],
            fields: vec![],
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        choice_type: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.ChoiceGroup::<init>({this:?}, {label:?}, {choice_type})");

        let string_elements = jvm.instantiate_array("[Ljava/lang/String;", 0).await?;

        let _: () = jvm
            .invoke_special(
                &this,
                "javax/microedition/lcdui/ChoiceGroup",
                "<init>",
                "(Ljava/lang/String;I[Ljava/lang/String;[Ljavax/microedition/lcdui/Image;)V",
                (label, choice_type, string_elements, None),
            )
            .await?;

        Ok(())
    }

    async fn init_with_elements(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        label: ClassInstanceRef<String>,
        choice_type: i32,
        string_elements: ClassInstanceRef<Array<ClassInstanceRef<String>>>,
        image_elements: ClassInstanceRef<Array<ClassInstanceRef<Image>>>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.ChoiceGroup::<init>({this:?}, {label:?}, {choice_type}, {string_elements:?}, {image_elements:?})");

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Item", "<init>", "()V", ()).await?;

        Ok(())
    }
}
