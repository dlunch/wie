use alloc::{borrow::Cow, boxed::Box, format, vec, vec::Vec};
use core::marker::PhantomData;

use bytemuck::{Zeroable, cast_slice, cast_vec};

use jvm::{
    Array, ArrayRawBufferMut, ClassInstanceRef, Jvm, Result as JvmResult,
    runtime::{JavaIoInputStream, JavaLangClassLoader, JavaLangString},
};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_backend::canvas::{
    ArgbPixel, Canvas, Color, Image as BackendImage, ImageBuffer, ImageBufferCanvas, PixelType, Rgb332Pixel, Rgb565Pixel, decode_image,
};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::Graphics;

// class javax.microedition.lcdui.Image
pub struct Image;

impl Image {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Image",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                // SKVM exposes a public no-argument constructor.
                JavaMethodProto::new("<init>", "()V", Self::init_empty, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(II[BI)V", Self::init, MethodAccessFlags::empty()),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isMutable", "()Z", Self::is_mutable, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getRGB", "([IIIIIII)V", Self::get_rgb, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "createRGBImage",
                    "([IIIZ)Ljavax/microedition/lcdui/Image;",
                    Self::create_rgb_image,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getGraphics",
                    "()Ljavax/microedition/lcdui/Graphics;",
                    Self::get_graphics,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    Self::create_image,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "([BII)Ljavax/microedition/lcdui/Image;",
                    Self::create_image_from_data,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(Ljava/lang/String;)Ljavax/microedition/lcdui/Image;",
                    Self::create_image_from_name,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(Ljavax/microedition/lcdui/Image;)Ljavax/microedition/lcdui/Image;",
                    Self::create_image_from_image,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("w", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("h", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("imgData", "[B", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("bpl", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("mutable", "Z", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init_empty(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Image::<init>({this:?})");

        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        width: i32,
        height: i32,
        img_data: ClassInstanceRef<Array<i8>>,
        bpl: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Image::<init>({this:?}, {width}, {height}, {img_data:?}, {bpl})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "javax/microedition/lcdui/Image", "w", "I", width).await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/Image", "h", "I", height).await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/Image", "imgData", "[B", img_data)
            .await?;
        jvm.put_field(&mut this, "javax/microedition/lcdui/Image", "bpl", "I", bpl).await?;

        Ok(())
    }

    async fn create_image(jvm: &Jvm, _: &mut WieJvmContext, width: i32, height: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createImage({width}, {height})");

        let bytes_per_pixel = 4;

        let mut image = Self::create_image_instance(
            jvm,
            width as _,
            height as _,
            &vec![255; (width * height * bytes_per_pixel) as usize],
            bytes_per_pixel as _,
        )
        .await?;
        jvm.put_field(&mut image, "javax/microedition/lcdui/Image", "mutable", "Z", true).await?;

        Ok(image)
    }

    async fn create_rgb_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        rgb: ClassInstanceRef<Array<i32>>,
        width: i32,
        height: i32,
        process_alpha: bool,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createRGBImage({rgb:?}, {width}, {height}, {process_alpha})");

        if rgb.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "rgb is null").await);
        }
        if width <= 0 || height <= 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "image dimensions must be positive")
                .await);
        }
        let count = i64::from(width) * i64::from(height);
        if count > jvm.array_length(&rgb).await? as i64 {
            return Err(jvm.exception("java/lang/ArrayIndexOutOfBoundsException", "rgb is too small").await);
        }

        let mut pixels: Vec<i32> = jvm.load_array(&rgb, 0, count as usize).await?;
        if !process_alpha {
            for pixel in &mut pixels {
                *pixel |= 0xff000000u32 as i32;
            }
        }
        Self::create_image_instance(jvm, width as _, height as _, cast_slice(&pixels), 4).await
    }

    async fn create_image_from_name(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createImage({name:?})");

        let name = JavaLangString::to_rust_string(jvm, &name).await?;

        let class_loader = JavaLangClassLoader::get_system_class_loader(jvm).await?;
        let stream = JavaLangClassLoader::get_resource_as_stream(jvm, &class_loader, &name).await?;
        let Some(stream) = stream else {
            let exception = jvm
                .exception("java/io/FileNotFoundException", &format!("Resource not found: {name}"))
                .await;
            return Err(exception);
        };

        let image_data = JavaIoInputStream::read_until_end(jvm, &stream).await?;
        let image_data_len = image_data.len() as i32;

        let mut image_array = jvm.instantiate_array("B", image_data_len as _).await?;
        jvm.array_raw_buffer_mut(&mut image_array).await?.write(0, &image_data)?;

        jvm.invoke_static(
            "javax/microedition/lcdui/Image",
            "createImage",
            "([BII)Ljavax/microedition/lcdui/Image;",
            (image_array, 0, image_data_len),
        )
        .await
    }

    async fn create_image_from_data(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        data: ClassInstanceRef<Array<i8>>,
        image_offset: i32,
        image_length: i32,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createImage({data:?}, {image_offset}, {image_length})");

        let mut image_data = vec![0; image_length as usize];
        jvm.array_raw_buffer(&data).await?.read(image_offset as _, &mut image_data)?;

        let image = {
            let result = decode_image(&cast_vec(image_data));
            if let Ok(image) = result {
                image
            } else {
                tracing::error!("Failed to decode image: {:?}", result.err());
                let exception = jvm.exception("java/lang/IllegalArgumentException", "Failed to decode image").await;

                return Err(exception);
            }
        };

        Self::create_image_instance(jvm, image.width(), image.height(), &image.raw(), image.bytes_per_pixel()).await
    }

    async fn create_image_from_image(jvm: &Jvm, _: &mut WieJvmContext, image: ClassInstanceRef<Image>) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createImage({image:?})");

        let src_image = Image::image(jvm, &image).await?;

        Self::create_image_instance(jvm, src_image.width(), src_image.height(), &src_image.raw(), src_image.bytes_per_pixel()).await
    }

    async fn get_graphics(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Graphics>> {
        tracing::debug!("javax.microedition.lcdui.Image::getGraphics({this:?})");

        let mutable: bool = jvm.get_field(&this, "javax/microedition/lcdui/Image", "mutable", "Z").await?;
        if !mutable {
            return Err(jvm.exception("java/lang/IllegalStateException", "image is immutable").await);
        }

        let instance = jvm
            .new_class(
                "javax/microedition/lcdui/Graphics",
                "(Ljavax/microedition/lcdui/Image;)V",
                (this.clone(),),
            )
            .await?;

        Ok(instance.into())
    }

    async fn get_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Image::getWidth({this:?})");

        jvm.get_field(&this, "javax/microedition/lcdui/Image", "w", "I").await
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Image::getHeight({this:?})");

        jvm.get_field(&this, "javax/microedition/lcdui/Image", "h", "I").await
    }

    async fn is_mutable(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("javax.microedition.lcdui.Image::isMutable({this:?})");

        jvm.get_field(&this, "javax/microedition/lcdui/Image", "mutable", "Z").await
    }

    #[allow(clippy::too_many_arguments)]
    async fn get_rgb(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        mut rgb_data: ClassInstanceRef<Array<i32>>,
        offset: i32,
        scanlength: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Image::getRGB({this:?}, {rgb_data:?}, {offset}, {scanlength}, {x}, {y}, {width}, {height})");

        let image_width: i32 = jvm.get_field(&this, "javax/microedition/lcdui/Image", "w", "I").await?;
        let image_height: i32 = jvm.get_field(&this, "javax/microedition/lcdui/Image", "h", "I").await?;
        if x < 0 || y < 0 || i64::from(x) + i64::from(width) > i64::from(image_width) || i64::from(y) + i64::from(height) > i64::from(image_height) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "region exceeds image bounds").await);
        }
        if width <= 0 || height <= 0 {
            return Ok(());
        }
        if rgb_data.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "rgbData is null").await);
        }
        if i64::from(scanlength).abs() < i64::from(width) {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "scanlength is smaller than width")
                .await);
        }
        let first_row = i64::from(offset);
        let last_row = first_row + i64::from(height - 1) * i64::from(scanlength);
        let length = jvm.array_length(&rgb_data).await? as i64;
        if first_row.min(last_row) < 0 || first_row.max(last_row) + i64::from(width) > length {
            return Err(jvm
                .exception("java/lang/ArrayIndexOutOfBoundsException", "destination is too small")
                .await);
        }

        let image = Self::image(jvm, &this).await?;
        for row in 0..height {
            let pixels: Vec<i32> = (0..width)
                .map(|column| ArgbPixel::from_color(image.get_pixel(x + column, y + row)) as i32)
                .collect();
            let destination = first_row + i64::from(row) * i64::from(scanlength);
            jvm.store_array(&mut rgb_data, destination as _, pixels).await?;
        }
        Ok(())
    }

    pub async fn image(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Box<dyn BackendImage>> {
        let width: i32 = jvm.get_field(this, "javax/microedition/lcdui/Image", "w", "I").await?;
        let bpl: i32 = jvm.get_field(this, "javax/microedition/lcdui/Image", "bpl", "I").await?;

        let bytes_per_pixel = bpl / width;

        Ok(match bytes_per_pixel {
            1 => Box::new(JavaImageBuffer::<Rgb332Pixel>::new(jvm, this).await?) as _,
            2 => Box::new(JavaImageBuffer::<Rgb565Pixel>::new(jvm, this).await?) as _,
            4 => Box::new(JavaImageBuffer::<ArgbPixel>::new(jvm, this).await?) as _,
            _ => unimplemented!("Unsupported pixel format: {bytes_per_pixel}"),
        })
    }

    pub async fn canvas(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Box<dyn Canvas>> {
        let width: i32 = jvm.get_field(this, "javax/microedition/lcdui/Image", "w", "I").await?;
        let bpl: i32 = jvm.get_field(this, "javax/microedition/lcdui/Image", "bpl", "I").await?;

        let bytes_per_pixel = bpl / width;

        Ok(match bytes_per_pixel {
            1 => Box::new(ImageBufferCanvas::new(JavaImageBuffer::<Rgb332Pixel>::new(jvm, this).await?)) as _,
            2 => Box::new(ImageBufferCanvas::new(JavaImageBuffer::<Rgb565Pixel>::new(jvm, this).await?)) as _,
            4 => Box::new(ImageBufferCanvas::new(JavaImageBuffer::<ArgbPixel>::new(jvm, this).await?)) as _,
            _ => unimplemented!("Unsupported pixel format: {bytes_per_pixel}"),
        })
    }

    async fn create_image_instance(jvm: &Jvm, width: u32, height: u32, data: &[u8], bytes_per_pixel: u32) -> JvmResult<ClassInstanceRef<Image>> {
        let mut data_array = jvm.instantiate_array("B", data.len() as _).await?;
        jvm.array_raw_buffer_mut(&mut data_array).await?.write(0, data)?;

        Ok(jvm
            .new_class(
                "javax/microedition/lcdui/Image",
                "(II[BI)V",
                (width as i32, height as i32, data_array, (width * bytes_per_pixel) as i32),
            )
            .await?
            .into())
    }
}

struct JavaImageBuffer<T>
where
    T: PixelType,
{
    width: i32,
    height: i32,
    raw_buffer: Box<dyn ArrayRawBufferMut>,
    _phantom: PhantomData<T>,
}

impl<T> JavaImageBuffer<T>
where
    T: PixelType,
{
    pub async fn new(jvm: &Jvm, this: &ClassInstanceRef<Image>) -> JvmResult<Self> {
        let mut java_img_data = jvm.get_field(this, "javax/microedition/lcdui/Image", "imgData", "[B").await?;
        let raw_buffer = jvm.array_raw_buffer_mut(&mut java_img_data).await?;

        let width: i32 = jvm.get_field(this, "javax/microedition/lcdui/Image", "w", "I").await?;
        let height: i32 = jvm.get_field(this, "javax/microedition/lcdui/Image", "h", "I").await?;

        Ok(Self {
            width,
            height,
            raw_buffer,
            _phantom: PhantomData,
        })
    }
}

impl<T> BackendImage for JavaImageBuffer<T>
where
    T: PixelType,
{
    fn width(&self) -> u32 {
        self.width as _
    }

    fn height(&self) -> u32 {
        self.height as _
    }

    fn bytes_per_pixel(&self) -> u32 {
        size_of::<T::DataType>() as _
    }

    fn get_pixel(&self, x: i32, y: i32) -> Color {
        let offset = (((y as u32) * self.width() + (x as u32)) * self.bytes_per_pixel()) as usize;

        let mut raw = T::DataType::zeroed();
        self.raw_buffer.read(offset, bytemuck::bytes_of_mut(&mut raw)).unwrap();

        T::to_color(raw)
    }

    fn raw(&self) -> Cow<'_, [u8]> {
        let size = self.width() * self.height() * self.bytes_per_pixel();
        let mut buffer = vec![0; size as usize];
        self.raw_buffer.read(0, &mut buffer).unwrap();

        Cow::Owned(buffer)
    }

    fn colors(&self) -> Vec<Color> {
        let buffer = self.raw();

        let data: Vec<T::DataType> = bytemuck::pod_collect_to_vec(&buffer);

        data.into_iter().map(T::to_color).collect()
    }
}

impl<T> ImageBuffer for JavaImageBuffer<T>
where
    T: PixelType,
{
    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x > self.width || y > self.height || x < 0 || y < 0 {
            return;
        }

        let offset = (((y as u32) * self.width() + (x as u32)) * self.bytes_per_pixel()) as usize;

        let raw = T::from_color(color);
        let raw_bytes = bytemuck::bytes_of(&raw);

        self.raw_buffer.write(offset as _, raw_bytes).unwrap();
    }

    fn put_pixels(&mut self, x: i32, y: i32, _width: u32, colors: &[Color]) {
        if x > self.width || y > self.height || x < 0 || y < 0 {
            return;
        }

        let offset = (((y as u32) * self.width() + (x as u32)) * self.bytes_per_pixel()) as usize;

        let raw = colors.iter().copied().map(T::from_color).collect::<Vec<_>>();
        self.raw_buffer.write(offset, bytemuck::cast_slice(&raw)).unwrap();
    }

    fn xor_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }

        let offset = (((y as u32) * self.width() + (x as u32)) * self.bytes_per_pixel()) as usize;

        let mut raw = T::DataType::zeroed();
        self.raw_buffer.read(offset, bytemuck::bytes_of_mut(&mut raw)).unwrap();

        let raw = T::xor_color(raw, color);
        self.raw_buffer.write(offset as _, bytemuck::bytes_of(&raw)).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use jvm::JavaError;

    use super::*;

    #[test]
    fn rgb_images_copy_pixels_and_validate_reads() -> wie_util::Result<()> {
        test_utils::run_jvm_test(Box::new([crate::get_protos().into()]), |jvm| async move {
            let blank: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    (2, 1),
                )
                .await?;
            assert!(
                jvm.invoke_virtual::<_, bool>(&blank, "javax/microedition/lcdui/Image", "isMutable", "()Z", ())
                    .await?
            );
            let blank_pixels = jvm.instantiate_array("I", 2).await?;
            let _: () = jvm
                .invoke_virtual(
                    &blank,
                    "javax/microedition/lcdui/Image",
                    "getRGB",
                    "([IIIIIII)V",
                    (blank_pixels.clone(), 0, 2, 0, 0, 2, 1),
                )
                .await?;
            assert_eq!(jvm.load_array::<i32>(&blank_pixels, 0, 2).await?, [-1, -1]);
            let source = [0x00112233, 0x80445566u32 as i32, 0xff778899u32 as i32, 0xffaabbccu32 as i32];
            let mut pixels = jvm.instantiate_array("I", source.len()).await?;
            jvm.store_array(&mut pixels, 0, source).await?;
            for process_alpha in [true, false] {
                let image: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createRGBImage",
                        "([IIIZ)Ljavax/microedition/lcdui/Image;",
                        (pixels.clone(), 2, 2, process_alpha),
                    )
                    .await?;
                assert!(
                    !jvm.invoke_virtual::<_, bool>(&image, "javax/microedition/lcdui/Image", "isMutable", "()Z", ())
                        .await?
                );
                let graphics = jvm
                    .invoke_virtual::<_, ClassInstanceRef<Graphics>>(
                        &image,
                        "javax/microedition/lcdui/Image",
                        "getGraphics",
                        "()Ljavax/microedition/lcdui/Graphics;",
                        (),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = graphics else {
                    panic!("RGB image is mutable")
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalStateException"));

                jvm.store_array(&mut pixels, 0, [0i32; 4]).await?;
                for (offset, scanlength, positions) in [(1, 3, [1, 2, 4, 5]), (4, -3, [4, 5, 1, 2])] {
                    let mut output = jvm.instantiate_array("I", 7).await?;
                    jvm.store_array(&mut output, 0, [-1i32; 7]).await?;
                    let _: () = jvm
                        .invoke_virtual(
                            &image,
                            "javax/microedition/lcdui/Image",
                            "getRGB",
                            "([IIIIIII)V",
                            (output.clone(), offset, scanlength, 0, 0, 2, 2),
                        )
                        .await?;
                    let mut expected = [-1i32; 7];
                    for (position, color) in positions.into_iter().zip(source) {
                        expected[position] = if process_alpha { color } else { color | 0xff000000u32 as i32 };
                    }
                    assert_eq!(jvm.load_array::<i32>(&output, 0, 7).await?, expected);
                }

                let mut output = jvm.instantiate_array("I", 4).await?;
                let _: () = jvm
                    .invoke_virtual(
                        &image,
                        "javax/microedition/lcdui/Image",
                        "getRGB",
                        "([IIIIIII)V",
                        (output.clone(), 0, 1, 1, 1, 1, 1),
                    )
                    .await?;
                assert_eq!(jvm.load_array::<i32>(&output, 0, 1).await?, [source[3]]);
                jvm.store_array(&mut output, 0, [42i32; 4]).await?;
                for (offset, scanlength, x, y, width, height, error) in [
                    (0, 2, -1, 0, 1, 1, "java/lang/IllegalArgumentException"),
                    (0, 2, 2, 0, 1, 1, "java/lang/IllegalArgumentException"),
                    (0, 2, 0, 2, 1, 1, "java/lang/IllegalArgumentException"),
                    (0, 1, 0, 0, 2, 1, "java/lang/IllegalArgumentException"),
                    (0, -2, 0, 0, 2, 2, "java/lang/ArrayIndexOutOfBoundsException"),
                    (1, 2, 0, 0, 2, 2, "java/lang/ArrayIndexOutOfBoundsException"),
                ] {
                    let result = jvm
                        .invoke_virtual::<_, ()>(
                            &image,
                            "javax/microedition/lcdui/Image",
                            "getRGB",
                            "([IIIIIII)V",
                            (output.clone(), offset, scanlength, x, y, width, height),
                        )
                        .await;
                    let Err(JavaError::JavaException(exception)) = result else {
                        panic!("invalid read succeeded")
                    };
                    assert!(jvm.is_instance(&*exception, error));
                    assert_eq!(jvm.load_array::<i32>(&output, 0, 4).await?, [42; 4]);
                }
                let null: ClassInstanceRef<Array<i32>> = None.into();
                let _: () = jvm
                    .invoke_virtual(
                        &image,
                        "javax/microedition/lcdui/Image",
                        "getRGB",
                        "([IIIIIII)V",
                        (null.clone(), -1, 0, 0, 0, 0, 2),
                    )
                    .await?;
                let result = jvm
                    .invoke_virtual::<_, ()>(
                        &image,
                        "javax/microedition/lcdui/Image",
                        "getRGB",
                        "([IIIIIII)V",
                        (null, 0, 2, 0, 0, 2, 2),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = result else {
                    panic!("null read succeeded")
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));
                jvm.store_array(&mut pixels, 0, source).await?;
            }
            for (width, height, null, error) in [
                (0, 2, false, "java/lang/IllegalArgumentException"),
                (2, -1, false, "java/lang/IllegalArgumentException"),
                (3, 2, false, "java/lang/ArrayIndexOutOfBoundsException"),
                (2, 2, true, "java/lang/NullPointerException"),
            ] {
                let data: ClassInstanceRef<Array<i32>> = if null { None.into() } else { pixels.clone().into() };
                let result = jvm
                    .invoke_static::<_, ClassInstanceRef<Image>>(
                        "javax/microedition/lcdui/Image",
                        "createRGBImage",
                        "([IIIZ)Ljavax/microedition/lcdui/Image;",
                        (data, width, height, true),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = result else {
                    panic!("invalid image creation succeeded")
                };
                assert!(jvm.is_instance(&*exception, error));
            }
            Ok(())
        })
    }

    #[test]
    fn image_can_be_constructed_without_pixel_data() -> wie_util::Result<()> {
        test_utils::run_jvm_test(Box::new([crate::get_protos().into()]), |jvm| async move {
            let image = jvm.new_class("javax/microedition/lcdui/Image", "()V", ()).await?;
            let width: i32 = jvm
                .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getWidth", "()I", ())
                .await?;
            let height: i32 = jvm
                .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getHeight", "()I", ())
                .await?;
            assert_eq!((width, height), (0, 0));
            Ok(())
        })
    }

    #[test]
    fn lbmp_images_preserve_grayscale_and_transparency_when_drawn() -> wie_util::Result<()> {
        test_utils::run_jvm_test(Box::new([crate::get_protos().into()]), |jvm| async move {
            for kind in [8u32, 16, 2] {
                for masked in [0u32, 1] {
                    let pixels = match kind {
                        8 => vec![0xe0; 18],
                        16 => [0x00, 0xf8].repeat(18),
                        // Vertical bitplanes: first plane carries the high bit.
                        _ => vec![0xcc, 0xcc, 0, 1, 0xaa, 0xaa, 0, 1],
                    };
                    let size = if kind == 2 { pixels.len() / 2 } else { pixels.len() };
                    let mut data = vec![0];
                    data.extend_from_slice(b"LBMP");
                    for value in [kind, 2, 9, size as u32, masked] {
                        data.extend_from_slice(&value.to_le_bytes());
                    }
                    data.extend_from_slice(&pixels);
                    if masked != 0 {
                        data.extend_from_slice(&[1, 0, 0, 1]);
                    }
                    let mut array = jvm.instantiate_array("B", data.len()).await?;
                    jvm.array_raw_buffer_mut(&mut array).await?.write(0, &data)?;
                    let image: ClassInstanceRef<Image> = jvm
                        .invoke_static(
                            "javax/microedition/lcdui/Image",
                            "createImage",
                            "([BII)Ljavax/microedition/lcdui/Image;",
                            (array, 1, (data.len() - 1) as i32),
                        )
                        .await?;
                    let background = [0xff0000ffu32; 18];
                    let target = Image::create_image_instance(&jvm, 2, 9, bytemuck::cast_slice(&background), 4).await?;
                    let graphics = jvm
                        .new_class(
                            "javax/microedition/lcdui/Graphics",
                            "(Ljavax/microedition/lcdui/Image;)V",
                            (target.clone(),),
                        )
                        .await?;
                    let _: () = jvm
                        .invoke_virtual(
                            &graphics,
                            "javax/microedition/lcdui/Graphics",
                            "drawImage",
                            "(Ljavax/microedition/lcdui/Image;III)V",
                            (image, 0, 0, 0),
                        )
                        .await?;
                    let rendered = Image::image(&jvm, &target).await?;
                    for y in 0..9 {
                        for x in 0..2 {
                            let expected = if masked != 0 && ((x, y) == (0, 0) || (x, y) == (1, 8)) {
                                (255, 0, 0, 255)
                            } else if kind == 2 {
                                let gray = if y == 8 {
                                    if x == 0 { 255 } else { 0 }
                                } else {
                                    255 - (y % 4) as u8 * 85
                                };
                                (255, gray, gray, gray)
                            } else {
                                (255, if kind == 8 { 252 } else { 255 }, 0, 0)
                            };
                            let actual = rendered.get_pixel(x, y);
                            assert_eq!(
                                (actual.a, actual.r, actual.g, actual.b),
                                expected,
                                "type {kind}, mask {masked}, ({x}, {y})"
                            );
                        }
                    }
                }
            }
            Ok(())
        })
    }

    async fn check_pixel_buffer<T: PixelType>(jvm: &Jvm, pixels: [T::DataType; 2]) -> JvmResult<()> {
        let size = size_of::<T::DataType>();
        let image = Image::create_image_instance(jvm, 2, 1, bytemuck::cast_slice(&pixels), size as u32).await?;
        let mut buffer = JavaImageBuffer::<T>::new(jvm, &image).await?;
        let actual = buffer.get_pixel(1, 0);
        let expected = T::to_color(pixels[1]);
        assert_eq!((actual.a, actual.r, actual.g, actual.b), (expected.a, expected.r, expected.g, expected.b));
        let colors = [T::to_color(pixels[1]), T::to_color(pixels[0])];
        buffer.put_pixels(0, 0, 2, &colors);
        assert_eq!(&*buffer.raw(), bytemuck::cast_slice(&[pixels[1], pixels[0]]));
        let output = jvm.instantiate_array("I", 2).await?;
        let _: () = jvm
            .invoke_virtual(
                &image,
                "javax/microedition/lcdui/Image",
                "getRGB",
                "([IIIIIII)V",
                (output.clone(), 0, 2, 0, 0, 2, 1),
            )
            .await?;
        assert_eq!(
            jvm.load_array::<i32>(&output, 0, 2).await?,
            colors.map(|color| ArgbPixel::from_color(color) as i32)
        );

        let color = Color {
            a: 255,
            r: 127,
            g: 63,
            b: 31,
        };
        buffer.xor_pixel(1, 0, color);
        let expected = [pixels[1], T::xor_color(pixels[0], color)];
        assert_eq!(&*buffer.raw(), bytemuck::cast_slice(&expected));
        Ok(())
    }

    #[test]
    fn java_pixel_buffers_preserve_native_formats() -> wie_util::Result<()> {
        test_utils::run_jvm_test(Box::new([crate::get_protos().into()]), |jvm| async move {
            check_pixel_buffer::<Rgb332Pixel>(&jvm, [0x13, 0xe7]).await?;
            check_pixel_buffer::<Rgb565Pixel>(&jvm, [0x1234, 0xabcd]).await?;
            check_pixel_buffer::<ArgbPixel>(&jvm, [0x10203040, 0x80abcdef]).await?;
            Ok(())
        })
    }
}
