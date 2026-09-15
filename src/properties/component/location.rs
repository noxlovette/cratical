use crate::{
    properties::{AltrepLanguageParams, SharedParams},
    values::Text,
};

/// This property is used to specify a name of the iCalendar object that
/// can be used by calendar user agents when presenting the calendar data
/// to a user.
///
/// This specification makes use of this property (defined in \[RFC7986\])
/// on the "VLOCATION" component (RFC 9073 §7.2) to name a location, e.g.
/// a venue.
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
#[derive(Debug)]
pub struct LocationType {
    value: Vec<Text>,
    params: SharedParams,
}

impl_try_from_bytes_list!(LocationType, Text, SharedParams);

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

    #[test]
    fn location_type_parses_a_single_value() {
        let line = "LOCATION-TYPE:HOTEL";
        let loctype =
            LocationType::try_from(line.as_bytes()[13..].as_ref()).unwrap();
        assert_eq!(loctype.to_string(), line);
    }

    #[test]
    fn location_type_parses_multiple_comma_separated_values() {
        let line = "LOCATION-TYPE:HOTEL,RESTAURANT";
        let loctype =
            LocationType::try_from(line.as_bytes()[13..].as_ref()).unwrap();
        assert_eq!(loctype.to_string(), line);
    }
}
