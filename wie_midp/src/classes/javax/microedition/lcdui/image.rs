use alloc::{boxed::Box, vec, vec::Vec};
use core::marker::PhantomData;

use bytemuck::cast_vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{
    Array, ArrayRawBufferMut, ClassInstanceRef, Jvm, Result as JvmResult,
    runtime::{JavaIoInputStream, JavaLangClassLoader, JavaLangString},
};

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
                JavaMethodProto::new("<init>", "(II[BI)V", Self::init, Default::default()),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new(
                    "getGraphics",
                    "()Ljavax/microedition/lcdui/Graphics;",
                    Self::get_graphics,
                    Default::default(),
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    Self::create_image,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "([BII)Ljavax/microedition/lcdui/Image;",
                    Self::create_image_from_data,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "createImage",
                    "(Ljava/lang/String;)Ljavax/microedition/lcdui/Image;",
                    Self::create_image_from_name,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("w", "I", Default::default()),
                JavaFieldProto::new("h", "I", Default::default()),
                JavaFieldProto::new("imgData", "[B", Default::default()),
                JavaFieldProto::new("bpl", "I", Default::default()),
            ],
        }
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
        tracing::debug!(
            "javax.microedition.lcdui.Image::<init>({:?}, {}, {}, {:?}, {})",
            &this,
            width,
            height,
            &img_data,
            bpl
        );

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "w", "I", width).await?;
        jvm.put_field(&mut this, "h", "I", height).await?;
        jvm.put_field(&mut this, "imgData", "[B", img_data).await?;
        jvm.put_field(&mut this, "bpl", "I", bpl).await?;

        Ok(())
    }

    async fn create_image(jvm: &Jvm, _: &mut WieJvmContext, width: i32, height: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createImage({}, {})", width, height);

        let bytes_per_pixel = 4;

        Self::create_image_instance(
            jvm,
            width as _,
            height as _,
            &vec![0; (width * height * bytes_per_pixel) as usize],
            bytes_per_pixel as _,
        )
        .await
    }

    async fn create_image_from_name(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createImage({:?})", &name);

        let name = JavaLangString::to_rust_string(jvm, &name).await?;

        let class_loader = jvm.current_class_loader().await?;
        let stream = JavaLangClassLoader::get_resource_as_stream(jvm, &class_loader, &name).await?.unwrap();

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
        offset: i32,
        length: i32,
    ) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("javax.microedition.lcdui.Image::createImage({:?}, {}, {})", &data, offset, length);

        let mut image_data = vec![0; length as usize];
        jvm.array_raw_buffer(&data).await?.read(offset as _, &mut image_data)?;

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

        Self::create_image_instance(jvm, image.width(), image.height(), image.raw(), image.bytes_per_pixel()).await
    }

    async fn get_graphics(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Graphics>> {
        tracing::debug!("javax.microedition.lcdui.Image::getGraphics({:?})", &this);

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
        tracing::debug!("javax.microedition.lcdui.Image::getWidth({:?})", &this);

        jvm.get_field(&this, "w", "I").await
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Image::getHeight({:?})", &this);

        jvm.get_field(&this, "h", "I").await
    }

    pub async fn image(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Box<dyn BackendImage>> {
        let width: i32 = jvm.get_field(this, "w", "I").await?;
        let bpl: i32 = jvm.get_field(this, "bpl", "I").await?;

        let bytes_per_pixel = bpl / width;

        Ok(match bytes_per_pixel {
            1 => Box::new(JavaImageBuffer::<Rgb332Pixel>::new(jvm, this).await?) as _,
            2 => Box::new(JavaImageBuffer::<Rgb565Pixel>::new(jvm, this).await?) as _,
            4 => Box::new(JavaImageBuffer::<ArgbPixel>::new(jvm, this).await?) as _,
            _ => unimplemented!("Unsupported pixel format: {}", bytes_per_pixel),
        })
    }

    pub async fn canvas(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Box<dyn Canvas>> {
        let width: i32 = jvm.get_field(this, "w", "I").await?;
        let bpl: i32 = jvm.get_field(this, "bpl", "I").await?;

        let bytes_per_pixel = bpl / width;

        Ok(match bytes_per_pixel {
            1 => Box::new(ImageBufferCanvas::new(JavaImageBuffer::<Rgb332Pixel>::new(jvm, this).await?)) as _,
            2 => Box::new(ImageBufferCanvas::new(JavaImageBuffer::<Rgb565Pixel>::new(jvm, this).await?)) as _,
            4 => Box::new(ImageBufferCanvas::new(JavaImageBuffer::<ArgbPixel>::new(jvm, this).await?)) as _,
            _ => unimplemented!("Unsupported pixel format: {}", bytes_per_pixel),
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
        let mut java_img_data = jvm.get_field(this, "imgData", "[B").await?;
        let raw_buffer = jvm.array_raw_buffer_mut(&mut java_img_data).await?;

        let width: i32 = jvm.get_field(this, "w", "I").await?;
        let height: i32 = jvm.get_field(this, "h", "I").await?;

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

        let mut buffer = vec![0; self.bytes_per_pixel() as usize];
        self.raw_buffer.read(offset as _, &mut buffer).unwrap();

        T::to_color(*bytemuck::from_bytes(&buffer[..size_of::<T::DataType>()]))
    }

    fn raw(&self) -> &[u8] {
        unimplemented!()
    }

    fn colors(&self) -> Vec<Color> {
        let size = self.width() * self.height() * self.bytes_per_pixel();
        let mut buffer = vec![0; size as usize];
        self.raw_buffer.read(0, &mut buffer).unwrap();

        buffer
            .chunks_exact(size_of::<T::DataType>())
            .map(|chunk| T::to_color(*bytemuck::from_bytes(chunk)))
            .collect()
    }
}

impl<T> ImageBuffer for JavaImageBuffer<T>
where
    T: PixelType,
{
    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        let offset = (((y as u32) * self.width() + (x as u32)) * self.bytes_per_pixel()) as usize;

        let raw = T::from_color(color);
        let raw_bytes = bytemuck::bytes_of(&raw);

        self.raw_buffer.write(offset as _, raw_bytes).unwrap();
    }

    fn put_pixels(&mut self, x: i32, y: i32, _width: u32, colors: &[Color]) {
        let offset = (((y as u32) * self.width() + (x as u32)) * self.bytes_per_pixel()) as usize;

        let raw_bytes = colors
            .iter()
            .flat_map(|color| bytemuck::bytes_of(&T::from_color(*color)).to_vec())
            .collect::<Vec<_>>();

        self.raw_buffer.write(offset as _, &raw_bytes).unwrap();
    }
}
