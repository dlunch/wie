use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::net::wie::{EventQueue, KeyboardEventType};

use crate::classes::{
    net::wie::WIPIKeyCode,
    org::kwis::msp::{
        lcdui::{Card, Display, Font, Graphics},
        lwc::Component,
    },
};

const PENDING: i32 = -2;
const DLG_TIMEOUT: i32 = 10;
const DLG_OK: i32 = 11;
const DLG_CANCEL: i32 = 12;
const INSET: i32 = 3;

// class org.kwis.msp.lwc.DialogComponent
pub struct DialogComponent;

impl DialogComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/DialogComponent",
            parent_class: Some("org/kwis/msp/lwc/ShellComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(I)V", Self::init_type, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lwc/Component;Ljava/lang/String;I)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lwc/Component;Ljava/lang/String;IIIII)V",
                    Self::init_bounds,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setType", "(I)V", Self::set_type, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getTimeout", "()I", Self::get_timeout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setTimeout", "(I)V", Self::set_timeout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getActionState", "()I", Self::get_action_state, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("doModal", "()I", Self::do_modal, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setButtonString",
                    "(ILjava/lang/String;)V",
                    Self::set_button_string,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("layout", "()V", Self::layout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "paintContent",
                    "(Lorg/kwis/msp/lcdui/Graphics;)V",
                    Self::paint_content,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "paintFrame",
                    "(Lorg/kwis/msp/lcdui/Graphics;)V",
                    Self::paint_frame,
                    MethodAccessFlags::PROTECTED,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("actionState", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("type", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("timeout", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("autoSize", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("title", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("okLabel", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("cancelLabel", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init_type(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, kind: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::<init>({this:?}, {kind})");
        jvm.invoke_special(
            &this,
            "org/kwis/msp/lwc/DialogComponent",
            "<init>",
            "(Lorg/kwis/msp/lwc/Component;Ljava/lang/String;I)V",
            (None, None, kind),
        )
        .await
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        work: ClassInstanceRef<Component>,
        title: ClassInstanceRef<String>,
        kind: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::<init>({this:?}, {work:?}, {title:?}, {kind})");
        Self::initialize(jvm, this, work, title, kind, None).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn init_bounds(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        work: ClassInstanceRef<Component>,
        title: ClassInstanceRef<String>,
        kind: i32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::<init>({this:?}, {work:?}, {title:?}, {kind}, {x}, {y}, {w}, {h})");
        if x < 0 || y < 0 || w <= 0 || h <= 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid dialog bounds").await);
        }
        Self::initialize(jvm, this, work, title, kind, Some((x, y, w, h))).await
    }

    async fn initialize(
        jvm: &Jvm,
        mut this: ClassInstanceRef<Self>,
        work: ClassInstanceRef<Component>,
        title: ClassInstanceRef<String>,
        kind: i32,
        bounds: Option<(i32, i32, i32, i32)>,
    ) -> JvmResult<()> {
        if let Some(rect) = bounds {
            let _: () = jvm
                .invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "(IIII)V", rect)
                .await?;
        } else {
            let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "()V", ()).await?;
        }
        jvm.put_field(&mut this, "org/kwis/msp/lwc/DialogComponent", "autoSize", "Z", bounds.is_none())
            .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/DialogComponent", "actionState", "I", PENDING)
            .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/DialogComponent", "title", "Ljava/lang/String;", title)
            .await?;
        for (field, label) in [("okLabel", "확인"), ("cancelLabel", "취소")] {
            let value = JavaLangString::from_rust_string(jvm, label).await?;
            jvm.put_field(&mut this, "org/kwis/msp/lwc/DialogComponent", field, "Ljava/lang/String;", value)
                .await?;
        }
        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lwc/DialogComponent", "setType", "(I)V", (kind,))
            .await?;
        let _: () = jvm
            .invoke_virtual(&this, "org/kwis/msp/lwc/Component", "setBackground", "(I)V", (0xffffff,))
            .await?;
        jvm.invoke_virtual(
            &this,
            "org/kwis/msp/lwc/ShellComponent",
            "setWorkComponent",
            "(Lorg/kwis/msp/lwc/Component;)V",
            (work,),
        )
        .await
    }

    async fn set_type(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, kind: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::setType({this:?}, {kind})");
        if !(0..=2).contains(&kind) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid dialog type").await);
        }
        jvm.put_field(&mut this, "org/kwis/msp/lwc/DialogComponent", "type", "I", kind).await?;
        jvm.put_field(
            &mut this,
            "org/kwis/msp/lwc/DialogComponent",
            "timeout",
            "I",
            if kind == 0 { 3000 } else { -1 },
        )
        .await?;
        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "invalidate", "()V", ()).await?;
        jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "repaint", "()V", ()).await
    }

    async fn set_timeout(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, timeout: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::setTimeout({this:?}, {timeout})");
        if timeout == -1 && jvm.get_field::<i32>(&this, "org/kwis/msp/lwc/DialogComponent", "type", "I").await? == 0 {
            return jvm
                .invoke_virtual(&this, "org/kwis/msp/lwc/DialogComponent", "setType", "(I)V", (1,))
                .await;
        }
        jvm.put_field(&mut this, "org/kwis/msp/lwc/DialogComponent", "timeout", "I", timeout)
            .await
    }

    async fn get_timeout(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::getTimeout({this:?})");
        jvm.get_field(&this, "org/kwis/msp/lwc/DialogComponent", "timeout", "I").await
    }

    async fn get_action_state(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::getActionState({this:?})");
        jvm.get_field(&this, "org/kwis/msp/lwc/DialogComponent", "actionState", "I").await
    }

    async fn do_modal(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::doModal({this:?})");
        let queue: ClassInstanceRef<EventQueue> = jvm
            .invoke_static("net/wie/EventQueue", "getEventQueue", "()Lnet/wie/EventQueue;", ())
            .await?;
        let mut event: ClassInstanceRef<Array<i32>> = jvm.instantiate_array("I", 4).await?.into();
        jvm.put_field(&mut this, "org/kwis/msp/lwc/DialogComponent", "actionState", "I", PENDING)
            .await?;
        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/ShellComponent", "show", "()V", ()).await?;
        let start = context.system().platform().now();
        let result = loop {
            let action: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/DialogComponent", "actionState", "I").await?;
            if action != PENDING {
                break action;
            }
            let timeout: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/DialogComponent", "timeout", "I").await?;
            if timeout >= 0 && context.system().platform().now() >= start + timeout as u64 {
                jvm.put_field(&mut this, "org/kwis/msp/lwc/DialogComponent", "actionState", "I", DLG_TIMEOUT)
                    .await?;
                break DLG_TIMEOUT;
            }
            if EventQueue::poll_event(jvm, context, &queue, &mut event).await? {
                let _: () = jvm
                    .invoke_virtual(&queue, "net/wie/EventQueue", "dispatchEvent", "([I)V", (event.clone(),))
                    .await?;
            }
        };
        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/ShellComponent", "hide", "()V", ()).await?;
        Ok(result)
    }

    async fn key_notify(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, event: i32, key: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::keyNotify({this:?}, {event}, {key})");
        let kind: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/DialogComponent", "type", "I").await?;
        let action = match WIPIKeyCode::from_raw(key) {
            Some(WIPIKeyCode::FIRE | WIPIKeyCode::LEFT_SOFT_KEY) if kind != 0 => Some(DLG_OK),
            Some(WIPIKeyCode::RIGHT_SOFT_KEY) if kind == 2 => Some(DLG_CANCEL),
            _ => None,
        };
        if let Some(action) = action {
            if event == KeyboardEventType::KeyPressed as i32 {
                jvm.put_field(&mut this, "org/kwis/msp/lwc/DialogComponent", "actionState", "I", action)
                    .await?;
            }
        } else {
            let _: bool = jvm
                .invoke_special(&this, "org/kwis/msp/lwc/ContainerComponent", "keyNotify", "(II)Z", (event, key))
                .await?;
        }
        // Modal input never falls through to the card underneath.
        Ok(true)
    }

    async fn set_button_string(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        button: i32,
        label: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::setButtonString({this:?}, {button}, {label:?})");
        if jvm.get_field::<i32>(&this, "org/kwis/msp/lwc/DialogComponent", "type", "I").await? == 0 {
            return Ok(());
        }
        let field = match button {
            20 => "okLabel",
            21 => "cancelLabel",
            _ => return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid dialog button").await),
        };
        jvm.put_field(&mut this, "org/kwis/msp/lwc/DialogComponent", field, "Ljava/lang/String;", label)
            .await?;
        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "invalidate", "()V", ()).await?;
        jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "repaint", "()V", ()).await
    }

    async fn captions(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<(ClassInstanceRef<Font>, i32, [ClassInstanceRef<String>; 3])> {
        let font: ClassInstanceRef<Font> = jvm
            .invoke_static("org/kwis/msp/lcdui/Font", "getDefaultFont", "()Lorg/kwis/msp/lcdui/Font;", ())
            .await?;
        let height: i32 = jvm.invoke_virtual(&font, "org/kwis/msp/lcdui/Font", "getHeight", "()I", ()).await?;
        let kind: i32 = jvm.get_field(this, "org/kwis/msp/lwc/DialogComponent", "type", "I").await?;
        let mut labels = [None.into(), None.into(), None.into()];
        for (slot, field) in labels.iter_mut().zip(["title", "okLabel", "cancelLabel"]).take(1 + kind as usize) {
            *slot = jvm
                .get_field(this, "org/kwis/msp/lwc/DialogComponent", field, "Ljava/lang/String;")
                .await?;
        }
        Ok((font, height + INSET, labels))
    }

    async fn layout(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::layout({this:?})");
        let (font, row, labels) = Self::captions(jvm, &this).await?;
        let top = INSET + if labels[0].is_null() { 0 } else { row };
        let bottom = INSET + if labels[1].is_null() && labels[2].is_null() { 0 } else { row };
        let work: ClassInstanceRef<Component> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ShellComponent", "cmpWork", "Lorg/kwis/msp/lwc/Component;")
            .await?;
        let (width, height) = if jvm.get_field::<bool>(&this, "org/kwis/msp/lwc/DialogComponent", "autoSize", "Z").await? {
            let display: ClassInstanceRef<Display> = jvm
                .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", ())
                .await?;
            let screen_w: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getWidth", "()I", ()).await?;
            let screen_h: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getHeight", "()I", ()).await?;
            let mut widths = [0; 3];
            for (w, label) in widths.iter_mut().zip(labels) {
                if label.is_null() {
                    continue;
                }
                *w = jvm
                    .invoke_virtual(&font, "org/kwis/msp/lcdui/Font", "stringWidth", "(Ljava/lang/String;)I", (label,))
                    .await?;
            }
            let preferred_w: i32 = if work.is_null() {
                0
            } else {
                jvm.invoke_virtual(&work, "org/kwis/msp/lwc/Component", "getPreferredWidth", "()I", ())
                    .await?
            };
            let w = (preferred_w.max(widths[0]).max(widths[1] + widths[2] + INSET) + 2 * INSET).min(screen_w);
            let preferred_h: i32 = if work.is_null() {
                0
            } else {
                jvm.invoke_virtual(&work, "org/kwis/msp/lwc/Component", "getPreferredHeight", "(I)I", (w - 2 * INSET,))
                    .await?
            };
            let h = (top + preferred_h + bottom).min(screen_h);
            let (x, y) = ((screen_w - w) / 2, (screen_h - h) / 2);
            let card: ClassInstanceRef<Card> = jvm
                .get_field(&this, "org/kwis/msp/lwc/ShellComponent", "cd", "Lorg/kwis/msp/lcdui/Card;")
                .await?;
            let _: () = jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "move", "(II)V", (x, y)).await?;
            let _: () = jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "resize", "(II)V", (w, h)).await?;
            for (field, value) in [("x", x), ("y", y), ("w", w), ("h", h)] {
                jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", field, "I", value).await?;
            }
            (w, h)
        } else {
            (
                jvm.get_field(&this, "org/kwis/msp/lwc/Component", "w", "I").await?,
                jvm.get_field(&this, "org/kwis/msp/lwc/Component", "h", "I").await?,
            )
        };
        if work.is_null() {
            return Ok(());
        }
        jvm.invoke_virtual(
            &work,
            "org/kwis/msp/lwc/Component",
            "configure",
            "(IIIII)V",
            (INSET, top, (width - 2 * INSET).max(0), (height - top - bottom).max(0), 3),
        )
        .await
    }

    async fn paint_content(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::paintContent({this:?}, {g:?})");
        let _: () = jvm
            .invoke_special(
                &this,
                "org/kwis/msp/lwc/Component",
                "paintContent",
                "(Lorg/kwis/msp/lcdui/Graphics;)V",
                (g.clone(),),
            )
            .await?;
        jvm.invoke_virtual(
            &this,
            "org/kwis/msp/lwc/DialogComponent",
            "paintFrame",
            "(Lorg/kwis/msp/lcdui/Graphics;)V",
            (g,),
        )
        .await
    }

    async fn paint_frame(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.DialogComponent::paintFrame({this:?}, {g:?})");
        let (font, row, labels) = Self::captions(jvm, &this).await?;
        let w: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "w", "I").await?;
        let h: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "h", "I").await?;
        let fg: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "fg", "I").await?;
        let _: () = jvm.invoke_virtual(&g, "org/kwis/msp/lcdui/Graphics", "setColor", "(I)V", (fg,)).await?;
        let _: () = jvm
            .invoke_virtual(&g, "org/kwis/msp/lcdui/Graphics", "setFont", "(Lorg/kwis/msp/lcdui/Font;)V", (font,))
            .await?;
        let _: () = jvm
            .invoke_virtual(&g, "org/kwis/msp/lcdui/Graphics", "drawRect", "(IIII)V", (0, 0, w - 1, h - 1))
            .await?;
        for (label, (x, y, anchor)) in labels
            .into_iter()
            .zip([(w / 2, INSET, 1), (INSET, h - row - INSET, 4), (w - INSET, h - row - INSET, 8)])
        {
            if label.is_null() {
                continue;
            }
            let _: () = jvm
                .invoke_virtual(
                    &g,
                    "org/kwis/msp/lcdui/Graphics",
                    "drawString",
                    "(Ljava/lang/String;III)V",
                    (label, x, y, anchor),
                )
                .await?;
        }
        Ok(())
    }
}
