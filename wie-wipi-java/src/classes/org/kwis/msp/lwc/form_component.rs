use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;
use wie_midp::classes::net::wie::KeyboardEventType;

use crate::classes::{
    net::wie::WIPIKeyCode,
    org::kwis::msp::lwc::{Component, ContainerComponent},
};

// class org.kwis.msp.lwc.FormComponent
#[java_class(name = "org/kwis/msp/lwc/FormComponent", extends = "org/kwis/msp/lwc/ContainerComponent", public)]
pub struct FormComponent {
    #[field(private)]
    vertical: bool,
}

#[java_methods]
impl FormComponent {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.FormComponent::<init>({this:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/FormComponent", "<init>", "(Z)V", (true,))
            .await
    }

    #[method(constructor, public)]
    async fn init_with_orientation(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, vertical: bool) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.FormComponent::<init>({this:?}, {vertical})");
        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lwc/ContainerComponent", "<init>", "()V", ())
            .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/FormComponent", "vertical", "Z", vertical)
            .await
    }

    #[method(protected)]
    async fn calc_preferred_size(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, width: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.FormComponent::calcPreferredSize({this:?}, {width})");
        let vertical: bool = jvm.get_field(&this, "org/kwis/msp/lwc/FormComponent", "vertical", "Z").await?;
        let (mut preferred_width, mut preferred_height) = (0, 0);
        for child in ContainerComponent::children(jvm, &this).await? {
            let w: i32 = jvm
                .invoke_virtual(&child, "org/kwis/msp/lwc/Component", "getPreferredWidth", "()I", ())
                .await?;
            let available = if vertical && width >= 0 { width } else { w };
            let h: i32 = jvm
                .invoke_virtual(&child, "org/kwis/msp/lwc/Component", "getPreferredHeight", "(I)I", (available,))
                .await?;
            if vertical {
                preferred_width = preferred_width.max(w);
                preferred_height += h;
            } else {
                preferred_width += w;
                preferred_height = preferred_height.max(h);
            }
        }
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "prefW", "I", preferred_width)
            .await?;
        jvm.put_field(&mut this, "org/kwis/msp/lwc/Component", "prefH", "I", preferred_height)
            .await
    }

    #[method(public)]
    async fn layout(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.FormComponent::layout({this:?})");
        let vertical: bool = jvm.get_field(&this, "org/kwis/msp/lwc/FormComponent", "vertical", "Z").await?;
        let width: i32 = jvm.get_field(&this, "org/kwis/msp/lwc/Component", "w", "I").await?;
        let children = ContainerComponent::children(jvm, &this).await?;
        let (mut x, mut y) = (0, 0);
        for child in children {
            let w: i32 = if vertical {
                width
            } else {
                jvm.invoke_virtual(&child, "org/kwis/msp/lwc/Component", "getPreferredWidth", "()I", ())
                    .await?
            };
            let h: i32 = jvm
                .invoke_virtual(&child, "org/kwis/msp/lwc/Component", "getPreferredHeight", "(I)I", (w,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&child, "org/kwis/msp/lwc/Component", "configure", "(IIIII)V", (x, y, w, h, 3))
                .await?;
            if vertical {
                y += h;
            } else {
                x += w;
            }
        }
        Ok(())
    }

    #[method(protected)]
    async fn key_notify(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, kind: i32, key: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.FormComponent::keyNotify({this:?}, {kind}, {key})");
        if jvm
            .invoke_special::<_, bool>(&this, "org/kwis/msp/lwc/ContainerComponent", "keyNotify", "(II)Z", (kind, key))
            .await?
        {
            return Ok(true);
        }
        if !matches!(
            KeyboardEventType::from_raw(kind),
            Some(KeyboardEventType::KeyPressed | KeyboardEventType::KeyRepeated)
        ) {
            return Ok(false);
        }
        let backwards = match WIPIKeyCode::from_raw(key) {
            Some(WIPIKeyCode::UP) => true,
            Some(WIPIKeyCode::DOWN) => false,
            _ => return Ok(false),
        };
        let mut children = ContainerComponent::children(jvm, &this).await?;
        if backwards {
            children.reverse();
        }
        let focus: ClassInstanceRef<Component> = jvm
            .get_field(&this, "org/kwis/msp/lwc/ContainerComponent", "cmpFocus", "Lorg/kwis/msp/lwc/Component;")
            .await?;
        let mut past_focus = focus.is_null();
        for child in children {
            if !past_focus {
                past_focus = child.equals(&**focus)?;
            } else if jvm
                .invoke_virtual::<_, bool>(&child, "org/kwis/msp/lwc/Component", "canHandleInput", "()Z", ())
                .await?
            {
                let _: () = jvm.invoke_virtual(&child, "org/kwis/msp/lwc/Component", "setFocus", "()V", ()).await?;
                return Ok(true);
            }
        }
        Ok(false)
    }
}
