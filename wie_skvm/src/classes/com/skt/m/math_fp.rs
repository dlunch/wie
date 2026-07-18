use alloc::{format, string::String as RustString, vec};

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

const SCALE: i64 = 1_000_000_000;
const HALF: i64 = SCALE / 2;
const E: i64 = 2_718_281_828;
const PI: i64 = 3_141_592_654;
const HALF_PI: i64 = 1_570_796_327;
const TAU: i64 = 6_283_185_308;
const LN_2: i64 = 693_147_181;

// class com.skt.m.MathFP
pub struct MathFP;

impl MathFP {
    pub fn as_proto() -> WieJavaClassProto {
        let public_static = MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC;
        let public_static_final = FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL;

        WieJavaClassProto {
            name: "com/skt/m/MathFP",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("abs", "(J)J", Self::abs, public_static),
                JavaMethodProto::new("acos", "(J)J", Self::acos, public_static),
                JavaMethodProto::new("add", "(JJ)J", Self::add, public_static),
                JavaMethodProto::new("asin", "(J)J", Self::asin, public_static),
                JavaMethodProto::new("atan", "(J)J", Self::atan, public_static),
                JavaMethodProto::new("cos", "(J)J", Self::cos, public_static),
                JavaMethodProto::new("divide", "(JJ)J", Self::divide, public_static),
                JavaMethodProto::new("exp", "(J)J", Self::exp, public_static),
                JavaMethodProto::new("log", "(J)J", Self::log, public_static),
                JavaMethodProto::new("max", "(JJ)J", Self::max, public_static),
                JavaMethodProto::new("min", "(JJ)J", Self::min, public_static),
                JavaMethodProto::new("multiply", "(JJ)J", Self::multiply, public_static),
                JavaMethodProto::new("parseFP", "(J)J", Self::parse_fp, public_static),
                JavaMethodProto::new("parseFPString", "(Ljava/lang/String;)J", Self::parse_fp_string, public_static),
                JavaMethodProto::new("pow", "(JJ)J", Self::pow, public_static),
                JavaMethodProto::new("round", "(J)J", Self::round, public_static),
                JavaMethodProto::new("sin", "(J)J", Self::sin, public_static),
                JavaMethodProto::new("sqrt", "(J)J", Self::sqrt, public_static),
                JavaMethodProto::new("sub", "(JJ)J", Self::sub, public_static),
                JavaMethodProto::new("tan", "(J)J", Self::tan, public_static),
                JavaMethodProto::new("toLong", "(J)J", Self::to_long, public_static),
                JavaMethodProto::new("toStringE", "(J)Ljava/lang/String;", Self::to_string_e, public_static),
                JavaMethodProto::new("toStringLF", "(JI)Ljava/lang/String;", Self::to_string_lf, public_static),
            ],
            fields: vec![
                JavaFieldProto::new("E", "J", public_static_final),
                JavaFieldProto::new("MAX_VALUE", "J", public_static_final),
                JavaFieldProto::new("MIN_VALUE", "J", public_static_final),
                JavaFieldProto::new("PI", "J", public_static_final),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.skt.m.MathFP::<clinit>()");

        jvm.put_static_field("com/skt/m/MathFP", "E", "J", E).await?;
        jvm.put_static_field("com/skt/m/MathFP", "MAX_VALUE", "J", i64::MAX).await?;
        jvm.put_static_field("com/skt/m/MathFP", "MIN_VALUE", "J", i64::MIN).await?;
        jvm.put_static_field("com/skt/m/MathFP", "PI", "J", PI).await
    }

    async fn abs(_jvm: &Jvm, _context: &mut WieJvmContext, value: i64) -> JvmResult<i64> {
        Ok(value.saturating_abs())
    }

    async fn acos(_jvm: &Jvm, _context: &mut WieJvmContext, value: i64) -> JvmResult<i64> {
        Ok(HALF_PI.saturating_sub(Self::asin_fixed(value)))
    }

    async fn add(_jvm: &Jvm, _context: &mut WieJvmContext, a: i64, b: i64) -> JvmResult<i64> {
        Ok(a.saturating_add(b))
    }

    async fn asin(_jvm: &Jvm, _context: &mut WieJvmContext, value: i64) -> JvmResult<i64> {
        Ok(Self::asin_fixed(value))
    }

    async fn atan(_jvm: &Jvm, _context: &mut WieJvmContext, value: i64) -> JvmResult<i64> {
        Ok(Self::atan_fixed(value))
    }

    async fn cos(_jvm: &Jvm, _context: &mut WieJvmContext, value: i64) -> JvmResult<i64> {
        Ok(Self::sin_fixed(value as i128 + HALF_PI as i128))
    }

    async fn divide(jvm: &Jvm, _context: &mut WieJvmContext, a: i64, b: i64) -> JvmResult<i64> {
        match Self::fixed_div(a, b) {
            Some(value) => Ok(value),
            None => Err(jvm.exception("java/lang/ArithmeticException", "division by zero").await),
        }
    }

    async fn exp(_jvm: &Jvm, _context: &mut WieJvmContext, value: i64) -> JvmResult<i64> {
        Ok(Self::exp_fixed(value))
    }

    async fn log(_jvm: &Jvm, _context: &mut WieJvmContext, value: i64) -> JvmResult<i64> {
        Ok(Self::log_fixed(value))
    }

    async fn max(_jvm: &Jvm, _context: &mut WieJvmContext, a: i64, b: i64) -> JvmResult<i64> {
        Ok(a.max(b))
    }

    async fn min(_jvm: &Jvm, _context: &mut WieJvmContext, a: i64, b: i64) -> JvmResult<i64> {
        Ok(a.min(b))
    }

    async fn multiply(_jvm: &Jvm, _context: &mut WieJvmContext, a: i64, b: i64) -> JvmResult<i64> {
        Ok(Self::fixed_mul(a, b))
    }

    async fn parse_fp(_jvm: &Jvm, _context: &mut WieJvmContext, value: i64) -> JvmResult<i64> {
        Ok(Self::saturating_i128_to_i64(value as i128 * SCALE as i128))
    }

    async fn parse_fp_string(jvm: &Jvm, _context: &mut WieJvmContext, value: ClassInstanceRef<String>) -> JvmResult<i64> {
        tracing::debug!("com.skt.m.MathFP::parseFPString({value:?})");

        if value.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "value is null").await);
        }

        let value = JavaLangString::to_rust_string(jvm, &value).await?;
        let input = value.trim();
        let bytes = input.as_bytes();
        let mut index = 0_usize;
        let negative = match bytes.first() {
            Some(b'-') => {
                index = 1;
                true
            }
            Some(b'+') => {
                index = 1;
                false
            }
            _ => false,
        };
        let mut magnitude = 0_i128;
        let mut digits = 0_usize;
        let mut fractional_digits = 0_i64;
        let mut decimal_seen = false;

        while index < bytes.len() {
            match bytes[index] {
                digit @ b'0'..=b'9' => {
                    magnitude = match magnitude.checked_mul(10).and_then(|value| value.checked_add((digit - b'0') as i128)) {
                        Some(value) => value,
                        None => return Err(jvm.exception("java/lang/NumberFormatException", &value).await),
                    };
                    digits += 1;
                    if decimal_seen {
                        fractional_digits += 1;
                    }
                    index += 1;
                }
                b'.' if !decimal_seen => {
                    decimal_seen = true;
                    index += 1;
                }
                b'e' | b'E' => break,
                _ => return Err(jvm.exception("java/lang/NumberFormatException", &value).await),
            }
        }

        if digits == 0 {
            return Err(jvm.exception("java/lang/NumberFormatException", &value).await);
        }

        let mut exponent = 0_i64;
        if index < bytes.len() {
            index += 1;
            let exponent_negative = match bytes.get(index) {
                Some(b'-') => {
                    index += 1;
                    true
                }
                Some(b'+') => {
                    index += 1;
                    false
                }
                _ => false,
            };
            let exponent_start = index;
            while index < bytes.len() {
                let digit = bytes[index];
                if !digit.is_ascii_digit() {
                    return Err(jvm.exception("java/lang/NumberFormatException", &value).await);
                }
                exponent = match exponent.checked_mul(10).and_then(|value| value.checked_add((digit - b'0') as i64)) {
                    Some(value) => value,
                    None => return Err(jvm.exception("java/lang/NumberFormatException", &value).await),
                };
                index += 1;
            }
            if index == exponent_start {
                return Err(jvm.exception("java/lang/NumberFormatException", &value).await);
            }
            if exponent_negative {
                exponent = -exponent;
            }
        }

        let decimal_shift = match exponent.checked_add(9).and_then(|value| value.checked_sub(fractional_digits)) {
            Some(value) => value,
            None => return Err(jvm.exception("java/lang/NumberFormatException", &value).await),
        };
        let raw_magnitude = if magnitude == 0 {
            0
        } else if decimal_shift >= 0 {
            if decimal_shift > 38 {
                return Err(jvm.exception("java/lang/NumberFormatException", &value).await);
            }
            match magnitude.checked_mul(10_i128.pow(decimal_shift as u32)) {
                Some(value) => value,
                None => return Err(jvm.exception("java/lang/NumberFormatException", &value).await),
            }
        } else if decimal_shift < -38 {
            0
        } else {
            magnitude / 10_i128.pow((-decimal_shift) as u32)
        };
        let raw = if negative { raw_magnitude.checked_neg() } else { Some(raw_magnitude) };
        let Some(raw) = raw.and_then(|value| i64::try_from(value).ok()) else {
            return Err(jvm.exception("java/lang/NumberFormatException", &value).await);
        };

        Ok(raw)
    }

    async fn pow(_jvm: &Jvm, _context: &mut WieJvmContext, base: i64, exponent: i64) -> JvmResult<i64> {
        if exponent == 0 {
            return Ok(SCALE);
        }
        if base == 0 {
            return Ok(if exponent > 0 { 0 } else { i64::MAX });
        }
        if base < 0 && exponent % SCALE != 0 {
            return Ok(0);
        }

        let magnitude = Self::exp_fixed(Self::fixed_mul(Self::log_fixed(base.saturating_abs()), exponent));
        if base < 0 && (exponent / SCALE) % 2 != 0 {
            Ok(magnitude.saturating_neg())
        } else {
            Ok(magnitude)
        }
    }

    async fn round(_jvm: &Jvm, _context: &mut WieJvmContext, value: i64) -> JvmResult<i64> {
        let rounded = (value as i128 + HALF as i128).div_euclid(SCALE as i128) * SCALE as i128;
        Ok(Self::saturating_i128_to_i64(rounded))
    }

    async fn sin(_jvm: &Jvm, _context: &mut WieJvmContext, value: i64) -> JvmResult<i64> {
        Ok(Self::sin_fixed(value as i128))
    }

    async fn sqrt(_jvm: &Jvm, _context: &mut WieJvmContext, value: i64) -> JvmResult<i64> {
        Ok(Self::sqrt_fixed(value))
    }

    async fn sub(_jvm: &Jvm, _context: &mut WieJvmContext, a: i64, b: i64) -> JvmResult<i64> {
        Ok(a.saturating_sub(b))
    }

    async fn tan(jvm: &Jvm, _context: &mut WieJvmContext, value: i64) -> JvmResult<i64> {
        let sin = Self::sin_fixed(value as i128);
        let cos = Self::sin_fixed(value as i128 + HALF_PI as i128);
        match Self::fixed_div(sin, cos) {
            Some(value) => Ok(value),
            None => Err(jvm.exception("java/lang/ArithmeticException", "undefined tangent").await),
        }
    }

    async fn to_long(_jvm: &Jvm, _context: &mut WieJvmContext, value: i64) -> JvmResult<i64> {
        Ok(value / SCALE)
    }

    async fn to_string_e(jvm: &Jvm, _context: &mut WieJvmContext, value: i64) -> JvmResult<ClassInstanceRef<String>> {
        let magnitude = value.unsigned_abs();
        let result = if magnitude == 0 {
            RustString::from("0e0")
        } else {
            let digits = format!("{magnitude}");
            let significant = digits.trim_end_matches('0');
            let exponent = digits.len() as i32 - 10;
            let coefficient = if significant.len() == 1 {
                RustString::from(significant)
            } else {
                format!("{}.{}", &significant[..1], &significant[1..])
            };
            format!("{}{coefficient}e{exponent}", if value < 0 { "-" } else { "" })
        };
        Ok(JavaLangString::from_rust_string(jvm, &result).await?.into())
    }

    async fn to_string_lf(jvm: &Jvm, _context: &mut WieJvmContext, value: i64, precision: i32) -> JvmResult<ClassInstanceRef<String>> {
        if !(0..=9).contains(&precision) {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "precision must be between 0 and 9")
                .await);
        }

        let divisor = 10_u128.pow((9 - precision) as u32);
        let magnitude = value.unsigned_abs() as u128;
        let rounded = (magnitude + divisor / 2) / divisor * divisor;
        let integer = rounded / SCALE as u128;
        let result = if precision == 0 {
            format!("{}{integer}", if value < 0 { "-" } else { "" })
        } else {
            let fraction = (rounded % SCALE as u128) / divisor;
            format!(
                "{}{integer}.{fraction:0width$}",
                if value < 0 { "-" } else { "" },
                width = precision as usize
            )
        };

        Ok(JavaLangString::from_rust_string(jvm, &result).await?.into())
    }

    fn saturating_i128_to_i64(value: i128) -> i64 {
        if value > i64::MAX as i128 {
            i64::MAX
        } else if value < i64::MIN as i128 {
            i64::MIN
        } else {
            value as i64
        }
    }

    fn fixed_mul(a: i64, b: i64) -> i64 {
        Self::saturating_i128_to_i64(a as i128 * b as i128 / SCALE as i128)
    }

    fn fixed_div(a: i64, b: i64) -> Option<i64> {
        if b == 0 {
            None
        } else {
            Some(Self::saturating_i128_to_i64(a as i128 * SCALE as i128 / b as i128))
        }
    }

    fn sin_fixed(angle: i128) -> i64 {
        let mut x = angle % TAU as i128;
        if x > PI as i128 {
            x -= TAU as i128;
        } else if x < -(PI as i128) {
            x += TAU as i128;
        }
        if x > HALF_PI as i128 {
            x = PI as i128 - x;
        } else if x < -(HALF_PI as i128) {
            x = -(PI as i128) - x;
        }

        let x = x as i64;
        let x_squared = Self::fixed_mul(x, x);
        let mut result = x;
        let mut term = x;
        for (index, divisor) in [6_i64, 20, 42, 72, 110, 156].into_iter().enumerate() {
            term = Self::fixed_mul(term, x_squared) / divisor;
            result = if index % 2 == 0 {
                result.saturating_sub(term)
            } else {
                result.saturating_add(term)
            };
        }
        result
    }

    fn atan_fixed(value: i64) -> i64 {
        let negative = value < 0;
        let magnitude = value.saturating_abs();
        let (x, reciprocal) = if magnitude > SCALE {
            let reciprocal = Self::saturating_i128_to_i64(SCALE as i128 * SCALE as i128 / magnitude as i128);
            (reciprocal, true)
        } else {
            (magnitude, false)
        };

        // A compact approximation with less than 0.004 radians error on [0, 1].
        let correction = Self::fixed_mul(273_000_000, SCALE - x);
        let mut result = Self::fixed_mul(x, 785_398_164_i64.saturating_add(correction));
        if reciprocal {
            result = HALF_PI.saturating_sub(result);
        }
        if negative { result.saturating_neg() } else { result }
    }

    fn asin_fixed(value: i64) -> i64 {
        if !(-SCALE..=SCALE).contains(&value) {
            return 0;
        }
        if value == SCALE {
            return HALF_PI;
        }
        if value == -SCALE {
            return -HALF_PI;
        }

        let complement = SCALE.saturating_sub(Self::fixed_mul(value, value));
        let root = Self::sqrt_fixed(complement);
        match Self::fixed_div(value, root) {
            Some(ratio) => Self::atan_fixed(ratio),
            None => 0,
        }
    }

    fn sqrt_fixed(value: i64) -> i64 {
        if value <= 0 {
            return 0;
        }

        let radicand = value as u128 * SCALE as u128;
        let mut estimate = radicand;
        let mut next = estimate.div_ceil(2);
        while next < estimate {
            estimate = next;
            next = (estimate + radicand / estimate) / 2;
        }
        estimate as i64
    }

    fn exp_fixed(value: i64) -> i64 {
        let exponent = (value as i128).div_euclid(LN_2 as i128);
        if exponent > 62 {
            return i64::MAX;
        }
        if exponent < -62 {
            return 0;
        }

        let remainder = (value as i128 - exponent * LN_2 as i128) as i64;
        let mut result = SCALE;
        let mut term = SCALE;
        for divisor in 1_i64..=16 {
            term = Self::fixed_mul(term, remainder) / divisor;
            result = result.saturating_add(term);
        }

        if exponent >= 0 {
            Self::saturating_i128_to_i64((result as i128) << exponent as u32)
        } else {
            result >> (-exponent) as u32
        }
    }

    fn log_fixed(value: i64) -> i64 {
        if value <= 0 {
            return 0;
        }

        let mut normalized = value;
        let mut exponent = 0_i64;
        while normalized >= SCALE * 2 {
            normalized /= 2;
            exponent += 1;
        }
        while normalized < SCALE {
            normalized *= 2;
            exponent -= 1;
        }

        let y = Self::saturating_i128_to_i64((normalized - SCALE) as i128 * SCALE as i128 / (normalized + SCALE) as i128);
        let y_squared = Self::fixed_mul(y, y);
        let mut term = y;
        let mut series = y;
        for divisor in (3_i64..=19).step_by(2) {
            term = Self::fixed_mul(term, y_squared);
            series = series.saturating_add(term / divisor);
        }

        Self::saturating_i128_to_i64(series as i128 * 2 + exponent as i128 * LN_2 as i128)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;
    use jvm::{ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use test_utils::run_jvm_test;

    use crate::get_protos;

    use super::E;

    #[test]
    fn test_fixed_point_arithmetic() {
        let result = run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let two_point_five: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "2.5").await?.into();
            let one_point_two_five: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "1.25").await?.into();

            let a: i64 = jvm
                .invoke_static("com/skt/m/MathFP", "parseFPString", "(Ljava/lang/String;)J", (two_point_five,))
                .await?;
            let b: i64 = jvm
                .invoke_static("com/skt/m/MathFP", "parseFPString", "(Ljava/lang/String;)J", (one_point_two_five,))
                .await?;

            assert_eq!(a, 2_500_000_000);
            assert_eq!(b, 1_250_000_000);
            assert_eq!(
                jvm.invoke_static::<_, i64>("com/skt/m/MathFP", "add", "(JJ)J", (a, b)).await?,
                3_750_000_000
            );
            assert_eq!(
                jvm.invoke_static::<_, i64>("com/skt/m/MathFP", "sub", "(JJ)J", (a, b)).await?,
                1_250_000_000
            );
            assert_eq!(
                jvm.invoke_static::<_, i64>("com/skt/m/MathFP", "multiply", "(JJ)J", (a, b)).await?,
                3_125_000_000
            );
            assert_eq!(
                jvm.invoke_static::<_, i64>("com/skt/m/MathFP", "divide", "(JJ)J", (a, b)).await?,
                2_000_000_000
            );
            assert_eq!(
                jvm.invoke_static::<_, i64>("com/skt/m/MathFP", "parseFP", "(J)J", (4_i64,)).await?,
                4_000_000_000
            );

            let pi: i64 = jvm.get_static_field("com/skt/m/MathFP", "PI", "J").await?;
            let sine: i64 = jvm.invoke_static("com/skt/m/MathFP", "sin", "(J)J", (pi / 2,)).await?;
            let square_root: i64 = jvm.invoke_static("com/skt/m/MathFP", "sqrt", "(J)J", (4_000_000_000_i64,)).await?;
            let exponential: i64 = jvm.invoke_static("com/skt/m/MathFP", "exp", "(J)J", (1_000_000_000_i64,)).await?;
            let logarithm: i64 = jvm.invoke_static("com/skt/m/MathFP", "log", "(J)J", (exponential,)).await?;

            assert!((sine - 1_000_000_000).abs() < 1_000);
            assert_eq!(square_root, 2_000_000_000);
            assert!((exponential - E).abs() < 1_000);
            assert!((logarithm - 1_000_000_000).abs() < 1_000);

            Ok(())
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_fp_string_reports_number_format_exception() {
        let result = run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let invalid: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "not-a-number").await?.into();
            let parse_result: JvmResult<i64> = jvm
                .invoke_static("com/skt/m/MathFP", "parseFPString", "(Ljava/lang/String;)J", (invalid,))
                .await;
            let Err(JavaError::JavaException(exception)) = parse_result else {
                panic!("parseFPString accepted an invalid number");
            };

            assert!(jvm.is_instance(&*exception, "java/lang/NumberFormatException"));

            let null_value = ClassInstanceRef::<String>::new(None);
            let null_result: JvmResult<i64> = jvm
                .invoke_static("com/skt/m/MathFP", "parseFPString", "(Ljava/lang/String;)J", (null_value,))
                .await;
            let Err(JavaError::JavaException(exception)) = null_result else {
                panic!("parseFPString accepted null");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

            let overflow: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "9223372036.854775808").await?.into();
            let overflow_result: JvmResult<i64> = jvm
                .invoke_static("com/skt/m/MathFP", "parseFPString", "(Ljava/lang/String;)J", (overflow,))
                .await;
            let Err(JavaError::JavaException(exception)) = overflow_result else {
                panic!("parseFPString accepted a value above the raw long range");
            };
            assert!(jvm.is_instance(&*exception, "java/lang/NumberFormatException"));
            Ok(())
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_large_fixed_point_values_preserve_raw_precision() {
        let result = run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let decimal: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "9000000.000000001").await?.into();
            let raw: i64 = jvm
                .invoke_static("com/skt/m/MathFP", "parseFPString", "(Ljava/lang/String;)J", (decimal,))
                .await?;
            assert_eq!(raw, 9_000_000_000_000_001);

            let exponential: ClassInstanceRef<String> = jvm
                .invoke_static("com/skt/m/MathFP", "toStringE", "(J)Ljava/lang/String;", (raw,))
                .await?;
            assert_eq!(JavaLangString::to_rust_string(&jvm, &exponential).await?, "9.000000000000001e6");
            assert_eq!(
                jvm.invoke_static::<_, i64>("com/skt/m/MathFP", "parseFPString", "(Ljava/lang/String;)J", (exponential,))
                    .await?,
                raw
            );

            let fixed: ClassInstanceRef<String> = jvm
                .invoke_static("com/skt/m/MathFP", "toStringLF", "(JI)Ljava/lang/String;", (raw, 9))
                .await?;
            assert_eq!(JavaLangString::to_rust_string(&jvm, &fixed).await?, "9000000.000000001");

            let minimum: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "-9223372036.854775808").await?.into();
            assert_eq!(
                jvm.invoke_static::<_, i64>("com/skt/m/MathFP", "parseFPString", "(Ljava/lang/String;)J", (minimum,))
                    .await?,
                i64::MIN
            );
            Ok(())
        });

        assert!(result.is_ok());
    }
}
