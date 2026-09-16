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

/// Builder for [`VLocation`]. RFC 9073 §7.2 places no cross-field rules on
/// `VLOCATION` beyond `UID` being required, so unlike most of this
/// module's other builders, this one is infallible — there's nothing left
/// to validate in [`Self::build`].
#[derive(Debug)]
pub struct VLocationBuilder {
    uid: Uid,
    name: Option<Name>,
    description: Option<Description>,
    geo: Option<Geo>,
    loctype: Option<LocationType>,
    url: Option<UniformResourceLocator>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
}

impl VLocationBuilder {
    /// Starts building a `VLOCATION` from its one required property,
    /// `UID` (RFC 9073 §7.2).
    pub fn new(uid: Uid) -> Self {
        Self {
            uid,
            name: None,
            description: None,
            geo: None,
            loctype: None,
            url: None,
            xprop: Vec::new(),
            iana: Vec::new(),
        }
    }

    /// Sets `NAME`.
    pub fn name(mut self, v: Name) -> Self {
        self.name = Some(v);
        self
    }

    /// Sets `DESCRIPTION`.
    pub fn description(mut self, v: Description) -> Self {
        self.description = Some(v);
        self
    }

    /// Sets `GEO`.
    pub fn geo(mut self, v: Geo) -> Self {
        self.geo = Some(v);
        self
    }

    /// Sets `LOCATION-TYPE`.
    pub fn loctype(mut self, v: LocationType) -> Self {
        self.loctype = Some(v);
        self
    }

    /// Sets `URL`. See [`VLocation`]'s own docs for why this crate models
    /// it, despite it being absent from RFC 9073's formal `locprop`
    /// grammar.
    pub fn url(mut self, v: UniformResourceLocator) -> Self {
        self.url = Some(v);
        self
    }

    /// Adds a non-standard (`X-`) property.
    pub fn xprop(mut self, v: Xprop) -> Self {
        self.xprop.push(v);
        self
    }

    /// Adds an IANA-registered property this crate doesn't otherwise model.
    pub fn iana(mut self, v: Iana) -> Self {
        self.iana.push(v);
        self
    }

    /// Assembles the finished [`VLocation`].
    pub fn build(self) -> VLocation {
        VLocation {
            uid: self.uid,
            name: self.name,
            description: self.description,
            geo: self.geo,
            loctype: self.loctype,
            url: self.url,
            xprop: self.xprop,
            iana: self.iana,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // rustfmt's `format_strings` wrapping of this literal corrupts its
    // runtime value (it splits inside a `\r\n` escape pair, not between
    // whole escapes) — skip it here rather than let a formatting pass
    // silently reintroduce that bug.
    #[rustfmt::skip]
    fn vlocation_builder_round_trips_a_minimal_vlocation() {
        let vlocation =
            VLocationBuilder::new(Uid::new("123456-abcdef-98765432".into()))
                .name(
                    crate::properties::NameBuilder::new("Office".into())
                        .build(),
                )
                .url(UniformResourceLocator::new(
                    crate::values::Uri::parse("geo:40.443,-79.945;u=10")
                        .unwrap(),
                ))
                .build();
        assert_eq!(
            vlocation.to_string(),
            "BEGIN:VLOCATION\r\nUID:123456-abcdef-98765432\r\nNAME:Office\r\nURL:geo:40.443,-79.945;u=10\r\nEND:VLOCATION\r\n"
        );
    }
}
