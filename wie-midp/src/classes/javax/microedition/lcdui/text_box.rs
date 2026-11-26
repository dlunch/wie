use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

use crate::classes::{
    javax::microedition::lcdui::{Font, Graphics, Item, TextField},
    net::wie::{KeyboardEventType, MIDPKeyCode},
};

// class javax.microedition.lcdui.TextBox
#[java_class(name = "javax/microedition/lcdui/TextBox", extends = "javax/microedition/lcdui/Screen", public)]
pub struct TextBox {
    #[field(private)]
    editor: ClassInstanceRef<TextField>,
    #[field(private)]
    scrollY: i32,
}

#[java_methods]
impl TextBox {
    #[method(constructor, public)]
    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        title: ClassInstanceRef<String>,
        text: ClassInstanceRef<String>,
        max_size: i32,
        constraints: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::<init>({this:?}, {title:?}, {text:?}, {max_size}, {constraints})");

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Screen", "<init>", "()V", ()).await?;
        let field: ClassInstanceRef<TextField> = jvm
            .new_class(
                "javax/microedition/lcdui/TextField",
                "(Ljava/lang/String;Ljava/lang/String;II)V",
                (None, text, max_size, constraints),
            )
            .await?
            .into();
        let _: () = jvm
            .invoke_virtual(
                &field,
                "javax/microedition/lcdui/Item",
                "setOwner",
                "(Ljavax/microedition/lcdui/Displayable;)V",
                (this.clone(),),
            )
            .await?;
        jvm.put_field(
            &mut this,
            "javax/microedition/lcdui/TextBox",
            "editor",
            "Ljavax/microedition/lcdui/TextField;",
            field,
        )
        .await?;
        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Displayable",
            "setTitle",
            "(Ljava/lang/String;)V",
            (title,),
        )
        .await
    }

    #[method(public)]
    async fn get_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.lcdui.TextBox::getString({this:?})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        jvm.invoke_virtual(&field, "javax/microedition/lcdui/TextField", "getString", "()Ljava/lang/String;", ())
            .await
    }

    #[method(public)]
    async fn set_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::setString({this:?}, {text:?})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        jvm.invoke_virtual(
            &field,
            "javax/microedition/lcdui/TextField",
            "setString",
            "(Ljava/lang/String;)V",
            (text,),
        )
        .await
    }

    #[method(public)]
    async fn get_chars(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, data: ClassInstanceRef<Array<JavaChar>>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextBox::getChars({this:?}, {data:?})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        jvm.invoke_virtual(&field, "javax/microedition/lcdui/TextField", "getChars", "([C)I", (data,))
            .await
    }

    #[method(public)]
    async fn set_chars(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::setChars({this:?}, {data:?}, {offset}, {length})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        jvm.invoke_virtual(
            &field,
            "javax/microedition/lcdui/TextField",
            "setChars",
            "([CII)V",
            (data, offset, length),
        )
        .await
    }

    #[method(name = "insert", public)]
    async fn insert_string(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        text: ClassInstanceRef<String>,
        position: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::insert({this:?}, {text:?}, {position})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        jvm.invoke_virtual(
            &field,
            "javax/microedition/lcdui/TextField",
            "insert",
            "(Ljava/lang/String;I)V",
            (text, position),
        )
        .await
    }

    #[method(name = "insert", public)]
    async fn insert_chars(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        length: i32,
        position: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::insert({this:?}, {data:?}, {offset}, {length}, {position})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        jvm.invoke_virtual(
            &field,
            "javax/microedition/lcdui/TextField",
            "insert",
            "([CIII)V",
            (data, offset, length, position),
        )
        .await
    }

    #[method(public)]
    async fn delete(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, offset: i32, length: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::delete({this:?}, {offset}, {length})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        jvm.invoke_virtual(&field, "javax/microedition/lcdui/TextField", "delete", "(II)V", (offset, length))
            .await
    }

    #[method(public)]
    async fn get_max_size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextBox::getMaxSize({this:?})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        jvm.invoke_virtual(&field, "javax/microedition/lcdui/TextField", "getMaxSize", "()I", ())
            .await
    }

    #[method(public)]
    async fn set_max_size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, max_size: i32) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextBox::setMaxSize({this:?}, {max_size})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        jvm.invoke_virtual(&field, "javax/microedition/lcdui/TextField", "setMaxSize", "(I)I", (max_size,))
            .await
    }

    #[method(public)]
    async fn size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextBox::size({this:?})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        jvm.invoke_virtual(&field, "javax/microedition/lcdui/TextField", "size", "()I", ()).await
    }

    #[method(public)]
    async fn get_caret_position(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextBox::getCaretPosition({this:?})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        jvm.invoke_virtual(&field, "javax/microedition/lcdui/TextField", "getCaretPosition", "()I", ())
            .await
    }

    #[method(public)]
    async fn get_constraints(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.TextBox::getConstraints({this:?})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        jvm.invoke_virtual(&field, "javax/microedition/lcdui/TextField", "getConstraints", "()I", ())
            .await
    }

    #[method(public)]
    async fn set_constraints(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, constraints: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::setConstraints({this:?}, {constraints})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        jvm.invoke_virtual(&field, "javax/microedition/lcdui/TextField", "setConstraints", "(I)V", (constraints,))
            .await
    }

    #[method(public)]
    async fn set_initial_input_mode(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        subset: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::setInitialInputMode({this:?}, {subset:?})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        jvm.invoke_virtual(
            &field,
            "javax/microedition/lcdui/TextField",
            "setInitialInputMode",
            "(Ljava/lang/String;)V",
            (subset,),
        )
        .await
    }

    #[method]
    async fn item_invalidated(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        item: ClassInstanceRef<Item>,
        layout_changed: bool,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::itemInvalidated({this:?}, {item:?}, {layout_changed})");
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "requestRepaint", "()V", ())
            .await
    }

    #[method]
    async fn handle_key_event(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, event_type: i32, key: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::handleKeyEvent({this:?}, {event_type}, {key})");
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        if event_type == KeyboardEventType::KeyPressed as i32 && key == MIDPKeyCode::FIRE as i32 {
            let _: i32 = jvm
                .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "handleItemKey", "(I)I", (key,))
                .await?;
            return Ok(());
        }
        if event_type != KeyboardEventType::KeyPressed as i32 && event_type != KeyboardEventType::KeyRepeated as i32 {
            return Ok(());
        }
        if key != MIDPKeyCode::UP as i32 && key != MIDPKeyCode::DOWN as i32 {
            return Ok(());
        }
        let width: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getWidth", "()I", ())
            .await?;
        let height: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
            .await?;
        let content: i32 = jvm
            .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "preferredContentHeight", "(I)I", (width,))
            .await?;
        let scroll: i32 = jvm.get_field(&this, "javax/microedition/lcdui/TextBox", "scrollY", "I").await?;
        let delta = if key == MIDPKeyCode::UP as i32 { -Font::HEIGHT } else { Font::HEIGHT };
        let scroll = (scroll + delta).clamp(0, (content - height).max(0));
        jvm.put_field(&mut this, "javax/microedition/lcdui/TextBox", "scrollY", "I", scroll)
            .await?;
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "requestRepaint", "()V", ())
            .await
    }

    #[method]
    async fn handle_paint_event(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.TextBox::handlePaintEvent({this:?}, {graphics:?})");
        let _: () = jvm
            .invoke_special(
                &this,
                "javax/microedition/lcdui/Screen",
                "handlePaintEvent",
                "(Ljavax/microedition/lcdui/Graphics;)V",
                (graphics.clone(),),
            )
            .await?;
        let width: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getWidth", "()I", ())
            .await?;
        let height: i32 = jvm
            .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
            .await?;
        let field: ClassInstanceRef<TextField> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/TextBox",
                "editor",
                "Ljavax/microedition/lcdui/TextField;",
            )
            .await?;
        let content: i32 = jvm
            .invoke_virtual(&field, "javax/microedition/lcdui/TextField", "preferredContentHeight", "(I)I", (width,))
            .await?;
        let scroll: i32 = jvm.get_field(&this, "javax/microedition/lcdui/TextBox", "scrollY", "I").await?;
        let scroll = scroll.min((content - height).max(0));
        jvm.put_field(&mut this, "javax/microedition/lcdui/TextBox", "scrollY", "I", scroll)
            .await?;
        jvm.invoke_virtual(
            &field,
            "javax/microedition/lcdui/TextField",
            "paintContent",
            "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
            (graphics, 0, -scroll, width, height.max(content), true),
        )
        .await
    }
}
