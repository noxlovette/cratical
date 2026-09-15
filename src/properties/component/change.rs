use crate::{
    properties::SharedParams,
    values::{DateTime, Integer},
};

/// This property specifies the date and time that the calendar information
/// was created by the calendar user agent in the calendar store.
///
/// Example:
///
/// > CREATED:19960329T133000Z
///
/// [Section 3.8.7.1](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.7.1)
#[derive(Debug)]
pub struct DateTimeCreated {
    value: DateTime,
    params: SharedParams,
}

impl_try_from_bytes!(DateTimeCreated, DateTime);
impl_simple_property!(DateTimeCreated, DateTime);

impl std::fmt::Display for DateTimeCreated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CREATED{}:{}", self.params, self.value)
    }
}

/// This property specifies the date and time that the instance of the iCalendar
/// object was created (when `METHOD` is present), or the date and time that the
/// calendar component was last revised in the calendar store (when `METHOD` is
/// absent).
///
/// Example:
///
/// > DTSTAMP:19971210T080000Z
///
/// [Section 3.8.7.2](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.7.2)
#[derive(Debug)]
pub struct DateTimeStamp {
    value: DateTime,
    params: SharedParams,
}

impl_try_from_bytes!(DateTimeStamp, DateTime);
impl_simple_property!(DateTimeStamp, DateTime);

impl std::fmt::Display for DateTimeStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DTSTAMP{}:{}", self.params, self.value)
    }
}

/// This property specifies the date and time that the information associated
/// with the calendar component was last revised in the calendar store.
///
/// Example:
///
/// > LAST-MODIFIED:19960817T133000Z
///
/// [Section 3.8.7.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.7.3)
#[derive(Debug)]
pub struct LastModified {
    value: DateTime,
    params: SharedParams,
}

impl_try_from_bytes!(LastModified, DateTime);
impl_simple_property!(LastModified, DateTime);

impl std::fmt::Display for LastModified {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LAST-MODIFIED{}:{}", self.params, self.value)
    }
}

/// This property defines the revision sequence number of the calendar component
/// within a sequence of revisions.
///
/// Example:
///
/// > SEQUENCE:0
///
/// [Section 3.8.7.4](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.7.4)
#[derive(Debug)]
pub struct Sequence {
    value: Integer,
    params: SharedParams,
}

impl_try_from_bytes!(Sequence, Integer);
impl_simple_property!(Sequence, Integer);

impl std::fmt::Display for Sequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SEQUENCE{}:{}", self.params, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_constructors_match_their_parsed_equivalent() {
        let dt = DateTime::try_from(b"19960329T133000Z".as_slice()).unwrap();
        assert_eq!(
            DateTimeCreated::new(dt).to_string(),
            "CREATED:19960329T133000Z"
        );
        assert_eq!(
            DateTimeStamp::new(dt).to_string(),
            "DTSTAMP:19960329T133000Z"
        );
        assert_eq!(
            LastModified::new(dt).to_string(),
            "LAST-MODIFIED:19960329T133000Z"
        );
        assert_eq!(Sequence::new(Integer::new(0)).to_string(), "SEQUENCE:0");
    }
}
