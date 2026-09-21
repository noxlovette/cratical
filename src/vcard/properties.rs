//! vCard properties (RFC 6350 §6).
//!
//! One type per property, each holding its group, its value from
//! [`values`] and its parameters from
//! [`params`](super::params), plus the `X-`/IANA passthrough ([`Xprop`],
//! [`Iana`]) for what has no type of its own.
//!
//! Only what is local to one property is checked here: that its value is
//! its value type, that its `VALUE` parameter names a type the property can
//! have, and the few parameters the RFC forbids outright. How often a
//! property may occur (`*1` against `*`), which ones a card must have, and
//! what properties require of each other (`MEMBER` needs `KIND:group`) is
//! for the card, once it has them all.

// The wire examples in the docs are URIs, as the RFC's are.
#![allow(rustdoc::bare_urls)]

use super::{
    ParseError,
    params::{Parameters, TypeValue},
    value_start, values,
};
use std::fmt;

/// The group a property belongs to.
///
/// The group construct is used to group related properties together. The
/// group name is a syntactic convention used to indicate that all property
/// names prefaced with the same group name SHOULD be grouped together when
/// displayed by an application. It has no other significance.
/// Implementations that do not understand or support grouping MAY simply
/// strip off any text before a "." to the left of the type name and present
/// the types and values as normal.
///
/// The group is case-insensitive; the case it was written in is kept.
///
/// Example:
///
/// > item1.TEL:+1-555-555-5555
///
/// [Section 3.3](https://datatracker.ietf.org/doc/html/rfc6350#section-3.3)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group(String);

impl Group {
    /// The group name as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&[u8]> for Group {
    type Error = ParseError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        if v.is_empty()
            || !v.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'-')
        {
            return Err(ParseError::Group(
                String::from_utf8_lossy(v).into_owned(),
            ));
        }
        Ok(Self(std::str::from_utf8(v)?.to_owned()))
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The version of the vCard specification used to format this vCard.
///
/// This property MUST be present in the vCard object, and it must appear
/// immediately after BEGIN:VCARD. The value MUST be "4.0" if the vCard
/// corresponds to this specification. Note that earlier versions of vCard
/// allowed this property to be placed anywhere in the vCard object, or even
/// to be absent.
///
/// Example:
///
/// > VERSION:4.0
///
/// [Section 6.7.9](https://datatracker.ietf.org/doc/html/rfc6350#section-6.7.9)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    group: Option<Group>,
    value: values::Version,
    params: Parameters,
}

impl Version {
    /// The group the property was written with, if any.
    pub fn group(&self) -> Option<&Group> {
        self.group.as_ref()
    }

    /// The version.
    pub fn value(&self) -> values::Version {
        self.value
    }

    /// The parameters the property was written with.
    pub fn params(&self) -> &Parameters {
        &self.params
    }

    fn parse(
        group: Option<Group>,
        remainder: &[u8],
    ) -> Result<Self, ParseError> {
        let colon = value_start(remainder)?;
        Ok(Self {
            group,
            params: Parameters::try_from(&remainder[..colon])?,
            value: values::Version::try_from(&remainder[colon + 1..])?,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(group) = &self.group {
            write!(f, "{group}.")?;
        }
        write!(f, "VERSION{}:{}", self.params, self.value)
    }
}

macro_rules! passthrough_property {
    ($(#[$doc:meta])* $ty:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $ty {
            group: Option<Group>,
            name: values::Raw,
            value: values::Raw,
            params: Parameters,
        }

        impl $ty {
            /// The group the property was written with, if any.
            pub fn group(&self) -> Option<&Group> {
                self.group.as_ref()
            }

            /// The property name, upper-cased.
            pub fn name(&self) -> &str {
                self.name.as_str()
            }

            /// The value, exactly as written.
            pub fn value(&self) -> &values::Raw {
                &self.value
            }

            /// The parameters the property was written with.
            pub fn params(&self) -> &Parameters {
                &self.params
            }

            fn parse(
                group: Option<Group>,
                name: &[u8],
                remainder: &[u8],
            ) -> Result<Self, ParseError> {
                let colon = value_start(remainder)?;
                Ok(Self {
                    group,
                    name: values::Raw::try_from(name)?,
                    params: Parameters::try_from(&remainder[..colon])?,
                    value: values::Raw::try_from(&remainder[colon + 1..])?,
                })
            }
        }

        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if let Some(group) = &self.group {
                    write!(f, "{group}.")?;
                }
                write!(f, "{}{}:{}", self.name, self.params, self.value)
            }
        }
    };
}

passthrough_property! {
    /// A non-standard, private property whose name starts with "X-".
    ///
    /// Non-standard, private properties and parameters with a name starting
    /// with "X-" may be defined bilaterally between two cooperating agents
    /// without outside registration or standardization.
    ///
    /// Example:
    ///
    /// > item1.X-ABLabel:Anniversary
    ///
    /// [Section 6.10](https://datatracker.ietf.org/doc/html/rfc6350#section-6.10)
    Xprop
}

passthrough_property! {
    /// A property with no typed representation in this crate, e.g. one
    /// registered with IANA after RFC 6350.
    ///
    /// Never an error: extension properties are open-ended by design, so
    /// an unrecognized name is kept, not rejected.
    ///
    /// Example:
    ///
    /// > EXAMPLE-IANA-PROPERTY:value
    ///
    /// [Section 6.10](https://datatracker.ietf.org/doc/html/rfc6350#section-6.10)
    Iana
}

/// Which value type the property's value is written as: the one its `VALUE`
/// parameter names, or `default` when it has none. `allowed` is what the
/// property's ABNF lets `VALUE` be.
fn value_type(
    name: &'static str,
    params: &Parameters,
    allowed: &[&'static str],
    default: &'static str,
) -> Result<&'static str, ParseError> {
    let Some(value) = params.value() else {
        return Ok(default);
    };
    let token = value.token();
    allowed
        .iter()
        .find(|allowed| allowed.eq_ignore_ascii_case(token))
        .copied()
        .ok_or_else(|| ParseError::ValueType {
            property: name,
            value: token.to_owned(),
        })
}

/// Parses `raw` as a `V`.
fn convert<V>(raw: &[u8]) -> Result<V, ParseError>
where
    V: for<'a> TryFrom<&'a [u8]>,
    for<'a> <V as TryFrom<&'a [u8]>>::Error: Into<values::ValueError>,
{
    V::try_from(raw).map_err(|e| match e.into() {
        // Invalid UTF-8 is the same error wherever it was found.
        values::ValueError::Utf8(e)
        | values::ValueError::Text(crate::values::ValueError::Utf8(e)) => {
            ParseError::Utf8(e)
        }
        e => ParseError::Value(e),
    })
}

/// A property whose one value type is `ty`.
fn typed<V>(
    name: &'static str,
    params: &Parameters,
    raw: &[u8],
    ty: &'static str,
) -> Result<V, ParseError>
where
    V: for<'a> TryFrom<&'a [u8]>,
    for<'a> <V as TryFrom<&'a [u8]>>::Error: Into<values::ValueError>,
{
    value_type(name, params, &[ty], ty)?;
    convert(raw)
}

/// A property that is a URI or text, `default` unless `VALUE` says otherwise.
fn text_or_uri(
    name: &'static str,
    params: &Parameters,
    raw: &[u8],
    default: &'static str,
) -> Result<values::TextOrUri, ParseError> {
    match value_type(name, params, &["text", "uri"], default)? {
        "uri" => convert(raw).map(values::TextOrUri::Uri),
        _ => convert(raw).map(values::TextOrUri::Text),
    }
}

/// `UID`: a URI by default, text when `VALUE=text`, and, when `VALUE` is
/// absent and the value isn't a URI, text anyway.
fn uid(params: &Parameters, raw: &[u8]) -> Result<values::Uid, ParseError> {
    match value_type("UID", params, &["text", "uri"], "uri")? {
        "text" => convert(raw).map(values::Uid::Text),
        _ if params.value().is_none() => convert(raw),
        _ => convert(raw).map(values::Uid::Uri),
    }
}

/// `TZ`: text by default, or the URI or UTC offset `VALUE` names.
fn tz(params: &Parameters, raw: &[u8]) -> Result<values::Timezone, ParseError> {
    match value_type("TZ", params, &["text", "uri", "utc-offset"], "text")? {
        "uri" => convert(raw).map(values::Timezone::Uri),
        "utc-offset" => convert(raw).map(values::Timezone::UtcOffset),
        _ => convert(raw).map(values::Timezone::Text),
    }
}

/// `BDAY` and `ANNIVERSARY`: a date-and-or-time by default, text when
/// `VALUE=text`.
fn date_or_text(
    name: &'static str,
    params: &Parameters,
    raw: &[u8],
) -> Result<values::DateAndOrTimeOrText, ParseError> {
    match value_type(
        name,
        params,
        &["date-and-or-time", "text"],
        "date-and-or-time",
    )? {
        "text" => convert(raw).map(values::DateAndOrTimeOrText::Text),
        _ => convert(raw).map(values::DateAndOrTimeOrText::DateAndOrTime),
    }
}

/// What RFC 6350 forbids of a property's parameters outright: the `TYPE`
/// values of `TEL` and `RELATED` on any other property (§5.6), and `PID` on
/// `CLIENTPIDMAP` (§6.7.7).
fn check_params(
    name: &'static str,
    params: &Parameters,
) -> Result<(), ParseError> {
    for value in params.types() {
        let only = match value {
            TypeValue::Tel(_) => "TEL",
            TypeValue::Related(_) => "RELATED",
            _ => continue,
        };
        if name != only {
            return Err(ParseError::TypeValue {
                property: name,
                value: value.to_string(),
                only,
            });
        }
    }
    if name == "CLIENTPIDMAP" && !params.pid().is_empty() {
        return Err(ParseError::ParamNotAllowed {
            property: name,
            param: "PID",
        });
    }
    Ok(())
}

/// Defines a typed property: its struct with the group, value and
/// parameters (all private, read through accessors), its parser, and its
/// `Display`.
///
/// The value is either read by the closure, which gets the parameters and
/// the raw value, or, given a `VALUE` type name instead, is the property's
/// one value type.
macro_rules! property {
    (
        $(#[$doc:meta])*
        $ty:ident, $name:literal, $value:ty, |$params:ident, $raw:ident| $parse:expr
    ) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $ty {
            group: Option<Group>,
            value: $value,
            params: Parameters,
        }

        impl $ty {
            /// The group the property was written with, if any.
            pub fn group(&self) -> Option<&Group> {
                self.group.as_ref()
            }

            /// The value.
            pub fn value(&self) -> &$value {
                &self.value
            }

            /// The parameters the property was written with.
            pub fn params(&self) -> &Parameters {
                &self.params
            }

            fn parse(
                group: Option<Group>,
                remainder: &[u8],
            ) -> Result<Self, ParseError> {
                let colon = value_start(remainder)?;
                let params = Parameters::try_from(&remainder[..colon])?;
                check_params($name, &params)?;
                let value = {
                    let $params = &params;
                    let $raw = &remainder[colon + 1..];
                    $parse
                }?;
                Ok(Self { group, value, params })
            }
        }

        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if let Some(group) = &self.group {
                    write!(f, "{group}.")?;
                }
                write!(f, "{}{}:{}", $name, self.params, self.value)
            }
        }
    };
    (
        $(#[$doc:meta])*
        $ty:ident, $name:literal, $value:ty, $value_type:literal
    ) => {
        property! {
            $(#[$doc])*
            $ty, $name, $value, |params, raw| {
                typed::<$value>($name, params, raw, $value_type)
            }
        }
    };
}

property! {
    /// The source of the directory information contained in the content
    /// type.
    ///
    /// To identify the source of directory information contained in the
    /// content type.
    ///
    /// The SOURCE property is used to provide the means by which
    /// applications knowledgable in the given directory service protocol can
    /// obtain additional or more up-to-date information from the directory
    /// service. It contains a URI as defined in \[RFC3986\] and/or other
    /// information referencing the vCard to which the information pertains.
    /// When directory information is available from more than one source,
    /// the sending entity can pick what it considers to be the best source,
    /// or multiple SOURCE properties can be included.
    ///
    /// Example:
    ///
    /// > SOURCE:ldap://ldap.example.com/cn=Babs%20Jensen,%20o=Babsco,%20c=US
    ///
    /// [Section 6.1.3](https://datatracker.ietf.org/doc/html/rfc6350#section-6.1.3)
    Source, "SOURCE", values::Uri, "uri"
}

property! {
    /// The kind of object the vCard represents.
    ///
    /// To specify the kind of object the vCard represents.
    ///
    /// The value may be one of "individual", "group", "org" or "location",
    /// an x-name, or an iana-token; see [`values::Kind`]. If this property
    /// is absent, "individual" MUST be assumed as the default.
    ///
    /// Example:
    ///
    /// > KIND:org
    ///
    /// [Section 6.1.4](https://datatracker.ietf.org/doc/html/rfc6350#section-6.1.4)
    Kind, "KIND", values::Kind, "text"
}

property! {
    /// Extended XML-encoded vCard data in a plain vCard.
    ///
    /// To include extended XML-encoded vCard data in a plain vCard.
    ///
    /// The content of this property is a single XML 1.0 element whose
    /// namespace MUST be explicitly specified using the xmlns attribute and
    /// MUST NOT be the vCard 4 namespace ("urn:ietf:params:xml:ns:vcard-4.0").
    /// (This implies that it cannot duplicate a standard vCard property.)
    /// The element is to be interpreted as if it was contained in a `<vcard>`
    /// element, as defined in \[RFC6351\].
    ///
    /// The fragment is subject to normal line folding and escaping, i.e.,
    /// replace all backslashes with "\\", then replace all newlines with
    /// "\n", then fold long lines.
    ///
    /// Support for this property is OPTIONAL, but implementations of this
    /// specification MUST preserve instances of this property when
    /// propagating vCards.
    ///
    /// Example:
    ///
    /// > XML:`<x xmlns="urn:example:x"/>`
    ///
    /// [Section 6.1.5](https://datatracker.ietf.org/doc/html/rfc6350#section-6.1.5)
    Xml, "XML", values::Text, "text"
}

property! {
    /// The formatted text corresponding to the name of the object the vCard
    /// represents.
    ///
    /// To specify the formatted text corresponding to the name of the object
    /// the vCard represents.
    ///
    /// This property is based on the semantics of the X.520 Common Name
    /// attribute \[CCITT.X520.1988\]. The property MUST be present in the
    /// vCard object.
    ///
    /// Example:
    ///
    /// > FN:Mr. John Q. Public\, Esq.
    ///
    /// [Section 6.2.1](https://datatracker.ietf.org/doc/html/rfc6350#section-6.2.1)
    FormattedName, "FN", values::Text, "text"
}

property! {
    /// The components of the name of the object the vCard represents.
    ///
    /// To specify the components of the name of the object the vCard
    /// represents.
    ///
    /// The structured property value corresponds, in sequence, to the Family
    /// Names (also known as surnames), Given Names, Additional Names,
    /// Honorific Prefixes, and Honorific Suffixes. The text components are
    /// separated by the SEMICOLON character (U+003B). Individual text
    /// components can include multiple text values separated by the COMMA
    /// character (U+002C). This property is based on the semantics of the
    /// X.520 individual name attributes \[CCITT.X520.1988\]. The property
    /// SHOULD be present in the vCard object when the name of the object the
    /// vCard represents follows the X.520 model.
    ///
    /// The SORT-AS parameter MAY be applied to this property.
    ///
    /// Example:
    ///
    /// > N:Stevenson;John;Philip,Paul;Dr.;Jr.,M.D.,A.C.P.
    ///
    /// [Section 6.2.2](https://datatracker.ietf.org/doc/html/rfc6350#section-6.2.2)
    Name, "N", values::Name, "text"
}

property! {
    /// The nickname of the object the vCard represents.
    ///
    /// To specify the text corresponding to the nickname of the object the
    /// vCard represents.
    ///
    /// The nickname is the descriptive name given instead of or in addition
    /// to the one belonging to the object the vCard represents. It can also
    /// be used to specify a familiar form of a proper name specified by the
    /// FN or N properties.
    ///
    /// Example:
    ///
    /// > NICKNAME:Jim,Jimmie
    ///
    /// [Section 6.2.3](https://datatracker.ietf.org/doc/html/rfc6350#section-6.2.3)
    Nickname, "NICKNAME", values::TextList, "text"
}

property! {
    /// An image or photograph that annotates some aspect of the object the
    /// vCard represents.
    ///
    /// To specify an image or photograph information that annotates some
    /// aspect of the object the vCard represents.
    ///
    /// Example:
    ///
    /// > PHOTO:http://www.example.com/pub/photos/jqpublic.gif
    ///
    /// [Section 6.2.4](https://datatracker.ietf.org/doc/html/rfc6350#section-6.2.4)
    Photo, "PHOTO", values::Uri, "uri"
}

property! {
    /// The birth date of the object the vCard represents.
    ///
    /// To specify the birth date of the object the vCard represents.
    ///
    /// The default is a single date-and-or-time value. It can also be reset
    /// to a single text value with `VALUE=text`. The `CALSCALE` parameter
    /// can only be present when the value is a date-and-or-time that
    /// actually contains a date or date-time.
    ///
    /// Example:
    ///
    /// > BDAY:--0415
    ///
    /// [Section 6.2.5](https://datatracker.ietf.org/doc/html/rfc6350#section-6.2.5)
    Birthday, "BDAY", values::DateAndOrTimeOrText, |params, raw| {
        date_or_text("BDAY", params, raw)
    }
}

property! {
    /// The date of marriage, or equivalent, of the object the vCard
    /// represents.
    ///
    /// The default is a single date-and-or-time value. It can also be reset
    /// to a single text value with `VALUE=text`. The `CALSCALE` parameter
    /// can only be present when the value is a date-and-or-time that
    /// actually contains a date or date-time.
    ///
    /// Example:
    ///
    /// > ANNIVERSARY:19960415
    ///
    /// [Section 6.2.6](https://datatracker.ietf.org/doc/html/rfc6350#section-6.2.6)
    Anniversary, "ANNIVERSARY", values::DateAndOrTimeOrText, |params, raw| {
        date_or_text("ANNIVERSARY", params, raw)
    }
}

property! {
    /// The sex and gender identity of the object the vCard represents.
    ///
    /// To specify the components of the sex and gender identity of the
    /// object the vCard represents.
    ///
    /// The components correspond, in sequence, to the sex (biological), and
    /// gender identity. Each component is optional.
    ///
    /// Sex component: A single letter. M stands for "male", F stands for
    /// "female", O stands for "other", N stands for "none or not
    /// applicable", U stands for "unknown".
    ///
    /// Gender identity component: Free-form text.
    ///
    /// Example:
    ///
    /// > GENDER:M;Fellow
    ///
    /// [Section 6.2.7](https://datatracker.ietf.org/doc/html/rfc6350#section-6.2.7)
    Gender, "GENDER", values::Gender, "text"
}

property! {
    /// The delivery address of the vCard object.
    ///
    /// To specify the components of the delivery address for the vCard
    /// object.
    ///
    /// The structured type value consists of a sequence of address
    /// components. The component values MUST be specified in their
    /// corresponding position. The structured type value corresponds, in
    /// sequence, to the post office box; the extended address (e.g.,
    /// apartment or suite number); the street address; the locality (e.g.,
    /// city); the region (e.g., state or province); the postal code; the
    /// country name (full name in the language specified in Section 5.1).
    ///
    /// When a component value is missing, the associated component
    /// separator MUST still be specified.
    ///
    /// The property can include the "PREF" parameter to indicate the
    /// preferred delivery address when more than one address is specified.
    ///
    /// The GEO and TZ parameters MAY be used with this property.
    ///
    /// The property can also include a "LABEL" parameter to present a
    /// delivery address label for the address. Its value is a plain-text
    /// string representing the formatted address. Newlines are encoded as
    /// \n, as they are for property values.
    ///
    /// Example:
    ///
    /// > ADR;GEO="geo:12.3457,78.910";LABEL="Mr. John Q. Public, Esq.\n
    /// >  Mail Drop: TNE QB\n123 Main Street\nAny Town, CA  91921-1234\n
    /// >  U.S.A.":;;123 Main Street;Any Town;CA;91921-1234;U.S.A.
    ///
    /// [Section 6.3.1](https://datatracker.ietf.org/doc/html/rfc6350#section-6.3.1)
    Address, "ADR", values::Address, "text"
}

property! {
    /// The telephone number for telephony communication with the object the
    /// vCard represents.
    ///
    /// To specify the telephone number for telephony communication with the
    /// object the vCard represents.
    ///
    /// By default, it is a single free-form text value (for backward
    /// compatibility with vCard 3), but it SHOULD be reset to a URI value
    /// with `VALUE=uri`. It is expected that the URI scheme will be "tel",
    /// as specified in \[RFC3966\], but other schemes MAY be used. Real data
    /// has numbers that are neither clean text nor a URI (`+1 (555)
    /// 123-4567`), and they are kept as text.
    ///
    /// The property can include the "PREF" parameter to indicate a
    /// preferred-use telephone number.
    ///
    /// The property can include the parameter "TYPE" to specify intended use
    /// for the telephone number: "text", "voice", "fax", "cell", "video",
    /// "pager" or "textphone". The default type is "voice".
    ///
    /// Example:
    ///
    /// > TEL;VALUE=uri;PREF=1;TYPE="voice,home":tel:+1-555-555-5555;ext=5555
    ///
    /// [Section 6.4.1](https://datatracker.ietf.org/doc/html/rfc6350#section-6.4.1)
    Telephone, "TEL", values::TextOrUri, |params, raw| {
        text_or_uri("TEL", params, raw, "text")
    }
}

property! {
    /// The electronic mail address for communication with the object the
    /// vCard represents.
    ///
    /// To specify the electronic mail address for communication with the
    /// object the vCard represents.
    ///
    /// The property can include tye "PREF" parameter to indicate a
    /// preferred-use email address when more than one is specified.
    ///
    /// Even though the value is free-form UTF-8 text, it is likely to be
    /// interpreted by a Mail User Agent (MUA) as an "addr-spec", as defined
    /// in \[RFC5322\], Section 3.4.1. Readers should also be aware of the
    /// current work toward internationalized email addresses \[RFC5335bis\].
    ///
    /// Example:
    ///
    /// > EMAIL;TYPE=work:jqpublic@xyz.example.com
    ///
    /// [Section 6.4.2](https://datatracker.ietf.org/doc/html/rfc6350#section-6.4.2)
    Email, "EMAIL", values::Text, "text"
}

property! {
    /// The URI for instant messaging and presence protocol communications
    /// with the object the vCard represents.
    ///
    /// To specify the URI for instant messaging and presence protocol
    /// communications with the object the vCard represents.
    ///
    /// The property may include the "PREF" parameter to indicate that this
    /// is a preferred address and has the same semantics as the "PREF"
    /// parameter in a TEL property.
    ///
    /// If this property's value is a URI that can be used for voice and/or
    /// video, the TEL property (Section 6.4.1) SHOULD be used in addition to
    /// this property.
    ///
    /// Example:
    ///
    /// > IMPP;PREF=1:xmpp:alice@example.com
    ///
    /// [Section 6.4.3](https://datatracker.ietf.org/doc/html/rfc6350#section-6.4.3)
    Impp, "IMPP", values::Uri, "uri"
}

property! {
    /// The language(s) that may be used for contacting the entity associated
    /// with the vCard.
    ///
    /// To specify the language(s) that may be used for contacting the entity
    /// associated with the vCard.
    ///
    /// Example:
    ///
    /// > LANG;TYPE=work;PREF=1:en
    ///
    /// [Section 6.4.4](https://datatracker.ietf.org/doc/html/rfc6350#section-6.4.4)
    Lang, "LANG", values::LanguageTag, "language-tag"
}

property! {
    /// Information related to the time zone of the object the vCard
    /// represents.
    ///
    /// To specify information related to the time zone of the object the
    /// vCard represents.
    ///
    /// The default is a single text value. It can also be reset to a single
    /// URI or utc-offset value.
    ///
    /// It is expected that names from the public-domain Olson database
    /// \[TZ-DB\] will be used, but this is not a restriction. See also
    /// \[IANA-TZ\].
    ///
    /// Note that utc-offset values SHOULD NOT be used because the UTC offset
    /// varies with time -- not just because of the usual daylight saving
    /// time shifts that occur in may regions, but often entire regions will
    /// "re-base" their overall offset. The actual offset may be +/- 1 hour
    /// (or perhaps a little more) than the one given.
    ///
    /// Example:
    ///
    /// > TZ;VALUE=utc-offset:-0500
    ///
    /// [Section 6.5.1](https://datatracker.ietf.org/doc/html/rfc6350#section-6.5.1)
    Tz, "TZ", values::Timezone, |params, raw| tz(params, raw)
}

property! {
    /// Information related to the global positioning of the object the vCard
    /// represents.
    ///
    /// To specify information related to the global positioning of the
    /// object the vCard represents.
    ///
    /// The "geo" URI scheme \[RFC5870\] is particularly well suited for this
    /// property, but other schemes MAY be used.
    ///
    /// Example:
    ///
    /// > GEO:geo:37.386013,-122.082932
    ///
    /// [Section 6.5.2](https://datatracker.ietf.org/doc/html/rfc6350#section-6.5.2)
    Geo, "GEO", values::Uri, "uri"
}

property! {
    /// The position or job of the object the vCard represents.
    ///
    /// To specify the position or job of the object the vCard represents.
    ///
    /// This property is based on the X.520 Title attribute
    /// \[CCITT.X520.1988\].
    ///
    /// Example:
    ///
    /// > TITLE:Research Scientist
    ///
    /// [Section 6.6.1](https://datatracker.ietf.org/doc/html/rfc6350#section-6.6.1)
    Title, "TITLE", values::Text, "text"
}

property! {
    /// The function or part played in a particular situation by the object
    /// the vCard represents.
    ///
    /// To specify the function or part played in a particular situation by
    /// the object the vCard represents.
    ///
    /// This property is based on the X.520 Business Category explanatory
    /// attribute \[CCITT.X520.1988\]. This property is included as an
    /// organizational type to avoid confusion with the semantics of the
    /// TITLE property and incorrect usage of that property when the
    /// semantics of this property is intended.
    ///
    /// Example:
    ///
    /// > ROLE:Project Leader
    ///
    /// [Section 6.6.2](https://datatracker.ietf.org/doc/html/rfc6350#section-6.6.2)
    Role, "ROLE", values::Text, "text"
}

property! {
    /// A graphic image of a logo associated with the object the vCard
    /// represents.
    ///
    /// To specify a graphic image of a logo associated with the object the
    /// vCard represents.
    ///
    /// Example:
    ///
    /// > LOGO:http://www.example.com/pub/logos/abccorp.jpg
    ///
    /// [Section 6.6.3](https://datatracker.ietf.org/doc/html/rfc6350#section-6.6.3)
    Logo, "LOGO", values::Uri, "uri"
}

property! {
    /// The organizational name and units associated with the vCard.
    ///
    /// To specify the organizational name and units associated with the
    /// vCard.
    ///
    /// The property is based on the X.520 Organization Name and Organization
    /// Unit attributes \[CCITT.X520.1988\]. The property value is a structured
    /// type consisting of the organization name, followed by zero or more
    /// levels of organizational unit names.
    ///
    /// The SORT-AS parameter MAY be applied to this property.
    ///
    /// Example:
    ///
    /// > ORG:ABC\, Inc.;North American Division;Marketing
    ///
    /// [Section 6.6.4](https://datatracker.ietf.org/doc/html/rfc6350#section-6.6.4)
    Organization, "ORG", values::Organization, "text"
}

property! {
    /// A member in the group this vCard represents.
    ///
    /// To include a member in the group this vCard represents.
    ///
    /// A single URI. It MAY refer to something other than a vCard object.
    /// For example, an email distribution list could employ the "mailto" URI
    /// scheme \[RFC6068\] for efficiency.
    ///
    /// This property MUST NOT be present unless the value of the KIND
    /// property is "group".
    ///
    /// Example:
    ///
    /// > MEMBER:urn:uuid:03a0e51f-d1aa-4385-8a53-e29025acd8af
    ///
    /// [Section 6.6.5](https://datatracker.ietf.org/doc/html/rfc6350#section-6.6.5)
    Member, "MEMBER", values::Uri, "uri"
}

property! {
    /// A relationship between another entity and the entity represented by
    /// this vCard.
    ///
    /// To specify a relationship between another entity and the entity
    /// represented by this vCard.
    ///
    /// A single URI. It can also be reset to a single text value. The text
    /// value can be used to specify textual information.
    ///
    /// The TYPE parameter MAY be used to characterize the related entity. It
    /// contains a comma-separated list of values that are registered with
    /// IANA as described in Section 10.2. The registry is pre-populated with
    /// the values defined in \[xfn\]. This document also specifies two
    /// additional values:
    ///
    /// agent: an entity who may sometimes act on behalf of the entity
    /// associated with the vCard.
    ///
    /// emergency: indicates an emergency contact
    ///
    /// Example:
    ///
    /// > RELATED;TYPE=friend:urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6
    ///
    /// [Section 6.6.6](https://datatracker.ietf.org/doc/html/rfc6350#section-6.6.6)
    Related, "RELATED", values::TextOrUri, |params, raw| {
        text_or_uri("RELATED", params, raw, "uri")
    }
}

property! {
    /// Application category information about the vCard, also known as
    /// "tags".
    ///
    /// To specify application category information about the vCard, also
    /// known as "tags".
    ///
    /// Example:
    ///
    /// > CATEGORIES:INTERNET,IETF,INDUSTRY,INFORMATION TECHNOLOGY
    ///
    /// [Section 6.7.1](https://datatracker.ietf.org/doc/html/rfc6350#section-6.7.1)
    Categories, "CATEGORIES", values::TextList, "text"
}

property! {
    /// Supplemental information or a comment that is associated with the
    /// vCard.
    ///
    /// To specify supplemental information or a comment that is associated
    /// with the vCard.
    ///
    /// The property is based on the X.520 Description attribute
    /// \[CCITT.X520.1988\].
    ///
    /// Example:
    ///
    /// > NOTE:This fax number is operational 0800 to 1715 EST\, Mon-Fri.
    ///
    /// [Section 6.7.2](https://datatracker.ietf.org/doc/html/rfc6350#section-6.7.2)
    Note, "NOTE", values::Text, "text"
}

property! {
    /// The identifier for the product that created the vCard object.
    ///
    /// To specify the identifier for the product that created the vCard
    /// object.
    ///
    /// Implementations SHOULD use a method such as that specified for Formal
    /// Public Identifiers in \[ISO9070\] or for Universal Resource Names in
    /// \[RFC3406\] to ensure that the text value is unique.
    ///
    /// Example:
    ///
    /// > PRODID:-//ONLINE DIRECTORY//NONSGML Version 1//EN
    ///
    /// [Section 6.7.3](https://datatracker.ietf.org/doc/html/rfc6350#section-6.7.3)
    ProductId, "PRODID", values::Text, "text"
}

property! {
    /// Revision information about the current vCard.
    ///
    /// To specify revision information about the current vCard.
    ///
    /// The value distinguishes the current revision of the information in
    /// this vCard for other renditions of the information.
    ///
    /// Example:
    ///
    /// > REV:19951031T222710Z
    ///
    /// [Section 6.7.4](https://datatracker.ietf.org/doc/html/rfc6350#section-6.7.4)
    Revision, "REV", values::Timestamp, "timestamp"
}

property! {
    /// A digital sound content information that annotates some aspect of
    /// the vCard.
    ///
    /// To specify a digital sound content information that annotates some
    /// aspect of the vCard. This property is often used to specify the
    /// proper pronunciation of the name property value of the vCard.
    ///
    /// Example:
    ///
    /// > SOUND:CID:JOHNQPUBLIC.part8.19960229T080000.xyzMail@example.com
    ///
    /// [Section 6.7.5](https://datatracker.ietf.org/doc/html/rfc6350#section-6.7.5)
    Sound, "SOUND", values::Uri, "uri"
}

property! {
    /// A value that represents a globally unique identifier corresponding to
    /// the entity associated with the vCard.
    ///
    /// To specify a value that represents a globally unique identifier
    /// corresponding to the entity associated with the vCard.
    ///
    /// This property is used to uniquely identify the object that the vCard
    /// represents. The "uuid" URN namespace defined in \[RFC4122\] is
    /// particularly well suited to this task, but other URI schemes MAY be
    /// used. Free-form text MAY also be used.
    ///
    /// The value is a URI by default and text when it isn't one; see
    /// [`values::Uid`].
    ///
    /// Example:
    ///
    /// > UID:urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6
    ///
    /// [Section 6.7.6](https://datatracker.ietf.org/doc/html/rfc6350#section-6.7.6)
    Uid, "UID", values::Uid, |params, raw| uid(params, raw)
}

property! {
    /// A global meaning to a local PID source identifier.
    ///
    /// To give a global meaning to a local PID source identifier.
    ///
    /// PID source identifiers (the source identifier is the second field in
    /// a PID parameter instance) are small integers that only have
    /// significance within the scope of a single vCard instance. Each
    /// distinct source identifier present in a vCard MUST have an associated
    /// CLIENTPIDMAP. See Section 7 for more details on the usage of
    /// CLIENTPIDMAP.
    ///
    /// PID source identifiers MUST be strictly positive. Zero is not
    /// allowed.
    ///
    /// As a special exception, the PID parameter MUST NOT be applied to this
    /// property.
    ///
    /// Example:
    ///
    /// > CLIENTPIDMAP:1;urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b
    ///
    /// [Section 6.7.7](https://datatracker.ietf.org/doc/html/rfc6350#section-6.7.7)
    ClientPidMap, "CLIENTPIDMAP", values::ClientPidMap, |_params, raw| {
        convert::<values::ClientPidMap>(raw)
    }
}

property! {
    /// A uniform resource locator associated with the object to which the
    /// vCard refers.
    ///
    /// To specify a uniform resource locator associated with the object to
    /// which the vCard refers. Examples for individuals include personal web
    /// sites, blogs, and social networking site identifiers.
    ///
    /// Example:
    ///
    /// > URL:http://example.org/restaurant.french/~chezchic.html
    ///
    /// [Section 6.7.8](https://datatracker.ietf.org/doc/html/rfc6350#section-6.7.8)
    Url, "URL", values::Uri, "uri"
}

property! {
    /// A public key or authentication certificate associated with the
    /// object that the vCard represents.
    ///
    /// To specify a public key or authentication certificate associated with
    /// the object that the vCard represents.
    ///
    /// A single URI. It can also be reset to a text value with
    /// `VALUE=text`.
    ///
    /// Example:
    ///
    /// > KEY;MEDIATYPE=application/pgp-keys:ftp://example.com/keys/jdoe
    ///
    /// [Section 6.8.1](https://datatracker.ietf.org/doc/html/rfc6350#section-6.8.1)
    Key, "KEY", values::TextOrUri, |params, raw| {
        text_or_uri("KEY", params, raw, "uri")
    }
}

property! {
    /// The URI for the busy time associated with the object that the vCard
    /// represents.
    ///
    /// To specify the URI for the busy time associated with the object that
    /// the vCard represents.
    ///
    /// Where multiple FBURL properties are specified, the default FBURL
    /// property is indicated with the PREF parameter. The FTP \[RFC1738\] or
    /// HTTP \[RFC2616\] type of URI points to an iCalendar \[RFC5545\] object
    /// associated with a snapshot of the next few weeks or months of busy
    /// time data. If the iCalendar object is represented as a file or
    /// document, its file extension should be ".ifb".
    ///
    /// Example:
    ///
    /// > FBURL;PREF=1:http://www.example.com/busy/janedoe
    ///
    /// [Section 6.9.1](https://datatracker.ietf.org/doc/html/rfc6350#section-6.9.1)
    FreeBusyUrl, "FBURL", values::Uri, "uri"
}

property! {
    /// The calendar user address to which a scheduling request should be
    /// sent for the object represented by the vCard.
    ///
    /// To specify the calendar user address \[RFC5545\] to which a scheduling
    /// request \[RFC5546\] should be sent for the object represented by the
    /// vCard.
    ///
    /// Where multiple CALADRURI properties are specified, the default
    /// CALADRURI property is indicated with the PREF parameter.
    ///
    /// Example:
    ///
    /// > CALADRURI;PREF=1:mailto:janedoe@example.com
    ///
    /// [Section 6.9.2](https://datatracker.ietf.org/doc/html/rfc6350#section-6.9.2)
    CalendarAddressUri, "CALADRURI", values::Uri, "uri"
}

property! {
    /// The URI for a calendar associated with the object represented by the
    /// vCard.
    ///
    /// To specify the URI for a calendar associated with the object
    /// represented by the vCard.
    ///
    /// Where multiple CALURI properties are specified, the default CALURI
    /// property is indicated with the PREF parameter. The property should
    /// contain a URI pointing to an iCalendar \[RFC5545\] object associated
    /// with a snapshot of the user's calendar store. If the iCalendar object
    /// is represented as a file or document, its file extension should be
    /// ".ics".
    ///
    /// Example:
    ///
    /// > CALURI;PREF=1:http://cal.example.com/calA
    ///
    /// [Section 6.9.3](https://datatracker.ietf.org/doc/html/rfc6350#section-6.9.3)
    CalendarUri, "CALURI", values::Uri, "uri"
}

/// Defines [`Property`], one variant per property type, and what they all
/// share.
macro_rules! properties {
    ($($ty:ident),* $(,)?) => {
        /// A vCard content line.
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Property {
            $(
                #[doc = concat!("A [`", stringify!($ty), "`] property.")]
                $ty($ty),
            )*
        }

        impl Property {
            /// The group the property was written with, if any.
            pub fn group(&self) -> Option<&Group> {
                match self {
                    $(Self::$ty(p) => p.group(),)*
                }
            }

            /// The parameters the property was written with.
            pub fn params(&self) -> &Parameters {
                match self {
                    $(Self::$ty(p) => p.params(),)*
                }
            }
        }

        impl fmt::Display for Property {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Self::$ty(p) => write!(f, "{p}"),)*
                }
            }
        }
    };
}

properties! {
    Version, Source, Kind, Xml, FormattedName, Name, Nickname, Photo,
    Birthday, Anniversary, Gender, Address, Telephone, Email, Impp, Lang, Tz,
    Geo, Title, Role, Logo, Organization, Member, Related, Categories, Note,
    ProductId, Revision, Sound, Uid, ClientPidMap, Url, Key, FreeBusyUrl,
    CalendarAddressUri, CalendarUri, Xprop, Iana,
}

type PropertyParser = fn(Option<Group>, &[u8]) -> Result<Property, ParseError>;

/// Maps an upper-cased property name to the parser of its typed
/// [`Property`] variant.
static PROPERTY_DISPATCH: phf::Map<&'static [u8], PropertyParser> = phf::phf_map! {
    b"VERSION" => |g, v| Version::parse(g, v).map(Property::Version),
    b"SOURCE" => |g, v| Source::parse(g, v).map(Property::Source),
    b"KIND" => |g, v| Kind::parse(g, v).map(Property::Kind),
    b"XML" => |g, v| Xml::parse(g, v).map(Property::Xml),
    b"FN" => |g, v| FormattedName::parse(g, v).map(Property::FormattedName),
    b"N" => |g, v| Name::parse(g, v).map(Property::Name),
    b"NICKNAME" => |g, v| Nickname::parse(g, v).map(Property::Nickname),
    b"PHOTO" => |g, v| Photo::parse(g, v).map(Property::Photo),
    b"BDAY" => |g, v| Birthday::parse(g, v).map(Property::Birthday),
    b"ANNIVERSARY" => |g, v| Anniversary::parse(g, v).map(Property::Anniversary),
    b"GENDER" => |g, v| Gender::parse(g, v).map(Property::Gender),
    b"ADR" => |g, v| Address::parse(g, v).map(Property::Address),
    b"TEL" => |g, v| Telephone::parse(g, v).map(Property::Telephone),
    b"EMAIL" => |g, v| Email::parse(g, v).map(Property::Email),
    b"IMPP" => |g, v| Impp::parse(g, v).map(Property::Impp),
    b"LANG" => |g, v| Lang::parse(g, v).map(Property::Lang),
    b"TZ" => |g, v| Tz::parse(g, v).map(Property::Tz),
    b"GEO" => |g, v| Geo::parse(g, v).map(Property::Geo),
    b"TITLE" => |g, v| Title::parse(g, v).map(Property::Title),
    b"ROLE" => |g, v| Role::parse(g, v).map(Property::Role),
    b"LOGO" => |g, v| Logo::parse(g, v).map(Property::Logo),
    b"ORG" => |g, v| Organization::parse(g, v).map(Property::Organization),
    b"MEMBER" => |g, v| Member::parse(g, v).map(Property::Member),
    b"RELATED" => |g, v| Related::parse(g, v).map(Property::Related),
    b"CATEGORIES" => |g, v| Categories::parse(g, v).map(Property::Categories),
    b"NOTE" => |g, v| Note::parse(g, v).map(Property::Note),
    b"PRODID" => |g, v| ProductId::parse(g, v).map(Property::ProductId),
    b"REV" => |g, v| Revision::parse(g, v).map(Property::Revision),
    b"SOUND" => |g, v| Sound::parse(g, v).map(Property::Sound),
    b"UID" => |g, v| Uid::parse(g, v).map(Property::Uid),
    b"CLIENTPIDMAP" => |g, v| ClientPidMap::parse(g, v).map(Property::ClientPidMap),
    b"URL" => |g, v| Url::parse(g, v).map(Property::Url),
    b"KEY" => |g, v| Key::parse(g, v).map(Property::Key),
    b"FBURL" => |g, v| FreeBusyUrl::parse(g, v).map(Property::FreeBusyUrl),
    b"CALADRURI" => |g, v| CalendarAddressUri::parse(g, v).map(Property::CalendarAddressUri),
    b"CALURI" => |g, v| CalendarUri::parse(g, v).map(Property::CalendarUri),
};

impl Property {
    /// Parses a property token's optional group, upper-cased name and raw
    /// remainder (`*(";" param) ":" value`) into the matching [`Property`]
    /// variant, dispatching on the name via [`PROPERTY_DISPATCH`]. A name
    /// absent from the table isn't an error: extension properties are
    /// open-ended by design (RFC 6350 §6.10), so it falls back to
    /// [`Xprop`]/[`Iana`].
    pub(crate) fn parse(
        group: Option<&[u8]>,
        name: &[u8],
        remainder: &[u8],
    ) -> Result<Self, ParseError> {
        let group = group.map(Group::try_from).transpose()?;
        if let Some(parse) = PROPERTY_DISPATCH.get(name) {
            parse(group, remainder)
        } else if name.starts_with(b"X-") {
            Xprop::parse(group, name, remainder).map(Self::Xprop)
        } else {
            Iana::parse(group, name, remainder).map(Self::Iana)
        }
    }
}
