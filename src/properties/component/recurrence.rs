use chrono_tz::Tz;

use crate::{
    params::{TimeZoneIdentifier, ValueDataType},
    properties::{
        ParameterError, SharedParams, param_name, param_segments, param_value,
    },
    values::{DateOrDatetime, DateTimePeriod, Recur},
};

/// This property defines the list of DATE-TIME exceptions for recurring events,
/// to-dos, journal entries, or time zone definitions.
///
/// Example:
///
/// > EXDATE:19960402T010000Z,19960403T010000Z,19960404T010000Z
///
/// [Section 3.8.5.1](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.5.1)
#[derive(Debug)]
pub struct ExceptionDateTimes {
    value: Vec<DateOrDatetime>,
    params: ExDateParams,
}

impl TryFrom<&[u8]> for ExceptionDateTimes {
    type Error = crate::ast::parser::ParseError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let colon = crate::properties::value_start(v)?;
        let params = ExDateParams::try_from(&v[..colon])?;
        let value = crate::ast::split_unescaped(&v[colon + 1..], b',')
            .into_iter()
            .map(DateOrDatetime::try_from)
            .map(|r| r.map(|v| v.resolve_tzid(params.tzid.as_ref())))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { value, params })
    }
}

impl std::fmt::Display for ExceptionDateTimes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EXDATE{}:", self.params)?;
        crate::properties::fmt_comma_list(f, &self.value)
    }
}

impl ExceptionDateTimes {
    /// The parsed `EXDATE` values — used by `build()` to cross-check their
    /// value type (DATE vs DATE-TIME) against the component's `DTSTART`
    /// (RFC 5545 §3.8.5.1).
    pub(crate) fn value(&self) -> &[DateOrDatetime] {
        &self.value
    }

    /// This `EXDATE`'s `TZID` parameter, if any — used by `build()` to
    /// cross-check it against the component's `DTSTART` (RFC 5545 §3.8.5.1).
    pub(crate) fn tzid(&self) -> Option<Tz> {
        self.params.tzid.as_ref().map(TimeZoneIdentifier::tz)
    }
}

/// This property defines the list of DATE-TIME values for recurring events,
/// to-dos, journal entries, or time zone definitions.
///
/// Example:
///
/// > RDATE;TZID=America/New_York:19970714T083000
///
/// [Section 3.8.5.2](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.5.2)
#[derive(Debug)]
pub struct RecurrenceDateTimes {
    value: Vec<DateTimePeriod>,
    params: RDateParams,
}

impl TryFrom<&[u8]> for RecurrenceDateTimes {
    type Error = crate::ast::parser::ParseError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let colon = crate::properties::value_start(v)?;
        let params = RDateParams::try_from(&v[..colon])?;
        let value = crate::ast::split_unescaped(&v[colon + 1..], b',')
            .into_iter()
            .map(DateTimePeriod::try_from)
            .map(|r| r.map(|v| v.resolve_tzid(params.tzid.as_ref())))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { value, params })
    }
}

impl std::fmt::Display for RecurrenceDateTimes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RDATE{}:", self.params)?;
        crate::properties::fmt_comma_list(f, &self.value)
    }
}

impl RecurrenceDateTimes {
    /// The parsed `RDATE` values — used by `build()` to cross-check their
    /// value type (DATE vs DATE-TIME vs PERIOD) against the component's
    /// `DTSTART` (RFC 5545 §3.8.5.2).
    pub(crate) fn value(&self) -> &[DateTimePeriod] {
        &self.value
    }

    /// This `RDATE`'s `TZID` parameter, if any — used by `build()` to
    /// cross-check it against the component's `DTSTART` (RFC 5545 §3.8.5.2).
    pub(crate) fn tzid(&self) -> Option<Tz> {
        self.params.tzid.as_ref().map(TimeZoneIdentifier::tz)
    }
}

/// This property defines a rule or repeating pattern for recurring events,
/// to-dos, journal entries, or time zone definitions.
///
/// Example:
///
/// > RRULE:FREQ=DAILY;COUNT=10
///
/// [Section 3.8.5.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.5.3)
#[derive(Debug)]
pub struct RRule {
    value: Recur,
    params: SharedParams,
}

impl_try_from_bytes!(RRule, Recur);

impl std::fmt::Display for RRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RRULE{}:{}", self.params, self.value)
    }
}

impl RRule {
    /// The parsed `RECUR` value — used by component builders to cross-check
    /// `UNTIL` against the enclosing property's `DTSTART` (RFC 5545 §3.3.10).
    pub(crate) fn recur(&self) -> &Recur {
        &self.value
    }
}

/// Parameter bundle for [`ExceptionDateTimes`].
#[derive(Debug, Default)]
struct ExDateParams {
    shared: SharedParams,
    data_type: Option<ValueDataType>,
    tzid: Option<TimeZoneIdentifier>,
}

impl TryFrom<&[u8]> for ExDateParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"VALUE" => {
                    params.data_type =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                b"TZID" => {
                    params.tzid =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

impl std::fmt::Display for ExDateParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.data_type {
            write!(f, ";VALUE={v}")?;
        }
        if let Some(v) = &self.tzid {
            write!(f, ";TZID={v}")?;
        }
        write!(f, "{}", self.shared)
    }
}

#[cfg(test)]
mod exdate_tests {
    use super::*;

    #[test]
    fn exception_date_times_list() {
        let exdate = ExceptionDateTimes::try_from(
            b":19960402T010000Z,19960403T010000Z".as_slice(),
        )
        .unwrap();
        assert_eq!(exdate.value.len(), 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrule_keeps_the_internal_semicolons_in_the_value() {
        // Regression test: RRULE's Recur value is itself ';'-delimited
        // ("FREQ=DAILY;COUNT=10") — a naive first-';' split (the old
        // behavior) would treat ";COUNT=10" as property-level params
        // instead of part of the value, silently stuffing "COUNT=10" into
        // the property's own SharedParams.iana bucket. With the colon-based
        // split, RRULE has no property-level params here at all.
        let rrule =
            RRule::try_from(b":FREQ=DAILY;COUNT=10".as_slice()).unwrap();
        assert!(rrule.params.iana.is_empty());
        assert!(rrule.params.xname.is_empty());
    }

    #[test]
    fn recurrence_date_times_list() {
        let rdate = RecurrenceDateTimes::try_from(
            b":19970714T083000,19970715T083000".as_slice(),
        )
        .unwrap();
        assert_eq!(rdate.value.len(), 2);
    }
}

/// Parameter bundle for [`RecurrenceDateTimes`].
#[derive(Debug, Default)]
struct RDateParams {
    shared: SharedParams,
    data_type: Option<ValueDataType>,
    tzid: Option<TimeZoneIdentifier>,
}

impl TryFrom<&[u8]> for RDateParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"VALUE" => {
                    params.data_type =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                b"TZID" => {
                    params.tzid =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

impl std::fmt::Display for RDateParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.data_type {
            write!(f, ";VALUE={v}")?;
        }
        if let Some(v) = &self.tzid {
            write!(f, ";TZID={v}")?;
        }
        write!(f, "{}", self.shared)
    }
}
