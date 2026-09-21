//! Structured values (RFC 6350 §6): `N`, `ADR`, `ORG`, `GENDER` and
//! `CLIENTPIDMAP`.
//!
//! The components are separated by a SEMICOLON, and a SEMICOLON inside one
//! is escaped. Where a component is a `list-component` it is itself a list
//! separated by COMMAs.

use super::{Text, Uri, ValueError, malformed};
use crate::ast::{split_once, split_unescaped};
use std::fmt;

/// One `list-component`. An empty component has no elements, which is the
/// same on the wire as one empty element.
fn list_component(v: &[u8]) -> Result<Vec<Text>, ValueError> {
    if v.is_empty() {
        return Ok(Vec::new());
    }
    split_unescaped(v, b',')
        .into_iter()
        .map(|item| Ok(Text::try_from(item)?))
        .collect()
}

fn write_list(f: &mut fmt::Formatter<'_>, items: &[Text]) -> fmt::Result {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            f.write_str(",")?;
        }
        write!(f, "{item}")?;
    }
    Ok(())
}

/// Splits `v` into its `N` list-components. A value with fewer components
/// than the grammar has is accepted, since real data drops trailing empty
/// ones (`N:Doe`), and how many were written is returned so it writes back
/// the same. One with more is refused.
fn parse_parts<const N: usize>(
    v: &[u8],
    expected: &'static str,
) -> Result<([Vec<Text>; N], usize), ValueError> {
    let components = split_unescaped(v, b';');
    if components.len() > N {
        return Err(malformed(expected, v));
    }
    let mut parts: [Vec<Text>; N] = std::array::from_fn(|_| Vec::new());
    for (part, component) in parts.iter_mut().zip(&components) {
        *part = list_component(component)?;
    }
    Ok((parts, components.len()))
}

fn write_parts(
    f: &mut fmt::Formatter<'_>,
    parts: &[Vec<Text>],
    written: usize,
) -> fmt::Result {
    for (i, part) in parts.iter().take(written).enumerate() {
        if i > 0 {
            f.write_str(";")?;
        }
        write_list(f, part)?;
    }
    Ok(())
}

/// The components of the name of the object the vCard represents.
///
/// The value is a single structured value with five components, in order:
/// family names (also known as surnames), given names, additional names,
/// honorific prefixes, and honorific suffixes. Each component can have
/// multiple values, delimited by a comma.
///
/// A value with fewer than five components is accepted, and is written back
/// with as many as it came with, so `Doe` and `Doe;;;;` round-trip as
/// different text.
///
/// Example:
///
/// > Public;John;Quinlan;Mr.;Esq.
/// >
/// > Stevenson;John;Philip,Paul;Dr.;Jr.,M.D.,A.C.P.
///
/// [Section 6.2.2](https://datatracker.ietf.org/doc/html/rfc6350#section-6.2.2)
#[derive(Debug, Clone, PartialEq)]
pub struct Name {
    parts: [Vec<Text>; 5],
    written: usize,
}

impl Name {
    /// Builds a name with all five components.
    pub fn new(
        family: Vec<Text>,
        given: Vec<Text>,
        additional: Vec<Text>,
        prefixes: Vec<Text>,
        suffixes: Vec<Text>,
    ) -> Self {
        Self {
            parts: [family, given, additional, prefixes, suffixes],
            written: 5,
        }
    }

    /// The family names (surnames).
    pub fn family(&self) -> &[Text] {
        &self.parts[0]
    }

    /// The given names.
    pub fn given(&self) -> &[Text] {
        &self.parts[1]
    }

    /// The additional names.
    pub fn additional(&self) -> &[Text] {
        &self.parts[2]
    }

    /// The honorific prefixes.
    pub fn prefixes(&self) -> &[Text] {
        &self.parts[3]
    }

    /// The honorific suffixes.
    pub fn suffixes(&self) -> &[Text] {
        &self.parts[4]
    }
}

impl TryFrom<&[u8]> for Name {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let (parts, written) =
            parse_parts(v, "N value of at most 5 components")?;
        Ok(Self { parts, written })
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_parts(f, &self.parts, self.written)
    }
}

/// The components of the delivery address.
///
/// The value is a single structured value with seven components, in order:
/// the post office box; the extended address (e.g., apartment or suite
/// number); the street address; the locality (e.g., city); the region (e.g.,
/// state or province); the postal code; the country name (full name in the
/// language specified in Section 5.1).
///
/// The structured type value corresponds, in sequence, to the post office
/// box; the extended address (e.g., apartment or suite number); the street
/// address; the locality (e.g., city); the region (e.g., state or province);
/// the postal code; the country name (full name). When a component value is
/// missing, the associated component separator MUST still be specified.
///
/// A value with fewer than seven components is accepted, and is written back
/// with as many as it came with.
///
/// Example:
///
/// > ;;123 Main Street;Any Town;CA;91921-1234;U.S.A.
///
/// [Section 6.3.1](https://datatracker.ietf.org/doc/html/rfc6350#section-6.3.1)
#[derive(Debug, Clone, PartialEq)]
pub struct Address {
    parts: [Vec<Text>; 7],
    written: usize,
}

impl Address {
    /// Builds an address with all seven components.
    pub fn new(
        pobox: Vec<Text>,
        extended: Vec<Text>,
        street: Vec<Text>,
        locality: Vec<Text>,
        region: Vec<Text>,
        code: Vec<Text>,
        country: Vec<Text>,
    ) -> Self {
        Self {
            parts: [pobox, extended, street, locality, region, code, country],
            written: 7,
        }
    }

    /// The post office box.
    pub fn pobox(&self) -> &[Text] {
        &self.parts[0]
    }

    /// The extended address (apartment or suite number).
    pub fn extended(&self) -> &[Text] {
        &self.parts[1]
    }

    /// The street address.
    pub fn street(&self) -> &[Text] {
        &self.parts[2]
    }

    /// The locality (city).
    pub fn locality(&self) -> &[Text] {
        &self.parts[3]
    }

    /// The region (state or province).
    pub fn region(&self) -> &[Text] {
        &self.parts[4]
    }

    /// The postal code.
    pub fn code(&self) -> &[Text] {
        &self.parts[5]
    }

    /// The country name.
    pub fn country(&self) -> &[Text] {
        &self.parts[6]
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let (parts, written) =
            parse_parts(v, "ADR value of at most 7 components")?;
        Ok(Self { parts, written })
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_parts(f, &self.parts, self.written)
    }
}

/// The organizational name and units associated with the vCard.
///
/// The property value is a structured type consisting of the organization
/// name, followed by zero or more levels of organizational unit names.
///
/// Unlike `N` and `ADR`, no component is a list: a COMMA in a name is
/// escaped.
///
/// Example:
///
/// > ABC\, Inc.;North American Division;Marketing
///
/// [Section 6.6.4](https://datatracker.ietf.org/doc/html/rfc6350#section-6.6.4)
#[derive(Debug, Clone, PartialEq)]
pub struct Organization {
    name: Text,
    units: Vec<Text>,
}

impl Organization {
    /// Builds an organization from its name and its units, top to bottom.
    pub fn new(name: Text, units: Vec<Text>) -> Self {
        Self { name, units }
    }

    /// The organization name.
    pub fn name(&self) -> &Text {
        &self.name
    }

    /// The organizational units, from the biggest down.
    pub fn units(&self) -> &[Text] {
        &self.units
    }
}

impl TryFrom<&[u8]> for Organization {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut components = split_unescaped(v, b';').into_iter();
        // `split_unescaped` always yields at least one.
        let name = Text::try_from(components.next().unwrap_or_default())?;
        let units = components
            .map(|unit| Ok(Text::try_from(unit)?))
            .collect::<Result<_, ValueError>>()?;
        Ok(Self { name, units })
    }
}

impl fmt::Display for Organization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)?;
        for unit in &self.units {
            write!(f, ";{unit}")?;
        }
        Ok(())
    }
}

/// The sex component of a `GENDER`: a single letter.
///
/// M stands for "male", F stands for "female", O stands for "other", N
/// stands for "none or not applicable", U stands for "unknown".
///
/// Example:
///
/// > M
///
/// [Section 6.2.7](https://datatracker.ietf.org/doc/html/rfc6350#section-6.2.7)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sex {
    /// `M`, male.
    Male,
    /// `F`, female.
    Female,
    /// `O`, other.
    Other,
    /// `N`, none or not applicable.
    NotApplicable,
    /// `U`, unknown.
    Unknown,
}

impl Sex {
    fn letter(self) -> char {
        match self {
            Self::Male => 'M',
            Self::Female => 'F',
            Self::Other => 'O',
            Self::NotApplicable => 'N',
            Self::Unknown => 'U',
        }
    }
}

/// The components of the sex and gender identity of the object the vCard
/// represents.
///
/// A single structured value with two components. Each component has a
/// single text value. The components correspond, in sequence, to the sex
/// (biological), and gender identity. Each component is optional.
///
/// Sex component: A single letter. M stands for "male", F stands for
/// "female", O stands for "other", N stands for "none or not applicable", U
/// stands for "unknown".
///
/// Gender identity component: Free-form text.
///
/// ```text
/// GENDER-value = sex [";" text]
/// sex = "" / "M" / "F" / "O" / "N" / "U"
/// ```
///
/// Whether the identity was written at all is kept, so `M` and `M;` each
/// round-trip as they came.
///
/// Example:
///
/// > M
/// >
/// > F
/// >
/// > M;Fellow
/// >
/// > F;grrrl
/// >
/// > O;intersex
/// >
/// > ;it's complicated
///
/// [Section 6.2.7](https://datatracker.ietf.org/doc/html/rfc6350#section-6.2.7)
#[derive(Debug, Clone, PartialEq)]
pub struct Gender {
    sex: Option<Sex>,
    identity: Option<Text>,
}

impl Gender {
    /// Builds a gender from its two optional components.
    pub fn new(sex: Option<Sex>, identity: Option<Text>) -> Self {
        Self { sex, identity }
    }

    /// The sex, if the component isn't empty.
    pub fn sex(&self) -> Option<Sex> {
        self.sex
    }

    /// The gender identity, if it was written.
    pub fn identity(&self) -> Option<&Text> {
        self.identity.as_ref()
    }
}

impl TryFrom<&[u8]> for Gender {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let err = || malformed("GENDER value (sex [\";\" text])", v);
        let components = split_unescaped(v, b';');
        if components.len() > 2 {
            return Err(err());
        }
        let sex = match components[0].to_ascii_uppercase().as_slice() {
            b"" => None,
            b"M" => Some(Sex::Male),
            b"F" => Some(Sex::Female),
            b"O" => Some(Sex::Other),
            b"N" => Some(Sex::NotApplicable),
            b"U" => Some(Sex::Unknown),
            _ => return Err(err()),
        };
        let identity = components
            .get(1)
            .map(|identity| Text::try_from(*identity))
            .transpose()?;
        Ok(Self { sex, identity })
    }
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(sex) = self.sex {
            write!(f, "{}", sex.letter())?;
        }
        if let Some(identity) = &self.identity {
            write!(f, ";{identity}")?;
        }
        Ok(())
    }
}

/// A mapping from a PID source identifier to a globally unique URI.
///
/// The CLIENTPIDMAP property is used for synchronization. The first field is
/// a small integer, the same as that of the second field of the PID
/// parameter. The second field is a URI that identifies the source, the
/// same URI on every vCard that shares the source.
///
/// ```text
/// CLIENTPIDMAP-value = 1*DIGIT ";" URI
/// ```
///
/// Example:
///
/// > 1;urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b
///
/// [Section 6.7.7](https://datatracker.ietf.org/doc/html/rfc6350#section-6.7.7)
#[derive(Debug, Clone, PartialEq)]
pub struct ClientPidMap {
    pid: u32,
    uri: Uri,
}

impl ClientPidMap {
    /// Builds a map entry.
    pub fn new(pid: u32, uri: Uri) -> Self {
        Self { pid, uri }
    }

    /// The source identifier, as used in the second field of `PID`.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// The URI of the source.
    pub fn uri(&self) -> &Uri {
        &self.uri
    }
}

impl TryFrom<&[u8]> for ClientPidMap {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let err = || malformed("CLIENTPIDMAP value (1*DIGIT \";\" URI)", v);
        let (pid, uri) = split_once(v, b';').ok_or_else(err)?;
        if pid.is_empty() || !pid.iter().all(u8::is_ascii_digit) {
            return Err(err());
        }
        Ok(Self {
            pid: std::str::from_utf8(pid)?.parse().map_err(|_| err())?,
            uri: Uri::try_from(uri)?,
        })
    }
}

impl fmt::Display for ClientPidMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{};{}", self.pid, self.uri)
    }
}
