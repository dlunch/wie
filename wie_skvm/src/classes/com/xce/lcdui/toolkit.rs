use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{
    ClassInstanceRef, Jvm, Result as JvmResult,
    runtime::{JavaIoInputStream, JavaLangClassLoader, JavaLangString},
};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::{
    lcdui::{Display, Font, Graphics, Image},
    midlet::MIDlet,
};

macro_rules! define_image_accessors {
    ($(($getter:ident, $setter:ident, $field:literal)),+ $(,)?) => {
        $(
            async fn $getter(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
                jvm.get_static_field("com/xce/lcdui/Toolkit", $field, "Ljavax/microedition/lcdui/Image;").await
            }

            async fn $setter(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
                jvm.put_static_field("com/xce/lcdui/Toolkit", $field, "Ljavax/microedition/lcdui/Image;", image).await
            }
        )+
    };
}

// class com.xce.lcdui.Toolkit
pub struct Toolkit;

impl Toolkit {
    pub fn as_proto() -> WieJavaClassProto {
        let mut methods = vec![
            JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
            JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
            JavaMethodProto::new(
                "createImage",
                "(Ljava/lang/String;)Ljavax/microedition/lcdui/Image;",
                Self::create_image,
                MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            ),
            JavaMethodProto::new(
                "createExImage",
                "(Ljava/lang/String;Ljava/lang/String;)Ljavax/microedition/lcdui/Image;",
                Self::create_ex_image,
                MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            ),
            JavaMethodProto::new(
                "splitString",
                "(Ljava/lang/String;I)I",
                Self::split_string,
                MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            ),
            JavaMethodProto::new(
                "paintPopup",
                "(Ljava/lang/String;Ljava/lang/String;)V",
                Self::paint_popup,
                MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            ),
            JavaMethodProto::new(
                "paintPopup",
                "(Ljava/lang/String;Ljava/lang/String;Z)V",
                Self::paint_popup_with_button,
                MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            ),
        ];
        let mut fields = vec![
            JavaFieldProto::new(
                "DEFAULT_FONT",
                "Ljavax/microedition/lcdui/Font;",
                FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
            ),
            JavaFieldProto::new(
                "FONT_HEIGHT",
                "I",
                FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
            ),
            JavaFieldProto::new(
                "FONT_GAP",
                "I",
                FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
            ),
            JavaFieldProto::new(
                "MAX_CHARWIDTH",
                "I",
                FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
            ),
            JavaFieldProto::new(
                "BLACK",
                "I",
                FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
            ),
            JavaFieldProto::new(
                "DK_GRAY",
                "I",
                FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
            ),
            JavaFieldProto::new(
                "LT_GRAY",
                "I",
                FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
            ),
            JavaFieldProto::new(
                "WHITE",
                "I",
                FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
            ),
            JavaFieldProto::new(
                "black",
                "I",
                FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
            ),
            JavaFieldProto::new(
                "dk_gray",
                "I",
                FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
            ),
            JavaFieldProto::new(
                "lt_gray",
                "I",
                FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
            ),
            JavaFieldProto::new(
                "white",
                "I",
                FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
            ),
            JavaFieldProto::new(
                "graphics",
                "Ljavax/microedition/lcdui/Graphics;",
                FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
            ),
            JavaFieldProto::new("IMG_HEIGHT", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
            JavaFieldProto::new("IS_KOREAN", "Z", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
            JavaFieldProto::new("selected", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
            JavaFieldProto::new("ext", "Ljava/lang/String;", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
            JavaFieldProto::new("iconsDir", "Ljava/lang/String;", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
            JavaFieldProto::new("MIDP_RES", "Lcom/xce/lcdui/MIDPRes;", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
        ];

        macro_rules! register_image_accessors {
            ($(($getter_name:literal, $getter:ident, $setter_name:literal, $setter:ident, $field:literal)),+ $(,)?) => {
                $(
                    methods.push(JavaMethodProto::new(
                        $getter_name,
                        "()Ljavax/microedition/lcdui/Image;",
                        Self::$getter,
                        MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                    ));
                    methods.push(JavaMethodProto::new(
                        $setter_name,
                        "(Ljavax/microedition/lcdui/Image;)V",
                        Self::$setter,
                        MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                    ));
                    fields.push(JavaFieldProto::new(
                        $field,
                        "Ljavax/microedition/lcdui/Image;",
                        FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
                    ));
                )+
            };
        }

        register_image_accessors!(
            ("titleImg", title_img, "setTitleImg", set_title_img, "titleImg"),
            (
                "buttonBackLeftImg",
                button_back_left_img,
                "setButtonBackLeftImg",
                set_button_back_left_img,
                "buttonBackLeftImg"
            ),
            (
                "buttonBackRightImg",
                button_back_right_img,
                "setButtonBackRightImg",
                set_button_back_right_img,
                "buttonBackRightImg"
            ),
            ("screenIconImg", screen_icon_img, "setScreenIconImg", set_screen_icon_img, "screenIconImg"),
            ("backIconImg", back_icon_img, "setBackIconImg", set_back_icon_img, "backIconImg"),
            ("cancelIconImg", cancel_icon_img, "setCancelIconImg", set_cancel_icon_img, "cancelIconImg"),
            ("helpIconImg", help_icon_img, "setHelpIconImg", set_help_icon_img, "helpIconImg"),
            ("okIconImg", ok_icon_img, "setOkIconImg", set_ok_icon_img, "okIconImg"),
            ("stopIconImg", stop_icon_img, "setStopIconImg", set_stop_icon_img, "stopIconImg"),
            ("exitIconImg", exit_icon_img, "setExitIconImg", set_exit_icon_img, "exitIconImg"),
            ("itemIconImg", item_icon_img, "setItemIconImg", set_item_icon_img, "itemIconImg"),
            ("menuIconImg", menu_icon_img, "setMenuIconImg", set_menu_icon_img, "menuIconImg"),
            ("ueimImg", ueim_img, "setUEimImg", set_ueim_img, "ueimImg"),
            ("leimImg", leim_img, "setLEimImg", set_leim_img, "leimImg"),
            ("kimImg", kim_img, "setKimImg", set_kim_img, "kimImg"),
            ("simImg", sim_img, "setSimImg", set_sim_img, "simImg"),
            ("nimImg", nim_img, "setNimImg", set_nim_img, "nimImg"),
            ("imHintImg", im_hint_img, "setIMHintImg", set_im_hint_img, "imHintImg"),
            ("sExclusive", s_exclusive, "setSExclusive", set_s_exclusive, "sExclusive"),
            ("uExclusive", u_exclusive, "setUExclusive", set_u_exclusive, "uExclusive"),
            ("sMultiple", s_multiple, "setSMultiple", set_s_multiple, "sMultiple"),
            ("uMultiple", u_multiple, "setUMultiple", set_u_multiple, "uMultiple"),
            ("sBackImg", s_back_img, "setSBackImg", set_s_back_img, "sBackImg"),
            ("gBackImg", g_back_img, "setGBackImg", set_g_back_img, "gBackImg"),
            ("gForeImg", g_fore_img, "setGForeImg", set_g_fore_img, "gForeImg"),
            ("scrollImg", scroll_img, "setScrollImg", set_scroll_img, "scrollImg"),
        );
        methods.push(JavaMethodProto::new(
            "appImg",
            "()Ljavax/microedition/lcdui/Image;",
            Self::app_img,
            MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
        ));
        methods.push(JavaMethodProto::new(
            "castleImg",
            "()Ljavax/microedition/lcdui/Image;",
            Self::castle_img,
            MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
        ));

        WieJavaClassProto {
            name: "com/xce/lcdui/Toolkit",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods,
            fields,
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::<clinit>");

        let font: ClassInstanceRef<Font> = jvm
            .invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
            .await?;
        jvm.put_static_field("com/xce/lcdui/Toolkit", "DEFAULT_FONT", "Ljavax/microedition/lcdui/Font;", font.clone())
            .await?;

        let font_height: i32 = jvm.invoke_virtual(&font, "getHeight", "()I", ()).await?;
        jvm.put_static_field("com/xce/lcdui/Toolkit", "FONT_HEIGHT", "I", font_height).await?;
        jvm.put_static_field("com/xce/lcdui/Toolkit", "FONT_GAP", "I", 0).await?;

        let max_char_width: i32 = jvm.invoke_virtual(&font, "charWidth", "(C)I", ('W' as u16,)).await?;
        jvm.put_static_field("com/xce/lcdui/Toolkit", "MAX_CHARWIDTH", "I", max_char_width)
            .await?;

        for (field, value) in [
            ("BLACK", 0x000000),
            ("DK_GRAY", 0x555555),
            ("LT_GRAY", 0xaaaaaa),
            ("WHITE", 0xffffff),
            ("black", 0x000000),
            ("dk_gray", 0x555555),
            ("lt_gray", 0xaaaaaa),
            ("white", 0xffffff),
        ] {
            jvm.put_static_field("com/xce/lcdui/Toolkit", field, "I", value).await?;
        }

        let current_midlet: ClassInstanceRef<MIDlet> = jvm
            .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
            .await?;

        if !current_midlet.is_null() {
            let display: ClassInstanceRef<Display> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Display",
                    "getDisplay",
                    "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                    (current_midlet,),
                )
                .await?;
            let graphics: ClassInstanceRef<Graphics> = Display::screen_graphics(jvm, &display).await?;

            jvm.put_static_field("com/xce/lcdui/Toolkit", "graphics", "Ljavax/microedition/lcdui/Graphics;", graphics)
                .await?;
        }

        Ok(())
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::<init>({this:?})");
        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    async fn create_image(jvm: &Jvm, _context: &mut WieJvmContext, file_name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::createImage({file_name:?})");

        if file_name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "fileName is null").await);
        }

        let resource_name = JavaLangString::to_rust_string(jvm, &file_name).await?;
        let class_loader = JavaLangClassLoader::get_system_class_loader(jvm).await?;
        let Some(stream) = JavaLangClassLoader::get_resource_as_stream(jvm, &class_loader, &resource_name).await? else {
            return Err(jvm.exception("java/io/IOException", "image resource was not found").await);
        };

        let image_data = match JavaIoInputStream::read_until_end(jvm, &stream).await {
            Ok(image_data) => {
                let _: () = jvm.invoke_virtual(&stream, "close", "()V", ()).await?;
                image_data
            }
            Err(error) => {
                let _: JvmResult<()> = jvm.invoke_virtual(&stream, "close", "()V", ()).await;
                return Err(error);
            }
        };
        let Ok(image_data_len) = i32::try_from(image_data.len()) else {
            return Err(jvm.exception("java/io/IOException", "image resource is too large").await);
        };
        let mut image_array = jvm.instantiate_array("B", image_data.len()).await?;
        jvm.array_raw_buffer_mut(&mut image_array).await?.write(0, &image_data)?;

        jvm.invoke_static(
            "javax/microedition/lcdui/Image",
            "createImage",
            "([BII)Ljavax/microedition/lcdui/Image;",
            (image_array, 0, image_data_len),
        )
        .await
    }

    async fn create_ex_image(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        dir_name: ClassInstanceRef<String>,
        file_name: ClassInstanceRef<String>,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::createExImage({dir_name:?}, {file_name:?})");

        if dir_name.is_null() || file_name.is_null() {
            return Err(jvm
                .exception("java/lang/NullPointerException", "dirName and fileName must not be null")
                .await);
        }

        let mut path = JavaLangString::to_rust_string(jvm, &dir_name).await?;
        let file_name = JavaLangString::to_rust_string(jvm, &file_name).await?;
        if !path.is_empty() && !path.ends_with('/') {
            path.push('/');
        }
        path.push_str(&file_name);
        let path: ClassInstanceRef<String> = JavaLangString::from_rust_string(jvm, &path).await?.into();

        jvm.invoke_static(
            "com/xce/lcdui/Toolkit",
            "createImage",
            "(Ljava/lang/String;)Ljavax/microedition/lcdui/Image;",
            (path,),
        )
        .await
    }

    async fn split_string(jvm: &Jvm, _context: &mut WieJvmContext, string: ClassInstanceRef<String>, max_width: i32) -> JvmResult<i32> {
        tracing::debug!("com.xce.lcdui.Toolkit::splitString({string:?}, {max_width})");

        if string.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "string is null").await);
        }
        if max_width <= 0 {
            return Ok(0);
        }

        let font: ClassInstanceRef<Font> = jvm
            .get_static_field("com/xce/lcdui/Toolkit", "DEFAULT_FONT", "Ljavax/microedition/lcdui/Font;")
            .await?;
        let string_length: i32 = jvm.invoke_virtual(&string, "length", "()I", ()).await?;
        let mut fitting_length = 0;
        for length in 1..=string_length {
            let width: i32 = jvm
                .invoke_virtual(&font, "substringWidth", "(Ljava/lang/String;II)I", (string.clone(), 0, length))
                .await?;
            if width > max_width {
                break;
            }
            fitting_length = length;
        }

        Ok(fitting_length)
    }

    async fn paint_popup(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        message_1: ClassInstanceRef<String>,
        message_2: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.Toolkit::paintPopup({message_1:?}, {message_2:?})");
        Ok(())
    }

    async fn paint_popup_with_button(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        message_1: ClassInstanceRef<String>,
        message_2: ClassInstanceRef<String>,
        button: bool,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.Toolkit::paintPopup({message_1:?}, {message_2:?}, {button})");
        Ok(())
    }

    async fn app_img(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::warn!("stub com.xce.lcdui.Toolkit::appImg()");
        Ok(ClassInstanceRef::new(None))
    }

    async fn castle_img(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::warn!("stub com.xce.lcdui.Toolkit::castleImg()");
        Ok(ClassInstanceRef::new(None))
    }

    define_image_accessors!(
        (title_img, set_title_img, "titleImg"),
        (button_back_left_img, set_button_back_left_img, "buttonBackLeftImg"),
        (button_back_right_img, set_button_back_right_img, "buttonBackRightImg"),
        (screen_icon_img, set_screen_icon_img, "screenIconImg"),
        (back_icon_img, set_back_icon_img, "backIconImg"),
        (cancel_icon_img, set_cancel_icon_img, "cancelIconImg"),
        (help_icon_img, set_help_icon_img, "helpIconImg"),
        (ok_icon_img, set_ok_icon_img, "okIconImg"),
        (stop_icon_img, set_stop_icon_img, "stopIconImg"),
        (exit_icon_img, set_exit_icon_img, "exitIconImg"),
        (item_icon_img, set_item_icon_img, "itemIconImg"),
        (menu_icon_img, set_menu_icon_img, "menuIconImg"),
        (ueim_img, set_ueim_img, "ueimImg"),
        (leim_img, set_leim_img, "leimImg"),
        (kim_img, set_kim_img, "kimImg"),
        (sim_img, set_sim_img, "simImg"),
        (nim_img, set_nim_img, "nimImg"),
        (im_hint_img, set_im_hint_img, "imHintImg"),
        (s_exclusive, set_s_exclusive, "sExclusive"),
        (u_exclusive, set_u_exclusive, "uExclusive"),
        (s_multiple, set_s_multiple, "sMultiple"),
        (u_multiple, set_u_multiple, "uMultiple"),
        (s_back_img, set_s_back_img, "sBackImg"),
        (g_back_img, set_g_back_img, "gBackImg"),
        (g_fore_img, set_g_fore_img, "gForeImg"),
        (scroll_img, set_scroll_img, "scrollImg"),
    );
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;
    use jvm::{ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_midp::classes::javax::microedition::lcdui::Image;

    use super::Toolkit;

    #[test]
    fn image_accessor_preserves_the_image_reference() {
        let result = run_jvm_test(
            Box::new([wie_midp::get_protos().into(), [Toolkit::as_proto()].into()]),
            |jvm| async move {
                let image: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createImage",
                        "(II)Ljavax/microedition/lcdui/Image;",
                        (2, 3),
                    )
                    .await?;

                let _: () = jvm
                    .invoke_static(
                        "com/xce/lcdui/Toolkit",
                        "setTitleImg",
                        "(Ljavax/microedition/lcdui/Image;)V",
                        (image.clone(),),
                    )
                    .await?;
                let returned: ClassInstanceRef<Image> = jvm
                    .invoke_static("com/xce/lcdui/Toolkit", "titleImg", "()Ljavax/microedition/lcdui/Image;", ())
                    .await?;
                assert!(returned.equals(&**image)?);

                let app_image: ClassInstanceRef<Image> = jvm
                    .invoke_static("com/xce/lcdui/Toolkit", "appImg", "()Ljavax/microedition/lcdui/Image;", ())
                    .await?;
                let castle_image: ClassInstanceRef<Image> = jvm
                    .invoke_static("com/xce/lcdui/Toolkit", "castleImg", "()Ljavax/microedition/lcdui/Image;", ())
                    .await?;
                assert!(app_image.is_null());
                assert!(castle_image.is_null());

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }

    #[test]
    fn split_string_returns_a_utf16_index_and_missing_images_throw_io_exception() {
        let result = run_jvm_test(
            Box::new([wie_midp::get_protos().into(), [Toolkit::as_proto()].into()]),
            |jvm| async move {
                let text: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "A\u{1f600}B").await?.into();
                let font = jvm
                    .get_static_field("com/xce/lcdui/Toolkit", "DEFAULT_FONT", "Ljavax/microedition/lcdui/Font;")
                    .await?;
                let width: i32 = jvm.invoke_virtual(&font, "stringWidth", "(Ljava/lang/String;)I", (text.clone(),)).await?;
                let split: i32 = jvm
                    .invoke_static("com/xce/lcdui/Toolkit", "splitString", "(Ljava/lang/String;I)I", (text, width))
                    .await?;
                assert_eq!(split, 4);

                let missing: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "/missing.png").await?.into();
                let image_result: JvmResult<ClassInstanceRef<Image>> = jvm
                    .invoke_static(
                        "com/xce/lcdui/Toolkit",
                        "createImage",
                        "(Ljava/lang/String;)Ljavax/microedition/lcdui/Image;",
                        (missing,),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = image_result else {
                    panic!("Toolkit.createImage accepted a missing resource");
                };
                assert!(jvm.is_instance(&*exception, "java/io/IOException"));

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
