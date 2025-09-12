use serde::de::{Error as _, Unexpected};
use swc_ecma_ast::{
    BigInt, Expr, ExprOrSpread, JSXText, Lit, Number, Prop, PropName, Regex, SpreadElement,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "parser")]
    #[error("JavaScript parsing error")]
    EcmaParse(swc_ecma_parser::error::Error),
    #[error("Invalid object key")]
    InvalidObjectKey(PropName),
    #[error("Invalid number")]
    InvalidNumber(Number),
    #[error("Invalid literal")]
    InvalidLiteral(Lit),
    #[error("Invalid prop")]
    InvalidProp(Box<Prop>),
    #[error("Invalid array element")]
    InvalidArrayElement(Option<ExprOrSpread>),
    #[error("Unexpected big integer")]
    UnexpectedBigInt(BigInt),
    #[error("Unexpected JSX text")]
    UnexpectedJsxText(JSXText),
    #[error("Unexpected regex")]
    UnexpectedRegex(Regex),
    #[error("Unexpected spread")]
    UnexpectedSpread(SpreadElement),
    #[error("Unexpected property")]
    UnexpectedProp(Box<Prop>),
    #[error("Unexpected expression")]
    UnexpectedExpr(Expr),
    #[error("Expected field value")]
    ExpectedFieldValue,
    #[error("Serde error")]
    Serde(serde::de::value::Error),
}

impl Error {
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
            Lit::Str(str) => Self::invalid_type(Unexpected::Str(str.value.as_str()), &expected),
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

    fn invalid_type(unexp: Unexpected, exp: &dyn serde::de::Expected) -> Self {
        Self::Serde(serde::de::value::Error::invalid_type(unexp, exp))
    }

    fn invalid_value(unexp: Unexpected, exp: &dyn serde::de::Expected) -> Self {
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
