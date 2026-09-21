//! vCard property parameters (RFC 6350 §5).
//!
//! Per the crate's rule enforcement, vCard properties hold only parameters
//! from here. The three ways to spell a multi-valued parameter, `TYPE=a,b`,
//! `TYPE="a,b"` and `TYPE=a;TYPE=b`, all parse to the same [`Parameters`].
//! RFC 6868 caret-encoding is decoded in every value, after the quotes and
//! commas have been read, so an encoded DQUOTE can't be mistaken for one.

use super::values::{
    LanguageTag, MediaType, ParamValue, Uri, UtcOffset, ValueError,
};
use crate::{ast::split_once, properties::param_segments};
use std::{fmt, str::Utf8Error};
use thiserror::Error;

/// A parameter that doesn't follow the grammar of RFC 6350 §5.
#[derive(Debug, Error)]
pub enum ParamError {
    /// The segment isn't `NAME=VALUE`, or its value isn't the shape the
    /// parameter needs.
    #[error(
        "Malformed {param} parameter. Expected {expected}, got {received:?}"
    )]
    Malformed {
        /// The parameter it's about.
        param: String,
        /// What was expected.
        expected: &'static str,
        /// What was found.
        received: String,
    },

    /// A parameter that can be given once was given again.
    #[error("The {0} parameter is given more than once")]
    Duplicate(&'static str),

    /// The value of a parameter failed to parse as its value type.
    #[error(transparent)]
    Value(#[from] ValueError),

    /// Text that isn't UTF-8.
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
}

fn malformed(
    param: impl Into<String>,
    expected: &'static str,
    received: &[u8],
) -> ParamError {
    ParamError::Malformed {
        param: param.into(),
        expected,
        received: String::from_utf8_lossy(received).into_owned(),
    }
}

/// `iana-token = 1*(ALPHA / DIGIT / "-")`, which `x-name` is a subset of.
fn is_token(v: &[u8]) -> bool {
    !v.is_empty() && v.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'-')
}

/// One comma-separated element of a parameter value.
struct Element<'a> {
    /// What's between the quotes, if it had any, otherwise all of it.
    inner: &'a [u8],
    quoted: bool,
}

/// Splits the `param-value *("," param-value)` of a parameter on its
/// unquoted commas. A DQUOTE may only delimit a whole element.
fn elements<'a>(
    param: &str,
    v: &'a [u8],
) -> Result<Vec<Element<'a>>, ParamError> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    let mut pieces = Vec::new();
    for (i, &b) in v.iter().enumerate() {
        match b {
            b'"' => in_quotes = !in_quotes,
            b',' if !in_quotes => {
                pieces.push(&v[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    pieces.push(&v[start..]);
    for piece in pieces {
        let element = match crate::ast::strip_quoted_string(piece) {
            Some(inner) => Element {
                inner,
                quoted: true,
            },
            None => Element {
                inner: piece,
                quoted: false,
            },
        };
        if element.inner.contains(&b'"') {
            return Err(malformed(
                param,
                "param-value without a stray DQUOTE",
                v,
            ));
        }
        out.push(element);
    }
    Ok(out)
}

/// The one element of a parameter that takes just one.
fn single<'a>(
    param: &str,
    elements: &[Element<'a>],
    whole: &[u8],
) -> Result<&'a [u8], ParamError> {
    match elements {
        [one] => Ok(one.inner),
        _ => Err(malformed(param, "a single value", whole)),
    }
}

/// The elements of a list parameter as values. A quoted element is a list
/// itself, since `TYPE="work,voice"` means `TYPE=work,voice` (and so does
/// `SORT-AS="Harten,Rene"`).
fn flattened(elements: &[Element]) -> Result<Vec<ParamValue>, ParamError> {
    let mut out = Vec::new();
    for element in elements {
        if element.quoted {
            for part in element.inner.split(|b| *b == b',') {
                out.push(ParamValue::decode(part)?);
            }
        } else {
            out.push(ParamValue::decode(element.inner)?);
        }
    }
    Ok(out)
}

/// Writes `values` comma-separated, quoting the ones that need it.
fn write_values(
    f: &mut fmt::Formatter<'_>,
    values: &[ParamValue],
) -> fmt::Result {
    for (i, value) in values.iter().enumerate() {
        if i > 0 {
            f.write_str(",")?;
        }
        write!(f, "{value}")?;
    }
    Ok(())
}

fn set_once<T>(
    slot: &mut Option<T>,
    name: &'static str,
    value: T,
) -> Result<(), ParamError> {
    if slot.replace(value).is_some() {
        return Err(ParamError::Duplicate(name));
    }
    Ok(())
}

/// The value type of a property's value, as carried by the `VALUE`
/// parameter.
///
/// The VALUE parameter is OPTIONAL, used to identify the value type (data
/// type) and format of the value. The use of these predefined formats is
/// encouraged even if the value parameter is not explicitly used. By
/// defining a standard set of value types and their formats, existing
/// parsing and processing code can be leveraged. The predefined data type
/// values MUST NOT be repeated in COMMA-separated value lists except within
/// the N, NICKNAME, ADR, and CATEGORIES properties.
///
/// ```text
/// value-type = "text" / "uri" / "date" / "time" / "date-time"
///            / "date-and-or-time" / "timestamp" / "boolean" / "integer"
///            / "float" / "utc-offset" / "language-tag"
///            / iana-token / x-name
/// ```
///
/// A token that isn't one of the predefined types (e.g. vCard 3.0's
/// `VALUE=binary`) is kept as [`ValueDataType::Other`].
///
/// Example:
///
/// > BDAY;VALUE=text:circa 1800
///
/// [Section 5.2](https://datatracker.ietf.org/doc/html/rfc6350#section-5.2)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueDataType {
    /// `text`.
    Text,
    /// `uri`.
    Uri,
    /// `date`.
    Date,
    /// `time`.
    Time,
    /// `date-time`.
    DateTime,
    /// `date-and-or-time`.
    DateAndOrTime,
    /// `timestamp`.
    Timestamp,
    /// `boolean`.
    Boolean,
    /// `integer`.
    Integer,
    /// `float`.
    Float,
    /// `utc-offset`.
    UtcOffset,
    /// `language-tag`.
    LanguageTag,
    /// An `iana-token` or `x-name` this crate has no type for, as written.
    Other(String),
}

impl ValueDataType {
    fn token(&self) -> &str {
        match self {
            Self::Text => "text",
            Self::Uri => "uri",
            Self::Date => "date",
            Self::Time => "time",
            Self::DateTime => "date-time",
            Self::DateAndOrTime => "date-and-or-time",
            Self::Timestamp => "timestamp",
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::Float => "float",
            Self::UtcOffset => "utc-offset",
            Self::LanguageTag => "language-tag",
            Self::Other(token) => token,
        }
    }
}

impl TryFrom<&[u8]> for ValueDataType {
    type Error = ParamError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        if !is_token(v) {
            return Err(malformed("VALUE", "a value type", v));
        }
        Ok(match v.to_ascii_lowercase().as_slice() {
            b"text" => Self::Text,
            b"uri" => Self::Uri,
            b"date" => Self::Date,
            b"time" => Self::Time,
            b"date-time" => Self::DateTime,
            b"date-and-or-time" => Self::DateAndOrTime,
            b"timestamp" => Self::Timestamp,
            b"boolean" => Self::Boolean,
            b"integer" => Self::Integer,
            b"float" => Self::Float,
            b"utc-offset" => Self::UtcOffset,
            b"language-tag" => Self::LanguageTag,
            _ => Self::Other(std::str::from_utf8(v)?.to_owned()),
        })
    }
}

impl fmt::Display for ValueDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

/// The level of preference of a property instance.
///
/// The PREF parameter is OPTIONAL and is used to indicate that the
/// corresponding instance of a property is preferred by the vCard author.
/// Its value MUST be an integer between 1 and 100 that quantifies the level
/// of preference. Lower values correspond to a higher level of preference,
/// with 1 being most preferred.
///
/// When the parameter is absent, the default MUST be to interpret the
/// property instance as being least preferred.
///
/// Note that the value of this parameter is to be interpreted only in
/// relation to values assigned to other instances of the same property in
/// the same vCard. A given value, or the absence of a value, MUST NOT be
/// interpreted on its own.
///
/// This parameter MAY be applied to any property that allows multiple
/// instances.
///
/// ```text
/// pref-param = "PREF=" (1*2DIGIT / "100")
///                      ; An integer between 1 and 100.
/// ```
///
/// Example:
///
/// > TEL;PREF=1;TYPE="work,voice":+1-555-555-5555
///
/// [Section 5.3](https://datatracker.ietf.org/doc/html/rfc6350#section-5.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pref(u8);

impl Pref {
    /// Builds a preference, refusing anything outside 1 to 100.
    pub fn new(value: u8) -> Result<Self, ParamError> {
        if (1..=100).contains(&value) {
            Ok(Self(value))
        } else {
            Err(malformed(
                "PREF",
                "an integer from 1 to 100",
                value.to_string().as_bytes(),
            ))
        }
    }

    /// The preference, 1 (most preferred) to 100.
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl TryFrom<&[u8]> for Pref {
    type Error = ParamError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let err = || malformed("PREF", "an integer from 1 to 100", v);
        if v.is_empty() || v.len() > 3 || !v.iter().all(u8::is_ascii_digit) {
            return Err(err());
        }
        let value = std::str::from_utf8(v)?.parse().map_err(|_| err())?;
        Self::new(value).map_err(|_| err())
    }
}

impl fmt::Display for Pref {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// The identifier of one property instance, `PID`'s
/// `1*DIGIT ["." 1*DIGIT]`: a small integer, then optionally the source it
/// was assigned by, which a `CLIENTPIDMAP` maps to a URI.
///
/// The PID parameter is used to identify a specific property among multiple
/// instances. It plays a role analogous to the UID property on a
/// per-property instead of per-vCard basis. It MAY appear more than once in
/// a given property. It MUST NOT appear on properties that may have only one
/// instance per vCard. Its value is either a single small positive integer
/// or a pair of small positive integers separated by a dot. Multiple values
/// may be encoded in a single PID parameter by separating the values with a
/// comma ",".
///
/// Example:
///
/// > EMAIL;PID=1.1,2.1:jdoe@example.com
///
/// [Section 5.5](https://datatracker.ietf.org/doc/html/rfc6350#section-5.5)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pid {
    first: u32,
    second: Option<u32>,
}

impl Pid {
    /// Builds an identifier.
    pub fn new(first: u32, second: Option<u32>) -> Self {
        Self { first, second }
    }

    /// The integer before the dot: the instance within its source.
    pub fn first(&self) -> u32 {
        self.first
    }

    /// The integer after the dot, if any: the `CLIENTPIDMAP` source.
    pub fn second(&self) -> Option<u32> {
        self.second
    }
}

impl TryFrom<&[u8]> for Pid {
    type Error = ParamError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let err = || malformed("PID", "1*DIGIT [\".\" 1*DIGIT]", v);
        let number = |d: &[u8]| -> Result<u32, ParamError> {
            if d.is_empty() || !d.iter().all(u8::is_ascii_digit) {
                return Err(err());
            }
            std::str::from_utf8(d)?.parse().map_err(|_| err())
        };
        match split_once(v, b'.') {
            Some((first, second)) => {
                Ok(Self::new(number(first)?, Some(number(second)?)))
            }
            None => Ok(Self::new(number(v)?, None)),
        }
    }
}

impl fmt::Display for Pid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.first.fmt(f)?;
        if let Some(second) = self.second {
            write!(f, ".{second}")?;
        }
        Ok(())
    }
}

/// A `TYPE` for a `TEL`.
///
/// ```text
/// type-param-tel = "text" / "voice" / "fax" / "cell" / "video"
///                / "pager" / "textphone" / iana-token / x-name
///   ; type-param-tel MUST NOT be used with a property other than TEL.
/// ```
///
/// Example:
///
/// > TEL;VALUE=uri;TYPE="voice,home":tel:+1-555-555-5555;ext=5555
///
/// [Section 6.4.1](https://datatracker.ietf.org/doc/html/rfc6350#section-6.4.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelType {
    /// `text`: the telephone number supports text messages (SMS).
    Text,
    /// `voice`: a voice telephone number.
    Voice,
    /// `fax`: a facsimile telephone number.
    Fax,
    /// `cell`: a cellular telephone number.
    Cell,
    /// `video`: a video conferencing telephone number.
    Video,
    /// `pager`: a paging device telephone number.
    Pager,
    /// `textphone`: a telecommunication device for people with hearing or
    /// speech difficulties.
    Textphone,
}

/// A `TYPE` for a `RELATED`.
///
/// ```text
/// related-type-value = "contact" / "acquaintance" / "friend" / "met"
///                    / "co-worker" / "colleague" / "co-resident"
///                    / "neighbor" / "child" / "parent"
///                    / "sibling" / "spouse" / "kin" / "muse"
///                    / "crush" / "date" / "sweetheart" / "me"
///                    / "agent" / "emergency"
/// ```
///
/// Example:
///
/// > RELATED;TYPE=friend:urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6
///
/// [Section 6.6.6](https://datatracker.ietf.org/doc/html/rfc6350#section-6.6.6)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelatedType {
    /// `contact`.
    Contact,
    /// `acquaintance`.
    Acquaintance,
    /// `friend`.
    Friend,
    /// `met`.
    Met,
    /// `co-worker`.
    CoWorker,
    /// `colleague`.
    Colleague,
    /// `co-resident`.
    CoResident,
    /// `neighbor`.
    Neighbor,
    /// `child`.
    Child,
    /// `parent`.
    Parent,
    /// `sibling`.
    Sibling,
    /// `spouse`.
    Spouse,
    /// `kin`.
    Kin,
    /// `muse`.
    Muse,
    /// `crush`.
    Crush,
    /// `date`.
    Date,
    /// `sweetheart`.
    Sweetheart,
    /// `me`.
    Me,
    /// `agent`.
    Agent,
    /// `emergency`.
    Emergency,
}

macro_rules! token_enum {
    ($ty:ident { $($variant:ident => $token:literal),+ $(,)? }) => {
        impl $ty {
            fn token(self) -> &'static str {
                match self { $(Self::$variant => $token),+ }
            }

            fn from_token(lowercase: &str) -> Option<Self> {
                match lowercase { $($token => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}

token_enum!(TelType {
    Text => "text",
    Voice => "voice",
    Fax => "fax",
    Cell => "cell",
    Video => "video",
    Pager => "pager",
    Textphone => "textphone",
});

token_enum!(RelatedType {
    Contact => "contact",
    Acquaintance => "acquaintance",
    Friend => "friend",
    Met => "met",
    CoWorker => "co-worker",
    Colleague => "colleague",
    CoResident => "co-resident",
    Neighbor => "neighbor",
    Child => "child",
    Parent => "parent",
    Sibling => "sibling",
    Spouse => "spouse",
    Kin => "kin",
    Muse => "muse",
    Crush => "crush",
    Date => "date",
    Sweetheart => "sweetheart",
    Me => "me",
    Agent => "agent",
    Emergency => "emergency",
});

/// One value of the `TYPE` parameter.
///
/// The TYPE parameter has multiple, different uses. In general, it is a way
/// of specifying class characteristics of the associated property. Most of
/// the time, its value is a comma-separated subset of a predefined
/// enumeration.
///
/// The "work" and "home" values act like tags. The "work" value implies that
/// the property is related to an individual's work place, while the "home"
/// value implies that the property is related to an individual's personal
/// life. When neither "work" nor "home" is present, it is implied that the
/// property is related to both an individual's work place and personal life
/// in the case that the KIND property's value is "individual", or to none in
/// other cases.
///
/// ```text
/// type-param = "TYPE=" type-value *("," type-value)
/// type-value = "work" / "home" / type-param-tel
///            / type-param-related / iana-token / x-name
///   ; This is further defined in individual property sections.
/// ```
///
/// Which values a property may carry is up to the property; this only says
/// what a value is. One this crate doesn't know isn't rejected, since real
/// data has `TYPE=CELL`, `TYPE=pref`, `TYPE=INTERNET` and the like: it is
/// kept as written in [`TypeValue::Other`].
///
/// Example:
///
/// > TEL;TYPE=work,voice:+1-555-555-5555
///
/// [Section 5.6](https://datatracker.ietf.org/doc/html/rfc6350#section-5.6)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeValue {
    /// `work`.
    Work,
    /// `home`.
    Home,
    /// A value that only `TEL` takes.
    Tel(TelType),
    /// A value that only `RELATED` takes.
    Related(RelatedType),
    /// Anything else, as written.
    Other(ParamValue),
}

impl From<ParamValue> for TypeValue {
    fn from(value: ParamValue) -> Self {
        let lowercase = value.as_str().to_ascii_lowercase();
        match lowercase.as_str() {
            "work" => Self::Work,
            "home" => Self::Home,
            other => TelType::from_token(other)
                .map(Self::Tel)
                .or_else(|| RelatedType::from_token(other).map(Self::Related))
                .unwrap_or(Self::Other(value)),
        }
    }
}

impl fmt::Display for TypeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Work => f.write_str("work"),
            Self::Home => f.write_str("home"),
            Self::Tel(t) => f.write_str(t.token()),
            Self::Related(r) => f.write_str(r.token()),
            Self::Other(other) => other.fmt(f),
        }
    }
}

/// The calendar system in which a date or date-time value is expressed.
///
/// The CALSCALE parameter is identical to the CALSCALE property in iCalendar
/// (see RFC 5545, Section 3.7.1). It is used to define the calendar system
/// in which a date or date-time value is expressed. The only value specified
/// by iCalendar is "gregorian", which stands for the Gregorian system. It is
/// the default when the parameter is absent. Additional values may be
/// defined in extension documents and registered with IANA (see Section
/// 10.3.4). A vCard implementation MUST ignore properties with a CALSCALE
/// parameter value that it does not understand.
///
/// ```text
/// calscale-value = "gregorian" / iana-token / x-name
/// ```
///
/// Example:
///
/// > BDAY;CALSCALE=gregorian:19960415
///
/// [Section 5.8](https://datatracker.ietf.org/doc/html/rfc6350#section-5.8)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalScale {
    /// `gregorian`.
    Gregorian,
    /// Any other `iana-token` or `x-name`, as written.
    Other(String),
}

impl TryFrom<&[u8]> for CalScale {
    type Error = ParamError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        if !is_token(v) {
            return Err(malformed("CALSCALE", "a calendar scale", v));
        }
        if v.eq_ignore_ascii_case(b"gregorian") {
            Ok(Self::Gregorian)
        } else {
            Ok(Self::Other(std::str::from_utf8(v)?.to_owned()))
        }
    }
}

impl fmt::Display for CalScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gregorian => f.write_str("gregorian"),
            Self::Other(other) => f.write_str(other),
        }
    }
}

/// The strings to sort by.
///
/// The "sort-as" parameter is used to specify the string to be used for
/// national-language-specific sorting. Without this information, sorting
/// algorithms could incorrectly sort this vCard within a sequence of sorted
/// vCards. When this property is present in a vCard, then the given strings
/// are used for sorting the vCard.
///
/// This parameter's value is a comma-separated list that MUST have as many
/// or fewer elements as the corresponding property value has components.
/// This parameter's value is case-sensitive.
///
/// ```text
/// sort-as-param = "SORT-AS=" sort-as-value
/// sort-as-value = param-value *("," param-value)
/// ```
///
/// Example:
///
/// > N;SORT-AS="Harten,Rene":van der Harten;Rene,J.;Sir;R.D.O.N.
///
/// [Section 5.9](https://datatracker.ietf.org/doc/html/rfc6350#section-5.9)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortAs(Vec<ParamValue>);

impl SortAs {
    /// Builds the parameter from its strings.
    pub fn new(values: Vec<ParamValue>) -> Self {
        Self(values)
    }

    /// The strings, one per component of the property, in order.
    pub fn values(&self) -> &[ParamValue] {
        &self.0
    }
}

impl fmt::Display for SortAs {
    /// Written as the examples of §5.9 are: one quoted string holding the
    /// comma-separated list.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let joined: Vec<&str> = self.0.iter().map(ParamValue::as_str).collect();
        write!(f, "{}", ParamValue::new(joined.join(",")))
    }
}

/// Global positioning information that is specific to an address.
///
/// The GEO parameter can be used to indicate global positioning information
/// that is specific to an address. Its value is the same as that of the GEO
/// property.
///
/// ```text
/// geo-parameter = "GEO=" DQUOTE URI DQUOTE
/// ```
///
/// Example:
///
/// > ADR;GEO="geo:12.3457,78.910":;;123 Main Street;Any
/// > Town;CA;91921-1234;U.S.A.
///
/// [Section 5.10](https://datatracker.ietf.org/doc/html/rfc6350#section-5.10)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Geo(Uri);

impl Geo {
    /// Builds the parameter from its URI.
    pub fn new(uri: Uri) -> Self {
        Self(uri)
    }

    /// The `geo:` URI.
    pub fn uri(&self) -> &Uri {
        &self.0
    }
}

impl fmt::Display for Geo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

/// Time zone information that is specific to an address.
///
/// The TZ parameter can be used to indicate time zone information that is
/// specific to an address. Its value is the same as that of the TZ property.
///
/// ```text
/// tz-parameter = "TZ=" (param-value / DQUOTE URI DQUOTE)
/// ```
///
/// With no `VALUE` to say which, the form tells: a `utc-offset` if it is
/// one, else a URI if it's quoted and is one, else text.
///
/// Example:
///
/// > ADR;TZ=America/New_York:;;123 Main Street;Any Town;CA;91921-1234;U.S.A.
///
/// [Section 5.11](https://datatracker.ietf.org/doc/html/rfc6350#section-5.11)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tz {
    /// A time zone name.
    Text(ParamValue),
    /// A URI, always written quoted.
    Uri(Uri),
    /// An offset from UTC.
    UtcOffset(UtcOffset),
}

impl fmt::Display for Tz {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(text) => text.fmt(f),
            Self::Uri(uri) => write!(f, "\"{uri}\""),
            Self::UtcOffset(offset) => offset.fmt(f),
        }
    }
}

/// An `iana-token` or `x-name` parameter this crate has no type for.
///
/// Applications MUST ignore x-param and iana-param values they don't
/// recognize.
///
/// ```text
/// any-param = (iana-token / x-name) "=" param-value *("," param-value)
/// ```
///
/// Its name is kept as written.
///
/// Example:
///
/// > X-GENERATED-BY=hand
///
/// [Section 5](https://datatracker.ietf.org/doc/html/rfc6350#section-5)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extension {
    name: String,
    values: Vec<ParamValue>,
}

impl Extension {
    /// The name, as written.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The values, in order.
    pub fn values(&self) -> &[ParamValue] {
        &self.values
    }

    /// Whether the name starts with "x-", which makes it experimental
    /// instead of registered with IANA.
    pub fn is_experimental(&self) -> bool {
        self.name
            .get(..2)
            .is_some_and(|p| p.eq_ignore_ascii_case("x-"))
    }
}

/// The `*(";" param)` parameter list of a vCard content line.
///
/// A property can have attributes associated with it. These "property
/// parameters" contain meta-information about the property or the property
/// value. In some cases, the property parameter can be multi-valued in which
/// case the property parameter value elements are separated by a COMMA
/// (U+002C).
///
/// Property parameter value elements that contain the COLON (U+003A),
/// SEMICOLON (U+003B), or COMMA (U+002C) character separators MUST be
/// specified as quoted-string text values. Property parameter values MUST
/// NOT contain the DQUOTE (U+0022) character. The DQUOTE character is used
/// as a delimiter for parameter values that contain restricted characters or
/// URI text.
///
/// Applications MUST ignore x-param and iana-param values they don't
/// recognize.
///
/// A parameter's name is case-insensitive, and so are the unquoted values
/// that come from a fixed set. A parameter that takes one value (all but
/// `PID`, `TYPE`, `SORT-AS` and the extensions) is an error if given twice.
/// `PID` and `TYPE` given twice are merged, in order, which is how vCard 3.0
/// spells a list. Written back, the parameters come out in a fixed order,
/// each list on one parameter.
///
/// Example:
///
/// > TEL;TYPE="work,voice";PREF=1:+1-555-555-5555
///
/// [Section 5](https://datatracker.ietf.org/doc/html/rfc6350#section-5)
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Parameters {
    language: Option<LanguageTag>,
    value: Option<ValueDataType>,
    pref: Option<Pref>,
    altid: Option<ParamValue>,
    pid: Vec<Pid>,
    types: Vec<TypeValue>,
    mediatype: Option<MediaType>,
    calscale: Option<CalScale>,
    sort_as: Option<SortAs>,
    geo: Option<Geo>,
    tz: Option<Tz>,
    label: Option<ParamValue>,
    extensions: Vec<Extension>,
}

impl Parameters {
    /// Whether the property has no parameters.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// The `LANGUAGE` of the value.
    pub fn language(&self) -> Option<&LanguageTag> {
        self.language.as_ref()
    }

    /// The `VALUE` type of the value.
    pub fn value(&self) -> Option<&ValueDataType> {
        self.value.as_ref()
    }

    /// The `PREF`erence of this instance among those of its property.
    pub fn pref(&self) -> Option<Pref> {
        self.pref
    }

    /// The `ALTID` tying this instance to its alternative representations.
    pub fn altid(&self) -> Option<&ParamValue> {
        self.altid.as_ref()
    }

    /// The `PID`s of this instance, from every `PID` parameter.
    pub fn pid(&self) -> &[Pid] {
        &self.pid
    }

    /// The `TYPE` values, from every `TYPE` parameter.
    pub fn types(&self) -> &[TypeValue] {
        &self.types
    }

    /// The `MEDIATYPE` of what the URI value points to.
    pub fn mediatype(&self) -> Option<&MediaType> {
        self.mediatype.as_ref()
    }

    /// The `CALSCALE` of the date or date-time value.
    pub fn calscale(&self) -> Option<&CalScale> {
        self.calscale.as_ref()
    }

    /// The `SORT-AS` strings.
    pub fn sort_as(&self) -> Option<&SortAs> {
        self.sort_as.as_ref()
    }

    /// The `GEO` position of an address.
    pub fn geo(&self) -> Option<&Geo> {
        self.geo.as_ref()
    }

    /// The `TZ` of an address.
    pub fn tz(&self) -> Option<&Tz> {
        self.tz.as_ref()
    }

    /// The `LABEL` of an address.
    pub fn label(&self) -> Option<&ParamValue> {
        self.label.as_ref()
    }

    /// The parameters this crate has no type for, in the order they came.
    pub fn extensions(&self) -> &[Extension] {
        &self.extensions
    }
}

impl TryFrom<&[u8]> for Parameters {
    type Error = ParamError;

    /// Parses `*(";" param)`, the text between a property's name and its
    /// `:`, leading `;` included.
    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            let (name, raw) = split_once(segment, b'=')
                .ok_or_else(|| malformed("(none)", "NAME=VALUE", segment))?;
            if !is_token(name) {
                return Err(malformed(
                    String::from_utf8_lossy(name),
                    "a parameter name of letters, digits and '-'",
                    name,
                ));
            }
            let display_name = String::from_utf8_lossy(name).into_owned();
            let elements = elements(&display_name, raw)?;
            match name.to_ascii_uppercase().as_slice() {
                b"LANGUAGE" => set_once(
                    &mut params.language,
                    "LANGUAGE",
                    LanguageTag::try_from(single("LANGUAGE", &elements, raw)?)?,
                )?,
                b"VALUE" => set_once(
                    &mut params.value,
                    "VALUE",
                    ValueDataType::try_from(single("VALUE", &elements, raw)?)?,
                )?,
                b"PREF" => set_once(
                    &mut params.pref,
                    "PREF",
                    Pref::try_from(single("PREF", &elements, raw)?)?,
                )?,
                b"ALTID" => set_once(
                    &mut params.altid,
                    "ALTID",
                    ParamValue::decode(single("ALTID", &elements, raw)?)?,
                )?,
                b"PID" => {
                    for element in &elements {
                        params.pid.push(Pid::try_from(element.inner)?);
                    }
                }
                b"TYPE" => params.types.extend(
                    flattened(&elements)?.into_iter().map(TypeValue::from),
                ),
                b"MEDIATYPE" => set_once(
                    &mut params.mediatype,
                    "MEDIATYPE",
                    MediaType::try_from(single("MEDIATYPE", &elements, raw)?)
                        .map_err(|e| ParamError::Value(e.into()))?,
                )?,
                b"CALSCALE" => set_once(
                    &mut params.calscale,
                    "CALSCALE",
                    CalScale::try_from(single("CALSCALE", &elements, raw)?)?,
                )?,
                b"SORT-AS" => set_once(
                    &mut params.sort_as,
                    "SORT-AS",
                    SortAs(flattened(&elements)?),
                )?,
                b"GEO" => {
                    if !matches!(elements.as_slice(), [e] if e.quoted) {
                        return Err(malformed(
                            "GEO",
                            "a URI in a quoted-string",
                            raw,
                        ));
                    }
                    set_once(
                        &mut params.geo,
                        "GEO",
                        Geo(Uri::try_from(elements[0].inner)
                            .map_err(|e| ParamError::Value(e.into()))?),
                    )?;
                }
                b"TZ" => {
                    let [element] = elements.as_slice() else {
                        return Err(malformed("TZ", "a single value", raw));
                    };
                    let tz = if let Ok(offset) =
                        UtcOffset::try_from(element.inner)
                    {
                        Tz::UtcOffset(offset)
                    } else if let Some(uri) = element
                        .quoted
                        .then(|| Uri::try_from(element.inner).ok())
                        .flatten()
                    {
                        Tz::Uri(uri)
                    } else {
                        Tz::Text(ParamValue::decode(element.inner)?)
                    };
                    set_once(&mut params.tz, "TZ", tz)?;
                }
                b"LABEL" => set_once(
                    &mut params.label,
                    "LABEL",
                    ParamValue::decode(single("LABEL", &elements, raw)?)?,
                )?,
                _ => params.extensions.push(Extension {
                    name: display_name,
                    values: elements
                        .iter()
                        .map(|e| ParamValue::decode(e.inner))
                        .collect::<Result<_, _>>()?,
                }),
            }
        }
        Ok(params)
    }
}

impl fmt::Display for Parameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(language) = &self.language {
            write!(f, ";LANGUAGE={language}")?;
        }
        if let Some(value) = &self.value {
            write!(f, ";VALUE={value}")?;
        }
        if let Some(pref) = self.pref {
            write!(f, ";PREF={pref}")?;
        }
        if let Some(altid) = &self.altid {
            write!(f, ";ALTID={altid}")?;
        }
        for (i, pid) in self.pid.iter().enumerate() {
            f.write_str(if i == 0 { ";PID=" } else { "," })?;
            pid.fmt(f)?;
        }
        for (i, kind) in self.types.iter().enumerate() {
            f.write_str(if i == 0 { ";TYPE=" } else { "," })?;
            kind.fmt(f)?;
        }
        if let Some(mediatype) = &self.mediatype {
            write!(f, ";MEDIATYPE={}", ParamValue::new(mediatype.to_string()))?;
        }
        if let Some(calscale) = &self.calscale {
            write!(f, ";CALSCALE={calscale}")?;
        }
        if let Some(sort_as) = &self.sort_as {
            write!(f, ";SORT-AS={sort_as}")?;
        }
        if let Some(geo) = &self.geo {
            write!(f, ";GEO={geo}")?;
        }
        if let Some(tz) = &self.tz {
            write!(f, ";TZ={tz}")?;
        }
        if let Some(label) = &self.label {
            write!(f, ";LABEL={label}")?;
        }
        for extension in &self.extensions {
            write!(f, ";{}=", extension.name)?;
            write_values(f, &extension.values)?;
        }
        Ok(())
    }
}
