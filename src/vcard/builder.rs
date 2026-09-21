use super::{
    VCard, ValidationError,
    properties::{FormattedName, Property, Version},
    values,
};
use std::collections::HashSet;

/// The properties RFC 6350 §6 gives a cardinality of `*1` ("may be present
/// at most once"), the ones `VERSION` (exactly one) aside.
const AT_MOST_ONE: &[&str] = &[
    "KIND",
    "N",
    "BDAY",
    "ANNIVERSARY",
    "GENDER",
    "PRODID",
    "REV",
    "UID",
];

/// Builds a [`VCard`], checking what needs the whole card once it has it.
///
/// A vCard is built from `FN`, the one property it can't do without, and
/// whatever else goes into it, each added with [`VCardBuilder::property`].
/// [`build`](VCardBuilder::build) then checks that the properties agree with
/// each other, which can't be told while they come in one at a time:
///
/// - a property whose cardinality is `*1` occurs once, where instances with the
///   same `ALTID` count as one (§5.4);
/// - `MEMBER` only comes with `KIND:group` (§6.6.5);
/// - each distinct PID source identifier has its `CLIENTPIDMAP` (§6.7.7).
///
/// Whatever is local to one property, that its value is its type and that its
/// parameters are ones it can have, was checked when the property was made.
///
/// The card is vCard 4.0 and gets its `VERSION` from the builder. Reading a
/// card goes through the same checks: [`VCard::parse`] feeds what it reads to
/// a builder.
///
/// Example:
///
/// > BEGIN:VCARD
/// > VERSION:4.0
/// > FN:Simon Perreault
/// > EMAIL;TYPE=work:simon.perreault@viagenie.ca
/// > END:VCARD
///
/// [Section 6](https://datatracker.ietf.org/doc/html/rfc6350#section-6)
#[derive(Debug, Clone)]
pub struct VCardBuilder {
    version: Version,
    properties: Vec<Property>,
}

impl VCardBuilder {
    /// A builder for a vCard 4.0 named `formatted_name`. `FN` is the one
    /// property a card must have (§6.2.1), so it can't be left out.
    pub fn new(formatted_name: FormattedName) -> Self {
        Self {
            version: Version::new(values::Version::V4_0),
            properties: vec![Property::FormattedName(formatted_name)],
        }
    }

    /// A builder for a card read as `version` with no properties yet, for
    /// the parser, which learns the `VERSION` before the card ends and
    /// whether there is an `FN` as it reads.
    pub(crate) fn with_version(version: Version) -> Self {
        Self {
            version,
            properties: Vec::new(),
        }
    }

    /// Adds a property, after those already in the builder. The card's
    /// `VERSION` is written first whatever the order, so adding one sets it
    /// instead.
    pub fn property(mut self, property: impl Into<Property>) -> Self {
        self.ingest(property.into());
        self
    }

    /// Takes in one property as it arrives. What is local to a property was
    /// checked when it was made; anything that needs the other properties
    /// waits for [`build`](Self::build).
    pub(crate) fn ingest(&mut self, property: Property) {
        match property {
            Property::Version(v) => self.version = v,
            other => self.properties.push(other),
        }
    }

    /// Checks the card as a whole and builds it.
    ///
    /// # Errors
    ///
    /// A [`ValidationError`] for the first rule broken; see the type's docs
    /// for the list.
    pub fn build(self) -> Result<VCard, ValidationError> {
        let Self {
            version,
            properties,
        } = self;

        if !properties
            .iter()
            .any(|p| matches!(p, Property::FormattedName(_)))
        {
            return Err(ValidationError::MissingFormattedName);
        }
        check_cardinality(&properties)?;
        check_member(&properties)?;
        check_pids(&properties)?;

        Ok(VCard {
            version,
            properties,
        })
    }
}

/// §5.4: instances of one property with the same `ALTID` count as one
/// toward its cardinality, and one without an `ALTID` is never an
/// alternative of another. So a `*1` property is over its limit when it has
/// two instances with no `ALTID` between them, or with two different ones.
fn check_cardinality(properties: &[Property]) -> Result<(), ValidationError> {
    for &name in AT_MOST_ONE {
        let mut alternatives = HashSet::new();
        let mut unlabelled = 0;
        for p in properties.iter().filter(|p| p.name() == name) {
            match p.params().altid() {
                Some(altid) => {
                    alternatives.insert(altid.as_str());
                }
                None => unlabelled += 1,
            }
        }
        if alternatives.len() + unlabelled > 1 {
            return Err(ValidationError::TooMany(name));
        }
    }
    Ok(())
}

/// §6.6.5: MEMBER "MUST NOT be present unless the value of the KIND
/// property is 'group'". An absent KIND is "individual" (§6.1.4).
fn check_member(properties: &[Property]) -> Result<(), ValidationError> {
    let is_group = properties.iter().any(
        |p| matches!(p, Property::Kind(k) if *k.value() == values::Kind::Group),
    );
    let has_member =
        properties.iter().any(|p| matches!(p, Property::Member(_)));
    if has_member && !is_group {
        return Err(ValidationError::MemberWithoutGroup);
    }
    Ok(())
}

/// §6.7.7: "Each distinct source identifier present in a vCard MUST have an
/// associated CLIENTPIDMAP."
fn check_pids(properties: &[Property]) -> Result<(), ValidationError> {
    let mapped: HashSet<u32> = properties
        .iter()
        .filter_map(|p| match p {
            Property::ClientPidMap(m) => Some(m.value().pid()),
            _ => None,
        })
        .collect();
    for p in properties {
        for pid in p.params().pid() {
            match pid.second() {
                Some(source) if !mapped.contains(&source) => {
                    return Err(ValidationError::UnmappedPidSource(source));
                }
                _ => {}
            }
        }
    }
    Ok(())
}
