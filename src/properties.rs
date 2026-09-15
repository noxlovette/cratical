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

/// Adds a `new` constructor for a client-facing property whose params are
/// exactly [`SharedParams`] — i.e. it has no RFC-defined parameters of its
/// own to set, only the value.
macro_rules! impl_simple_property {
    ($ty:ident, $value_ty:ty) => {
        impl $ty {
            /// Constructs a new property from its value, with no
            /// parameters set.
            pub fn new(value: $value_ty) -> Self {
                Self {
                    value,
                    params: crate::properties::SharedParams::default(),
                }
            }
        }
    };
}

/// Adds a `<$builder>` type for a client-facing property whose params are
/// exactly [`AltrepLanguageParams`] (`ALTREP` + `LANGUAGE`), e.g. `COMMENT`,
/// `DESCRIPTION`, `SUMMARY`.
macro_rules! impl_altrep_language_builder {
    ($builder:ident, $prop:ident, $value_ty:ty) => {
        /// Builder for the property this macro was invoked for.
        #[derive(Debug)]
        pub struct $builder {
            value: $value_ty,
            params: crate::properties::AltrepLanguageParams,
        }

        impl $builder {
            /// Starts a new builder from the property's required value.
            pub fn new(value: $value_ty) -> Self {
                Self {
                    value,
                    params: Default::default(),
                }
            }

            /// Sets the `ALTREP` parameter.
            pub fn altrep(mut self, altrep: crate::params::Altrep) -> Self {
                self.params.altrep = Some(altrep);
                self
            }

            /// Sets the `LANGUAGE` parameter.
            pub fn language(
                mut self,
                language: crate::params::Language,
            ) -> Self {
                self.params.language = Some(language);
                self
            }

            /// Finishes the builder, producing the property.
            pub fn build(self) -> $prop {
                $prop {
                    value: self.value,
                    params: self.params,
                }
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
    name: Text,
    value: Text,
    params: SharedParams,
}

impl Xprop {
    /// Parses a non-standard `X-` prefixed property, unlike every other
    /// property type: `name` isn't a compile-time-known literal here (it's
    /// arbitrary, caller-defined text), so it has to be captured alongside
    /// `value`/`params` rather than assumed.
    pub(crate) fn parse(
        name: &[u8],
        remainder: &[u8],
    ) -> Result<Self, crate::ast::parser::ParseError> {
        let colon = value_start(remainder)?;
        let params = SharedParams::try_from(&remainder[..colon])?;
        let value = Text::try_from(&remainder[colon + 1..])?;
        let name = Text::try_from(name)?;
        Ok(Self {
            name,
            value,
            params,
        })
    }
}

#[derive(Debug)]
/// IANA Propery
pub struct Iana {
    name: Text,
    value: Text,
    params: SharedParams,
}

impl Iana {
    /// Parses an IANA-registered property with no dispatch-table entry of
    /// its own (see [`Xprop::parse`] for why `name` is captured explicitly
    /// here rather than assumed).
    pub(crate) fn parse(
        name: &[u8],
        remainder: &[u8],
    ) -> Result<Self, crate::ast::parser::ParseError> {
        let colon = value_start(remainder)?;
        let params = SharedParams::try_from(&remainder[..colon])?;
        let value = Text::try_from(&remainder[colon + 1..])?;
        let name = Text::try_from(name)?;
        Ok(Self {
            name,
            value,
            params,
        })
    }
}

impl Xprop {
    /// Constructs a non-standard `X-` prefixed property from an explicit
    /// name and value, with no parameters set.
    pub fn build(name: Text, value: Text) -> Self {
        Self {
            name,
            value,
            params: SharedParams::default(),
        }
    }
}

impl Iana {
    /// Constructs an IANA-registered property with no dedicated type, from
    /// an explicit name and value, with no parameters set.
    pub fn build(name: Text, value: Text) -> Self {
        Self {
            name,
            value,
            params: SharedParams::default(),
        }
    }
}

impl std::fmt::Display for Xprop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}:{}", self.name.as_str(), self.params, self.value)
    }
}

impl std::fmt::Display for Iana {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}:{}", self.name.as_str(), self.params, self.value)
    }
}

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

/// The `VALUE` half of a `NAME=VALUE` parameter segment, with RFC 6868
/// caret-encoding ([`crate::ast::decode_caret`]) decoded.
pub(crate) fn param_value(segment: &[u8]) -> Result<Vec<u8>, ParameterError> {
    split_once(segment, b'=')
        .map(|(_, v)| crate::ast::decode_caret(v))
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
        let text: Text =
            crate::ast::decode_caret(segment).as_slice().try_into()?;
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

impl std::fmt::Display for SharedParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Each entry is already the full raw `NAME=VALUE` segment text (see
        // `absorb`), not a TEXT value, so it's written as-is rather than
        // through `Text`'s own (TEXT-escaping) `Display`.
        for t in self.iana.iter().chain(&self.xname) {
            write!(f, ";{}", t.as_str())?;
        }
        Ok(())
    }
}

/// Renders `items` as a COMMA-separated list of values, for a property
/// whose value type is itself a list (e.g. `CATEGORIES`, `EXDATE`).
pub(crate) fn fmt_comma_list<T: std::fmt::Display>(
    f: &mut std::fmt::Formatter<'_>,
    items: &[T],
) -> std::fmt::Result {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            f.write_str(",")?;
        }
        write!(f, "{item}")?;
    }
    Ok(())
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
                    params.altrep =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                b"LANGUAGE" => {
                    params.language =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

impl std::fmt::Display for AltrepLanguageParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(altrep) = &self.altrep {
            write!(f, ";ALTREP={altrep}")?;
        }
        if let Some(language) = &self.language {
            write!(f, ";LANGUAGE={language}")?;
        }
        write!(f, "{}", self.shared)
    }
}

/// An error building or validating a typed property.
#[derive(Debug, Error)]
pub enum PropertyError {
    /// `PRIORITY`'s value was outside its valid range.
    #[error("invalid value for PRIORITY")]
    InvalidPriority,
    /// `PERCENT-COMPLETE`'s value was outside its valid range.
    #[error("invalid value for PERCENT-COMPLETE")]
    InvalidPercentComplete,
    /// `GEO`'s value couldn't be parsed as a latitude/longitude pair.
    #[error("invalid value for GEO")]
    InvalidGeo,
    /// The property isn't legal wherever it was encountered in the grammar.
    #[error("property is not valid at this position in the grammar")]
    UnexpectedProperty,
    /// A singleton property occurred more than once in the same component.
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

    #[test]
    fn param_value_decodes_rfc_6868_caret_sequences() {
        // ATTENDEE;CN=George Herman ^'Babe^' Ruth
        assert_eq!(
            param_value(b"CN=George Herman ^'Babe^' Ruth").unwrap(),
            b"George Herman \"Babe\" Ruth"
        );
        // X-PARAM;ALL=^^^'^n
        assert_eq!(param_value(b"ALL=^^^'^n").unwrap(), b"^\"\n");
    }

    #[test]
    fn xprop_decodes_rfc_6868_caret_sequences_in_its_params() {
        // collective-icalendar/calendars/rfc_6868.ics
        let xprop = Xprop::parse(
            b"X-PARAM",
            b";NEWLINE=^n;ALL=^^^'^n;UNKNOWN=^a^ ^asd:asd".as_slice(),
        )
        .unwrap();
        assert_eq!(xprop.params.iana[0].as_str(), "NEWLINE=\n");
        assert_eq!(xprop.params.iana[1].as_str(), "ALL=^\"\n");
        // ^a and a lone trailing ^ aren't defined sequences, so both are
        // left untouched per RFC 6868 §3.2.
        assert_eq!(xprop.params.iana[2].as_str(), "UNKNOWN=^a^ ^asd");
    }

    #[test]
    fn xprop_display_round_trips_the_content_line() {
        let xprop =
            Xprop::parse(b"X-WR-CALNAME", b":My Calendar".as_slice()).unwrap();
        assert_eq!(xprop.to_string(), "X-WR-CALNAME:My Calendar");
    }

    #[test]
    fn shared_params_absorb_decodes_caret_sequences_in_unknown_params() {
        // X-PARAM;NEWLINE=^n;ALL=^^^'^n — NEWLINE/ALL aren't modeled by any
        // property's own params struct, so they fall to SharedParams's
        // iana/xname passthrough bucket, which must still decode them.
        let params =
            SharedParams::try_from(b";NEWLINE=^n;ALL=^^^'^n".as_slice())
                .unwrap();
        assert_eq!(params.iana[0].as_str(), "NEWLINE=\n");
        assert_eq!(params.iana[1].as_str(), "ALL=^\"\n");
    }

    #[test]
    fn image_falls_back_to_iana_across_uri_and_binary_value_forms() {
        // collective-icalendar/calendars/rfc_7986_image.ics — IMAGE (RFC
        // 7986 §5.10) has no PROPERTY_DISPATCH entry, so it must round-trip
        // through the Iana fallback rather than erroring or truncating,
        // across both the VALUE=URI and VALUE=BINARY (base64) forms.
        let uri = Iana::parse(
            b"IMAGE",
            b";VALUE=URI;DISPLAY=BADGE;FMTTYPE=image/png:http://example.com/images/party.png"
                .as_slice(),
        )
        .unwrap();
        assert_eq!(uri.params.iana[0].as_str(), "VALUE=URI");
        assert_eq!(uri.params.iana[1].as_str(), "DISPLAY=BADGE");
        assert_eq!(uri.params.iana[2].as_str(), "FMTTYPE=image/png");
        assert_eq!(uri.value.as_str(), "http://example.com/images/party.png");

        let binary = Iana::parse(
            b"IMAGE",
            b";ENCODING=BASE64;VALUE=BINARY;FMTTYPE=image/png:iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACXBIWXMAAAAnAAAAJwEqCZFPAAAAGXRFWHRTb2Z0d2FyZQB3d3cuaW5rc2NhcGUub3Jnm+48GgAAAA1JREFUCJlj+P//PwMACPwC/oXNqzQAAAAASUVORK5CYII="
                .as_slice(),
        )
        .unwrap();
        assert_eq!(binary.params.iana[0].as_str(), "ENCODING=BASE64");
        assert_eq!(binary.params.iana[1].as_str(), "VALUE=BINARY");
        assert_eq!(binary.params.iana[2].as_str(), "FMTTYPE=image/png");
        assert!(binary.value.as_str().starts_with("iVBORw0KGgo"));
    }

    #[test]
    fn image_falls_back_to_iana_across_uri_binary_text_and_unknown_forms() {
        // collective-icalendar/calendars/issue_1561_image_value.ics
        let uri = Iana::parse(
            b"IMAGE",
            b";VALUE=URI:https://example.com/a.png".as_slice(),
        )
        .unwrap();
        assert_eq!(uri.params.iana[0].as_str(), "VALUE=URI");
        assert_eq!(uri.value.as_str(), "https://example.com/a.png");

        let binary = Iana::parse(
            b"IMAGE",
            b";ENCODING=BASE64;VALUE=BINARY:AP+A".as_slice(),
        )
        .unwrap();
        assert_eq!(binary.params.iana[0].as_str(), "ENCODING=BASE64");
        assert_eq!(binary.params.iana[1].as_str(), "VALUE=BINARY");
        assert_eq!(binary.value.as_str(), "AP+A");

        // VALUE=TEXT round-trips through Text's own BACKSLASH-escape
        // decoding (RFC 5545 §3.3.11) — the escaped `;`/`,` must be decoded,
        // not mistaken for real param/value-list delimiters.
        let text =
            Iana::parse(b"IMAGE", br";VALUE=TEXT:a\;b\,c".as_slice()).unwrap();
        assert_eq!(text.params.iana[0].as_str(), "VALUE=TEXT");
        assert_eq!(text.value.as_str(), "a;b,c");

        // No recognized VALUE param at all — still falls back cleanly.
        let unknown =
            Iana::parse(b"IMAGE", b":https://example.com/b.png".as_slice())
                .unwrap();
        assert!(unknown.params.iana.is_empty());
        assert_eq!(unknown.value.as_str(), "https://example.com/b.png");
    }

    #[test]
    fn iana_display_round_trips_the_content_line() {
        let iana = Iana::parse(
            b"IMAGE",
            b";VALUE=URI:http://example.com/images/party.png".as_slice(),
        )
        .unwrap();
        assert_eq!(
            iana.to_string(),
            "IMAGE;VALUE=URI:http://example.com/images/party.png"
        );
    }

    #[test]
    fn conference_falls_back_to_iana_with_folded_multi_value_feature_param() {
        // collective-icalendar/calendars/rfc_7986_conferences.ics, unfolded
        // per RFC 5545 §3.1 (the CRLF/LF + single leading SPACE that splits
        // each of these across two physical lines is removed, with no space
        // inserted, before Property::parse ever sees it — see
        // `ast::lexer::unfold`, exercised end-to-end for a different
        // fixture in
        // `link_falls_back_to_iana_and_unfolds_across_rfc_9253_examples`
        // below). CONFERENCE (RFC 7986 §5.11) has no PROPERTY_DISPATCH
        // entry either.
        let moderator = Iana::parse(
            b"CONFERENCE",
            b";VALUE=URI;FEATURE=PHONE,MODERATOR;LABEL=Moderator dial-in:tel:+1-412-555-0123,,,654321"
                .as_slice(),
        )
        .unwrap();
        assert_eq!(moderator.params.iana[0].as_str(), "VALUE=URI");
        // FEATURE's comma-separated multi-value list is preserved verbatim
        // as one passthrough param text, not comma-split into typed values
        // (only list-*valued properties* get that treatment, not params).
        assert_eq!(
            moderator.params.iana[1].as_str(),
            "FEATURE=PHONE,MODERATOR"
        );
        assert_eq!(
            moderator.params.iana[2].as_str(),
            "LABEL=Moderator dial-in"
        );
        assert_eq!(moderator.value.as_str(), "tel:+1-412-555-0123,,,654321");

        let video = Iana::parse(
            b"CONFERENCE",
            b";VALUE=URI;FEATURE=AUDIO,VIDEO;LABEL=Attendee dial-in:https://chat.example.com/audio?id=123456"
                .as_slice(),
        )
        .unwrap();
        assert_eq!(video.params.iana[1].as_str(), "FEATURE=AUDIO,VIDEO");
        assert_eq!(
            video.value.as_str(),
            "https://chat.example.com/audio?id=123456"
        );
    }

    #[test]
    fn link_falls_back_to_iana_and_unfolds_across_rfc_9253_examples() {
        // collective-icalendar/calendars/rfc_9253_examples.ics — LINK (RFC
        // 9253 §8) has no PROPERTY_DISPATCH entry either, and unlike the
        // IMAGE/CONFERENCE tests above, this one goes through the real
        // `Calendar::parse` pipeline (real lexer, real unfolding) rather
        // than hand-assembled bytes, since this fixture's LINK values are
        // folded across up to four physical lines — worth confirming the
        // fallback survives real multi-fold reconstruction, not just a
        // hand-unfolded stand-in.
        let calendar = crate::Calendar::parse(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/collective-icalendar/calendars/rfc_9253_examples.\
             ics"
        )))
        .unwrap();
        let components = calendar.components();
        assert_eq!(components.len(), 2);

        let crate::Component::Event(links) = &components[0] else {
            panic!("expected first component to be a VEVENT");
        };
        assert_eq!(links.uid.as_str(), "links-rfc-9253-section-8.2");
        assert_eq!(links.iana.len(), 3);

        assert_eq!(links.iana[0].params.iana[0].as_str(), "LINKREL=SOURCE");
        assert_eq!(links.iana[0].params.iana[1].as_str(), "LABEL=Venue");
        assert_eq!(links.iana[0].params.iana[2].as_str(), "VALUE=URI");
        assert_eq!(links.iana[0].value.as_str(), "https://example.com/events");
        // The real parser (unlike the hand-assembled Iana::parse calls in
        // the sibling tests above) is what proves `name` survives end to
        // end: `Property::parse` captured "LINK" itself, not a value this
        // test supplied.
        assert_eq!(
            links.iana[0].to_string(),
            "LINK;LINKREL=SOURCE;LABEL=Venue;VALUE=URI:https://example.com/events"
        );

        assert_eq!(
            links.iana[1].params.iana[0].as_str(),
            r#"LINKREL="https://example.com/linkrel/derivedFrom""#
        );
        assert_eq!(links.iana[1].params.iana[1].as_str(), "VALUE=URI");
        assert_eq!(
            links.iana[1].value.as_str(),
            "https://example.com/tasks/01234567-abcd1234.ics"
        );

        // Spans 4 physical (folded) lines end to end — confirms multi-fold
        // reconstruction, not just a single fold.
        assert_eq!(
            links.iana[2].params.iana[0].as_str(),
            r#"LINKREL="https://example.com/linkrel/costStructure""#
        );
        assert_eq!(
            links.iana[2].params.iana[1].as_str(),
            "VALUE=XML-REFERENCE"
        );
        assert_eq!(
            links.iana[2].value.as_str(),
            "https://example.com/xmlDocs/bidFramework.xml#xpointer(descendant::CostStruc/range-to(following::CostStrucEND[1]))"
        );

        let crate::Component::Event(reference) = &components[1] else {
            panic!("expected second component to be a VEVENT");
        };
        assert_eq!(reference.uid.as_str(), "links-rfc-9253-uid");
        assert_eq!(reference.iana.len(), 1);
        assert_eq!(
            reference.iana[0].params.iana[0].as_str(),
            "LINKREL=REFERENCE"
        );
        assert_eq!(reference.iana[0].params.iana[1].as_str(), "VALUE=UID");
        assert_eq!(
            reference.iana[0].value.as_str(),
            "links-rfc-9253-section-8.2"
        );
    }
}
