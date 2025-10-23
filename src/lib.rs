#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(clippy::missing_errors_doc)]
#![forbid(unsafe_code)]
use serde::de::{
    EnumAccess, Error as _, IntoDeserializer, MapAccess, SeqAccess, Unexpected, VariantAccess,
    Visitor,
};
use std::borrow::Cow;
use swc_ecma_ast::{ArrayLit, Expr, ExprOrSpread, Lit, ObjectLit, Prop, PropName, PropOrSpread};

pub mod error;
mod number;

use error::Error;

#[cfg(feature = "parser")]
pub fn from_str<'a: 'de, 'de, T: serde::Deserialize<'de>>(expr_str: &'a str) -> Result<T, Error> {
    from_str_with_version(expr_str, swc_ecma_ast::EsVersion::default())
}

#[cfg(feature = "parser")]
pub fn from_str_with_version<'a: 'de, 'de, T: serde::Deserialize<'de>>(
    expr_str: &'a str,
    version: swc_ecma_ast::EsVersion,
) -> Result<T, Error> {
    let lexer = swc_ecma_parser::Lexer::new(
        swc_ecma_parser::Syntax::Es(swc_ecma_parser::EsSyntax::default()),
        version,
        swc_ecma_parser::StringInput::new(
            expr_str,
            swc_common::BytePos(0),
            swc_common::BytePos(u32::try_from(expr_str.len()).unwrap_or(u32::MAX)),
        ),
        None,
    );

    let mut parser = swc_ecma_parser::Parser::new_from(lexer);
    let expr = parser.parse_expr().map_err(Error::EcmaParse)?;

    T::deserialize(Deserializer {
        expr: std::borrow::Cow::Owned(*expr),
    })
}

pub fn from_expr<'a: 'de, 'de, T: serde::Deserialize<'de>>(expr: &'a Expr) -> Result<T, Error> {
    T::deserialize(Deserializer {
        expr: std::borrow::Cow::Borrowed(expr),
    })
}

pub struct Deserializer<'de> {
    expr: std::borrow::Cow<'de, Expr>,
}

impl<'de> serde::de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    fn is_human_readable(&self) -> bool {
        true
    }

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match &*self.expr {
            Expr::Array(_) => self.deserialize_seq(visitor),
            Expr::Object(_) => self.deserialize_map(visitor),
            Expr::Lit(lit) => match lit {
                Lit::Bool(bool) => visitor.visit_bool(bool.value),
                Lit::Num(number) => {
                    if number::is_integer(number) {
                        self.deserialize_i64(visitor)
                    } else {
                        self.deserialize_f64(visitor)
                    }
                }
                Lit::Null(_) => visitor.visit_none(),
                Lit::Str(_) => self.deserialize_str(visitor),
                _ => Err(Self::Error::UnexpectedExpr(self.expr.into_owned())),
            },
            Expr::Ident(_) => self.deserialize_str(visitor),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "boolean";

        match &*self.expr {
            Expr::Lit(Lit::Bool(value)) => visitor.visit_bool(value.value),
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "bytes";

        match self.expr {
            Cow::Borrowed(Expr::Lit(Lit::Str(str))) => {
                visitor.visit_borrowed_bytes(str.value.as_bytes())
            }
            Cow::Owned(Expr::Lit(Lit::Str(str))) => visitor.visit_bytes(str.value.as_bytes()),
            other => match &*other {
                Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
                Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
                Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
                other => Err(Self::Error::UnexpectedExpr(other.clone())),
            },
        }
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "character";

        match &*self.expr {
            Expr::Lit(Lit::Str(str)) => {
                let mut chars = str.value.chars();

                chars.next().map_or_else(
                    || {
                        Err(Self::Error::invalid_value(
                            Unexpected::Str(str.value.as_str()),
                            &expected,
                        ))
                    },
                    |ch| {
                        if chars.next().is_none() {
                            visitor.visit_char(ch)
                        } else {
                            Err(Self::Error::invalid_value(
                                Unexpected::Str(str.value.as_str()),
                                &expected,
                            ))
                        }
                    },
                )
            }
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let expected = "enumeration";

        match self.expr {
            Cow::Borrowed(Expr::Lit(Lit::Str(str))) => {
                visitor.visit_enum(str.value.as_str().into_deserializer())
            }
            Cow::Owned(Expr::Lit(Lit::Str(str))) => {
                visitor.visit_enum(str.value.as_str().into_deserializer())
            }
            Cow::Borrowed(Expr::Object(ObjectLit { props, .. })) => {
                if props.len() == 1 {
                    match &props[0] {
                        PropOrSpread::Prop(prop) => match &**prop {
                            Prop::KeyValue(kvp) => {
                                let key = prop_name_to_str(&kvp.key).ok_or_else(|| {
                                    Self::Error::UnexpectedProp(Box::new(*prop.clone()))
                                })?;

                                visitor.visit_enum(Enum {
                                    key: Cow::Borrowed(key),
                                    value: Cow::Borrowed(&kvp.value),
                                })
                            }
                            other => Err(Self::Error::UnexpectedProp(Box::new(other.clone()))),
                        },
                        PropOrSpread::Spread(spread) => {
                            Err(Self::Error::UnexpectedSpread(spread.clone()))
                        }
                    }
                } else {
                    Err(Self::Error::invalid_length(props.len(), &"1"))
                }
            }
            Cow::Owned(Expr::Object(ObjectLit { mut props, .. })) => {
                if props.len() == 1 {
                    match props.pop() {
                        Some(PropOrSpread::Prop(prop)) => match *prop {
                            Prop::KeyValue(kvp) => {
                                let key = prop_name_to_str(&kvp.key).ok_or_else(|| {
                                    Self::Error::InvalidObjectKey(kvp.key.clone())
                                })?;

                                visitor.visit_enum(Enum {
                                    key: Cow::Owned(key.to_string()),
                                    value: Cow::Owned(*kvp.value),
                                })
                            }
                            other => Err(Self::Error::UnexpectedProp(Box::new(other))),
                        },
                        Some(PropOrSpread::Spread(spread)) => {
                            Err(Self::Error::UnexpectedSpread(spread))
                        }
                        None => {
                            // We already checked the length, but here for completeness.
                            Err(Self::Error::invalid_length(props.len(), &"1"))
                        }
                    }
                } else {
                    Err(Self::Error::invalid_length(props.len(), &"1"))
                }
            }
            other => match &*other {
                Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
                Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
                Expr::Ident(ident) => visitor.visit_enum(ident.sym.as_str().into_deserializer()),
                other => Err(Self::Error::UnexpectedExpr(other.clone())),
            },
        }
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "f32";

        match &*self.expr {
            Expr::Lit(Lit::Num(number)) =>
            {
                #[allow(clippy::cast_possible_truncation)]
                visitor.visit_f32(number.value as f32)
            }
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "f64";

        match &*self.expr {
            Expr::Lit(Lit::Num(number)) => visitor.visit_f64(number.value),
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "i128";

        match &*self.expr {
            Expr::Lit(lit @ Lit::Num(number)) => number::number_to_i128(number)
                .ok_or_else(|| Error::unexpected_lit(lit, expected))
                .and_then(|value| visitor.visit_i128(value)),
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "i16";

        match &*self.expr {
            Expr::Lit(lit @ Lit::Num(number)) => number::number_to_i16(number)
                .ok_or_else(|| Error::unexpected_lit(lit, expected))
                .and_then(|value| visitor.visit_i16(value)),
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "i32";

        match &*self.expr {
            Expr::Lit(lit @ Lit::Num(number)) => number::number_to_i32(number)
                .ok_or_else(|| Error::unexpected_lit(lit, expected))
                .and_then(|value| visitor.visit_i32(value)),
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "i64";

        match &*self.expr {
            Expr::Lit(lit @ Lit::Num(number)) => number::number_to_i64(number)
                .ok_or_else(|| Error::unexpected_lit(lit, expected))
                .and_then(|value| visitor.visit_i64(value)),
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "i8";

        match &*self.expr {
            Expr::Lit(lit @ Lit::Num(number)) => number::number_to_i8(number)
                .ok_or_else(|| Error::unexpected_lit(lit, expected))
                .and_then(|value| visitor.visit_i8(value)),
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_any(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.expr {
            Cow::Borrowed(Expr::Object(ObjectLit { props, .. })) => {
                visitor.visit_map(Map::new(Cow::Borrowed(props)))
            }
            Cow::Owned(Expr::Object(ObjectLit { props, .. })) => {
                visitor.visit_map(Map::new(Cow::Owned(props)))
            }
            other => Err(Self::Error::UnexpectedExpr(other.into_owned())),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.expr {
            Cow::Borrowed(Expr::Array(ArrayLit { elems, .. })) => {
                visitor.visit_seq(Seq::new(Cow::Borrowed(elems)))
            }
            Cow::Owned(Expr::Array(ArrayLit { elems, .. })) => {
                visitor.visit_seq(Seq::new(Cow::Owned(elems)))
            }
            other => Err(Self::Error::UnexpectedExpr(other.into_owned())),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "string";

        match self.expr {
            Cow::Borrowed(Expr::Lit(Lit::Str(str))) => {
                visitor.visit_borrowed_str(str.value.as_str())
            }
            Cow::Owned(Expr::Lit(Lit::Str(str))) => visitor.visit_str(str.value.as_str()),
            Cow::Borrowed(Expr::Ident(ident)) => visitor.visit_borrowed_str(ident.sym.as_str()),
            Cow::Owned(Expr::Ident(ident)) => visitor.visit_str(ident.sym.as_str()),
            other => match &*other {
                Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
                Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
                Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
                other => Err(Self::Error::UnexpectedExpr(other.clone())),
            },
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_map(visitor)
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "u128";

        match &*self.expr {
            Expr::Lit(lit @ Lit::Num(number)) => number::number_to_u128(number)
                .ok_or_else(|| Error::unexpected_lit(lit, expected))
                .and_then(|value| visitor.visit_u128(value)),
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "u16";

        match &*self.expr {
            Expr::Lit(lit @ Lit::Num(number)) => number::number_to_u16(number)
                .ok_or_else(|| Error::unexpected_lit(lit, expected))
                .and_then(|value| visitor.visit_u16(value)),
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "u32";

        match &*self.expr {
            Expr::Lit(lit @ Lit::Num(number)) => number::number_to_u32(number)
                .ok_or_else(|| Error::unexpected_lit(lit, expected))
                .and_then(|value| visitor.visit_u32(value)),
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "u64";

        match &*self.expr {
            Expr::Lit(lit @ Lit::Num(number)) => number::number_to_u64(number)
                .ok_or_else(|| Error::unexpected_lit(lit, expected))
                .and_then(|value| visitor.visit_u64(value)),
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "u8";

        match &*self.expr {
            Expr::Lit(lit @ Lit::Num(number)) => number::number_to_u8(number)
                .ok_or_else(|| Error::unexpected_lit(lit, expected))
                .and_then(|value| visitor.visit_u8(value)),
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let expected = "null";

        match &*self.expr {
            Expr::Lit(Lit::Null(_)) => visitor.visit_unit(),
            Expr::Lit(lit) => Err(Error::unexpected_lit(lit, expected)),
            Expr::Object(_) => Err(Self::Error::invalid_type(Unexpected::Map, &expected)),
            Expr::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &expected)),
            other => Err(Self::Error::UnexpectedExpr(other.clone())),
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match &*self.expr {
            Expr::Lit(Lit::Null(_)) => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }
}

fn prop_name_to_str(prop_name: &PropName) -> Option<&str> {
    prop_name
        .as_str()
        .map(|str| str.value.as_str())
        .or_else(|| prop_name.as_ident().map(|ident| ident.sym.as_str()))
}

struct Seq<'de> {
    values: Cow<'de, [Option<ExprOrSpread>]>,
}

impl<'de> Seq<'de> {
    fn new(values: Cow<'de, [Option<ExprOrSpread>]>) -> Self {
        Self {
            values: match values {
                Cow::Borrowed(values) => Cow::Borrowed(values),
                Cow::Owned(mut values) => {
                    values.reverse();

                    Cow::Owned(values)
                }
            },
        }
    }
}

impl<'de> SeqAccess<'de> for Seq<'de> {
    type Error = Error;

    fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        match &mut self.values {
            Cow::Borrowed(values) => {
                if values.is_empty() {
                    Ok(None)
                } else {
                    let value = &values[0];

                    let expr_or_spread = value
                        .as_ref()
                        .ok_or_else(|| Error::InvalidArrayElement(value.clone()))?;

                    self.values = Cow::Borrowed(&values[1..]);

                    seed.deserialize(Deserializer {
                        expr: Cow::Borrowed(&expr_or_spread.expr),
                    })
                    .map(Some)
                }
            }
            Cow::Owned(values) => values
                .pop()
                .map(|value| {
                    let expr_or_spread = value.ok_or_else(|| Error::InvalidArrayElement(None))?;

                    seed.deserialize(Deserializer {
                        expr: Cow::Owned(*expr_or_spread.expr),
                    })
                })
                .map_or(Ok(None), |value| value.map(Some)),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.values.len())
    }
}

struct Map<'de> {
    fields: Cow<'de, [PropOrSpread]>,
    value: Option<Cow<'de, Expr>>,
}

impl<'de> Map<'de> {
    fn new(fields: Cow<'de, [PropOrSpread]>) -> Self {
        Self {
            fields: match fields {
                Cow::Borrowed(fields) => Cow::Borrowed(fields),
                Cow::Owned(mut fields) => {
                    fields.reverse();

                    Cow::Owned(fields)
                }
            },
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for Map<'de> {
    type Error = Error;

    fn next_key_seed<K: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match &mut self.fields {
            Cow::Borrowed(fields) => {
                if fields.is_empty() {
                    Ok(None)
                } else {
                    let prop_or_spread = &fields[0];

                    match prop_or_spread {
                        PropOrSpread::Prop(prop) => match &**prop {
                            Prop::KeyValue(kvp) => {
                                self.value = Some(Cow::Borrowed(&kvp.value));

                                let key_str = prop_name_to_str(&kvp.key).ok_or_else(|| {
                                    Error::UnexpectedProp(Box::new(*prop.clone()))
                                })?;

                                self.fields = Cow::Borrowed(&fields[1..]);

                                seed.deserialize(key_str.into_deserializer()).map(Some)
                            }
                            other => Err(Error::UnexpectedProp(Box::new(other.clone()))),
                        },
                        PropOrSpread::Spread(spread) => {
                            Err(Error::UnexpectedSpread(spread.clone()))
                        }
                    }
                }
            }
            Cow::Owned(fields) => fields
                .pop()
                .map(|prop_or_spread| match prop_or_spread {
                    PropOrSpread::Prop(prop) => match *prop {
                        Prop::KeyValue(kvp) => {
                            self.value = Some(Cow::Owned(*kvp.value));

                            let key_str = prop_name_to_str(&kvp.key)
                                .ok_or_else(|| Error::InvalidObjectKey(kvp.key.clone()))?;

                            seed.deserialize(key_str.into_deserializer())
                        }
                        other => Err(Error::UnexpectedProp(Box::new(other))),
                    },
                    PropOrSpread::Spread(spread) => Err(Error::UnexpectedSpread(spread)),
                })
                .map_or(Ok(None), |value| value.map(Some)),
        }
    }

    fn next_value_seed<V: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        self.value.take().map_or_else(
            || Err(Error::ExpectedFieldValue),
            |value| seed.deserialize(Deserializer { expr: value }),
        )
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.fields.len())
    }
}

struct Enum<'de> {
    key: Cow<'de, str>,
    value: Cow<'de, Expr>,
}

impl<'de> EnumAccess<'de> for Enum<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V: serde::de::DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let value = seed.deserialize(self.key.clone().into_deserializer())?;

        Ok((value, self))
    }
}

impl<'de> VariantAccess<'de> for Enum<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(Self::Error::UnexpectedExpr(self.value.into_owned()))
    }

    fn newtype_variant_seed<T: serde::de::DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<T::Value, Self::Error> {
        seed.deserialize(Deserializer { expr: self.value })
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        serde::de::Deserializer::deserialize_seq(Deserializer { expr: self.value }, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        serde::de::Deserializer::deserialize_map(Deserializer { expr: self.value }, visitor)
    }
}

#[cfg(test)]
mod test {
    use swc_common::BytePos;
    use swc_ecma_ast::{EsVersion, Expr};
    use swc_ecma_parser::{Parser, StringInput, Syntax, lexer::Lexer};

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Deserialization error")]
        EcmaDe(#[from] super::error::Error),
        #[error("Invalid example")]
        InvalidExample(String),
    }

    pub fn parse_js(script: &str, version: EsVersion) -> Result<Box<Expr>, Error> {
        let lexer = Lexer::new(
            Syntax::Es(Default::default()),
            version,
            StringInput::new(script, BytePos(0), BytePos(script.as_bytes().len() as u32)),
            None,
        );

        let mut parser = Parser::new_from(lexer);

        parser
            .parse_expr()
            .map_err(super::Error::EcmaParse)
            .map_err(Error::from)
    }

    #[test]
    fn google_play_chess_to_json() -> Result<(), Error> {
        let example_path = "../examples/google-play-chess.js";
        let script = parse_js(
            include_str!("../examples/google-play-chess.js"),
            Default::default(),
        )?;

        match &*script {
            Expr::Call(swc_ecma_ast::CallExpr { args, .. }) if args.len() == 1 => {
                let object_lit_expr = &args[0].expr;

                let json = super::from_expr::<serde_json::Value>(object_lit_expr)?;

                assert_eq!(json.as_object().map(|object| object.len()), Some(4));

                Ok(())
            }
            _ => Err(Error::InvalidExample(example_path.to_string())),
        }
    }

    #[derive(Debug, Eq, PartialEq, serde::Deserialize)]
    enum TestEnum {
        Orange,
        Apple {},
        Pear { name: String },
    }

    #[derive(Debug, Eq, PartialEq, serde::Deserialize)]
    struct TestStruct<'a> {
        foo: Option<u64>,
        bar: Vec<bool>,
        qux: std::borrow::Cow<'a, str>,
        fruit: Vec<TestEnum>,
    }

    const SCRIPT_STR: &str = r#"{ foo: 123, "bar": [true, false], qux: "hey", fruit: [Orange, { Apple: {} }, { "Pear": { name: "+?*" } } ] }"#;
    const JSON_STR: &str = r#"{ "foo": 123, "bar": [true, false], "qux": "hey", "fruit": ["Orange", { "Apple": {} }, { "Pear": { "name": "+?*" } }  ] }"#;

    #[test]
    fn test_struct() -> Result<(), Error> {
        let expected_test_value = TestStruct {
            foo: Some(123),
            bar: vec![true, false],
            qux: "hey".into(),
            fruit: vec![
                TestEnum::Orange,
                TestEnum::Apple {},
                TestEnum::Pear {
                    name: "+?*".to_string(),
                },
            ],
        };

        let expected_json_value = serde_json::from_str::<serde_json::Value>(JSON_STR).unwrap();

        let script_js = parse_js(SCRIPT_STR, Default::default())?;

        let test_value = super::from_expr::<TestStruct<'_>>(&script_js).unwrap();

        assert_eq!(test_value, expected_test_value);

        let json_value = super::from_expr::<serde_json::Value>(&script_js).unwrap();

        assert_eq!(json_value, expected_json_value);

        Ok(())
    }

    #[test]
    fn test_struct_owned() -> Result<(), Error> {
        let expected_test_value = TestStruct {
            foo: Some(123),
            bar: vec![true, false],
            qux: "hey".into(),
            fruit: vec![
                TestEnum::Orange,
                TestEnum::Apple {},
                TestEnum::Pear {
                    name: "+?*".to_string(),
                },
            ],
        };

        let expected_json_value = serde_json::from_str::<serde_json::Value>(JSON_STR).unwrap();

        let test_value = super::from_str::<TestStruct<'_>>(SCRIPT_STR)?;

        assert_eq!(test_value, expected_test_value);

        let json_value = super::from_str::<serde_json::Value>(SCRIPT_STR).unwrap();

        assert_eq!(json_value, expected_json_value);

        Ok(())
    }
}
