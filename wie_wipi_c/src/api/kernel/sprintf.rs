use alloc::{format, string::String};

use wie_util::{Result, read_null_terminated_string_bytes};

use crate::context::WIPICContext;

const MAX_WIDTH: usize = 4096;

pub fn sprintf(context: &mut dyn WIPICContext, format: &str, args: &[u32]) -> Result<String> {
    self::format(format, args, &mut |ptr| {
        let bytes = read_null_terminated_string_bytes(context, ptr)?;

        Ok(encoding_rs::EUC_KR.decode(&bytes).0.into_owned())
    })
}

fn format(format: &str, args: &[u32], read_string: &mut dyn FnMut(u32) -> Result<String>) -> Result<String> {
    let mut result = String::with_capacity(format.len());
    let mut chars = format.chars();
    let mut arg_iter = args.iter();

    while let Some(x) = chars.next() {
        if x != '%' {
            result.push(x);
            continue;
        }

        let mut spec = String::from("%");
        let mut flag = None;
        let mut width = None;
        let mut longs = 0u32;
        loop {
            let Some(c) = chars.next() else {
                // broken format: emit what we have as-is
                result.push_str(&spec);
                break;
            };
            spec.push(c);

            match c {
                '%' => {
                    result.push('%');
                    break;
                }
                'd' | 'u' => {
                    // ILP32 ABI: long is one word; only long long occupies two
                    let long = longs >= 2;
                    let raw = if long {
                        next_arg64(&mut arg_iter)
                    } else {
                        next_arg(&mut arg_iter) as u64
                    };
                    // core::fmt panics on width >= 65536; guest-controlled width must be clamped
                    let width = width.unwrap_or(0).min(MAX_WIDTH);
                    if c == 'd' {
                        let arg = if long { raw as i64 } else { raw as u32 as i32 as i64 };
                        if let Some('0') = flag {
                            result += &format!("{arg:0width$}");
                        } else {
                            result += &format!("{arg:width$}");
                        }
                    } else if let Some('0') = flag {
                        result += &format!("{raw:0width$}");
                    } else {
                        result += &format!("{raw:width$}");
                    }
                    break;
                }
                's' => {
                    let ptr = next_arg(&mut arg_iter);
                    if ptr == 0 {
                        result += "(null)";
                        break;
                    }

                    result += &read_string(ptr)?;
                    break;
                }
                'c' => {
                    result.push(next_arg(&mut arg_iter) as u8 as char);
                    break;
                }
                'x' => {
                    let arg = if longs >= 2 {
                        next_arg64(&mut arg_iter)
                    } else {
                        next_arg(&mut arg_iter) as u64
                    };
                    let width = width.unwrap_or(0).min(MAX_WIDTH);
                    if let Some('0') = flag {
                        result += &format!("{arg:0width$x}");
                    } else {
                        result += &format!("{arg:width$x}");
                    }
                    break;
                }
                'l' => longs += 1,
                '0' if width.is_none() => flag = Some('0'),
                '0'..='9' => width = Some(width.unwrap_or(0).saturating_mul(10).saturating_add(c.to_digit(10).unwrap() as usize)),
                _ => {
                    tracing::warn!("unsupported format specifier: {spec}");
                    result.push_str(&spec);
                    break;
                }
            }
        }
    }

    Ok(result)
}

fn next_arg<'a>(arg_iter: &mut impl Iterator<Item = &'a u32>) -> u32 {
    arg_iter.next().copied().unwrap_or_else(|| {
        tracing::warn!("printf: more format specifiers than arguments");
        0
    })
}

// long arguments occupy two consecutive words, low word first
fn next_arg64<'a>(arg_iter: &mut impl Iterator<Item = &'a u32>) -> u64 {
    let low = next_arg(arg_iter) as u64;
    let high = next_arg(arg_iter) as u64;

    (high << 32) | low
}

#[cfg(test)]
mod test {
    use alloc::string::{String, ToString};

    use wie_util::Result;

    fn format(format_string: &str, args: &[u32]) -> Result<String> {
        super::format(format_string, args, &mut |_| Ok("stub".to_string()))
    }

    #[test]
    fn test_unsigned() -> Result<()> {
        assert_eq!(format("%u", &[0xffff_ffff])?, "4294967295");

        Ok(())
    }

    #[test]
    fn test_unknown_specifier_passthrough() -> Result<()> {
        assert_eq!(format("a%qb", &[])?, "a%qb");

        Ok(())
    }

    #[test]
    fn test_more_specifiers_than_args() -> Result<()> {
        assert_eq!(format("%d %d %d %d %d", &[1, 2, 3, 4])?, "1 2 3 4 0");

        Ok(())
    }

    #[test]
    fn test_width_and_zero_flag() -> Result<()> {
        assert_eq!(format("%02d", &[1])?, "01");
        assert_eq!(format("%10d", &[42])?, "        42");
        assert_eq!(format("%d", &[0xffff_ffff])?, "-1");

        Ok(())
    }

    #[test]
    fn test_long_specifiers() -> Result<()> {
        // ILP32: long is one word, so %ld must not shift later arguments
        assert_eq!(format("%ld", &[0xffff_ffff])?, "-1");
        assert_eq!(format("%lu", &[0xffff_ffff])?, "4294967295");
        assert_eq!(format("%ld %d", &[1, 42])?, "1 42");

        // long long arguments are two words, low first
        assert_eq!(format("%lld", &[0xffff_ffff, 0xffff_ffff])?, "-1");
        assert_eq!(format("%llu", &[0xffff_ffff, 0xffff_ffff])?, "18446744073709551615");
        assert_eq!(format("%llx", &[0x9abc_def0, 0x1234_5678])?, "123456789abcdef0");
        assert_eq!(format("%lld %d", &[1, 0, 42])?, "1 42");

        Ok(())
    }

    #[test]
    fn test_hex_width() -> Result<()> {
        assert_eq!(format("%08x", &[0xbeef])?, "0000beef");
        assert_eq!(format("%8x", &[0xbeef])?, "    beef");

        Ok(())
    }

    #[test]
    fn test_huge_width_is_clamped() -> Result<()> {
        // core::fmt panics on width >= 65536; must not propagate guest width unclamped
        assert_eq!(format("%65536d", &[1])?.len(), 4096);
        assert_eq!(format("%99999999999999999999d", &[1])?.len(), 4096);

        Ok(())
    }

    #[test]
    fn test_string_and_null() -> Result<()> {
        assert_eq!(format("%s!", &[1])?, "stub!");
        assert_eq!(format("%s", &[0])?, "(null)");

        Ok(())
    }
}
