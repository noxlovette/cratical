//! vCard properties.
//!
//! Only `VERSION` and the `X-`/IANA passthrough exist so far; the typed
//! properties of RFC 6350 §6 are added to [`Property`] and the dispatch map
//! here.

use super::{
    ParseError,
    params::Parameters,
    value_start,
    values::{Raw, Version as VersionValue},
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
    value: VersionValue,
    params: Parameters,
}

impl Version {
    /// The group the property was written with, if any.
    pub fn group(&self) -> Option<&Group> {
        self.group.as_ref()
    }

    /// The version.
    pub fn value(&self) -> VersionValue {
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
            value: VersionValue::try_from(&remainder[colon + 1..])?,
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
            name: Raw,
            value: Raw,
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
            pub fn value(&self) -> &Raw {
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
                    name: Raw::try_from(name)?,
                    params: Parameters::try_from(&remainder[..colon])?,
                    value: Raw::try_from(&remainder[colon + 1..])?,
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

/// A vCard content line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Property {
    /// `VERSION`
    Version(Version),
    /// A non-standard `X-` property.
    Xprop(Xprop),
    /// A property with no dedicated type.
    Iana(Iana),
}

impl Property {
    /// The group the property was written with, if any.
    pub fn group(&self) -> Option<&Group> {
        match self {
            Self::Version(p) => p.group(),
            Self::Xprop(p) => p.group(),
            Self::Iana(p) => p.group(),
        }
    }
}

impl Property {
    /// The parameters the property was written with.
    pub fn params(&self) -> &Parameters {
        match self {
            Self::Version(p) => p.params(),
            Self::Xprop(p) => p.params(),
            Self::Iana(p) => p.params(),
        }
    }
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Version(p) => p.fmt(f),
            Self::Xprop(p) => p.fmt(f),
            Self::Iana(p) => p.fmt(f),
        }
    }
}

type PropertyParser = fn(Option<Group>, &[u8]) -> Result<Property, ParseError>;

/// Maps an upper-cased property name to the parser of its typed
/// [`Property`] variant.
static PROPERTY_DISPATCH: phf::Map<&'static [u8], PropertyParser> = phf::phf_map! {
    b"VERSION" => |g, v| Version::parse(g, v).map(Property::Version),
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
