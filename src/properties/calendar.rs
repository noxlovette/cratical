use crate::{properties::SharedParams, values::Text};

/// This property defines the calendar scale used for the calendar information
/// specified in the iCalendar object.  This memo is based on the Gregorian
/// calendar scale.  The Gregorian calendar scale is assumed if this property
/// is not specified in the iCalendar object.  It is expected that other
/// calendar scales will be defined in other specifications or by future
/// versions of this memo.
///
/// The value GREGORIAN indicates that the calendar scale of the iCalendar
/// object is Gregorian.  If the "CALSCALE" property is not present in the
/// iCalendar object, then the Gregorian calendar scale is assumed.  The
/// definitions below are defined and referenced as Monday, Tuesday,
/// Wednesday, Thursday, Friday, Saturday, and Sunday.
///
/// Example:
///
/// > CALSCALE:GREGORIAN
///
/// [Section 3.7.1](https://datatracker.ietf.org/doc/html/rfc5545#section-3.7.1)
#[derive(Debug)]
pub struct CalendarScale {
    value: Text,
    params: SharedParams,
}

/// This property defines the iCalendar object method associated with the
/// calendar object.  When used in a MIME message entity, the value of this
/// property MUST be the same as the Content-Type "method" parameter value.
///
/// No methods are defined by this specification.  This is the subject of
/// other specifications, such as the iCalendar Transport-independent
/// Interoperability Protocol (iTIP) defined by [RFC5546](https://datatracker.ietf.org/doc/html/rfc5546).
///
/// Applications MUST ignore x-name and iana-token values they don't
/// recognize.
///
/// Example:
///
/// > METHOD:REQUEST
///
/// [Section 3.7.2](https://datatracker.ietf.org/doc/html/rfc5545#section-3.7.2)
#[derive(Debug)]
pub struct Method {
    value: Text,
    params: SharedParams,
}

/// The vendor of the implementation SHOULD assure that this is a globally
/// unique identifier; using some technique such as an FPI value, as defined
/// in [ISO.9070.1991].
///
/// This property SHOULD NOT be used to alter the interpretation of an
/// iCalendar object beyond the semantics specified in this memo.  For
/// example, it is not to be used to further the understanding of
/// non-standard properties.
///
/// Example:
///
/// > PRODID:-//ABC Corporation//NONSGML My Product//EN
///
/// [Section 3.7.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.7.3)
#[derive(Debug)]
pub struct ProductIdentifier {
    value: Text,
    params: SharedParams,
}

/// A value of "2.0" corresponds to this memo.
///
/// Example:
///
/// > VERSION:2.0
///
/// [Section 3.7.4](https://datatracker.ietf.org/doc/html/rfc5545#section-3.7.4)
#[derive(Debug)]
pub struct Version {
    value: Text,
    params: SharedParams,
}

impl_try_from_bytes!(ProductIdentifier);
impl_try_from_bytes!(Version);
impl_try_from_bytes!(Method);
impl_try_from_bytes!(CalendarScale);

impl_simple_property!(ProductIdentifier, Text);
impl_simple_property!(Version, Text);
impl_simple_property!(Method, Text);
impl_simple_property!(CalendarScale, Text);

impl std::fmt::Display for CalendarScale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CALSCALE{}:{}", self.params, self.value)
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "METHOD{}:{}", self.params, self.value)
    }
}

impl std::fmt::Display for ProductIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PRODID{}:{}", self.params, self.value)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VERSION{}:{}", self.params, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_constructors_match_their_parsed_equivalent() {
        assert_eq!(
            CalendarScale::new("GREGORIAN".into()).to_string(),
            CalendarScale::try_from(b":GREGORIAN".as_slice())
                .unwrap()
                .to_string()
        );
        assert_eq!(
            Method::new("REQUEST".into()).to_string(),
            Method::try_from(b":REQUEST".as_slice())
                .unwrap()
                .to_string()
        );
        assert_eq!(
            ProductIdentifier::new("-//ABC//EN".into()).to_string(),
            ProductIdentifier::try_from(b":-//ABC//EN".as_slice())
                .unwrap()
                .to_string()
        );
        assert_eq!(
            Version::new("2.0".into()).to_string(),
            Version::try_from(b":2.0".as_slice()).unwrap().to_string()
        );
    }
}
