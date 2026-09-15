use crate::{
    components::write_lines,
    properties::{
        Description, Geo, Iana, LocationType, Name, Uid,
        UniformResourceLocator, Xprop,
    },
};

/// This component provides rich information about the location of an
/// event using the structured data property or, optionally, a plain-text
/// typed value.
///
/// There may be a number of locations associated with an event.  This
/// component provides detailed information about a location.
///
/// When used in a component, the value of this property provides
/// information about the event venue or of related services, such as
/// parking, dining, stations, etc.
///
/// **Deviation from the formal grammar:** RFC 9073's `locprop` ABNF only
/// lists `UID` (required), `DESCRIPTION`/`GEO`/`LOCATION-TYPE`/`NAME`
/// (optional, singleton), and `STRUCTURED-DATA`/`iana-prop` (optional,
/// repeatable) — it does not name `URL`. `STRUCTURED-DATA` itself isn't
/// modeled by this crate. RFC 9074 §8's own worked example nonetheless
/// specifies a `VLOCATION`'s location with a `URL` property carrying a
/// `geo:` URI ("used to indicate the actual location(s) to trigger off
/// of, specified with a URL property containing a 'geo' URI"), which this
/// crate follows: `URL` is modeled as an ordinary optional singleton
/// field here, the same way it's treated as a legal `iana-prop` fallback
/// per RFC 9073's own grammar.
///
/// Example:
///
/// > BEGIN:VLOCATION
/// >
/// > UID:123456-abcdef-98765432
/// >
/// > NAME:Office
/// >
/// > URL:geo:40.443,-79.945;u=10
/// >
/// > END:VLOCATION
/// >
///
/// [Section 7.2](https://datatracker.ietf.org/doc/html/rfc9073#section-7.2)
#[derive(Debug)]
pub struct VLocation {
    pub(crate) uid: Uid,
    pub(crate) name: Option<Name>,
    pub(crate) description: Option<Description>,
    pub(crate) geo: Option<Geo>,
    pub(crate) loctype: Option<LocationType>,
    pub(crate) url: Option<UniformResourceLocator>,
    pub(crate) xprop: Vec<Xprop>,
    pub(crate) iana: Vec<Iana>,
}

impl VLocation {
    /// The `UID` property.
    pub fn uid(&self) -> &Uid {
        &self.uid
    }

    /// The `NAME` property, if present.
    pub fn name(&self) -> Option<&Name> {
        self.name.as_ref()
    }

    /// The `DESCRIPTION` property, if present.
    pub fn description(&self) -> Option<&Description> {
        self.description.as_ref()
    }

    /// The `GEO` property, if present.
    pub fn geo(&self) -> Option<&Geo> {
        self.geo.as_ref()
    }

    /// The `LOCATION-TYPE` property, if present.
    pub fn loctype(&self) -> Option<&LocationType> {
        self.loctype.as_ref()
    }

    /// The `URL` property, if present. See this type's own docs for why
    /// this crate models it, despite it being absent from RFC 9073's
    /// formal `locprop` grammar.
    pub fn url(&self) -> Option<&UniformResourceLocator> {
        self.url.as_ref()
    }

    /// The non-standard (`X-`) properties.
    pub fn xprop(&self) -> &[Xprop] {
        &self.xprop
    }

    /// The IANA-registered properties this crate doesn't otherwise model.
    pub fn iana(&self) -> &[Iana] {
        &self.iana
    }
}

impl std::fmt::Display for VLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BEGIN:VLOCATION\r\n")?;
        write!(f, "{}\r\n", self.uid)?;
        if let Some(v) = &self.name {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.description {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.geo {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.loctype {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.url {
            write!(f, "{v}\r\n")?;
        }
        write_lines(f, &self.xprop)?;
        write_lines(f, &self.iana)?;
        write!(f, "END:VLOCATION\r\n")
    }
}
