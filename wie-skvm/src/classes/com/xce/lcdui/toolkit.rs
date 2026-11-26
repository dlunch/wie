use jvm::{
    ClassInstanceRef, Jvm, Result as JvmResult,
    runtime::{JavaIoInputStream, JavaLangClassLoader, JavaLangString},
};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;
use wie_midp::classes::javax::microedition::lcdui::{Font, Graphics, Image};

use super::MIDPRes;

// class com.xce.lcdui.Toolkit
#[java_class(name = "com/xce/lcdui/Toolkit", public)]
pub struct Toolkit {
    #[field(public, static, final)]
    DEFAULT_FONT: ClassInstanceRef<Font>,
    #[field(public, static, final)]
    FONT_HEIGHT: i32,
    #[field(public, static, final)]
    FONT_GAP: i32,
    #[field(public, static, final)]
    MAX_CHARWIDTH: i32,
    #[field(public, static, final)]
    BLACK: i32,
    #[field(public, static, final)]
    DK_GRAY: i32,
    #[field(public, static, final)]
    LT_GRAY: i32,
    #[field(public, static, final)]
    WHITE: i32,
    #[field(public, static, final)]
    black: i32,
    #[field(public, static, final)]
    dk_gray: i32,
    #[field(public, static, final)]
    lt_gray: i32,
    #[field(public, static, final)]
    white: i32,
    #[field(public, static, final)]
    graphics: ClassInstanceRef<Graphics>,
    #[field(public, static)]
    IMG_HEIGHT: i32,
    #[field(public, static)]
    IS_KOREAN: bool,
    #[field(public, static)]
    selected: i32,
    #[field(public, static)]
    ext: ClassInstanceRef<String>,
    #[field(public, static)]
    iconsDir: ClassInstanceRef<String>,
    #[field(public, static)]
    MIDP_RES: ClassInstanceRef<MIDPRes>,
    #[field(private, static)]
    titleImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    buttonBackLeftImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    buttonBackRightImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    screenIconImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    backIconImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    cancelIconImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    helpIconImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    okIconImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    stopIconImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    exitIconImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    itemIconImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    menuIconImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    ueimImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    leimImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    kimImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    simImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    nimImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    imHintImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    sExclusive: ClassInstanceRef<Image>,
    #[field(private, static)]
    uExclusive: ClassInstanceRef<Image>,
    #[field(private, static)]
    sMultiple: ClassInstanceRef<Image>,
    #[field(private, static)]
    uMultiple: ClassInstanceRef<Image>,
    #[field(private, static)]
    sBackImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    gBackImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    gForeImg: ClassInstanceRef<Image>,
    #[field(private, static)]
    scrollImg: ClassInstanceRef<Image>,
}

#[java_methods]
impl Toolkit {
    #[method(static_initializer)]
    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::<clinit>");

        let font: ClassInstanceRef<Font> = jvm
            .invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
            .await?;
        jvm.put_static_field("com/xce/lcdui/Toolkit", "DEFAULT_FONT", "Ljavax/microedition/lcdui/Font;", font.clone())
            .await?;

        let font_height: i32 = jvm.invoke_virtual(&font, "javax/microedition/lcdui/Font", "getHeight", "()I", ()).await?;
        jvm.put_static_field("com/xce/lcdui/Toolkit", "FONT_HEIGHT", "I", font_height).await?;
        jvm.put_static_field("com/xce/lcdui/Toolkit", "FONT_GAP", "I", 0).await?;

        let max_char_width: i32 = jvm
            .invoke_virtual(&font, "javax/microedition/lcdui/Font", "charWidth", "(C)I", ('W' as u16,))
            .await?;
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

        let graphics: ClassInstanceRef<Graphics> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Display",
                "getScreenGraphics",
                "()Ljavax/microedition/lcdui/Graphics;",
                (),
            )
            .await?;
        jvm.put_static_field("com/xce/lcdui/Toolkit", "graphics", "Ljavax/microedition/lcdui/Graphics;", graphics)
            .await?;

        Ok(())
    }

    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::<init>({this:?})");
        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    #[method(public, static)]
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
                let _: () = jvm.invoke_virtual(&stream, "java/io/InputStream", "close", "()V", ()).await?;
                image_data
            }
            Err(error) => {
                let _: JvmResult<()> = jvm.invoke_virtual(&stream, "java/io/InputStream", "close", "()V", ()).await;
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

    #[method(public, static)]
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

    #[method(public, static)]
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
        let string_length: i32 = jvm.invoke_virtual(&string, "java/lang/String", "length", "()I", ()).await?;
        let mut fitting_length = 0;
        for length in 1..=string_length {
            let width: i32 = jvm
                .invoke_virtual(
                    &font,
                    "javax/microedition/lcdui/Font",
                    "substringWidth",
                    "(Ljava/lang/String;II)I",
                    (string.clone(), 0, length),
                )
                .await?;
            if width > max_width {
                break;
            }
            fitting_length = length;
        }

        Ok(fitting_length)
    }

    #[method(public, static)]
    async fn paint_popup(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        message_1: ClassInstanceRef<String>,
        message_2: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.Toolkit::paintPopup({message_1:?}, {message_2:?})");
        Ok(())
    }

    #[method(name = "paintPopup", public, static)]
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

    #[method(public, static)]
    async fn app_img(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::warn!("stub com.xce.lcdui.Toolkit::appImg()");
        Ok(ClassInstanceRef::new(None))
    }

    #[method(public, static)]
    async fn castle_img(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::warn!("stub com.xce.lcdui.Toolkit::castleImg()");
        Ok(ClassInstanceRef::new(None))
    }

    #[method(public, static)]
    async fn title_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::titleImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "titleImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_title_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setTitleImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "titleImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn button_back_left_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::buttonBackLeftImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "buttonBackLeftImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_button_back_left_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setButtonBackLeftImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "buttonBackLeftImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn button_back_right_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::buttonBackRightImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "buttonBackRightImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_button_back_right_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setButtonBackRightImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "buttonBackRightImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn screen_icon_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::screenIconImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "screenIconImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_screen_icon_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setScreenIconImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "screenIconImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn back_icon_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::backIconImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "backIconImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_back_icon_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setBackIconImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "backIconImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn cancel_icon_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::cancelIconImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "cancelIconImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_cancel_icon_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setCancelIconImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "cancelIconImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn help_icon_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::helpIconImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "helpIconImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_help_icon_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setHelpIconImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "helpIconImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn ok_icon_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::okIconImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "okIconImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_ok_icon_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setOkIconImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "okIconImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn stop_icon_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::stopIconImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "stopIconImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_stop_icon_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setStopIconImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "stopIconImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn exit_icon_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::exitIconImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "exitIconImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_exit_icon_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setExitIconImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "exitIconImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn item_icon_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::itemIconImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "itemIconImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_item_icon_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setItemIconImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "itemIconImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn menu_icon_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::menuIconImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "menuIconImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_menu_icon_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setMenuIconImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "menuIconImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn ueim_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::ueimImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "ueimImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(name = "setUEimImg", public, static)]
    async fn set_ueim_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setUEimImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "ueimImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn leim_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::leimImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "leimImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(name = "setLEimImg", public, static)]
    async fn set_leim_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setLEimImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "leimImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn kim_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::kimImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "kimImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_kim_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setKimImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "kimImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn sim_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::simImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "simImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_sim_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setSimImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "simImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn nim_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::nimImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "nimImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_nim_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setNimImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "nimImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn im_hint_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::imHintImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "imHintImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(name = "setIMHintImg", public, static)]
    async fn set_im_hint_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setIMHintImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "imHintImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn s_exclusive(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::sExclusive()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "sExclusive", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_s_exclusive(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setSExclusive({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "sExclusive", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn u_exclusive(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::uExclusive()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "uExclusive", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_u_exclusive(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setUExclusive({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "uExclusive", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn s_multiple(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::sMultiple()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "sMultiple", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_s_multiple(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setSMultiple({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "sMultiple", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn u_multiple(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::uMultiple()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "uMultiple", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_u_multiple(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setUMultiple({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "uMultiple", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn s_back_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::sBackImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "sBackImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_s_back_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setSBackImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "sBackImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn g_back_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::gBackImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "gBackImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_g_back_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setGBackImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "gBackImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn g_fore_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::gForeImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "gForeImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_g_fore_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setGForeImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "gForeImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }

    #[method(public, static)]
    async fn scroll_img(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.xce.lcdui.Toolkit::scrollImg()");
        jvm.get_static_field("com/xce/lcdui/Toolkit", "scrollImg", "Ljavax/microedition/lcdui/Image;")
            .await
    }

    #[method(public, static)]
    async fn set_scroll_img(jvm: &Jvm, _context: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::setScrollImg({image:?})");
        jvm.put_static_field("com/xce/lcdui/Toolkit", "scrollImg", "Ljavax/microedition/lcdui/Image;", image)
            .await
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_midp::classes::javax::microedition::lcdui::{Font, Image};

    use super::Toolkit;

    #[test]
    fn split_string_returns_a_utf16_index_and_missing_images_throw_io_exception() {
        let result = run_jvm_test(
            Box::new([wie_midp::get_protos().into(), [Toolkit::as_proto()].into()]),
            |jvm| async move {
                let text: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "A\u{1f600}B").await?.into();
                let font: ClassInstanceRef<Font> = jvm
                    .get_static_field("com/xce/lcdui/Toolkit", "DEFAULT_FONT", "Ljavax/microedition/lcdui/Font;")
                    .await?;
                let width: i32 = jvm
                    .invoke_virtual(
                        &font,
                        "javax/microedition/lcdui/Font",
                        "stringWidth",
                        "(Ljava/lang/String;)I",
                        (text.clone(),),
                    )
                    .await?;
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
