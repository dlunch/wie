use alloc::{format, vec};
use core::cmp::min;

use bytemuck::cast_slice;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::{io::InputStream, lang::String};
use jvm::{runtime::JavaLangString, Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[repr(i32)]
#[derive(Eq, PartialEq)]
#[allow(dead_code)]
enum Mode {
    // wipi constant
    READ_ONLY = 1,
    WRITE = 2,
    WRITE_TRUNC = 3,
    READ_WRITE = 4,
}

// class org.kwis.msp.io.File
pub struct File;

impl File {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/io/File",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;II)V", Self::init_with_flag, Default::default()),
                JavaMethodProto::new("write", "([BII)I", Self::write, Default::default()),
                JavaMethodProto::new("read", "([B)I", Self::read, Default::default()),
                JavaMethodProto::new("read", "([BII)I", Self::read_with_offset_length, Default::default()),
                JavaMethodProto::new("seek", "(I)V", Self::seek, Default::default()),
                JavaMethodProto::new("close", "()V", Self::close, Default::default()),
                JavaMethodProto::new("sizeOf", "()I", Self::size_of, Default::default()),
                JavaMethodProto::new("openInputStream", "()Ljava/io/InputStream;", Self::open_input_stream, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("data", "[B", Default::default()),
                JavaFieldProto::new("pos", "I", Default::default()),
            ],
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, filename: ClassInstanceRef<String>, mode: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.io.File::<init>({:?}, {:?}, {:?})", &this, &filename, mode);

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/io/File", "<init>", "(Ljava/lang/String;II)V", (filename, mode, 0))
            .await?;

        Ok(())
    }

    async fn init_with_flag(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        filename: ClassInstanceRef<String>,
        mode: i32,
        flag: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.File::<init>({:?}, {:?}, {:?}, {:?})", &this, &filename, mode, flag);

        let filename = JavaLangString::to_rust_string(jvm, &filename).await?;
        tracing::debug!("Opening {}", filename);

        let mode = unsafe { core::mem::transmute::<i32, Mode>(mode) };

        let data = if mode == Mode::WRITE || mode == Mode::WRITE_TRUNC {
            // TODO: write not implemented
            Some(vec![])
        } else {
            let filesystem = context.system().filesystem();
            let data = filesystem.read(&filename);
            data.map(|x| cast_slice(x).to_vec())
        };

        if data.is_none() {
            return Err(jvm.exception("java/io/IOException", &format!("File not found: {}", filename)).await);
        }
        let data = data.unwrap();

        let mut data_array = jvm.instantiate_array("B", data.len() as _).await?;
        jvm.store_byte_array(&mut data_array, 0, data).await?;

        jvm.put_field(&mut this, "data", "[B", data_array).await?;
        jvm.put_field(&mut this, "pos", "I", 0).await?;

        Ok(())
    }

    async fn write(
        _jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        buf: ClassInstanceRef<ClassInstanceRef<Array<i8>>>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.File::write({:?}, {:?}, {:?}, {:?})", &this, &buf, offset, len);

        Ok(0)
    }

    async fn seek(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, pos: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.io.File::seek({:?}, {:?})", &this, pos);

        jvm.put_field(&mut this, "pos", "I", pos).await?;

        Ok(())
    }

    async fn read(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, buf: ClassInstanceRef<Array<i8>>) -> JvmResult<i32> {
        let length = jvm.array_length(&buf).await? as i32;

        jvm.invoke_virtual(&this, "read", "([BII)I", (buf, 0, length)).await
    }

    async fn read_with_offset_length(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        mut buf: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.io.File::read({:?}, {:?})", &this, &buf);

        let data_array = jvm.get_field(&this, "data", "[B").await?;
        let pos: i32 = jvm.get_field(&this, "pos", "I").await?;

        let data_len = jvm.array_length(&data_array).await? as i32;

        let length_to_read = min(data_len - pos, length);

        let data = jvm.load_byte_array(&data_array, pos as _, length_to_read as _).await?;
        jvm.store_byte_array(&mut buf, offset as _, data).await?;

        jvm.put_field(&mut this, "pos", "I", pos + length_to_read).await?;

        Ok(length_to_read as _)
    }

    async fn close(_jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.File::close({:?})", &this);

        Ok(())
    }

    async fn size_of(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.io.File::sizeOf({:?})", &this);

        let data_array = jvm.get_field(&this, "data", "[B").await?;
        let data_len = jvm.array_length(&data_array).await?;

        Ok(data_len as _)
    }

    async fn open_input_stream(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<InputStream>> {
        tracing::debug!("org.kwis.msp.io.File::openInputStream({:?})", &this);

        let data_array = jvm.get_field(&this, "data", "[B").await?;

        let input_stream = jvm.new_class("java/io/ByteArrayInputStream", "([B)V", [data_array]).await?;

        Ok(input_stream.into())
    }
}
