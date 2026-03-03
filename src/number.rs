use serde::de::Unexpected;
use swc_ecma_ast::Number;

/// Returns `true` if the number's raw source text contains no decimal point.
///
/// This uses the source representation rather than the value, so `1.0` returns `false` (has a
/// decimal) while `1e5` returns `true` (no decimal, value 100000).
///
/// A number with no raw source is treated as non-integer.
pub fn is_integer(number: &Number) -> bool {
    number
        .raw
        .as_ref()
        .is_some_and(|atom| !atom.as_str().contains('.'))
}

/// Returns a serde [`Unexpected`] variant classifying the number as signed, unsigned,
/// or float, based on its raw representation and value.
///
/// Returns `None` when the number is an integer but outside both `i64` and `u64` range.
pub fn number_to_unexpected(number: &Number) -> Option<Unexpected<'_>> {
    if is_integer(number) {
        let value = number.value;

        // `i64::MIN`` is exactly representable as `f64`.
        // `i64::MAX` rounds up to `2 ^ 63` when cast to `f64`, so we use that as an exclusive upper
        // bound to avoid incorrectly accepting that value.
        if (-9_223_372_036_854_775_808.0_f64..9_223_372_036_854_775_808.0_f64).contains(&value) {
            #[allow(clippy::cast_possible_truncation)]
            let signed = value as i64;

            Some(Unexpected::Signed(signed))
        } else if (0.0_f64..18_446_744_073_709_551_616.0_f64).contains(&value) {
            // Similarly, `u64::MAX` rounds up to `2 ^ 64` as `f64`, so use that as exclusive bound.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let unsigned = value as u64;

            Some(Unexpected::Unsigned(unsigned))
        } else {
            None
        }
    } else {
        Some(Unexpected::Float(number.value))
    }
}

// Generates a `pub fn number_to_T(number: &Number) -> Option<T>` for each integer type.
//
// All bounds are expressed as exact `f64` literals (powers of 2) in order to avoid
// `cast_precision_loss`. Wide-type max values round up when cast to `f64``, so we use exclusive
// upper bounds rather than inclusive ones, which would allow one out-of-range value through.
macro_rules! number_to_int {
    ($fn_name:ident, $ty:ty, $min_f64:literal, $max_excl_f64:literal) => {
        pub fn $fn_name(number: &Number) -> Option<$ty> {
            if is_integer(number) {
                let value = number.value;

                if ($min_f64..$max_excl_f64).contains(&value) {
                    // Truncation is intentional: we verified the value is a whole number (no decimal
                    // point) and within the target type's exact range.
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    Some(value as $ty)
                } else {
                    None
                }
            } else {
                None
            }
        }
    };
}

// Signed integer types.
number_to_int!(number_to_i8, i8, -128.0, 128.0);
number_to_int!(number_to_i16, i16, -32_768.0, 32_768.0);
number_to_int!(number_to_i32, i32, -2_147_483_648.0, 2_147_483_648.0);
number_to_int!(
    number_to_i64,
    i64,
    -9_223_372_036_854_775_808.0,
    9_223_372_036_854_775_808.0
);
number_to_int!(
    number_to_i128,
    i128,
    -170_141_183_460_469_231_731_687_303_715_884_105_728.0,
    170_141_183_460_469_231_731_687_303_715_884_105_728.0
);

// Unsigned integer types.
number_to_int!(number_to_u8, u8, 0.0, 256.0);
number_to_int!(number_to_u16, u16, 0.0, 65_536.0);
number_to_int!(number_to_u32, u32, 0.0, 4_294_967_296.0);
number_to_int!(number_to_u64, u64, 0.0, 18_446_744_073_709_551_616.0);
number_to_int!(
    number_to_u128,
    u128,
    0.0,
    340_282_366_920_938_463_463_374_607_431_768_211_456.0
);
