use serde::de::Unexpected;
use swc_ecma_ast::Number;

pub(super) fn is_integer(number: &Number) -> bool {
    number
        .raw
        .as_ref()
        .map(|raw| raw.as_str())
        .filter(|str| !str.contains('.'))
        .is_some()
}

pub(super) fn number_to_unexpected(number: &Number) -> Option<Unexpected<'_>> {
    if is_integer(number) {
        if number.value <= i64::MAX as f64 {
            if number.value >= i64::MIN as f64 {
                Some(Unexpected::Signed(number.value as i64))
            } else {
                None
            }
        } else if number.value <= u64::MAX as f64 && number.value >= 0.0 {
            Some(Unexpected::Unsigned(number.value as u64))
        } else {
            None
        }
    } else {
        Some(Unexpected::Float(number.value))
    }
}

pub(super) fn number_to_i128(number: &Number) -> Option<i128> {
    if is_integer(number) {
        if number.value <= i128::MAX as f64 && number.value >= i128::MIN as f64 {
            Some(number.value as i128)
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn number_to_i16(number: &Number) -> Option<i16> {
    if is_integer(number) {
        if number.value <= i16::MAX as f64 && number.value >= i16::MIN as f64 {
            Some(number.value as i16)
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn number_to_i32(number: &Number) -> Option<i32> {
    if is_integer(number) {
        if number.value <= i32::MAX as f64 && number.value >= i32::MIN as f64 {
            Some(number.value as i32)
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn number_to_i64(number: &Number) -> Option<i64> {
    if is_integer(number) {
        if number.value <= i64::MAX as f64 && number.value >= i64::MIN as f64 {
            Some(number.value as i64)
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn number_to_i8(number: &Number) -> Option<i8> {
    if is_integer(number) {
        if number.value <= i8::MAX as f64 && number.value >= i8::MIN as f64 {
            Some(number.value as i8)
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn number_to_u128(number: &Number) -> Option<u128> {
    if is_integer(number) {
        if number.value <= u128::MAX as f64 && number.value >= u128::MIN as f64 {
            Some(number.value as u128)
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn number_to_u16(number: &Number) -> Option<u16> {
    if is_integer(number) {
        if number.value <= u16::MAX as f64 && number.value >= u16::MIN as f64 {
            Some(number.value as u16)
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn number_to_u32(number: &Number) -> Option<u32> {
    if is_integer(number) {
        if number.value <= u32::MAX as f64 && number.value >= u32::MIN as f64 {
            Some(number.value as u32)
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn number_to_u64(number: &Number) -> Option<u64> {
    if is_integer(number) {
        if number.value <= u64::MAX as f64 && number.value >= u64::MIN as f64 {
            Some(number.value as u64)
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn number_to_u8(number: &Number) -> Option<u8> {
    if is_integer(number) {
        if number.value <= u8::MAX as f64 && number.value >= u8::MIN as f64 {
            Some(number.value as u8)
        } else {
            None
        }
    } else {
        None
    }
}
