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
    /// Constructs a new `EXDATE` from its list of values and an optional
    /// `TZID`, resolving each floating `DATE-TIME` value against it (RFC
    /// 5545 §3.8.5.1). A homogeneous list of `DATE` values automatically
    /// gets `VALUE=DATE`.
    pub fn new(
        value: Vec<DateOrDatetime>,
        tzid: Option<TimeZoneIdentifier>,
    ) -> Self {
        let value: Vec<_> = value
            .into_iter()
            .map(|v| v.resolve_tzid(tzid.as_ref()))
            .collect();
        let data_type = value
            .first()
            .filter(|v| matches!(v, DateOrDatetime::Date(_)))
            .map(|_| ValueDataType::Date);
        Self {
            value,
            params: ExDateParams {
                shared: SharedParams::default(),
                data_type,
                tzid,
            },
        }
    }

    /// The parsed `EXDATE` values — used by `build()` to cross-check their
    /// value type (DATE vs DATE-TIME) against the component's `DTSTART`
    /// (RFC 5545 §3.8.5.1).
    pub(crate) fn value(&self) -> &[DateOrDatetime] {
        &self.value
    }

    /// This `EXDATE`'s raw `TZID` parameter text, if any — used by
    /// `build()` to cross-check it against the component's `DTSTART` (RFC
    /// 5545 §3.8.5.1). Compared as text rather than a resolved
    /// [`chrono_tz::Tz`] so the check still works for a `TZID` this crate
    /// can't resolve (issue #27 bucket 3).
    pub(crate) fn tzid(&self) -> Option<&str> {
        self.params.tzid.as_ref().map(TimeZoneIdentifier::as_str)
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
    /// Constructs a new `RDATE` from its list of values and an optional
    /// `TZID`, resolving each floating `DATE-TIME`/`PERIOD` value against
    /// it (RFC 5545 §3.8.5.2). A homogeneous list of `DATE` or `PERIOD`
    /// values automatically gets the matching `VALUE` parameter.
    pub fn new(
        value: Vec<DateTimePeriod>,
        tzid: Option<TimeZoneIdentifier>,
    ) -> Self {
        let value: Vec<_> = value
            .into_iter()
            .map(|v| v.resolve_tzid(tzid.as_ref()))
            .collect();
        let data_type = value.first().and_then(|v| match v {
            DateTimePeriod::Date(_) => Some(ValueDataType::Date),
            DateTimePeriod::Period(_) => Some(ValueDataType::Period),
            DateTimePeriod::DateTime(_) => None,
        });
        Self {
            value,
            params: RDateParams {
                shared: SharedParams::default(),
                data_type,
                tzid,
            },
        }
    }

    /// The parsed `RDATE` values — used by `build()` to cross-check their
    /// value type (DATE vs DATE-TIME vs PERIOD) against the component's
    /// `DTSTART` (RFC 5545 §3.8.5.2).
    pub(crate) fn value(&self) -> &[DateTimePeriod] {
        &self.value
    }

    /// This `RDATE`'s raw `TZID` parameter text, if any — used by
    /// `build()` to cross-check it against the component's `DTSTART` (RFC
    /// 5545 §3.8.5.2). Compared as text rather than a resolved
    /// [`chrono_tz::Tz`] so the check still works for a `TZID` this crate
    /// can't resolve (issue #27 bucket 3).
    pub(crate) fn tzid(&self) -> Option<&str> {
        self.params.tzid.as_ref().map(TimeZoneIdentifier::as_str)
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
impl_simple_property!(RRule, Recur);

impl std::fmt::Display for RRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RRULE{}:{}", self.params, self.value)
    }
}

impl RRule {
    /// The parsed `RECUR` value. Also used internally by component builders
    /// to cross-check `UNTIL` against the enclosing property's `DTSTART`
    /// (RFC 5545 §3.3.10).
    pub fn recur(&self) -> &Recur {
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

    #[test]
    fn rrule_new_matches_the_parsed_equivalent() {
        let recur =
            crate::values::RecurBuilder::new(crate::values::Frequency::Daily)
                .count(10)
                .build()
                .unwrap();
        assert_eq!(RRule::new(recur).to_string(), "RRULE:FREQ=DAILY;COUNT=10");
    }

    #[test]
    fn rrule_recur_and_recur_accessors_read_back_the_parsed_rule() {
        let recur =
            crate::values::RecurBuilder::new(crate::values::Frequency::Monthly)
                .count(5)
                .interval(2)
                .by_day([(Some(1), crate::values::Weekday::Mo)])
                .by_month([3])
                .build()
                .unwrap();
        let rrule = RRule::new(recur);

        let recur = rrule.recur();
        assert!(matches!(recur.freq(), crate::values::Frequency::Monthly));
        assert_eq!(recur.count(), Some(5));
        assert_eq!(recur.interval(), Some(2));
        assert!(recur.until().is_none());

        let by_day = recur.by_day();
        assert_eq!(by_day.len(), 1);
        assert_eq!(by_day[0].ordinal(), Some(1));
        assert!(matches!(by_day[0].weekday(), crate::values::Weekday::Mo));

        let by_month = recur.by_month();
        assert_eq!(by_month.len(), 1);
        assert_eq!(*by_month[0], 3);

        // `value()`/`Deref` (from `impl_simple_property!`) reach the same
        // `Recur`.
        assert!(matches!(
            rrule.value().freq(),
            crate::values::Frequency::Monthly
        ));
    }

    #[test]
    fn exception_date_times_new_sets_value_date_for_date_values() {
        let date =
            crate::values::Date::try_from(b"19960402".as_slice()).unwrap();
        let exdate =
            ExceptionDateTimes::new(vec![DateOrDatetime::Date(date)], None);
        assert_eq!(exdate.to_string(), "EXDATE;VALUE=DATE:19960402");
    }

    #[test]
    fn exception_date_times_new_resolves_tzid() {
        let dt =
            crate::values::DateTime::try_from(b"19980119T020000".as_slice())
                .unwrap();
        let tzid: crate::params::TimeZoneIdentifier =
            b"America/New_York".as_slice().try_into().unwrap();
        let exdate = ExceptionDateTimes::new(
            vec![DateOrDatetime::DateTime(dt)],
            Some(tzid),
        );
        assert_eq!(
            exdate.to_string(),
            "EXDATE;TZID=America/New_York:19980119T070000Z"
        );
    }

    #[test]
    fn recurrence_date_times_new_sets_value_period() {
        let start =
            crate::values::DateTime::try_from(b"19970101T180000Z".as_slice())
                .unwrap();
        let end =
            crate::values::DateTime::try_from(b"19970102T070000Z".as_slice())
                .unwrap();
        let period = crate::values::Period::StartEnd { start, end };
        let rdate = RecurrenceDateTimes::new(
            vec![crate::values::DateTimePeriod::Period(period)],
            None,
        );
        assert_eq!(
            rdate.to_string(),
            "RDATE;VALUE=PERIOD:19970101T180000Z/19970102T070000Z"
        );
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
