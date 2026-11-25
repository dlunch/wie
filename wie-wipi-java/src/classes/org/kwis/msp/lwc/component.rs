use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::Object;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{
    lcdui::{Card, Graphics},
    lwc::{ContainerComponent, EventListener},
};

// class org.kwis.msp.lwc.Component
pub struct Component;

const VALID: i32 = 1;
const FOCUSED: i32 = 2;
const INPUT: i32 = 4;

impl Component {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/Component",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("focusNotify", "(Z)V", Self::focus_notify, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("showNotify", "(Z)V", Self::show_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("configure", "(IIIII)V", Self::configure, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setFocus", "()V", Self::set_focus, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getX", "()I", Self::get_x, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getY", "()I", Self::get_y, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getXOnScreen", "()I", Self::get_x_on_screen, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getYOnScreen", "()I", Self::get_y_on_screen, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("canHandleInput", "()Z", Self::can_handle_input, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hasFocus", "()Z", Self::has_focus, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isShown", "()Z", Self::is_shown, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isValid", "()Z", Self::is_valid, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("calcPreferredSize", "(I)V", Self::calc_preferred_size, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("getPreferredWidth", "()I", Self::get_preferred_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getPreferredHeight", "()I", Self::get_preferred_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getPreferredHeight",
                    "(I)I",
                    Self::get_preferred_height_for_width,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setBackground", "(I)V", Self::set_background, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setForeground", "(I)V", Self::set_foreground, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getBackground", "()I", Self::get_background, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getForeground", "()I", Self::get_foreground, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "paintContent",
                    "(Lorg/kwis/msp/lcdui/Graphics;)V",
                    Self::paint_content,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("invalidate", "()V", Self::invalidate, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("validate", "()V", Self::validate, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("layout", "()V", Self::layout, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("pointerNotify", "(III)Z", Self::pointer_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("processEvent", "(IIII)Z", Self::process_event, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("getCard", "()Lorg/kwis/msp/lcdui/Card;", Self::get_card, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setEventListener",
                    "(Lorg/kwis/msp/lwc/EventListener;Ljava/lang/Object;)V",
                    Self::set_event_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("repaint", "()V", Self::repaint, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("repaint", "(IIII)V", Self::repaint_area, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("serviceRepaints", "()V", Self::service_repaints, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("x", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("y", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("w", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("h", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("shown", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("mask", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("bg", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("fg", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("prefW", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("prefH", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("parent", "Lorg/kwis/msp/lwc/ContainerComponent;", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("evtListener", "Lorg/kwis/msp/lwc/EventListener;", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("evtListenerObj", "Ljava/lang/Object;", FieldAccessFlags::PROTECTED),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "bg", "I", -1).await
    }

    async fn key_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32, chr: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.Component::keyNotify({this:?}, {type:?}, {chr:?})");

        Ok(false)
    }

    async fn focus_notify(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, focus: bool) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::focusNotify({this:?}, {focus:?})");

        let mask: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "mask", "I").await?;
        jvm.put_field(
            &mut this,
            "org/kwis/msp/lwc/Component",
            "mask",
            "I",
            if focus { mask | FOCUSED } else { mask & !FOCUSED },
        )
        .await?;
        jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "repaint", "()V", ()).await
    }

    async fn show_notify(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, show: bool) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::showNotify({this:?}, {show:?})");

        if !show {
            return Ok(());
        }
        jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "validate", "()V", ()).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn configure(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        mask: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::configure({this:?}, {x}, {y}, {w}, {h}, {mask})");

        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "repaint", "()V", ()).await?;
        if mask & 1 != 0 {
            jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "x", "I", x).await?;
            jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "y", "I", y).await?;
        }
        if mask & 2 != 0 {
            jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "w", "I", w).await?;
            jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "h", "I", h).await?;
            let flags: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "mask", "I").await?;
            jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "mask", "I", flags & !VALID)
                .await?;
        }
        jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "repaint", "()V", ()).await
    }

    async fn set_focus(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::setFocus({this:?})");

        let mut parent: ClassInstanceRef<ContainerComponent> = jvm
            .get_field(&this, "org/kwis/msp/lwc/Component", "parent", "Lorg/kwis/msp/lwc/ContainerComponent;")
            .await?;
        if !parent.is_null() {
            let old: ClassInstanceRef<Self> = jvm
                .get_field(&parent, "org/kwis/msp/lwc/ContainerComponent", "cmpFocus", "Lorg/kwis/msp/lwc/Component;")
                .await?;
            if !old.is_null() && !old.equals(&**this)? {
                let _: bool = jvm
                    .invoke_virtual(&old, "org/kwis/msp/lwc/Component", "processEvent", "(IIII)Z", (1, 0, 0, 0))
                    .await?;
            }
            jvm.put_field(
                &mut parent,
                "org/kwis/msp/lwc/ContainerComponent",
                "cmpFocus",
                "Lorg/kwis/msp/lwc/Component;",
                this.clone(),
            )
            .await?;
            let _: () = jvm.invoke_virtual(&parent, "org/kwis/msp/lwc/Component", "setFocus", "()V", ()).await?;
        }
        if !jvm
            .invoke_virtual::<_, bool>(&this, "org/kwis/msp/lwc/Component", "hasFocus", "()Z", ())
            .await?
        {
            let _: bool = jvm
                .invoke_virtual(&this, "org/kwis/msp/lwc/Component", "processEvent", "(IIII)Z", (1, 1, 0, 0))
                .await?;
        }
        Ok(())
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.Component::getHeight({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lwc/Component", "h", "I").await
    }

    async fn get_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.Component::getWidth({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lwc/Component", "w", "I").await
    }

    async fn get_x(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.Component::getX({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lwc/Component", "x", "I").await
    }

    async fn repaint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::repaint({this:?})");

        let width: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "w", "I").await?;
        let height: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "h", "I").await?;
        jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "repaint", "(IIII)V", (0, 0, width, height))
            .await
    }

    async fn service_repaints(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::serviceRepaints({this:?})");

        let parent: ClassInstanceRef<ContainerComponent> = jvm
            .get_field(&this, "org/kwis/msp/lwc/Component", "parent", "Lorg/kwis/msp/lwc/ContainerComponent;")
            .await?;
        if parent.is_null() {
            return Ok(());
        }
        jvm.invoke_virtual(&parent, "org/kwis/msp/lwc/Component", "serviceRepaints", "()V", ())
            .await
    }

    async fn repaint_area(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::repaint({this:?}, {x}, {y}, {width}, {height})");

        let parent: ClassInstanceRef<ContainerComponent> = jvm
            .get_field(&this, "org/kwis/msp/lwc/Component", "parent", "Lorg/kwis/msp/lwc/ContainerComponent;")
            .await?;
        if parent.is_null() {
            return Ok(());
        }
        let left: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "x", "I").await?;
        let top: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "y", "I").await?;
        jvm.invoke_virtual(
            &parent,
            "org/kwis/msp/lwc/Component",
            "repaint",
            "(IIII)V",
            (x + left, y + top, width, height),
        )
        .await
    }

    async fn get_y(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.Component::getY({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lwc/Component", "y", "I").await
    }

    async fn get_x_on_screen(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.Component::getXOnScreen({this:?})");

        let parent: ClassInstanceRef<ContainerComponent> = jvm
            .get_field(&this, "org/kwis/msp/lwc/Component", "parent", "Lorg/kwis/msp/lwc/ContainerComponent;")
            .await?;
        let local: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "x", "I").await?;
        if parent.is_null() {
            return Ok(local);
        }
        let origin: i32 = jvm
            .invoke_virtual(&parent, "org/kwis/msp/lwc/Component", "getXOnScreen", "()I", ())
            .await?;
        Ok(origin + local)
    }

    async fn get_y_on_screen(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.Component::getYOnScreen({this:?})");

        let parent: ClassInstanceRef<ContainerComponent> = jvm
            .get_field(&this, "org/kwis/msp/lwc/Component", "parent", "Lorg/kwis/msp/lwc/ContainerComponent;")
            .await?;
        let local: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "y", "I").await?;
        if parent.is_null() {
            return Ok(local);
        }
        let origin: i32 = jvm
            .invoke_virtual(&parent, "org/kwis/msp/lwc/Component", "getYOnScreen", "()I", ())
            .await?;
        Ok(origin + local)
    }

    async fn can_handle_input(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.Component::canHandleInput({this:?})");

        let mask: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "mask", "I").await?;
        Ok(mask & INPUT != 0)
    }

    async fn has_focus(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.Component::hasFocus({this:?})");

        let mask: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "mask", "I").await?;
        Ok(mask & FOCUSED != 0)
    }

    async fn is_shown(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.Component::isShown({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lwc/Component", "shown", "Z").await
    }

    async fn is_valid(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.Component::isValid({this:?})");

        let mask: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "mask", "I").await?;
        Ok(mask & VALID != 0)
    }

    async fn calc_preferred_size(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, width: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::calcPreferredSize({this:?}, {width:?})");

        for (field, preferred) in [("w", "prefW"), ("h", "prefH")] {
            let value: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", field, "I").await?;
            jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", preferred, "I", value).await?;
        }
        Ok(())
    }

    async fn get_preferred_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.Component::getPreferredWidth({this:?})");

        let _: () = jvm
            .invoke_virtual(&this, "org/kwis/msp/lwc/Component", "calcPreferredSize", "(I)V", (-1,))
            .await?;
        jvm.get_field(&this, "org/kwis/msp/lwc/Component", "prefW", "I").await
    }

    async fn get_preferred_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.Component::getPreferredHeight({this:?})");

        let _: () = jvm
            .invoke_virtual(&this, "org/kwis/msp/lwc/Component", "calcPreferredSize", "(I)V", (-1,))
            .await?;
        jvm.get_field(&this, "org/kwis/msp/lwc/Component", "prefH", "I").await
    }

    async fn get_preferred_height_for_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, width: i32) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.Component::getPreferredHeight({this:?}, {width:?})");

        let _: () = jvm
            .invoke_virtual(&this, "org/kwis/msp/lwc/Component", "calcPreferredSize", "(I)V", (width,))
            .await?;
        jvm.get_field(&this, "org/kwis/msp/lwc/Component", "prefH", "I").await
    }

    async fn set_background(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, color: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::setBackground({this:?}, {color:?})");

        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "bg", "I", color).await?;
        jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "repaint", "()V", ()).await
    }

    async fn set_foreground(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, color: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::setForeground({this:?}, {color:?})");

        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "fg", "I", color).await?;
        jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "repaint", "()V", ()).await
    }

    async fn get_background(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.Component::getBackground({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lwc/Component", "bg", "I").await
    }

    async fn get_foreground(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.Component::getForeground({this:?})");

        jvm.get_field(&this, "org/kwis/msp/lwc/Component", "fg", "I").await
    }

    async fn paint_content(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::paintContent({this:?}, {graphics:?})");

        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "validate", "()V", ()).await?;
        let color: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "bg", "I").await?;
        if color == -1 {
            return Ok(());
        }
        let width: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "w", "I").await?;
        let height: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "h", "I").await?;
        let _: () = jvm
            .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "setColor", "(I)V", (color,))
            .await?;
        jvm.invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, width, height))
            .await
    }

    async fn invalidate(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::invalidate({this:?})");

        let mask: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "mask", "I").await?;
        let parent: ClassInstanceRef<ContainerComponent> = jvm
            .get_field(&this, "org/kwis/msp/lwc/Component", "parent", "Lorg/kwis/msp/lwc/ContainerComponent;")
            .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "mask", "I", mask & !VALID).await?;
        if parent.is_null() {
            return Ok(());
        }
        jvm.invoke_virtual(&parent, "org/kwis/msp/lwc/Component", "invalidate", "()V", ()).await
    }

    async fn validate(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::validate({this:?})");

        let mask: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "mask", "I").await?;
        if mask & VALID != 0 {
            return Ok(());
        }
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "mask", "I", mask | VALID).await?;
        jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "layout", "()V", ()).await
    }

    async fn layout(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::layout({this:?})");

        Ok(())
    }

    async fn pointer_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, event_type: i32, x: i32, y: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::pointerNotify({this:?}, {event_type:?}, {x:?}, {y:?})");

        Ok(false)
    }

    async fn process_event(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        event_type: i32,
        subtype: i32,
        param1: i32,
        param2: i32,
    ) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.Component::processEvent({this:?}, {event_type:?}, {subtype:?}, {param1:?}, {param2:?})");

        let listener: ClassInstanceRef<EventListener> = jvm
            .get_field(&this, "org/kwis/msp/lwc/Component", "evtListener", "Lorg/kwis/msp/lwc/EventListener;")
            .await?;
        if !listener.is_null() {
            let object: ClassInstanceRef<Object> = jvm
                .get_field(&this, "org/kwis/msp/lwc/Component", "evtListenerObj", "Ljava/lang/Object;")
                .await?;
            if jvm
                .invoke_virtual::<_, bool>(
                    &listener,
                    "org/kwis/msp/lwc/EventListener",
                    "eventNotify",
                    "(IIIILjava/lang/Object;)Z",
                    (event_type, subtype, param1, param2, object),
                )
                .await?
            {
                return Ok(true);
            }
        }
        match event_type {
            1 | 2 => {
                let method = if event_type == 1 { "focusNotify" } else { "showNotify" };
                let _: () = jvm
                    .invoke_virtual(&this, "org/kwis/msp/lwc/Component", method, "(Z)V", (subtype != 0,))
                    .await?;
                Ok(true)
            }
            3 => {
                jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "keyNotify", "(II)Z", (subtype, param1))
                    .await
            }
            4 => {
                jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "pointerNotify", "(III)Z", (subtype, param1, param2))
                    .await
            }
            _ => Ok(false),
        }
    }

    async fn get_card(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Card>> {
        tracing::debug!("org.kwis.msp.lwc.Component::getCard({this:?})");

        let parent: ClassInstanceRef<ContainerComponent> = jvm
            .get_field(&this, "org/kwis/msp/lwc/Component", "parent", "Lorg/kwis/msp/lwc/ContainerComponent;")
            .await?;
        if parent.is_null() {
            return Ok(None.into());
        }
        jvm.invoke_virtual(&parent, "org/kwis/msp/lwc/Component", "getCard", "()Lorg/kwis/msp/lcdui/Card;", ())
            .await
    }

    async fn set_event_listener(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<EventListener>,
        object: ClassInstanceRef<Object>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::setEventListener({this:?}, {listener:?}, {object:?})");

        jvm.put_field(
            &mut this,
            "org/kwis/msp/lwc/Component",
            "evtListener",
            "Lorg/kwis/msp/lwc/EventListener;",
            listener,
        )
        .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "evtListenerObj", "Ljava/lang/Object;", object)
            .await
    }
}
