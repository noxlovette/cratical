#[cfg(feature = "rfc_9074")]
use crate::properties::SharedParams;
use crate::{properties::AltrepLanguageParams, values::Text};

/// This property is used to specify a name of the iCalendar object that
/// can be used by calendar user agents when presenting the calendar data
/// to a user.
///
/// RFC 7986 §5.1 defines this property directly on `VCALENDAR`, where this
/// crate wires it in unconditionally (RFC 7986 is a core, always-on update
/// to RFC 5545 here — see the crate-level docs). RFC 9073 §7.2 separately
/// reuses this same property on the "VLOCATION" component, gated behind the
/// `rfc_9074` feature since that's where `VLOCATION` itself lives.
///
/// This property can be specified multiple times in an iCalendar object;
/// however, each property MUST represent the name of the calendar in a
/// different language.
///
/// Example:
///
/// > NAME:Company Vacation
///
/// [Section 5.1](https://datatracker.ietf.org/doc/html/rfc7986#section-5.1)
#[derive(Debug)]
pub struct Name {
    value: Text,
    params: AltrepLanguageParams,
}

impl_try_from_bytes!(Name, Text, AltrepLanguageParams);
impl_altrep_language_builder!(NameBuilder, Name, Text);

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NAME{}:{}", self.params, self.value)
    }
}

/// This property specifies the type(s) of a location.
///
/// This property MAY be specified in "VLOCATION" components and provides
/// a way to differentiate multiple locations.  For example, it allows
/// event producers to provide location information for the venue and the
/// parking. Multiple values may be used if the location has multiple
/// purposes, for example, a hotel and a restaurant.
///
/// Example:
///
/// > LOCATION-TYPE:HOTEL,RESTAURANT
///
/// [Section 6.1](https://datatracker.ietf.org/doc/html/rfc9073#section-6.1)
#[cfg(feature = "rfc_9074")]
#[derive(Debug)]
pub struct LocationType {
    value: Vec<Text>,
    params: SharedParams,
}

#[cfg(feature = "rfc_9074")]
impl_try_from_bytes_list!(LocationType, Text, SharedParams);
#[cfg(feature = "rfc_9074")]
impl_simple_property!(LocationType, Vec<Text>);

#[cfg(feature = "rfc_9074")]
impl std::fmt::Display for LocationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LOCATION-TYPE{}:", self.params)?;
        crate::properties::fmt_comma_list(f, &self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_parses_and_displays() {
        let line = "NAME:Company Vacation";
        let name = Name::try_from(line.as_bytes()[4..].as_ref()).unwrap();
        assert_eq!(name.to_string(), line);
    }

    #[cfg(feature = "rfc_9074")]
    #[test]
    fn location_type_parses_a_single_value() {
        let line = "LOCATION-TYPE:HOTEL";
        let loctype =
            LocationType::try_from(line.as_bytes()[13..].as_ref()).unwrap();
        assert_eq!(loctype.to_string(), line);
    }

    #[cfg(feature = "rfc_9074")]
    #[test]
    fn location_type_parses_multiple_comma_separated_values() {
        let line = "LOCATION-TYPE:HOTEL,RESTAURANT";
        let loctype =
            LocationType::try_from(line.as_bytes()[13..].as_ref()).unwrap();
        assert_eq!(loctype.to_string(), line);
    }

    #[test]
    fn name_builder_round_trips() {
        let name = NameBuilder::new("Company Vacation".into()).build();
        assert_eq!(name.to_string(), "NAME:Company Vacation");
    }

    #[cfg(feature = "rfc_9074")]
    #[test]
    fn location_type_new_matches_the_parsed_equivalent() {
        let loctype =
            LocationType::new(vec!["HOTEL".into(), "RESTAURANT".into()]);
        assert_eq!(loctype.to_string(), "LOCATION-TYPE:HOTEL,RESTAURANT");
    }
}
