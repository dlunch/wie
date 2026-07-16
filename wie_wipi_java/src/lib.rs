#![no_std]

extern crate alloc;

pub mod classes;

use wie_jvm_support::WieJavaClassProto;

pub fn get_protos() -> [WieJavaClassProto; 39] {
    [
        crate::classes::org::kwis::msf::io::Network::as_proto(),
        crate::classes::org::kwis::msf::io::SchemeNotFoundException::as_proto(),
        crate::classes::org::kwis::msp::db::DataBase::as_proto(),
        crate::classes::org::kwis::msp::db::DataComparator::as_proto(),
        crate::classes::org::kwis::msp::db::DataFilter::as_proto(),
        crate::classes::org::kwis::msp::db::DataBaseException::as_proto(),
        crate::classes::org::kwis::msp::db::DataBaseRecordException::as_proto(),
        crate::classes::org::kwis::msp::handset::BackLight::as_proto(),
        crate::classes::org::kwis::msp::handset::HandsetProperty::as_proto(),
        crate::classes::org::kwis::msp::io::File::as_proto(),
        crate::classes::org::kwis::msp::io::FileSystem::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Card::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Display::as_proto(),
        crate::classes::org::kwis::msp::lcdui::EventQueue::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Font::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Graphics::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Image::as_proto(),
        crate::classes::org::kwis::msp::lcdui::ImageObserver::as_proto(),
        crate::classes::org::kwis::msp::lcdui::InputMethodHandler::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Main::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Jlet::as_proto(),
        crate::classes::org::kwis::msp::lcdui::JletEventListener::as_proto(),
        crate::classes::org::kwis::msp::lwc::Component::as_proto(),
        crate::classes::org::kwis::msp::lwc::ContainerComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::EventListener::as_proto(),
        crate::classes::org::kwis::msp::lwc::ShellComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::AnnunciatorComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::TextComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::TextBoxComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::TextFieldComponent::as_proto(),
        crate::classes::org::kwis::msp::media::BaseClip::as_proto(),
        crate::classes::org::kwis::msp::media::Clip::as_proto(),
        crate::classes::org::kwis::msp::media::Player::as_proto(),
        crate::classes::org::kwis::msp::media::PlayListener::as_proto(),
        crate::classes::org::kwis::msp::media::Vibrator::as_proto(),
        crate::classes::org::kwis::msp::media::Volume::as_proto(),
        crate::classes::net::wie::CardCanvas::as_proto(),
        crate::classes::net::wie::WIPIFileOutputStream::as_proto(),
        crate::classes::net::wie::WIPIMIDlet::as_proto(),
    ]
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, collections::BTreeSet, string::String, vec, vec::Vec};

    use java_class_proto::{JavaFieldProto, JavaMethodProto};
    use java_constants::{ClassAccessFlags, MethodAccessFlags};
    use java_runtime::classes::java::lang::{Class, ClassLoader};
    use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_jvm_support::WieJvmContext;
    use wie_util::Result;

    use crate::classes::org::kwis::msp::lcdui::Image;

    use super::{WieJavaClassProto, get_protos};

    const INTERFACE_CALLER_CLASS_HEX: &str = concat!(
        "cafebabe0000003400230a000200030700040c000500060100106a6176612f6c616e672f4f626a6563740100063c696e",
        "69743e0100032829560b0008000907000a0c000b000c01001e6f72672f6b7769732f6d73702f64622f44617461436f6d",
        "70617261746f72010007636f6d70617265010007285b425b4229490b000e000f0700100c0011001201001a6f72672f6b",
        "7769732f6d73702f64622f4461746146696c74657201000666696c746572010005285b42295a0b001400150700160c00",
        "1700180100206f72672f6b7769732f6d73702f6c636475692f496d6167654f627365727665720100066e6f7469667901",
        "001e284c6f72672f6b7769732f6d73702f6c636475692f496d6167653b49295607001a010014746573742f496e746572",
        "6661636543616c6c6572010004436f646501000f4c696e654e756d6265725461626c65010027284c6f72672f6b776973",
        "2f6d73702f64622f44617461436f6d70617261746f723b5b425b422949010021284c6f72672f6b7769732f6d73702f64",
        "622f4461746146696c7465723b5b42295a01000e6e6f746966794f62736572766572010040284c6f72672f6b7769732f",
        "6d73702f6c636475692f496d6167654f627365727665723b4c6f72672f6b7769732f6d73702f6c636475692f496d6167",
        "653b49295601000a536f7572636546696c65010014496e7465726661636543616c6c65722e6a61766100310019000200",
        "00000000040001000500060001001b0000001d00010001000000052ab70001b100000001001c00000006000100000008",
        "0009000b001d0001001b0000002100030003000000092a2b2cb900070300ac00000001001c0000000600010000000a00",
        "090011001e0001001b0000002000020002000000082a2bb9000d0200ac00000001001c0000000600010000000e000900",
        "1f00200001001b0000002500030003000000092a2b1cb900130300b100000001001c0000000a00020000001200080013",
        "00010021000000020022",
    );

    struct InterfaceCallbacks;

    impl InterfaceCallbacks {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/InterfaceCallbacks",
                parent_class: Some("java/lang/Object"),
                interfaces: vec![
                    "org/kwis/msp/db/DataComparator",
                    "org/kwis/msp/db/DataFilter",
                    "org/kwis/msp/lcdui/ImageObserver",
                ],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                    JavaMethodProto::new("compare", "([B[B)I", Self::compare, Default::default()),
                    JavaMethodProto::new("filter", "([B)Z", Self::filter, Default::default()),
                    JavaMethodProto::new("notify", "(Lorg/kwis/msp/lcdui/Image;I)V", Self::notify, Default::default()),
                ],
                fields: vec![
                    JavaFieldProto::new("notifiedImage", "Lorg/kwis/msp/lcdui/Image;", Default::default()),
                    JavaFieldProto::new("notifiedStatus", "I", Default::default()),
                ],
                access_flags: Default::default(),
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
        }

        async fn compare(
            jvm: &Jvm,
            _: &mut WieJvmContext,
            _: ClassInstanceRef<Self>,
            first: ClassInstanceRef<Array<i8>>,
            second: ClassInstanceRef<Array<i8>>,
        ) -> JvmResult<i32> {
            Ok((jvm.array_length(&first).await? * 10 + jvm.array_length(&second).await?) as i32)
        }

        async fn filter(jvm: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>, data: ClassInstanceRef<Array<i8>>) -> JvmResult<bool> {
            let value = jvm.load_array::<i8>(&data, 0, 1).await?;
            Ok(value == [42])
        }

        async fn notify(
            jvm: &Jvm,
            _: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            image: ClassInstanceRef<Image>,
            status: i32,
        ) -> JvmResult<()> {
            jvm.put_field(&mut this, "notifiedImage", "Lorg/kwis/msp/lcdui/Image;", image).await?;
            jvm.put_field(&mut this, "notifiedStatus", "I", status).await
        }
    }

    fn assert_methods(
        protos: &[WieJavaClassProto],
        class_name: &str,
        expected: &[(&str, &str, bool, bool)],
    ) -> BTreeSet<(String, String, String, u16)> {
        let class = protos
            .iter()
            .find(|proto| proto.name == class_name)
            .unwrap_or_else(|| panic!("missing class {class_name}"));

        let mut matched_methods = BTreeSet::new();
        for (name, descriptor, is_static, is_protected) in expected {
            let mut matches = class
                .methods
                .iter()
                .filter(|method| method.name == *name && method.descriptor == *descriptor);
            let method = matches.next().unwrap_or_else(|| panic!("missing method {class_name}.{name}{descriptor}"));
            assert!(matches.next().is_none(), "duplicate method {class_name}.{name}{descriptor}");
            let relevant_flags = method.access_flags & (MethodAccessFlags::STATIC | MethodAccessFlags::PROTECTED);
            assert!(
                matched_methods.insert((
                    String::from(class_name),
                    String::from(*name),
                    String::from(*descriptor),
                    relevant_flags.bits(),
                )),
                "duplicate selected method tuple {class_name}.{name}{descriptor}",
            );

            assert_eq!(
                method.access_flags.contains(MethodAccessFlags::STATIC),
                *is_static,
                "wrong static flag for {class_name}.{name}{descriptor}",
            );
            assert_eq!(
                method.access_flags.contains(MethodAccessFlags::PROTECTED),
                *is_protected,
                "wrong protected flag for {class_name}.{name}{descriptor}",
            );
        }

        matched_methods
    }

    #[test]
    fn selected_wipi_api_prototypes_are_complete() {
        let protos = get_protos();
        let mut actual_methods = BTreeSet::new();

        actual_methods.extend(assert_methods(
            &protos,
            "org/kwis/msp/db/DataBase",
            &[
                ("deleteDataBase", "(Ljava/lang/String;I)V", true, false),
                ("deleteRecord", "(I)V", false, false),
                ("selectRecord", "(I[BI)V", false, false),
                (
                    "sortRecord",
                    "(Lorg/kwis/msp/db/DataFilter;Lorg/kwis/msp/db/DataComparator;)[I",
                    false,
                    false,
                ),
                ("listDataBases", "()[Ljava/lang/String;", true, false),
                ("getAccessMode", "(Ljava/lang/String;)I", true, false),
                ("getDataBaseName", "()Ljava/lang/String;", false, false),
                ("getDataBaseSize", "()I", false, false),
                ("getRecordSize", "()I", false, false),
                ("getSizeAvailable", "()I", false, false),
                ("getLastModified", "()J", false, false),
            ],
        ));
        actual_methods.extend(assert_methods(
            &protos,
            "org/kwis/msp/handset/BackLight",
            &[("on", "(III)V", true, false), ("off", "()V", true, false), ("before", "()V", true, false)],
        ));
        actual_methods.extend(assert_methods(
            &protos,
            "org/kwis/msp/handset/HandsetProperty",
            &[("setSystemProperty", "(Ljava/lang/String;Ljava/lang/String;)Z", true, false)],
        ));
        actual_methods.extend(assert_methods(
            &protos,
            "org/kwis/msp/io/File",
            &[
                ("openOutputStream", "()Ljava/io/OutputStream;", false, false),
                ("openDataOutputStream", "()Ljava/io/DataOutputStream;", false, false),
                ("write", "(I)I", false, false),
                ("read", "()I", false, false),
                ("tell", "()I", false, false),
            ],
        ));
        actual_methods.extend(assert_methods(
            &protos,
            "org/kwis/msp/io/FileSystem",
            &[
                ("<init>", "()V", false, false),
                ("getMaxFilenameLength", "()I", true, false),
                ("list", "(Ljava/lang/String;)Ljava/util/Vector;", true, false),
                ("list", "(Ljava/lang/String;I)Ljava/util/Vector;", true, false),
                ("remove", "(Ljava/lang/String;)V", true, false),
                ("remove", "(Ljava/lang/String;I)V", true, false),
                ("mkdir", "(Ljava/lang/String;)V", true, false),
                ("rmdir", "(Ljava/lang/String;)V", true, false),
                ("rmdir", "(Ljava/lang/String;I)V", true, false),
                ("toCString", "(Ljava/lang/String;)[B", true, false),
                ("isFile", "(Ljava/lang/String;I)Z", true, false),
                ("isDirectory", "(Ljava/lang/String;)Z", true, false),
                ("getCreationTime", "(Ljava/lang/String;)I", true, false),
                ("getCreationTime", "(Ljava/lang/String;I)I", true, false),
                ("rename", "(Ljava/lang/String;Ljava/lang/String;)V", true, false),
                ("rename", "(Ljava/lang/String;Ljava/lang/String;I)V", true, false),
            ],
        ));
        actual_methods.extend(assert_methods(
            &protos,
            "org/kwis/msp/lcdui/Card",
            &[
                ("<init>", "(Z)V", false, false),
                ("<init>", "(IIII)V", false, false),
                ("<init>", "(Lorg/kwis/msp/lcdui/Display;IIII)V", false, false),
                ("<init>", "(Lorg/kwis/msp/lcdui/Display;IIIIZ)V", false, false),
                ("move", "(II)V", false, false),
                ("resize", "(II)V", false, false),
                ("getX", "()I", false, false),
                ("getY", "()I", false, false),
                ("pointerNotify", "(III)Z", false, true),
                ("getDisplay", "()Lorg/kwis/msp/lcdui/Display;", false, false),
            ],
        ));
        actual_methods.extend(assert_methods(
            &protos,
            "org/kwis/msp/lcdui/Display",
            &[
                ("popCard", "()Lorg/kwis/msp/lcdui/Card;", false, false),
                ("removeCard", "(Lorg/kwis/msp/lcdui/Card;)Z", false, false),
                ("countCard", "()I", false, false),
                ("callSerially", "(Ljava/lang/Runnable;I)V", false, false),
                ("setDockedCard", "(Lorg/kwis/msp/lcdui/Card;I)V", false, false),
                ("isColor", "()Z", false, false),
                ("numColors", "()I", false, false),
                ("hasPointerEvents", "()Z", false, false),
                ("hasPointerMotionEvents", "()Z", false, false),
                ("hasRepeatEvents", "()Z", false, false),
                ("getKeyName", "(I)Ljava/lang/String;", true, false),
                ("getBitsPerPixel", "()I", false, false),
                ("flush", "()V", false, false),
                ("removeJletEventListener", "(Lorg/kwis/msp/lcdui/JletEventListener;)V", false, false),
                ("grabKey", "(ILorg/kwis/msp/lcdui/JletEventListener;)V", false, false),
                ("ungrabKey", "(I)V", false, false),
            ],
        ));
        actual_methods.extend(assert_methods(
            &protos,
            "org/kwis/msp/lcdui/EventQueue",
            &[("postEvent", "([I)Z", false, false), ("postEvent", "(I[I)V", true, false)],
        ));
        actual_methods.extend(assert_methods(
            &protos,
            "org/kwis/msp/lcdui/Font",
            &[
                ("getBaselinePosition", "()I", false, false),
                ("getFace", "()I", false, false),
                ("getSize", "()I", false, false),
                ("getStyle", "()I", false, false),
                ("isBold", "()Z", false, false),
                ("isItalic", "()Z", false, false),
                ("isPlain", "()Z", false, false),
                ("isUnderlined", "()Z", false, false),
            ],
        ));
        actual_methods.extend(assert_methods(
            &protos,
            "org/kwis/msp/lcdui/Graphics",
            &[
                ("fillPolygon", "([I[I)V", false, false),
                ("getBlueComponent", "()I", false, false),
                ("getGrayScale", "()I", false, false),
                ("getGreenComponent", "()I", false, false),
                ("getRedComponent", "()I", false, false),
                ("getStrokeStyle", "()I", false, false),
                ("setStrokeStyle", "(I)V", false, false),
                ("getPixel", "(II)I", false, false),
                ("getPixels", "(IIII[BII)V", false, false),
                ("setPixels", "(IIII[BII)V", false, false),
                ("drawPolygon", "([I[I)V", false, false),
                ("reset", "()V", false, false),
                ("getAlpha", "()I", false, false),
                ("isXORMode", "()Z", false, false),
            ],
        ));
        actual_methods.extend(assert_methods(
            &protos,
            "org/kwis/msp/lcdui/Image",
            &[
                ("<init>", "()V", false, true),
                (
                    "loadImage",
                    "(Ljava/lang/String;Lorg/kwis/msp/lcdui/ImageObserver;)Lorg/kwis/msp/lcdui/Image;",
                    true,
                    false,
                ),
                ("isMutable", "()Z", false, false),
                ("isAnimated", "()Z", false, false),
                ("play", "(Lorg/kwis/msp/lcdui/ImageObserver;)V", false, false),
                ("stop", "()V", false, false),
                ("stopImage", "(Lorg/kwis/msp/lcdui/ImageObserver;)V", true, false),
                ("drawImage", "(Lorg/kwis/msp/lcdui/Image;IIIIIIII)V", false, false),
                ("createSubImage", "(IIIIZ)Lorg/kwis/msp/lcdui/Image;", false, false),
                ("setTransparentColor", "(I)V", false, false),
            ],
        ));
        actual_methods.extend(assert_methods(
            &protos,
            "org/kwis/msp/media/Player",
            &[
                ("pause", "(Lorg/kwis/msp/media/BaseClip;)Z", true, false),
                ("stop", "(Lorg/kwis/msp/media/BaseClip;)Z", true, false),
                ("resume", "(Lorg/kwis/msp/media/BaseClip;)Z", true, false),
                ("play", "(Lorg/kwis/msp/media/BaseClip;Z)Z", true, false),
                ("record", "(Lorg/kwis/msp/media/BaseClip;)Z", true, false),
            ],
        ));
        actual_methods.extend(assert_methods(&protos, "org/kwis/msp/media/Vibrator", &[("off", "()V", true, false)]));
        actual_methods.extend(assert_methods(
            &protos,
            "org/kwis/msp/media/Volume",
            &[
                ("setMute", "(IZ)V", true, false),
                ("getMute", "(I)Z", true, false),
                ("setDefaultVolume", "(II)Z", true, false),
                ("getDefaultVolume", "(I)I", true, false),
            ],
        ));
        actual_methods.extend(assert_methods(
            &protos,
            "org/kwis/msp/media/Clip",
            &[
                ("getType", "()Ljava/lang/String;", false, false),
                ("setPosition", "(I)Z", false, false),
                ("getPosition", "()I", false, false),
                ("setStopTime", "(I)Z", false, false),
                ("getStopTime", "()I", false, false),
                ("getVolume", "()I", false, false),
            ],
        ));

        assert_eq!(actual_methods.len(), 112);
    }

    #[test]
    fn support_interfaces_expose_abstract_callbacks() {
        let protos = get_protos();
        let expected = [
            ("org/kwis/msp/db/DataComparator", "compare", "([B[B)I"),
            ("org/kwis/msp/db/DataFilter", "filter", "([B)Z"),
            ("org/kwis/msp/lcdui/ImageObserver", "notify", "(Lorg/kwis/msp/lcdui/Image;I)V"),
        ];

        for (class_name, method_name, descriptor) in expected {
            let class = protos
                .iter()
                .find(|proto| proto.name == class_name)
                .unwrap_or_else(|| panic!("missing interface {class_name}"));
            assert!(class.access_flags.contains(ClassAccessFlags::INTERFACE));

            let method = class
                .methods
                .iter()
                .find(|method| method.name == method_name && method.descriptor == descriptor)
                .unwrap_or_else(|| panic!("missing callback {class_name}.{method_name}{descriptor}"));
            assert!(method.access_flags.contains(MethodAccessFlags::ABSTRACT));
        }
    }

    #[test]
    fn support_callbacks_dispatch_through_invokeinterface_bytecode() -> Result<()> {
        let fixture: Box<[WieJavaClassProto]> = Vec::from([InterfaceCallbacks::as_proto()]).into_boxed_slice();
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), fixture]),
            |jvm| async move {
                let class_data: Vec<i8> = INTERFACE_CALLER_CLASS_HEX
                    .as_bytes()
                    .chunks_exact(2)
                    .map(|digits| u8::from_str_radix(core::str::from_utf8(digits).unwrap(), 16).unwrap() as i8)
                    .collect();
                let mut class_bytes = jvm.instantiate_array("B", class_data.len()).await?;
                jvm.store_array(&mut class_bytes, 0, class_data).await?;

                let loader: ClassInstanceRef<ClassLoader> = jvm
                    .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", ())
                    .await?;
                let class_name = JavaLangString::from_rust_string(&jvm, "test.InterfaceCaller").await?;
                let _: ClassInstanceRef<Class> = jvm
                    .invoke_virtual(
                        &loader,
                        "defineClass",
                        "(Ljava/lang/String;[BII)Ljava/lang/Class;",
                        (class_name, class_bytes, 0, (INTERFACE_CALLER_CLASS_HEX.len() / 2) as i32),
                    )
                    .await?;

                let callbacks: ClassInstanceRef<InterfaceCallbacks> = jvm.new_class("test/InterfaceCallbacks", "()V", ()).await?.into();
                let first = jvm.instantiate_array("B", 2).await?;
                let mut second = jvm.instantiate_array("B", 1).await?;
                jvm.store_array(&mut second, 0, [42i8]).await?;

                let compared: i32 = jvm
                    .invoke_static(
                        "test/InterfaceCaller",
                        "compare",
                        "(Lorg/kwis/msp/db/DataComparator;[B[B)I",
                        (callbacks.clone(), first, second.clone()),
                    )
                    .await?;
                let filtered: bool = jvm
                    .invoke_static(
                        "test/InterfaceCaller",
                        "filter",
                        "(Lorg/kwis/msp/db/DataFilter;[B)Z",
                        (callbacks.clone(), second),
                    )
                    .await?;
                let image: ClassInstanceRef<Image> = jvm
                    .invoke_static("org/kwis/msp/lcdui/Image", "createImage", "(II)Lorg/kwis/msp/lcdui/Image;", (1, 1))
                    .await?;
                let _: () = jvm
                    .invoke_static(
                        "test/InterfaceCaller",
                        "notifyObserver",
                        "(Lorg/kwis/msp/lcdui/ImageObserver;Lorg/kwis/msp/lcdui/Image;I)V",
                        (callbacks.clone(), image.clone(), 77),
                    )
                    .await?;

                let notified_image: ClassInstanceRef<Image> = jvm.get_field(&callbacks, "notifiedImage", "Lorg/kwis/msp/lcdui/Image;").await?;
                let notified_status: i32 = jvm.get_field(&callbacks, "notifiedStatus", "I").await?;
                let same_image: bool = jvm.invoke_virtual(&notified_image, "equals", "(Ljava/lang/Object;)Z", (image,)).await?;

                assert_eq!(compared, 21);
                assert!(filtered);
                assert!(same_image);
                assert_eq!(notified_status, 77);

                Ok(())
            },
        )
    }
}
