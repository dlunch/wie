use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::lcdui::{Font, Image};

// WIE guest record for one Choice element.
#[java_class(name = "net/wie/ChoiceElement")]
pub struct ChoiceElement {
    text: ClassInstanceRef<String>,
    image: ClassInstanceRef<Image>,
    displayImage: ClassInstanceRef<Image>,
    font: ClassInstanceRef<Font>,
    selected: bool,
}

#[java_methods]
impl ChoiceElement {
    #[method(constructor)]
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

        jvm.put_field(&mut this, "net/wie/ChoiceElement", "text", "Ljava/lang/String;", text)
            .await?;
        jvm.put_field(&mut this, "net/wie/ChoiceElement", "image", "Ljavax/microedition/lcdui/Image;", image)
            .await?;
        jvm.put_field(
            &mut this,
            "net/wie/ChoiceElement",
            "displayImage",
            "Ljavax/microedition/lcdui/Image;",
            display_image,
        )
        .await?;
        jvm.put_field(&mut this, "net/wie/ChoiceElement", "font", "Ljavax/microedition/lcdui/Font;", None)
            .await?;
        jvm.put_field(&mut this, "net/wie/ChoiceElement", "selected", "Z", selected).await?;

        Ok(())
    }
}
