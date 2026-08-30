use alloc::{boxed::Box, vec::Vec};

use jvm::{ClassInstance, JavaType, JavaValue};

pub trait NativeJavaValueCodec {
    fn object_from_raw(&self, raw: u32) -> Box<dyn ClassInstance>;
    fn object_to_raw(&self, object: &dyn ClassInstance) -> u32;

    fn decode_word(&self, raw: u32, r#type: &JavaType) -> JavaValue {
        match r#type {
            JavaType::Void => JavaValue::Void,
            JavaType::Boolean => JavaValue::Boolean(raw != 0),
            JavaType::Byte => JavaValue::Byte(raw as i8),
            JavaType::Short => JavaValue::Short(raw as i16),
            JavaType::Int => JavaValue::Int(raw as i32),
            JavaType::Float => JavaValue::Float(f32::from_bits(raw)),
            JavaType::Char => JavaValue::Char(raw as u16),
            JavaType::Class(_) | JavaType::Array(_) => JavaValue::Object((raw != 0).then(|| self.object_from_raw(raw))),
            JavaType::Long | JavaType::Double | JavaType::Method(_, _) => unreachable!(),
        }
    }

    fn decode_wide(&self, low: u32, high: u32, r#type: &JavaType) -> JavaValue {
        let raw = ((high as u64) << 32) | low as u64;
        match r#type {
            JavaType::Long => JavaValue::Long(raw as i64),
            JavaType::Double => JavaValue::Double(f64::from_bits(raw)),
            _ => unreachable!(),
        }
    }

    fn encode_word(&self, value: &JavaValue) -> u32 {
        match value {
            JavaValue::Void => 0,
            JavaValue::Boolean(value) => *value as u32,
            JavaValue::Byte(value) => *value as u32,
            JavaValue::Short(value) => *value as u32,
            JavaValue::Int(value) => *value as u32,
            JavaValue::Float(value) => value.to_bits(),
            JavaValue::Char(value) => *value as u32,
            JavaValue::Object(Some(object)) => self.object_to_raw(&**object),
            JavaValue::Object(None) => 0,
            JavaValue::Long(_) | JavaValue::Double(_) => unreachable!(),
        }
    }

    fn encode_wide(&self, value: &JavaValue) -> (u32, u32) {
        let raw = match value {
            JavaValue::Long(value) => *value as u64,
            JavaValue::Double(value) => value.to_bits(),
            _ => unreachable!(),
        };
        (raw as u32, (raw >> 32) as u32)
    }
}

pub fn method_argument_word_count(types: &[JavaType]) -> usize {
    types
        .iter()
        .map(|r#type| usize::from(matches!(r#type, JavaType::Long | JavaType::Double)) + 1)
        .sum()
}

pub fn encode_method_arguments(codec: &impl NativeJavaValueCodec, arguments: &[JavaValue]) -> Vec<u32> {
    let mut words = Vec::with_capacity(arguments.len());
    for argument in arguments {
        if matches!(argument, JavaValue::Long(_) | JavaValue::Double(_)) {
            let (low, high) = codec.encode_wide(argument);
            words.push(low);
            words.push(high);
        } else {
            words.push(codec.encode_word(argument));
        }
    }
    words
}

pub fn decode_method_arguments(codec: &impl NativeJavaValueCodec, types: &[JavaType], words: &[u32]) -> Vec<JavaValue> {
    let mut words = words.iter().copied();
    types
        .iter()
        .map(|r#type| {
            let low = words.next().unwrap();
            if matches!(r#type, JavaType::Long | JavaType::Double) {
                codec.decode_wide(low, words.next().unwrap(), r#type)
            } else {
                codec.decode_word(low, r#type)
            }
        })
        .collect()
}

pub fn array_element_size(r#type: &JavaType) -> usize {
    match r#type {
        JavaType::Boolean | JavaType::Byte => 1,
        JavaType::Char | JavaType::Short => 2,
        JavaType::Int | JavaType::Float | JavaType::Class(_) | JavaType::Array(_) => 4,
        JavaType::Long | JavaType::Double => 8,
        JavaType::Void | JavaType::Method(_, _) => unreachable!(),
    }
}

pub fn encode_array_values(codec: &impl NativeJavaValueCodec, element_type: &JavaType, values: &[JavaValue]) -> Vec<u8> {
    match array_element_size(element_type) {
        1 => values.iter().map(|value| codec.encode_word(value) as u8).collect(),
        2 => values.iter().flat_map(|value| (codec.encode_word(value) as u16).to_le_bytes()).collect(),
        4 => values.iter().flat_map(|value| codec.encode_word(value).to_le_bytes()).collect(),
        8 => values
            .iter()
            .flat_map(|value| {
                let (low, high) = codec.encode_wide(value);
                (((high as u64) << 32) | low as u64).to_le_bytes()
            })
            .collect(),
        _ => unreachable!(),
    }
}

pub fn decode_array_values(codec: &impl NativeJavaValueCodec, element_type: &JavaType, bytes: &[u8]) -> Vec<JavaValue> {
    match array_element_size(element_type) {
        1 => bytes.iter().map(|value| codec.decode_word(*value as u32, element_type)).collect(),
        2 => bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|value| codec.decode_word(u16::from_le_bytes(*value) as u32, element_type))
            .collect(),
        4 => bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|value| codec.decode_word(u32::from_le_bytes(*value), element_type))
            .collect(),
        8 => bytes
            .as_chunks::<8>()
            .0
            .iter()
            .map(|value| {
                let value = u64::from_le_bytes(*value);
                codec.decode_wide(value as u32, (value >> 32) as u32, element_type)
            })
            .collect(),
        _ => unreachable!(),
    }
}
