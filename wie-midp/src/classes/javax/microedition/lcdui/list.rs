use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

use crate::classes::{
    javax::microedition::lcdui::{ChoiceGroup, Command, Font, Graphics, Image, Item},
    net::wie::{KeyboardEventType, MIDPKeyCode},
};

// class javax.microedition.lcdui.List
#[java_class(
    name = "javax/microedition/lcdui/List",
    extends = "javax/microedition/lcdui/Screen",
    implements("javax/microedition/lcdui/Choice"),
    public
)]
pub struct List {
    #[field(public, static, final)]
    SELECT_COMMAND: ClassInstanceRef<Command>,
    #[field(private)]
    selectCommand: ClassInstanceRef<Command>,
    #[field(private)]
    choices: ClassInstanceRef<ChoiceGroup>,
    #[field(private)]
    scrollY: i32,
}

#[java_methods]
impl List {
    #[method(static_initializer)]
    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::<clinit>");
        let label = JavaLangString::from_rust_string(jvm, "").await?;
        let command = jvm
            .new_class("javax/microedition/lcdui/Command", "(Ljava/lang/String;II)V", (label, 1, 0))
            .await?;
        jvm.put_static_field(
            "javax/microedition/lcdui/List",
            "SELECT_COMMAND",
            "Ljavax/microedition/lcdui/Command;",
            command,
        )
        .await
    }

    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, title: ClassInstanceRef<String>, list_type: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::<init>({this:?}, {title:?}, {list_type})");
        let strings = jvm.instantiate_array("Ljava/lang/String;", 0).await?;
        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/List",
            "<init>",
            "(Ljava/lang/String;I[Ljava/lang/String;[Ljavax/microedition/lcdui/Image;)V",
            (title, list_type, strings, None),
        )
        .await
    }

    #[method(constructor, public)]
    async fn init_with_elements(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        title: ClassInstanceRef<String>,
        list_type: i32,
        strings: ClassInstanceRef<Array<ClassInstanceRef<String>>>,
        images: ClassInstanceRef<Array<ClassInstanceRef<Image>>>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::<init>({this:?}, {title:?}, {list_type}, {strings:?}, {images:?})");
        if !(1..=3).contains(&list_type) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid List type").await);
        }
        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Screen", "<init>", "()V", ()).await?;
        let group = jvm
            .new_class(
                "javax/microedition/lcdui/ChoiceGroup",
                "(Ljavax/microedition/lcdui/List;I[Ljava/lang/String;[Ljavax/microedition/lcdui/Image;)V",
                (this.clone(), list_type, strings, images),
            )
            .await?;
        jvm.put_field(
            &mut this,
            "javax/microedition/lcdui/List",
            "choices",
            "Ljavax/microedition/lcdui/ChoiceGroup;",
            group,
        )
        .await?;
        if list_type == 3 {
            let select: ClassInstanceRef<Command> = jvm
                .get_static_field("javax/microedition/lcdui/List", "SELECT_COMMAND", "Ljavax/microedition/lcdui/Command;")
                .await?;
            jvm.put_field(
                &mut this,
                "javax/microedition/lcdui/List",
                "selectCommand",
                "Ljavax/microedition/lcdui/Command;",
                select,
            )
            .await?;
        }
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
    async fn size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.List::size({this:?})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(&group, "javax/microedition/lcdui/ChoiceGroup", "size", "()I", ())
            .await
    }

    #[method(public)]
    async fn get_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("javax.microedition.lcdui.List::getString({this:?}, {index})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(
            &group,
            "javax/microedition/lcdui/ChoiceGroup",
            "getString",
            "(I)Ljava/lang/String;",
            (index,),
        )
        .await
    }

    #[method(public)]
    async fn get_image(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.List::getImage({this:?}, {index})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(
            &group,
            "javax/microedition/lcdui/ChoiceGroup",
            "getImage",
            "(I)Ljavax/microedition/lcdui/Image;",
            (index,),
        )
        .await
    }

    #[method(public)]
    async fn append(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        text: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
    ) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.List::append({this:?}, {text:?}, {image:?})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(
            &group,
            "javax/microedition/lcdui/ChoiceGroup",
            "append",
            "(Ljava/lang/String;Ljavax/microedition/lcdui/Image;)I",
            (text, image),
        )
        .await
    }

    #[method(public)]
    async fn insert(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        index: i32,
        text: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::insert({this:?}, {index}, {text:?}, {image:?})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(
            &group,
            "javax/microedition/lcdui/ChoiceGroup",
            "insert",
            "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V",
            (index, text, image),
        )
        .await
    }

    #[method(public)]
    async fn delete(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::delete({this:?}, {index})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(&group, "javax/microedition/lcdui/ChoiceGroup", "delete", "(I)V", (index,))
            .await
    }

    #[method(public)]
    async fn delete_all(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::deleteAll({this:?})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(&group, "javax/microedition/lcdui/ChoiceGroup", "deleteAll", "()V", ())
            .await
    }

    #[method(public)]
    async fn set(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        index: i32,
        text: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::set({this:?}, {index}, {text:?}, {image:?})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(
            &group,
            "javax/microedition/lcdui/ChoiceGroup",
            "set",
            "(ILjava/lang/String;Ljavax/microedition/lcdui/Image;)V",
            (index, text, image),
        )
        .await
    }

    #[method(public)]
    async fn is_selected(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.List::isSelected({this:?}, {index})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(&group, "javax/microedition/lcdui/ChoiceGroup", "isSelected", "(I)Z", (index,))
            .await
    }

    #[method(public)]
    async fn get_selected_index(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.List::getSelectedIndex({this:?})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(&group, "javax/microedition/lcdui/ChoiceGroup", "getSelectedIndex", "()I", ())
            .await
    }

    #[method(public)]
    async fn get_selected_flags(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        flags: ClassInstanceRef<Array<bool>>,
    ) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.List::getSelectedFlags({this:?}, {flags:?})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(&group, "javax/microedition/lcdui/ChoiceGroup", "getSelectedFlags", "([Z)I", (flags,))
            .await
    }

    #[method(public)]
    async fn set_selected_index(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32, selected: bool) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::setSelectedIndex({this:?}, {index}, {selected})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(
            &group,
            "javax/microedition/lcdui/ChoiceGroup",
            "setSelectedIndex",
            "(IZ)V",
            (index, selected),
        )
        .await
    }

    #[method(public)]
    async fn set_selected_flags(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        flags: ClassInstanceRef<Array<bool>>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::setSelectedFlags({this:?}, {flags:?})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(&group, "javax/microedition/lcdui/ChoiceGroup", "setSelectedFlags", "([Z)V", (flags,))
            .await
    }

    #[method(public)]
    async fn set_fit_policy(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, policy: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::setFitPolicy({this:?}, {policy})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(&group, "javax/microedition/lcdui/ChoiceGroup", "setFitPolicy", "(I)V", (policy,))
            .await
    }

    #[method(public)]
    async fn get_fit_policy(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.List::getFitPolicy({this:?})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(&group, "javax/microedition/lcdui/ChoiceGroup", "getFitPolicy", "()I", ())
            .await
    }

    #[method(public)]
    async fn set_font(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32, font: ClassInstanceRef<Font>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::setFont({this:?}, {index}, {font:?})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(
            &group,
            "javax/microedition/lcdui/ChoiceGroup",
            "setFont",
            "(ILjavax/microedition/lcdui/Font;)V",
            (index, font),
        )
        .await
    }

    #[method(public)]
    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::debug!("javax.microedition.lcdui.List::getFont({this:?}, {index})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        jvm.invoke_virtual(
            &group,
            "javax/microedition/lcdui/ChoiceGroup",
            "getFont",
            "(I)Ljavax/microedition/lcdui/Font;",
            (index,),
        )
        .await
    }

    #[method(public)]
    async fn set_select_command(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        command: ClassInstanceRef<Command>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::setSelectCommand({this:?}, {command:?})");
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        if jvm
            .get_field::<i32>(&group, "javax/microedition/lcdui/ChoiceGroup", "choiceType", "I")
            .await?
            == 3
        {
            let default: ClassInstanceRef<Command> = jvm
                .get_static_field("javax/microedition/lcdui/List", "SELECT_COMMAND", "Ljavax/microedition/lcdui/Command;")
                .await?;
            if !command.is_null() && command.identity() != default.identity() {
                let _: () = jvm
                    .invoke_special(
                        &this,
                        "javax/microedition/lcdui/Displayable",
                        "addCommand",
                        "(Ljavax/microedition/lcdui/Command;)V",
                        (command.clone(),),
                    )
                    .await?;
            }
            jvm.put_field(
                &mut this,
                "javax/microedition/lcdui/List",
                "selectCommand",
                "Ljavax/microedition/lcdui/Command;",
                command,
            )
            .await?;
        }
        Ok(())
    }

    #[method(public)]
    async fn remove_command(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, command: ClassInstanceRef<Command>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::removeCommand({this:?}, {command:?})");
        let select: ClassInstanceRef<Command> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "selectCommand",
                "Ljavax/microedition/lcdui/Command;",
            )
            .await?;
        if !command.is_null() && !select.is_null() && command.identity() == select.identity() {
            jvm.put_field(
                &mut this,
                "javax/microedition/lcdui/List",
                "selectCommand",
                "Ljavax/microedition/lcdui/Command;",
                None,
            )
            .await?;
        }
        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Displayable",
            "removeCommand",
            "(Ljavax/microedition/lcdui/Command;)V",
            (command,),
        )
        .await
    }

    #[method]
    async fn dispatch_command(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, command: ClassInstanceRef<Command>) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.List::dispatchCommand({this:?}, {command:?})");
        let select: ClassInstanceRef<Command> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "selectCommand",
                "Ljavax/microedition/lcdui/Command;",
            )
            .await?;
        if !select.is_null()
            && command.identity() == select.identity()
            && jvm
                .invoke_special::<_, i32>(&this, "javax/microedition/lcdui/List", "size", "()I", ())
                .await?
                == 0
        {
            return Ok(false);
        }
        jvm.invoke_special(
            &this,
            "javax/microedition/lcdui/Displayable",
            "dispatchCommand",
            "(Ljavax/microedition/lcdui/Command;)Z",
            (command,),
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
        tracing::debug!("javax.microedition.lcdui.List::itemInvalidated({this:?}, {item:?}, {layout_changed})");
        jvm.invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "requestRepaint", "()V", ())
            .await
    }

    // Both input and paint use the current row bounds, since the application can edit
    // the choices or change the available screen height between key events.
    async fn scroll_to_focus(
        jvm: &Jvm,
        this: &mut ClassInstanceRef<Self>,
        group: &ClassInstanceRef<ChoiceGroup>,
        width: i32,
        height: i32,
        page_delta: i32,
    ) -> JvmResult<(i32, bool)> {
        let bounds: ClassInstanceRef<Array<i32>> = jvm
            .invoke_virtual(group, "javax/microedition/lcdui/ChoiceGroup", "getFocusContentBounds", "(I)[I", (width,))
            .await?;
        if bounds.is_null() {
            return Ok((0, false));
        }
        let bounds = jvm.load_array::<i32>(&bounds, 0, 2).await?;
        let top = bounds[0];
        let last_page = bounds[1] - height;
        let previous: i32 = jvm.get_field(this, "javax/microedition/lcdui/List", "scrollY", "I").await?;
        let (scroll, paged) = if last_page > top {
            // Page through an oversized row before moving focus to its neighbor.
            let current = previous.clamp(top, last_page);
            let next = current.saturating_add(page_delta).clamp(top, last_page);
            (next, next != current)
        } else {
            let content_height: i32 = jvm
                .invoke_virtual(group, "javax/microedition/lcdui/ChoiceGroup", "preferredContentHeight", "(I)I", (width,))
                .await?;
            (previous.clamp(last_page, top).clamp(0, (content_height - height).max(0)), false)
        };
        jvm.put_field(this, "javax/microedition/lcdui/List", "scrollY", "I", scroll).await?;
        Ok((scroll, paged))
    }

    #[method]
    async fn handle_key_event(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, event_type: i32, code: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::handleKeyEvent({this:?}, {event_type}, {code})");
        let vertical = code == MIDPKeyCode::UP as i32 || code == MIDPKeyCode::DOWN as i32;
        if event_type != KeyboardEventType::KeyPressed as i32 && !(vertical && event_type == KeyboardEventType::KeyRepeated as i32) {
            return Ok(());
        }
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        if vertical {
            let width: i32 = jvm
                .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getWidth", "()I", ())
                .await?;
            let height: i32 = jvm
                .invoke_special(&this, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
                .await?;
            let delta = if code == MIDPKeyCode::UP as i32 { -height } else { height };
            let (_, paged) = Self::scroll_to_focus(jvm, &mut this, &group, width, height, delta).await?;
            if paged {
                return jvm
                    .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "requestRepaint", "()V", ())
                    .await;
            }
        }
        let _: i32 = jvm
            .invoke_virtual(&group, "javax/microedition/lcdui/ChoiceGroup", "handleItemKey", "(I)I", (code,))
            .await?;
        if code == MIDPKeyCode::FIRE as i32 {
            let command: ClassInstanceRef<Command> = jvm
                .get_field(
                    &this,
                    "javax/microedition/lcdui/List",
                    "selectCommand",
                    "Ljavax/microedition/lcdui/Command;",
                )
                .await?;
            if !command.is_null() {
                let _: bool = jvm
                    .invoke_virtual(
                        &this,
                        "javax/microedition/lcdui/List",
                        "dispatchCommand",
                        "(Ljavax/microedition/lcdui/Command;)Z",
                        (command,),
                    )
                    .await?;
            }
        }
        Ok(())
    }

    #[method]
    async fn handle_paint_event(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.List::handlePaintEvent({this:?}, {graphics:?})");
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
        let group: ClassInstanceRef<ChoiceGroup> = jvm
            .get_field(
                &this,
                "javax/microedition/lcdui/List",
                "choices",
                "Ljavax/microedition/lcdui/ChoiceGroup;",
            )
            .await?;
        let (scroll, _) = Self::scroll_to_focus(jvm, &mut this, &group, width, height, 0).await?;
        jvm.invoke_virtual(
            &group,
            "javax/microedition/lcdui/ChoiceGroup",
            "paintContent",
            "(Ljavax/microedition/lcdui/Graphics;IIIIZ)V",
            (graphics, 0, -scroll, width, height, true),
        )
        .await
    }
}
