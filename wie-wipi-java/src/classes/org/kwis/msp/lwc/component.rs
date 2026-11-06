use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::Object;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{
    lcdui::{Card, Graphics},
    lwc::EventListener,
};

// class org.kwis.msp.lwc.Component
pub struct Component;

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
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.Component::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn key_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32, chr: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::keyNotify({this:?}, {type:?}, {chr:?})");

        Ok(true)
    }

    async fn focus_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, focus: bool) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::focusNotify({this:?}, {focus:?})");

        Ok(())
    }

    async fn show_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, show: bool) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::showNotify({this:?}, {show:?})");

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn configure(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, w: i32, h: i32, mask: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::configure({this:?}, {x}, {y}, {w}, {h}, {mask})");

        Ok(())
    }

    async fn set_focus(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::setFocus({this:?})");

        Ok(())
    }

    async fn get_height(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getHeight({this:?})");

        Ok(0)
    }

    async fn get_width(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getWidth({this:?})");

        Ok(0)
    }

    async fn get_x(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getX({this:?})");

        Ok(0)
    }

    async fn repaint(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::repaint({this:?})");

        Ok(())
    }

    async fn service_repaints(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::serviceRepaints({this:?})");

        Ok(())
    }

    async fn repaint_area(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::repaint({this:?}, {x}, {y}, {width}, {height})");

        Ok(())
    }

    async fn get_y(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getY({this:?})");

        Ok(0)
    }

    async fn get_x_on_screen(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getXOnScreen({this:?})");

        Ok(0)
    }

    async fn get_y_on_screen(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getYOnScreen({this:?})");

        Ok(0)
    }

    async fn can_handle_input(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::canHandleInput({this:?})");

        Ok(false)
    }

    async fn has_focus(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::hasFocus({this:?})");

        Ok(false)
    }

    async fn is_shown(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::isShown({this:?})");

        Ok(false)
    }

    async fn is_valid(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::isValid({this:?})");

        Ok(false)
    }

    async fn calc_preferred_size(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, width: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::calcPreferredSize({this:?}, {width:?})");

        Ok(())
    }

    async fn get_preferred_width(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getPreferredWidth({this:?})");

        Ok(0)
    }

    async fn get_preferred_height(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getPreferredHeight({this:?})");

        Ok(0)
    }

    async fn get_preferred_height_for_width(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, width: i32) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getPreferredHeight({this:?}, {width:?})");

        Ok(0)
    }

    async fn set_background(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, color: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::setBackground({this:?}, {color:?})");

        Ok(())
    }

    async fn set_foreground(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, color: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::setForeground({this:?}, {color:?})");

        Ok(())
    }

    async fn get_background(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getBackground({this:?})");

        Ok(-1)
    }

    async fn get_foreground(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getForeground({this:?})");

        Ok(0)
    }

    async fn paint_content(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::paintContent({this:?}, {graphics:?})");

        Ok(())
    }

    async fn invalidate(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::invalidate({this:?})");

        Ok(())
    }

    async fn validate(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::validate({this:?})");

        Ok(())
    }

    async fn layout(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::layout({this:?})");

        Ok(())
    }

    async fn pointer_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, event_type: i32, x: i32, y: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::pointerNotify({this:?}, {event_type:?}, {x:?}, {y:?})");

        Ok(false)
    }

    async fn process_event(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        event_type: i32,
        subtype: i32,
        param1: i32,
        param2: i32,
    ) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::processEvent({this:?}, {event_type:?}, {subtype:?}, {param1:?}, {param2:?})");

        Ok(false)
    }

    async fn get_card(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Card>> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getCard({this:?})");

        Ok(None.into())
    }

    async fn set_event_listener(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<EventListener>,
        object: ClassInstanceRef<Object>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::setEventListener({this:?}, {listener:?}, {object:?})");

        Ok(())
    }
}
