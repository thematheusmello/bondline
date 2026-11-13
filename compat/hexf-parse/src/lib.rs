//! Independently implemented exact hexadecimal float conversion, not upstream hexf-parse.

use std::{borrow::Cow, fmt};

/// Invalid syntax, overflow, underflow, or an inexact IEEE-754 conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseHexfError;

impl fmt::Display for ParseHexfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("hexadecimal float is invalid or not exactly representable")
    }
}
impl std::error::Error for ParseHexfError {}

/// Parse an exactly representable binary32 literal for the pinned WGSL frontend.
pub fn parse_hexf32(input: &str, underscores: bool) -> Result<f32, ParseHexfError> {
    let bits = exact_bits(input, underscores, 24, -149, -126, 127)?;
    Ok(f32::from_bits(
        u32::try_from(bits).map_err(|_| ParseHexfError)?,
    ))
}

/// Parse an exactly representable binary64 literal for the pinned WGSL frontend.
pub fn parse_hexf64(input: &str, underscores: bool) -> Result<f64, ParseHexfError> {
    exact_bits(input, underscores, 53, -1074, -1022, 1023).map(f64::from_bits)
}

fn exact_bits(
    input: &str,
    underscores: bool,
    precision: u32,
    min_power: i64,
    min_normal: i64,
    bias: i64,
) -> Result<u64, ParseHexfError> {
    if input.len() > 65_536 {
        return Err(ParseHexfError);
    }
    let text: Cow<'_, str> = if underscores && input.contains('_') {
        Cow::Owned(input.chars().filter(|c| *c != '_').collect())
    } else {
        Cow::Borrowed(input)
    };
    let (negative, text) = if let Some(s) = text.strip_prefix('-') {
        (true, s)
    } else {
        (false, text.strip_prefix('+').unwrap_or(&text))
    };
    let sign = u64::from(negative) << (if precision == 24 { 31 } else { 63 });
    let text = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .ok_or(ParseHexfError)?;
    let (mantissa, exponent) = text.split_once(['p', 'P']).unwrap_or((text, "0"));
    let exponent = exponent.parse::<i64>().map_err(|_| ParseHexfError)?;
    let mut digits = Vec::new();
    let mut point = false;
    let mut fractional_digits = 0_i64;
    for c in mantissa.chars() {
        if c == '.' && !point {
            point = true;
            continue;
        }
        if !c.is_ascii_hexdigit() {
            return Err(ParseHexfError);
        }
        digits.push(c.to_digit(16).ok_or(ParseHexfError)?);
        if point {
            fractional_digits = fractional_digits.checked_add(1).ok_or(ParseHexfError)?;
        }
    }
    if digits.is_empty() {
        return Err(ParseHexfError);
    }
    let Some(first) = digits.iter().position(|d| *d != 0) else {
        return Ok(sign);
    };
    let last = digits.iter().rposition(|d| *d != 0).ok_or(ParseHexfError)?;
    let first_digit = *digits.get(first).ok_or(ParseHexfError)?;
    let last_digit = *digits.get(last).ok_or(ParseHexfError)?;
    let low_zeros = last_digit.trailing_zeros();
    let bits = u32::try_from(last - first)
        .map_err(|_| ParseHexfError)?
        .checked_mul(4)
        .and_then(|n| n.checked_add(32 - first_digit.leading_zeros()))
        .and_then(|n| n.checked_sub(low_zeros))
        .ok_or(ParseHexfError)?;
    if bits > precision {
        return Err(ParseHexfError);
    }
    let mut significand = 0_u64;
    for digit in digits.get(first..=last).ok_or(ParseHexfError)? {
        significand = significand
            .checked_mul(16)
            .and_then(|n| n.checked_add(u64::from(*digit)))
            .ok_or(ParseHexfError)?;
    }
    significand >>= low_zeros;
    let trailing_digits = i64::try_from(digits.len() - 1 - last).map_err(|_| ParseHexfError)?;
    let power = fractional_digits
        .checked_mul(4)
        .and_then(|n| exponent.checked_sub(n))
        .and_then(|n| n.checked_add(trailing_digits * 4))
        .and_then(|n| n.checked_add(i64::from(low_zeros)))
        .ok_or(ParseHexfError)?;
    let high_power = power
        .checked_add(i64::from(bits) - 1)
        .ok_or(ParseHexfError)?;
    if power < min_power || high_power > bias {
        return Err(ParseHexfError);
    }
    let fraction_bits = precision - 1;
    let payload = if high_power >= min_normal {
        let exponent_bits = u64::try_from(high_power + bias).map_err(|_| ParseHexfError)?;
        let fraction = (significand << (precision - bits)) & ((1_u64 << fraction_bits) - 1);
        (exponent_bits << fraction_bits) | fraction
    } else {
        significand
            .checked_shl(u32::try_from(power - min_power).map_err(|_| ParseHexfError)?)
            .ok_or(ParseHexfError)?
    };
    Ok(sign | payload)
}

#[cfg(test)]
mod tests {
    use super::{parse_hexf32, parse_hexf64};

    #[test]
    fn ieee_reference_vectors_are_exact() {
        // Independently justified IEEE-754 bit patterns; WGSL numeric literal rules.
        for (text, bits) in [
            ("0x1p0", 0x3ff0_0000_0000_0000),
            ("0x1.8p1", 0x4008_0000_0000_0000),
            ("-0x0p0", 0x8000_0000_0000_0000),
            ("0x1p-1074", 1),
            ("0x1p-1022", 0x0010_0000_0000_0000),
            ("0x0.fffffffffffffp-1022", 0x000f_ffff_ffff_ffff),
            ("0x1.fffffffffffffp1023", 0x7fef_ffff_ffff_ffff),
            ("0x0001000.0000p-12", 0x3ff0_0000_0000_0000),
        ] {
            assert_eq!(
                parse_hexf64(text, false).map(f64::to_bits),
                Ok(bits),
                "{text}"
            );
        }
        for (text, bits) in [
            ("0x1p-149", 1),
            ("0x1.fffffep127", 0x7f7f_ffff),
            ("0x1.8p1", 0x4040_0000),
        ] {
            assert_eq!(
                parse_hexf32(text, false).map(f32::to_bits),
                Ok(bits),
                "{text}"
            );
        }
    }

    #[test]
    fn rejects_inexact_overflow_underflow_and_malformed_literals() {
        for text in [
            "0x1.00000000000001p0",
            "0x1p1024",
            "0x1p-1075",
            "0x.p1",
            "0x1p",
            "0x1p+",
            "0x1.2.3p1",
            "NaN",
            "0x1p1p1",
            " 0x1p0",
            "0x1p99999999999999999999999",
        ] {
            assert!(parse_hexf64(text, false).is_err(), "{text}");
        }
        assert!(parse_hexf32("0x1.000001p0", false).is_err());
        assert!(parse_hexf32("0x1p-150", false).is_err());
    }

    #[test]
    fn underscore_permission_is_explicit() {
        assert!(parse_hexf64("0x1_0p0", false).is_err());
        assert_eq!(parse_hexf64("0x1_0p0", true), Ok(16.0));
    }

    #[test]
    fn oversized_literals_are_rejected_before_allocating_digit_storage() {
        let text = format!("0x{}p0", "0".repeat(65_536));
        assert!(parse_hexf64(&text, false).is_err());
    }
}
