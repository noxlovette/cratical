/// Both macros below expect `v` to be everything from (but not including)
/// the property name up to end-of-line — i.e. `*(";" param) ":" value` per
/// RFC 5545's content-line grammar (`name *(";" param) ":" value`; the
/// property name itself is stripped by whatever dispatches to this type by
/// name, e.g. a matched `TokenType`). The split point is the first
/// *unquoted* `:` — never the first `;`, since a value can itself contain
/// unquoted `;` (`GEO`'s `lat;lon`, `RRULE`'s `Recur` grammar,
/// `REQUEST-STATUS`'s `statcode;statdesc[;extdata]`), which a naive
/// first-`;` split would misroute into the params half and truncate the
/// value. Params (if any) are everything before that colon, individually
/// `;`-split by [`param_segments`].
fn value_start(v: &[u8]) -> Result<usize, crate::ast::parser::ParseError> {
    crate::ast::find_unquoted(v, b':').ok_or(
        crate::ast::parser::ParseError::Parameter {
            expected: "':' introducing the property value".into(),
            received: std::str::from_utf8(v).ok().map(|s| s.into()),
        },
    )
}

macro_rules! impl_try_from_bytes {
    ($ty:ident) => {
        impl_try_from_bytes!($ty, Text);
    };
    ($ty:ident, $value_ty:ty) => {
        impl_try_from_bytes!($ty, $value_ty, SharedParams);
    };
    ($ty:ident, $value_ty:ty, $param_ty:ty) => {
        impl TryFrom<&[u8]> for $ty {
            type Error = crate::ast::parser::ParseError;

            fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
                let colon = crate::properties::value_start(v)?;
                let params = <$param_ty>::try_from(&v[..colon])?;
                let value = <$value_ty>::try_from(&v[colon + 1..])?;
                Ok(Self { value, params })
            }
        }
    };
    ($ty:ident, $value_ty:ty, $param_ty:ty, $validate:expr) => {
        impl TryFrom<&[u8]> for $ty {
            type Error = crate::ast::parser::ParseError;

            fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
                let colon = crate::properties::value_start(v)?;
                let params = <$param_ty>::try_from(&v[..colon])?;
                let value = <$value_ty>::try_from(&v[colon + 1..])?;
                let validate: fn(&$value_ty) -> Result<(), Self::Error> =
                    $validate;
                validate(&value)?;
                Ok(Self { value, params })
            }
        }
    };
}

/// Like [`impl_try_from_bytes!`], but for properties whose value is a
/// COMMA-separated list (`value: Vec<$elem_ty>`), e.g. `CATEGORIES` or
/// `EXDATE`. See [`value_start`] for the value/params split rule.
macro_rules! impl_try_from_bytes_list {
    ($ty:ident, $elem_ty:ty, $param_ty:ty) => {
        impl TryFrom<&[u8]> for $ty {
            type Error = crate::ast::parser::ParseError;

            fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
                let colon = crate::properties::value_start(v)?;
                let params = <$param_ty>::try_from(&v[..colon])?;
                let value = crate::ast::split_unescaped(&v[colon + 1..], b',')
                    .into_iter()
                    .map(<$elem_ty>::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Self { value, params })
            }
        }
    };
}

/// Section 3.7
mod calendar;
/// Section 3.8
mod component;
pub use calendar::*;
pub use component::*;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug)]
/// X Property
pub struct Xprop {
    value: Text,
    params: SharedParams,
}
impl_try_from_bytes!(Xprop);

#[derive(Debug)]
/// IANA Propery
pub struct Iana {
    value: Text,
    params: SharedParams,
}
impl_try_from_bytes!(Iana);

use crate::{
    ast::split_once,
    params::{Altrep, Language, ParamError},
    values::{Text, ValueError},
};

/// Splits raw parameter bytes (e.g. `;FOO=BAR;X-BAZ="a;b"`) into its
/// `;`-separated `NAME=VALUE` segments, honoring DQUOTE-enclosed values (a
/// `;` inside quotes doesn't end the segment) and dropping the empty piece
/// before the leading `;`.
fn param_segments(v: &[u8]) -> Vec<&[u8]> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    for (i, &b) in v.iter().enumerate() {
        match b {
            b'"' => in_quotes = !in_quotes,
            b';' if !in_quotes => {
                segments.push(&v[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    segments.push(&v[start..]);
    segments.retain(|s| !s.is_empty());
    segments
}

/// Builds the [`ParameterError::Malformed`] shared by [`param_name`] and
/// [`param_value`] when a `NAME=VALUE` parameter segment has no `=`.
fn missing_equals(segment: &[u8]) -> ParameterError {
    ParameterError::Malformed {
        expected: "NAME=VALUE".into(),
        received: std::str::from_utf8(segment).ok().map(Into::into),
    }
}

/// The `NAME` half of a `NAME=VALUE` parameter segment.
pub(crate) fn param_name(segment: &[u8]) -> Result<&[u8], ParameterError> {
    split_once(segment, b'=')
        .map(|(n, _)| n)
        .ok_or_else(|| missing_equals(segment))
}

/// The `VALUE` half of a `NAME=VALUE` parameter segment.
pub(crate) fn param_value(segment: &[u8]) -> Result<&[u8], ParameterError> {
    split_once(segment, b'=')
        .map(|(_, v)| v)
        .ok_or_else(|| missing_equals(segment))
}

/// This trait ensures that all parameters as used in properties have iana and
/// x-name params 100% of the time
pub trait Params<'a>: Default + Debug + TryFrom<&'a [u8]> {
    /// returns the iana properties of a param
    fn get_iana(&self) -> &[Text];
    /// returns the xname properties of a param
    fn get_xname(&self) -> &[Text];
}

/// The params that every property has
///
/// That is, the IANA and non-standard property parameters
#[derive(Default, Debug)]
struct SharedParams {
    iana: Vec<Text>,
    xname: Vec<Text>,
}

impl SharedParams {
    /// Records an unrecognized `NAME=VALUE` parameter segment into the
    /// `iana` or `xname` bucket, based on whether `NAME` has the `X-`
    /// prefix. Used both by [`SharedParams`]'s own `TryFrom` and by every
    /// composite params struct's fallback arm for params it doesn't model.
    fn absorb(&mut self, segment: &[u8]) -> Result<(), ParameterError> {
        let name = param_name(segment)?;
        let text: Text = segment.try_into()?;
        if name.to_ascii_uppercase().starts_with(b"X-") {
            self.xname.push(text);
        } else {
            self.iana.push(text);
        }
        Ok(())
    }
}

impl TryFrom<&[u8]> for SharedParams {
    type Error = ParameterError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(value) {
            params.absorb(segment)?;
        }
        Ok(params)
    }
}

impl<'a> Params<'a> for SharedParams {
    fn get_iana(&self) -> &[Text] {
        &self.iana
    }

    fn get_xname(&self) -> &[Text] {
        &self.xname
    }
}

/// Shared + Altrep + Language
///
/// These params are shared by multiple properties:
///
/// Summary, Resources, Description, Location, Contact, etc.
#[derive(Debug, Default)]
struct AltrepLanguageParams {
    shared: SharedParams,
    altrep: Option<Altrep>,
    language: Option<Language>,
}

impl TryFrom<&[u8]> for AltrepLanguageParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"ALTREP" => {
                    params.altrep = Some(param_value(segment)?.try_into()?)
                }
                b"LANGUAGE" => {
                    params.language = Some(param_value(segment)?.try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

#[derive(Debug, Error)]
pub enum PropertyError {
    #[error("invalid value for PRIORITY")]
    InvalidPriority,
    #[error("invalid value for GEO")]
    InvalidGeo,
    #[error("property is not valid at this position in the grammar")]
    UnexpectedProperty,
    #[error("{0} MUST NOT occur more than once in this component")]
    DuplicateProperty(&'static str),
}

/// The `*(";" param)` parameter list of a content line failed to parse.
///
/// Distinct from [`crate::params::ParamError`], which covers one individual
/// parameter (`ALTREP`, `LANGUAGE`, ...) failing at its own value-parsing
/// step — that error surfaces here as [`ParameterError::Param`]. This type
/// is for the surrounding list syntax itself: a `NAME=VALUE` segment with no
/// `=`, or a modeled parameter appearing where [`SharedParams::absorb`]
/// falls back to iana/x-name text and that fails to parse as `TEXT`.
#[derive(Debug, Error)]
pub enum ParameterError {
    /// A parameter segment didn't match the `NAME=VALUE` shape expected of
    /// it.
    #[error(
        "parameter list parsing failed. Expected {expected}, got {received:?}"
    )]
    Malformed {
        /// What the segment is supposed to be
        expected: String,
        /// What we actually received
        received: Option<String>,
    },

    /// One parameter's own value failed to parse. See
    /// [`crate::params::ParamError`].
    #[error(transparent)]
    Param(#[from] ParamError),

    /// An iana/x-name parameter's value failed to parse as `TEXT`. See
    /// [`crate::values::ValueError`].
    #[error(transparent)]
    Value(#[from] ValueError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_start_finds_first_unquoted_colon() {
        assert_eq!(value_start(b":VALUE").unwrap(), 0);
        assert_eq!(value_start(b";FOO=BAR:VALUE").unwrap(), 8);
    }

    #[test]
    fn value_start_ignores_colon_inside_quotes() {
        // ALTREP="cid:part1":the-real-value — the colon inside the quoted
        // ALTREP value must not be mistaken for the value/params boundary.
        let v = br#";ALTREP="cid:part1":the-real-value"#;
        let colon = value_start(v).unwrap();
        assert_eq!(&v[colon + 1..], b"the-real-value");
    }

    #[test]
    fn value_start_errors_without_a_colon() {
        assert!(value_start(b"no colon here").is_err());
    }

    #[test]
    fn shared_params_buckets_iana_and_xname() {
        let params =
            SharedParams::try_from(b";SOME-IANA=1;X-CUSTOM=2".as_slice())
                .unwrap();
        assert_eq!(params.iana.len(), 1);
        assert_eq!(params.xname.len(), 1);
    }

    #[test]
    fn shared_params_empty_is_fine() {
        let params = SharedParams::try_from(b"".as_slice()).unwrap();
        assert!(params.iana.is_empty());
        assert!(params.xname.is_empty());
    }

    #[test]
    fn altrep_language_params_parses_known_and_falls_back() {
        let params = AltrepLanguageParams::try_from(
            br#";ALTREP="cid:part1.0001@example.org";LANGUAGE=en;X-EXTRA=1"#
                .as_slice(),
        )
        .unwrap();
        assert!(params.altrep.is_some());
        assert!(params.language.is_some());
        assert_eq!(params.shared.xname.len(), 1);
    }
}
