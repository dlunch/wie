use alloc::vec;

use bytemuck::cast_slice;
use encoding_rs::{CoderResult, EUC_KR};
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.xce.io.ByteToCharConverter
pub struct ByteToCharConverter;

impl ByteToCharConverter {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/io/ByteToCharConverter",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("flush", "([CII)I", Self::flush, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new_abstract("convert", "([BII[CII)I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getNumberOfChars", "([BII)I", MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT),
            ],
            fields: vec![
                JavaFieldProto::new("buf", "[B", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("blen", "I", FieldAccessFlags::PROTECTED),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.ByteToCharConverter::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        let buffer = jvm.instantiate_array("B", 0).await?;
        jvm.put_field(&mut this, "com/xce/io/ByteToCharConverter", "buf", "[B", buffer).await?;
        Ok(())
    }

    async fn flush(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        output: ClassInstanceRef<Array<u16>>,
        offset: i32,
        capacity: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.ByteToCharConverter::flush({this:?}, {output:?}, {offset}, {capacity})");

        Self::decode(jvm, this, &[], output, offset, capacity, true).await
    }

    pub async fn decode(
        jvm: &Jvm,
        mut this: ClassInstanceRef<Self>,
        input: &[i8],
        mut output: ClassInstanceRef<Array<u16>>,
        offset: i32,
        capacity: i32,
        last: bool,
    ) -> JvmResult<i32> {
        if output.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "output is null").await);
        }
        let output_length = jvm.array_length(&output).await? as i32;
        if offset < 0 || capacity < 0 || offset > output_length - capacity {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid output range").await);
        }

        let buffer: ClassInstanceRef<Array<i8>> = jvm.get_field(&this, "com/xce/io/ByteToCharConverter", "buf", "[B").await?;
        let length: i32 = jvm.get_field(&this, "com/xce/io/ByteToCharConverter", "blen", "I").await?;
        let mut bytes = jvm.load_array(&buffer, 0, length as usize).await?;
        bytes.extend_from_slice(input);
        let mut chars = vec![0; (capacity as usize).min(bytes.len())];
        let mut decoder = EUC_KR.new_decoder_without_bom_handling();
        let (result, mut read, written, _) = decoder.decode_to_utf16(cast_slice(&bytes), &mut chars, last);
        if !last || result == CoderResult::OutputFull {
            let (_, _, pending, _) = decoder.decode_to_utf16(&[], &mut [0; 2], true);
            if pending != 0 {
                // EUC-KR can retain one consumed byte; keep it in guest memory before dropping the decoder.
                read -= 1;
            }
        }

        let remaining = &bytes[read..];
        let mut buffer = jvm.instantiate_array("B", remaining.len()).await?;
        jvm.store_array(&mut buffer, 0, remaining.iter().copied()).await?;
        jvm.put_field(&mut this, "com/xce/io/ByteToCharConverter", "buf", "[B", buffer).await?;
        jvm.put_field(&mut this, "com/xce/io/ByteToCharConverter", "blen", "I", remaining.len() as i32)
            .await?;
        jvm.store_array(&mut output, offset as usize, chars.into_iter().take(written)).await?;
        Ok(written as i32)
    }
}
