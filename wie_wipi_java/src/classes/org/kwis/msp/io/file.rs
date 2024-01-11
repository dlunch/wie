use alloc::{format, vec};
use core::cmp::min;

use bytemuck::cast_slice;

use java_class_proto::{JavaFieldAccessFlag, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm};

use crate::{WIPIJavaClassProto, WIPIJavaContxt};

// class org.kwis.msp.io.File
pub struct File {}

impl File {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;II)V", Self::init_with_flag, JavaMethodFlag::NONE),
                JavaMethodProto::new("write", "([BII)I", Self::write, JavaMethodFlag::NONE),
                JavaMethodProto::new("read", "([B)I", Self::read, JavaMethodFlag::NONE),
                JavaMethodProto::new("close", "()V", Self::close, JavaMethodFlag::NONE),
                JavaMethodProto::new("sizeOf", "()I", Self::size_of, JavaMethodFlag::NONE),
            ],
            fields: vec![
                JavaFieldProto::new("data", "[B", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("pos", "I", JavaFieldAccessFlag::NONE),
            ],
        }
    }

    async fn init(
        jvm: &mut Jvm,
        _: &mut WIPIJavaContxt,
        this: ClassInstanceRef<Self>,
        filename: ClassInstanceRef<String>,
        mode: i32,
    ) -> JavaResult<()> {
        tracing::debug!("org.kwis.msp.io.File::<init>({:?}, {:?}, {:?})", &this, &filename, mode);

        jvm.invoke_special(&this, "org/kwis/msp/io/File", "<init>", "(Ljava/lang/String;II)V", (filename, mode, 0))
            .await?;

        Ok(())
    }

    #[allow(clippy::await_holding_refcell_ref)] // We manually drop Ref
    async fn init_with_flag(
        jvm: &mut Jvm,
        context: &mut WIPIJavaContxt,
        mut this: ClassInstanceRef<Self>,
        filename: ClassInstanceRef<String>,
        mode: i32,
        flag: i32,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.io.File::<init>({:?}, {:?}, {:?}, {:?})", &this, &filename, mode, flag);

        let filename = String::to_rust_string(jvm, &filename)?;
        tracing::debug!("Loading {}", filename);

        // TODO we don't have filesystem now, emulating file loading with resource for now..
        // P directory is on ktf archive

        let filename_on_resource = format!("P{}", filename);

        let resource = context.system().resource();
        let data = resource.data(resource.id(&filename_on_resource).unwrap());

        let data = cast_slice(data).to_vec();
        drop(resource);

        let mut data_array = jvm.instantiate_array("B", data.len() as _).await?;
        jvm.store_byte_array(&mut data_array, 0, data)?;

        jvm.put_field(&mut this, "data", "[B", data_array)?;
        jvm.put_field(&mut this, "pos", "I", 0)?;

        Ok(())
    }

    async fn write(
        _jvm: &mut Jvm,
        _: &mut WIPIJavaContxt,
        this: ClassInstanceRef<Self>,
        buf: ClassInstanceRef<ClassInstanceRef<Array<i8>>>,
        offset: i32,
        len: i32,
    ) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.File::write({:?}, {:?}, {:?}, {:?})", &this, &buf, offset, len);

        Ok(0)
    }

    async fn read(jvm: &mut Jvm, _: &mut WIPIJavaContxt, mut this: ClassInstanceRef<Self>, mut buf: ClassInstanceRef<Array<i8>>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.io.File::read({:?}, {:?})", &this, &buf);

        let data_array = jvm.get_field(&this, "data", "[B")?;
        let pos: i32 = jvm.get_field(&this, "pos", "I")?;

        let data_len = jvm.array_length(&data_array)?;
        let buf_len = jvm.array_length(&buf)?;

        let length_to_read = min(data_len - pos as usize, buf_len);

        let data = jvm.load_byte_array(&data_array, pos as _, length_to_read)?;
        jvm.store_byte_array(&mut buf, 0, data)?;

        jvm.put_field(&mut this, "pos", "I", pos + length_to_read as i32)?;

        Ok(length_to_read as _)
    }

    async fn close(_jvm: &mut Jvm, _: &mut WIPIJavaContxt, this: ClassInstanceRef<Self>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.io.File::close({:?})", &this);

        Ok(())
    }

    async fn size_of(jvm: &mut Jvm, _: &mut WIPIJavaContxt, this: ClassInstanceRef<Self>) -> JavaResult<i32> {
        tracing::debug!("org.kwis.msp.io.File::sizeOf({:?})", &this);

        let data_array = jvm.get_field(&this, "data", "[B")?;
        let data_len = jvm.array_length(&data_array)?;

        Ok(data_len as _)
    }
}
