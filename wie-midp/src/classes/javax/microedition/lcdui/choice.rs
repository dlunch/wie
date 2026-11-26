use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::lcdui::{Font, Image};

// interface javax.microedition.lcdui.Choice
#[java_class(name = "javax/microedition/lcdui/Choice", public, interface, abstract)]
pub struct Choice {
    #[field(public, static, final)]
    EXCLUSIVE: i32,
    #[field(public, static, final)]
    MULTIPLE: i32,
    #[field(public, static, final)]
    IMPLICIT: i32,
    #[field(public, static, final)]
    POPUP: i32,
    #[field(public, static, final)]
    TEXT_WRAP_DEFAULT: i32,
    #[field(public, static, final)]
    TEXT_WRAP_ON: i32,
    #[field(public, static, final)]
    TEXT_WRAP_OFF: i32,
}

#[java_methods]
impl Choice {
    #[method(public)]
    async fn size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32>;

    #[method(public)]
    async fn get_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<String>>;

    #[method(public)]
    async fn get_image(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<Image>>;

    #[method(public)]
    async fn append(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        text: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
    ) -> JvmResult<i32>;

    #[method(public)]
    async fn insert(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        index: i32,
        text: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
    ) -> JvmResult<()>;

    #[method(public)]
    async fn delete(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<()>;

    #[method(public)]
    async fn delete_all(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()>;

    #[method(public)]
    async fn set(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        index: i32,
        text: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
    ) -> JvmResult<()>;

    #[method(public)]
    async fn is_selected(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<bool>;

    #[method(public)]
    async fn get_selected_index(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32>;

    #[method(public)]
    async fn get_selected_flags(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        selected: ClassInstanceRef<Array<bool>>,
    ) -> JvmResult<i32>;

    #[method(public)]
    async fn set_selected_index(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32, selected: bool) -> JvmResult<()>;

    #[method(public)]
    async fn set_selected_flags(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        selected: ClassInstanceRef<Array<bool>>,
    ) -> JvmResult<()>;

    #[method(public)]
    async fn set_fit_policy(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, fit_policy: i32) -> JvmResult<()>;

    #[method(public)]
    async fn get_fit_policy(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32>;

    #[method(public)]
    async fn set_font(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32, font: ClassInstanceRef<Font>) -> JvmResult<()>;

    #[method(public)]
    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<Font>>;

    #[method(static_initializer)]
    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Choice::<clinit>");

        jvm.put_static_field("javax/microedition/lcdui/Choice", "EXCLUSIVE", "I", 1).await?;
        jvm.put_static_field("javax/microedition/lcdui/Choice", "MULTIPLE", "I", 2).await?;
        jvm.put_static_field("javax/microedition/lcdui/Choice", "IMPLICIT", "I", 3).await?;
        jvm.put_static_field("javax/microedition/lcdui/Choice", "POPUP", "I", 4).await?;
        jvm.put_static_field("javax/microedition/lcdui/Choice", "TEXT_WRAP_DEFAULT", "I", 0)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/Choice", "TEXT_WRAP_ON", "I", 1).await?;
        jvm.put_static_field("javax/microedition/lcdui/Choice", "TEXT_WRAP_OFF", "I", 2).await?;

        Ok(())
    }
}
