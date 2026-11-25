use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{lcdui::Graphics, lwc::Component};

// class org.kwis.msp.lwc.ContainerComponent
pub struct ContainerComponent;

impl ContainerComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/ContainerComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new(
                    "addComponent",
                    "(Lorg/kwis/msp/lwc/Component;)I",
                    Self::add_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("removeComponent", "(I)V", Self::remove_component_index, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "removeComponent",
                    "(Lorg/kwis/msp/lwc/Component;)V",
                    Self::remove_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("canHandleInput", "()Z", Self::can_handle_input, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("focusNotify", "(Z)V", Self::focus_notify, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PROTECTED),
            ],
            fields: vec![
                JavaFieldProto::new("cmps", "[Lorg/kwis/msp/lwc/Component;", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("ncomp", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("cmpFocus", "Lorg/kwis/msp/lwc/Component;", FieldAccessFlags::PROTECTED),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::<init>({this:?})");
        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;
        let array = jvm.instantiate_array("Lorg/kwis/msp/lwc/Component;", 0).await?;
        jvm.put_field(
            &mut this,
            "org/kwis/msp/lwc/ContainerComponent",
            "cmps",
            "[Lorg/kwis/msp/lwc/Component;",
            array,
        )
        .await
    }

    pub async fn children<T>(jvm: &Jvm, this: &ClassInstanceRef<T>) -> JvmResult<Vec<ClassInstanceRef<Component>>> {
        let array: ClassInstanceRef<Array<Component>> = jvm
            .get_field(this, "org/kwis/msp/lwc/ContainerComponent", "cmps", "[Lorg/kwis/msp/lwc/Component;")
            .await?;
        let count: i32 = jvm.get_field(this, "org/kwis/msp/lwc/ContainerComponent", "ncomp", "I").await?;
        jvm.load_array(&array, 0, count as usize).await
    }

    pub async fn notify_visibility(jvm: &Jvm, component: ClassInstanceRef<Component>, show: bool) -> JvmResult<()> {
        let mut pending = vec![(component, Some(show))];
        while let Some((mut component, requested)) = pending.pop() {
            let shown = if let Some(show) = requested {
                show
            } else {
                // A previous callback may have removed or reparented a sibling.
                let parent: ClassInstanceRef<Self> = jvm
                    .get_field(
                        &component,
                        "org/kwis/msp/lwc/Component",
                        "parent",
                        "Lorg/kwis/msp/lwc/ContainerComponent;",
                    )
                    .await?;
                !parent.is_null() && jvm.get_field::<bool>(&parent, "org/kwis/msp/lwc/Component", "shown", "Z").await?
            };
            if jvm.get_field::<bool>(&component, "org/kwis/msp/lwc/Component", "shown", "Z").await? == shown {
                continue;
            }
            jvm.put_field(&mut component, "org/kwis/msp/lwc/Component", "shown", "Z", shown).await?;
            let _: bool = jvm
                .invoke_virtual(
                    &component,
                    "org/kwis/msp/lwc/Component",
                    "processEvent",
                    "(IIII)Z",
                    (2, i32::from(shown), 0, 0),
                )
                .await?;
            if jvm.is_instance(&**component, "org/kwis/msp/lwc/ContainerComponent") {
                pending.extend(Self::children(jvm, &component).await?.into_iter().rev().map(|child| (child, None)));
            }
        }
        Ok(())
    }

    async fn store_children(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, children: Vec<ClassInstanceRef<Component>>) -> JvmResult<()> {
        let count = children.len();
        let mut array = jvm.instantiate_array("Lorg/kwis/msp/lwc/Component;", count).await?;
        jvm.store_array(&mut array, 0, children).await?;
        jvm.put_field(
            this,
            "org/kwis/msp/lwc/ContainerComponent",
            "cmps",
            "[Lorg/kwis/msp/lwc/Component;",
            array,
        )
        .await?;
        jvm.put_field(this, "org/kwis/msp/lwc/ContainerComponent", "ncomp", "I", count as i32)
            .await
    }

    async fn add_component(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        mut component: ClassInstanceRef<Component>,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::addComponent({this:?}, {component:?})");
        if component.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "null child").await);
        }
        let parent: ClassInstanceRef<Self> = jvm
            .get_field(
                &component,
                "org/kwis/msp/lwc/Component",
                "parent",
                "Lorg/kwis/msp/lwc/ContainerComponent;",
            )
            .await?;
        if !parent.is_null() {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "child already has a parent").await);
        }
        let mut children = Self::children(jvm, &this).await?;
        let index = children.len() as i32;
        children.push(component.clone());
        Self::store_children(jvm, &mut this, children).await?;
        jvm.put_field(
            &mut component,
            "org/kwis/msp/lwc/Component",
            "parent",
            "Lorg/kwis/msp/lwc/ContainerComponent;",
            this.clone(),
        )
        .await?;
        let focus: ClassInstanceRef<Component> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ContainerComponent", "cmpFocus", "Lorg/kwis/msp/lwc/Component;")
            .await?;
        if focus.is_null()
            && jvm
                .invoke_virtual::<_, bool>(&component, "org/kwis/msp/lwc/Component", "canHandleInput", "()Z", ())
                .await?
        {
            let _: () = jvm
                .invoke_virtual(&component, "org/kwis/msp/lwc/Component", "setFocus", "()V", ())
                .await?;
        }
        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "invalidate", "()V", ()).await?;
        if jvm.get_field::<bool>(&this, "org/kwis/msp/lwc/Component", "shown", "Z").await? {
            Self::notify_visibility(jvm, component, true).await?;
        }
        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "repaint", "()V", ()).await?;
        Ok(index)
    }

    async fn remove_component_index(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, index: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::removeComponent({this:?}, {index})");
        let mut children = Self::children(jvm, &this).await?;
        if index < 0 || index as usize >= children.len() {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "invalid child index").await);
        }
        let mut removed = children.remove(index as usize);
        Self::store_children(jvm, &mut this, children).await?;
        jvm.put_field(
            &mut removed,
            "org/kwis/msp/lwc/Component",
            "parent",
            "Lorg/kwis/msp/lwc/ContainerComponent;",
            ClassInstanceRef::<Self>::from(None),
        )
        .await?;
        let focus: ClassInstanceRef<Component> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ContainerComponent", "cmpFocus", "Lorg/kwis/msp/lwc/Component;")
            .await?;
        if !focus.is_null() && focus.equals(&**removed)? {
            jvm.put_field(
                &mut this,
                "org/kwis/msp/lwc/ContainerComponent",
                "cmpFocus",
                "Lorg/kwis/msp/lwc/Component;",
                ClassInstanceRef::<Component>::from(None),
            )
            .await?;
            let _: bool = jvm
                .invoke_virtual(&removed, "org/kwis/msp/lwc/Component", "processEvent", "(IIII)Z", (1, 0, 0, 0))
                .await?;
        }
        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "invalidate", "()V", ()).await?;
        Self::notify_visibility(jvm, removed, false).await?;
        jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "repaint", "()V", ()).await
    }

    async fn remove_component(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::removeComponent({this:?}, {component:?})");
        if component.is_null() {
            return Ok(());
        }
        for (index, child) in Self::children(jvm, &this).await?.into_iter().enumerate() {
            if child.equals(&**component)? {
                return jvm
                    .invoke_virtual(&this, "org/kwis/msp/lwc/ContainerComponent", "removeComponent", "(I)V", (index as i32,))
                    .await;
            }
        }
        Ok(())
    }

    async fn can_handle_input(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::canHandleInput({this:?})");
        let focus: ClassInstanceRef<Component> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ContainerComponent", "cmpFocus", "Lorg/kwis/msp/lwc/Component;")
            .await?;
        Ok(!focus.is_null())
    }

    async fn focus_notify(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, focused: bool) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::focusNotify({this:?}, {focused})");
        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lwc/Component", "focusNotify", "(Z)V", (focused,))
            .await?;
        let focus: ClassInstanceRef<Component> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ContainerComponent", "cmpFocus", "Lorg/kwis/msp/lwc/Component;")
            .await?;
        if !focus.is_null()
            && jvm
                .invoke_virtual::<_, bool>(&focus, "org/kwis/msp/lwc/Component", "hasFocus", "()Z", ())
                .await?
                != focused
        {
            let _: bool = jvm
                .invoke_virtual(
                    &focus,
                    "org/kwis/msp/lwc/Component",
                    "processEvent",
                    "(IIII)Z",
                    (1, i32::from(focused), 0, 0),
                )
                .await?;
        }
        Ok(())
    }

    async fn key_notify(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, event_type: i32, key: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::keyNotify({this:?}, {event_type}, {key})");
        let focus: ClassInstanceRef<Component> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ContainerComponent", "cmpFocus", "Lorg/kwis/msp/lwc/Component;")
            .await?;
        if focus.is_null() {
            return Ok(false);
        }
        jvm.invoke_virtual(&focus, "org/kwis/msp/lwc/Component", "processEvent", "(IIII)Z", (3, event_type, key, 0))
            .await
    }

    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::paint({this:?}, {graphics:?})");
        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "validate", "()V", ()).await?;
        let _: () = jvm
            .invoke_virtual(
                &this,
                "org/kwis/msp/lwc/Component",
                "paintContent",
                "(Lorg/kwis/msp/lcdui/Graphics;)V",
                (graphics.clone(),),
            )
            .await?;
        let clip_x: i32 = jvm
            .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "getClipX", "()I", ())
            .await?;
        let clip_y: i32 = jvm
            .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "getClipY", "()I", ())
            .await?;
        let clip_w: i32 = jvm
            .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "getClipWidth", "()I", ())
            .await?;
        let clip_h: i32 = jvm
            .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "getClipHeight", "()I", ())
            .await?;
        for child in Self::children(jvm, &this).await? {
            let x: i32 = jvm.get_field(&child, "org/kwis/msp/lwc/Component", "x", "I").await?;
            let y: i32 = jvm.get_field(&child, "org/kwis/msp/lwc/Component", "y", "I").await?;
            let width: i32 = jvm.get_field(&child, "org/kwis/msp/lwc/Component", "w", "I").await?;
            let height: i32 = jvm.get_field(&child, "org/kwis/msp/lwc/Component", "h", "I").await?;
            if width <= 0 || height <= 0 {
                continue;
            }
            let _: () = jvm
                .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "translate", "(II)V", (x, y))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "clipRect", "(IIII)V", (0, 0, width, height))
                .await?;
            let (class, method) = if jvm.is_instance(&**child, "org/kwis/msp/lwc/ContainerComponent") {
                ("org/kwis/msp/lwc/ContainerComponent", "paint")
            } else {
                ("org/kwis/msp/lwc/Component", "paintContent")
            };
            let _: () = jvm
                .invoke_virtual(&child, class, method, "(Lorg/kwis/msp/lcdui/Graphics;)V", (graphics.clone(),))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "org/kwis/msp/lcdui/Graphics", "translate", "(II)V", (-x, -y))
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "org/kwis/msp/lcdui/Graphics",
                    "setClip",
                    "(IIII)V",
                    (clip_x, clip_y, clip_w, clip_h),
                )
                .await?;
        }
        Ok(())
    }
}
