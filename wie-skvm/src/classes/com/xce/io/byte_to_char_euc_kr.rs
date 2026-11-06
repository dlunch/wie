use alloc::vec;

use bytemuck::cast_slice;
use encoding_rs::EUC_KR;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::com::xce::io::ByteToCharConverter;

// class com.xce.io.ByteToCharEUC_KR
pub struct ByteToCharEucKr;

impl ByteToCharEucKr {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/io/ByteToCharEUC_KR",
            parent_class: Some("com/xce/io/ByteToCharConverter"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("convert", "([BII[CII)I", Self::convert, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getNumberOfChars", "([BII)I", Self::get_number_of_chars, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.ByteToCharEUC_KR::<init>({this:?})");

        jvm.invoke_special(&this, "com/xce/io/ByteToCharConverter", "<init>", "()V", ()).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn convert(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<ByteToCharConverter>,
        input: ClassInstanceRef<Array<i8>>,
        input_offset: i32,
        input_length: i32,
        output: ClassInstanceRef<Array<u16>>,
        output_offset: i32,
        output_capacity: i32,
    ) -> JvmResult<i32> {
        tracing::debug!(
            "com.xce.io.ByteToCharEUC_KR::convert({this:?}, {input:?}, {input_offset}, {input_length}, {output:?}, {output_offset}, {output_capacity})"
        );

        if input.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "input is null").await);
        }
        let length = jvm.array_length(&input).await? as i32;
        if input_offset < 0 || input_length < 0 || input_offset > length - input_length {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid input range").await);
        }
        let bytes = jvm.load_array(&input, input_offset as usize, input_length as usize).await?;
        ByteToCharConverter::decode(jvm, this, &bytes, output, output_offset, output_capacity, false).await
    }

    async fn get_number_of_chars(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        input: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.ByteToCharEUC_KR::getNumberOfChars({this:?}, {input:?}, {offset}, {length})");

        if input.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "input is null").await);
        }
        let input_length = jvm.array_length(&input).await? as i32;
        if offset < 0 || length < 0 || offset > input_length - length {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid input range").await);
        }
        let bytes = jvm.load_array::<i8>(&input, offset as usize, length as usize).await?;
        let (chars, _) = EUC_KR.decode_without_bom_handling(cast_slice(&bytes));
        Ok(chars.encode_utf16().count() as i32)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{Array, ClassInstanceRef, JavaError};
    use test_utils::run_jvm_test;

    #[test]
    fn euc_kr_conversion_preserves_offsets_pending_bytes_and_flush_state() {
        let result = run_jvm_test(Box::new([wie_midp::get_protos().into(), crate::get_protos().into()]), |jvm| async move {
            let converter = jvm.new_class("com/xce/io/ByteToCharEUC_KR", "()V", ()).await?;
            let mut input = jvm.instantiate_array("B", 8).await?;
            jvm.store_array(&mut input, 0, [88i8, 65, -80, -95, -77, -86, 90, 89]).await?;
            let mut output = jvm.instantiate_array("C", 8).await?;
            jvm.store_array(&mut output, 0, [0x2603u16; 8]).await?;

            let count: i32 = jvm
                .invoke_virtual(
                    &converter,
                    "com/xce/io/ByteToCharConverter",
                    "getNumberOfChars",
                    "([BII)I",
                    (input.clone(), 1, 6),
                )
                .await?;
            assert_eq!(count, 4);
            let count: i32 = jvm
                .invoke_virtual(
                    &converter,
                    "com/xce/io/ByteToCharConverter",
                    "convert",
                    "([BII[CII)I",
                    (input.clone(), 1, 6, output.clone(), 2, 2),
                )
                .await?;
            assert_eq!(count, 2);
            for (offset, capacity, expected) in [(4, 1, 1), (5, 1, 1), (6, 2, 0), (6, 0, 0)] {
                let count: i32 = jvm
                    .invoke_virtual(
                        &converter,
                        "com/xce/io/ByteToCharConverter",
                        "flush",
                        "([CII)I",
                        (output.clone(), offset, capacity),
                    )
                    .await?;
                assert_eq!(count, expected);
            }
            assert_eq!(
                jvm.load_array::<u16>(&output, 0, 8).await?,
                [0x2603, 0x2603, 65, 0xac00, 0xb098, 90, 0x2603, 0x2603]
            );

            let count: i32 = jvm
                .invoke_virtual(
                    &converter,
                    "com/xce/io/ByteToCharConverter",
                    "convert",
                    "([BII[CII)I",
                    (input.clone(), 2, 1, output.clone(), 0, 1),
                )
                .await?;
            assert_eq!(count, 0);
            let count: i32 = jvm
                .invoke_virtual(
                    &converter,
                    "com/xce/io/ByteToCharConverter",
                    "getNumberOfChars",
                    "([BII)I",
                    (input.clone(), 4, 2),
                )
                .await?;
            assert_eq!(count, 1);
            let count: i32 = jvm
                .invoke_virtual(
                    &converter,
                    "com/xce/io/ByteToCharConverter",
                    "convert",
                    "([BII[CII)I",
                    (input.clone(), 3, 1, output.clone(), 0, 1),
                )
                .await?;
            assert_eq!(count, 1);
            assert_eq!(jvm.load_array::<u16>(&output, 0, 2).await?, [0xac00, 0x2603]);

            // Each output limit must retain exactly the bytes not represented in output.
            for (bytes, expected) in [
                (&[0xff, b'A', 0xb0][..], &[0xfffd, 65, 0xfffd][..]),
                (&[0xb0, b' ', 0xb0, 0xa1][..], &[0xfffd, 32, 0xac00][..]),
                (&[0xef, 0xbb, 0xbf][..], &[0x7664, 0xfffd][..]),
            ] {
                for capacity in 0..=4 {
                    let mut bytes_array = jvm.instantiate_array("B", bytes.len()).await?;
                    jvm.store_array(&mut bytes_array, 0, bytes.iter().map(|byte| *byte as i8)).await?;
                    jvm.store_array(&mut output, 0, [0x2603u16; 8]).await?;
                    let total: i32 = jvm
                        .invoke_virtual(
                            &converter,
                            "com/xce/io/ByteToCharConverter",
                            "getNumberOfChars",
                            "([BII)I",
                            (bytes_array.clone(), 0, bytes.len() as i32),
                        )
                        .await?;
                    assert_eq!(total, expected.len() as i32);
                    let count: i32 = jvm
                        .invoke_virtual(
                            &converter,
                            "com/xce/io/ByteToCharConverter",
                            "convert",
                            "([BII[CII)I",
                            (bytes_array, 0, bytes.len() as i32, output.clone(), 1, capacity),
                        )
                        .await?;
                    let empty: i32 = jvm
                        .invoke_virtual(
                            &converter,
                            "com/xce/io/ByteToCharConverter",
                            "flush",
                            "([CII)I",
                            (output.clone(), count + 1, 0),
                        )
                        .await?;
                    assert_eq!(empty, 0);
                    let mut written = count;
                    for _ in 0..=expected.len() {
                        let flushed: i32 = jvm
                            .invoke_virtual(
                                &converter,
                                "com/xce/io/ByteToCharConverter",
                                "flush",
                                "([CII)I",
                                (output.clone(), written + 1, 1),
                            )
                            .await?;
                        written += flushed;
                    }
                    assert_eq!(written, total);
                    assert_eq!(jvm.load_array::<u16>(&output, 1, written as usize).await?, expected);
                    assert_eq!(jvm.load_array::<u16>(&output, 0, 1).await?, [0x2603]);
                    assert_eq!(jvm.load_array::<u16>(&output, written as usize + 1, 1).await?, [0x2603]);
                }
            }

            jvm.store_array(&mut output, 0, [0x2603u16; 8]).await?;
            for (input_offset, input_length, output_offset, capacity) in [
                (-1, 1, 0, 8),
                (0, -1, 0, 8),
                (7, 2, 0, 8),
                (i32::MAX, 1, 0, 8),
                (0, 1, -1, 8),
                (0, 1, 0, -1),
                (0, 1, 7, 2),
            ] {
                let result = jvm
                    .invoke_virtual::<_, i32>(
                        &converter,
                        "com/xce/io/ByteToCharConverter",
                        "convert",
                        "([BII[CII)I",
                        (input.clone(), input_offset, input_length, output.clone(), output_offset, capacity),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = result else {
                    panic!("Expected invalid conversion range to fail");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IndexOutOfBoundsException"));
            }
            for (method, descriptor, array) in [("getNumberOfChars", "([BII)I", input.clone()), ("flush", "([CII)I", output.clone())] {
                let result = jvm
                    .invoke_virtual::<_, i32>(&converter, "com/xce/io/ByteToCharConverter", method, descriptor, (array, 7, 2))
                    .await;
                let Err(JavaError::JavaException(exception)) = result else {
                    panic!("Expected invalid {method} range to fail");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IndexOutOfBoundsException"));
            }
            for (source, destination) in [
                (ClassInstanceRef::<Array<i8>>::from(None), output.clone().into()),
                (input.clone().into(), ClassInstanceRef::<Array<u16>>::from(None)),
            ] {
                let result = jvm
                    .invoke_virtual::<_, i32>(
                        &converter,
                        "com/xce/io/ByteToCharConverter",
                        "convert",
                        "([BII[CII)I",
                        (source, 0, 0, destination, 0, 0),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = result else {
                    panic!("Expected null conversion array to fail");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));
            }
            assert_eq!(jvm.load_array::<u16>(&output, 0, 8).await?, [0x2603; 8]);
            let empty: i32 = jvm
                .invoke_virtual(&converter, "com/xce/io/ByteToCharConverter", "flush", "([CII)I", (output, 0, 8))
                .await?;
            assert_eq!(empty, 0);
            Ok(())
        });
        assert!(result.is_ok());
    }
}
