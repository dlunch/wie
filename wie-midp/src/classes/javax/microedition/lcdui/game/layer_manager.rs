use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::util::Vector;

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::lcdui::{Graphics, game::Layer};

// class javax.microedition.lcdui.game.LayerManager
#[java_class(name = "javax/microedition/lcdui/game/LayerManager", public)]
pub struct LayerManager {
    #[field(private)]
    layers: ClassInstanceRef<Vector>,
    #[field(private)]
    viewX: i32,
    #[field(private)]
    viewY: i32,
    #[field(private)]
    viewWidth: i32,
    #[field(private)]
    viewHeight: i32,
}

#[java_methods]
impl LayerManager {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.LayerManager::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let layers = jvm.new_class("java/util/Vector", "()V", ()).await?;
        jvm.put_field(
            &mut this,
            "javax/microedition/lcdui/game/LayerManager",
            "layers",
            "Ljava/util/Vector;",
            layers,
        )
        .await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/game/LayerManager", "viewWidth", "I", i32::MAX)
            .await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/game/LayerManager", "viewHeight", "I", i32::MAX)
            .await?;

        Ok(())
    }

    #[method(public)]
    async fn append(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, layer: ClassInstanceRef<Layer>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.LayerManager::append({this:?}, {layer:?})");

        // A layer that is already managed moves to the bottom.
        let _: () = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/game/LayerManager",
                "remove",
                "(Ljavax/microedition/lcdui/game/Layer;)V",
                (layer.clone(),),
            )
            .await?;
        let layers = Self::layers(jvm, &this).await?;
        jvm.invoke_virtual(&layers, "java/util/Vector", "addElement", "(Ljava/lang/Object;)V", (layer,))
            .await
    }

    #[method(public)]
    async fn insert(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, layer: ClassInstanceRef<Layer>, index: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.LayerManager::insert({this:?}, {layer:?}, {index})");

        let layers = Self::layers(jvm, &this).await?;
        let size: i32 = jvm.invoke_virtual(&layers, "java/util/Vector", "size", "()I", ()).await?;
        if index < 0 || index > size {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid layer index").await);
        }

        let _: () = jvm
            .invoke_virtual(
                &this,
                "javax/microedition/lcdui/game/LayerManager",
                "remove",
                "(Ljavax/microedition/lcdui/game/Layer;)V",
                (layer.clone(),),
            )
            .await?;
        // Removing an already managed layer can shorten the list below the requested index.
        let size: i32 = jvm.invoke_virtual(&layers, "java/util/Vector", "size", "()I", ()).await?;
        jvm.invoke_virtual(
            &layers,
            "java/util/Vector",
            "insertElementAt",
            "(Ljava/lang/Object;I)V",
            (layer, index.min(size)),
        )
        .await
    }

    #[method(public)]
    async fn get_layer_at(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<Layer>> {
        tracing::debug!("javax.microedition.lcdui.game.LayerManager::getLayerAt({this:?}, {index})");

        let layers = Self::layers(jvm, &this).await?;
        let size: i32 = jvm.invoke_virtual(&layers, "java/util/Vector", "size", "()I", ()).await?;
        if index < 0 || index >= size {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid layer index").await);
        }

        jvm.invoke_virtual(&layers, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
            .await
    }

    #[method(public)]
    async fn get_size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.game.LayerManager::getSize({this:?})");

        let layers = Self::layers(jvm, &this).await?;
        jvm.invoke_virtual(&layers, "java/util/Vector", "size", "()I", ()).await
    }

    #[method(public)]
    async fn remove(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, layer: ClassInstanceRef<Layer>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.LayerManager::remove({this:?}, {layer:?})");

        if layer.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "layer is null").await);
        }

        let layers = Self::layers(jvm, &this).await?;
        let _: bool = jvm
            .invoke_virtual(&layers, "java/util/Vector", "removeElement", "(Ljava/lang/Object;)Z", (layer,))
            .await?;

        Ok(())
    }

    #[method(public)]
    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.LayerManager::paint({this:?}, {g:?}, {x}, {y})");

        if g.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "g is null").await);
        }

        let view_x: i32 = jvm.get_field(&this, "javax/microedition/lcdui/game/LayerManager", "viewX", "I").await?;
        let view_y: i32 = jvm.get_field(&this, "javax/microedition/lcdui/game/LayerManager", "viewY", "I").await?;
        let view_width: i32 = jvm
            .get_field(&this, "javax/microedition/lcdui/game/LayerManager", "viewWidth", "I")
            .await?;
        let view_height: i32 = jvm
            .get_field(&this, "javax/microedition/lcdui/game/LayerManager", "viewHeight", "I")
            .await?;

        let mut clip = [0; 4];
        for (value, getter) in clip.iter_mut().zip(["getClipX", "getClipY", "getClipWidth", "getClipHeight"]) {
            *value = jvm.invoke_virtual(&g, "javax/microedition/lcdui/Graphics", getter, "()I", ()).await?;
        }

        // Layers paint in their own coordinate system, with the view window origin placed at (x, y).
        let (dx, dy) = (x.wrapping_sub(view_x), y.wrapping_sub(view_y));
        let _: () = jvm
            .invoke_virtual(&g, "javax/microedition/lcdui/Graphics", "translate", "(II)V", (dx, dy))
            .await?;
        let _: () = jvm
            .invoke_virtual(
                &g,
                "javax/microedition/lcdui/Graphics",
                "clipRect",
                "(IIII)V",
                (view_x, view_y, view_width, view_height),
            )
            .await?;

        let layers = Self::layers(jvm, &this).await?;
        let size: i32 = jvm.invoke_virtual(&layers, "java/util/Vector", "size", "()I", ()).await?;
        for index in (0..size).rev() {
            let layer: ClassInstanceRef<Layer> = jvm
                .invoke_virtual(&layers, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
                .await?;
            let visible: bool = jvm
                .invoke_virtual(&layer, "javax/microedition/lcdui/game/Layer", "isVisible", "()Z", ())
                .await?;
            if visible {
                let _: () = jvm
                    .invoke_virtual(
                        &layer,
                        "javax/microedition/lcdui/game/Layer",
                        "paint",
                        "(Ljavax/microedition/lcdui/Graphics;)V",
                        (g.clone(),),
                    )
                    .await?;
            }
        }

        let _: () = jvm
            .invoke_virtual(
                &g,
                "javax/microedition/lcdui/Graphics",
                "translate",
                "(II)V",
                (dx.wrapping_neg(), dy.wrapping_neg()),
            )
            .await?;
        jvm.invoke_virtual(
            &g,
            "javax/microedition/lcdui/Graphics",
            "setClip",
            "(IIII)V",
            (clip[0], clip[1], clip[2], clip[3]),
        )
        .await
    }

    #[method(public)]
    async fn set_view_window(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.LayerManager::setViewWindow({this:?}, {x}, {y}, {width}, {height})");

        if width < 0 || height < 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Negative view window size").await);
        }

        for (field, value) in [("viewX", x), ("viewY", y), ("viewWidth", width), ("viewHeight", height)] {
            jvm.put_field(&mut this, "javax/microedition/lcdui/game/LayerManager", field, "I", value)
                .await?;
        }

        Ok(())
    }

    async fn layers(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Vector>> {
        jvm.get_field(this, "javax/microedition/lcdui/game/LayerManager", "layers", "Ljava/util/Vector;")
            .await
    }
}
