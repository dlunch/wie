use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::Image;

// WIE guest record for one Choice element.
pub struct ChoiceElement;

impl ChoiceElement {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/ChoiceElement",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "<init>",
                "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;Z)V",
                Self::init,
                MethodAccessFlags::empty(),
            )],
            fields: vec![
                JavaFieldProto::new("text", "Ljava/lang/String;", FieldAccessFlags::empty()),
                JavaFieldProto::new("image", "Ljavax/microedition/lcdui/Image;", FieldAccessFlags::empty()),
                JavaFieldProto::new("displayImage", "Ljavax/microedition/lcdui/Image;", FieldAccessFlags::empty()),
                JavaFieldProto::new("font", "Ljavax/microedition/lcdui/Font;", FieldAccessFlags::empty()),
                JavaFieldProto::new("selected", "Z", FieldAccessFlags::empty()),
            ],
            access_flags: ClassAccessFlags::empty(),
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        text: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
        selected: bool,
    ) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let display_image: ClassInstanceRef<Image> = if image.is_null() {
            None.into()
        } else {
            jvm.invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(Ljavax/microedition/lcdui/Image;)Ljavax/microedition/lcdui/Image;",
                (image.clone(),),
            )
            .await?
        };

        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await?;
        jvm.put_field(&mut this, "image", "Ljavax/microedition/lcdui/Image;", image).await?;
        jvm.put_field(&mut this, "displayImage", "Ljavax/microedition/lcdui/Image;", display_image)
            .await?;
        jvm.put_field(&mut this, "font", "Ljavax/microedition/lcdui/Font;", None).await?;
        jvm.put_field(&mut this, "selected", "Z", selected).await?;

        Ok(())
    }
}
