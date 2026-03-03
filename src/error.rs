use serde::de::{Error as _, Unexpected};
use swc_ecma_ast::{
    BigInt, Expr, ExprOrSpread, JSXText, Lit, Number, Prop, PropName, Regex, SpreadElement,
};

/// Errors that can occur while deserializing a JavaScript AST expression.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The JavaScript source text could not be parsed.
    #[cfg(feature = "parser")]
    #[error("JavaScript parsing error")]
    EcmaParse(swc_ecma_parser::error::Error),
    /// An object property had a key that cannot be used as a Serde map key (e.g. a computed or
    /// numeric key).
    #[error("Invalid object key")]
    InvalidObjectKey(PropName),
    /// A number literal's value cannot be represented in the requested type.
    #[error("Invalid number")]
    InvalidNumber(Number),
    /// A literal value is invalid in context.
    #[error("Invalid literal")]
    InvalidLiteral(Lit),
    /// An object property has an unsupported form (e.g. shorthand or method).
    #[error("Invalid prop")]
    InvalidProp(Box<Prop>),
    /// An array element is a hole (`[,]`) or otherwise invalid.
    #[error("Invalid array element")]
    InvalidArrayElement(Option<ExprOrSpread>),
    /// A `BigInt` literal was encountered; these are not representable as Serde values.
    #[error("Unexpected big integer")]
    UnexpectedBigInt(BigInt),
    /// A JSX text node was encountered in a non-JSX context.
    #[error("Unexpected JSX text")]
    UnexpectedJsxText(JSXText),
    /// A regex literal was encountered; these are not representable as Serde values.
    #[error("Unexpected regex")]
    UnexpectedRegex(Regex),
    /// A spread element was encountered where a plain property was expected.
    #[error("Unexpected spread")]
    UnexpectedSpread(SpreadElement),
    /// A property with an unsupported form was encountered.
    #[error("Unexpected property")]
    UnexpectedProp(Box<Prop>),
    /// An expression type that cannot be mapped to a Serde value was encountered.
    #[error("Unexpected expression")]
    UnexpectedExpr(Expr),
    /// [`serde::de::MapAccess::next_value_seed`] was called before a key was consumed.
    #[error("Expected field value")]
    ExpectedFieldValue,
    /// A Serde-level error (e.g. unknown field, missing field, type mismatch).
    #[error("Serde error")]
    Serde(serde::de::value::Error),
}

impl Error {
    /// Construct an appropriate error for an unexpected literal value.
    ///
    /// Maps each [`Lit`] variant to the most informative serde [`Unexpected`] type,
    /// falling back to dedicated error variants for `BigInt`, `JSXText`, and `Regex`.
    pub(super) fn unexpected_lit(lit: &Lit, expected: &str) -> Self {
        match lit {
            Lit::Bool(bool) => Self::invalid_type(Unexpected::Bool(bool.value), &expected),
            Lit::BigInt(big_int) => Self::UnexpectedBigInt(big_int.clone()),
            Lit::JSXText(jsx_text) => Self::UnexpectedJsxText(jsx_text.clone()),
            Lit::Null(_) => Self::invalid_type(Unexpected::Option, &expected),
            Lit::Num(number) => super::number::number_to_unexpected(number).map_or_else(
                || Self::InvalidNumber(number.clone()),
                |unexpected| Self::invalid_type(unexpected, &expected),
            ),
            Lit::Regex(regex) => Self::UnexpectedRegex(regex.clone()),
            Lit::Str(str) => {
                Self::invalid_type(Unexpected::Str(&str.value.to_string_lossy()), &expected)
            }
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Serde(serde::de::value::Error::custom(msg))
    }

    fn duplicate_field(field: &'static str) -> Self {
        Self::Serde(serde::de::value::Error::duplicate_field(field))
    }

    fn invalid_length(len: usize, exp: &dyn serde::de::Expected) -> Self {
        Self::Serde(serde::de::value::Error::invalid_length(len, exp))
    }

    fn invalid_type(unexp: Unexpected<'_>, exp: &dyn serde::de::Expected) -> Self {
        Self::Serde(serde::de::value::Error::invalid_type(unexp, exp))
    }

    fn invalid_value(unexp: Unexpected<'_>, exp: &dyn serde::de::Expected) -> Self {
        Self::Serde(serde::de::value::Error::invalid_value(unexp, exp))
    }

    fn missing_field(field: &'static str) -> Self {
        Self::Serde(serde::de::value::Error::missing_field(field))
    }

    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        Self::Serde(serde::de::value::Error::unknown_field(field, expected))
    }

    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        Self::Serde(serde::de::value::Error::unknown_variant(variant, expected))
    }
}
