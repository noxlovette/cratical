use chrono_tz::Tz;

use crate::{
    ast::ComponentError,
    params::{Fbtype, TimeZoneIdentifier, ValueDataType},
    properties::{
        ExceptionDateTimes, ParameterError, RRule, RecurrenceDateTimes,
        SharedParams, param_name, param_segments, param_value,
    },
    values::{
        DateOrDatetime, DateTime, DateTimePeriod, Duration as DurationV,
        Period, ValueError,
    },
};

/// These params are shared by this module's component properties
#[derive(Debug, Default)]
struct DateTimeParams {
    shared: SharedParams,
    value_data_type: Option<ValueDataType>,
    tz_identifier: Option<TimeZoneIdentifier>,
}

impl TryFrom<&[u8]> for DateTimeParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"VALUE" => {
                    params.value_data_type =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                b"TZID" => {
                    params.tz_identifier =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

impl std::fmt::Display for DateTimeParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.value_data_type {
            write!(f, ";VALUE={v}")?;
        }
        if let Some(v) = &self.tz_identifier {
            write!(f, ";TZID={v}")?;
        }
        write!(f, "{}", self.shared)
    }
}

/// This property defines the date and time that a to-do was actually
/// completed.
///
/// Example:
///
/// > COMPLETED:19960401T150000Z
///
/// [Section 3.8.2.1](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.1)
#[derive(Debug)]
pub struct Completed {
    value: DateTime,
    params: SharedParams,
}

impl_try_from_bytes!(Completed, DateTime);
impl_simple_property!(Completed, DateTime);

impl std::fmt::Display for Completed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "COMPLETED{}:{}", self.params, self.value)
    }
}

/// This property specifies the date and time that a calendar component ends.
///
/// Example:
///
/// > DTEND:19960401T150000Z
/// >
/// > DTEND;VALUE=DATE:19980704
///
/// [Section 3.8.2.2](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.2)
#[derive(Debug)]
pub struct DateTimeEnd {
    value: DateOrDatetime,
    params: DateTimeParams,
}

impl TryFrom<&[u8]> for DateTimeEnd {
    type Error = crate::ast::parser::ParseError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let colon = crate::properties::value_start(v)?;
        let params = DateTimeParams::try_from(&v[..colon])?;
        let value = DateOrDatetime::try_from(&v[colon + 1..])?
            .resolve_tzid(params.tz_identifier.as_ref());
        Ok(Self { value, params })
    }
}

impl DateTimeEnd {
    /// The parsed `DTEND` value — used by `build()` to cross-check its
    /// value type (DATE vs DATE-TIME) against the component's `DTSTART`
    /// (RFC 5545 §3.8.2.2).
    pub(crate) fn value(&self) -> &DateOrDatetime {
        &self.value
    }
}

impl std::fmt::Display for DateTimeEnd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DTEND{}:{}", self.params, self.value)
    }
}

/// This property defines the date and time that a to-do is expected to be
/// completed.
///
/// Example:
///
/// > DUE:19980430T000000Z
///
/// [Section 3.8.2.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.3)
#[derive(Debug)]
pub struct DateTimeDue {
    value: DateOrDatetime,
    params: DateTimeParams,
}

impl TryFrom<&[u8]> for DateTimeDue {
    type Error = crate::ast::parser::ParseError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let colon = crate::properties::value_start(v)?;
        let params = DateTimeParams::try_from(&v[..colon])?;
        let value = DateOrDatetime::try_from(&v[colon + 1..])?
            .resolve_tzid(params.tz_identifier.as_ref());
        Ok(Self { value, params })
    }
}

impl DateTimeDue {
    /// The parsed `DUE` value — used by `build()` to cross-check its value
    /// type (DATE vs DATE-TIME) against the component's `DTSTART` (RFC 5545
    /// §3.8.2.3).
    pub(crate) fn value(&self) -> &DateOrDatetime {
        &self.value
    }
}

impl std::fmt::Display for DateTimeDue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DUE{}:{}", self.params, self.value)
    }
}

/// This property specifies when the calendar component begins.
///
/// Example:
///
/// > DTSTART:19980118T073000Z
///
/// [Section 3.8.2.4](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.4)
#[derive(Debug)]
pub struct DateTimeStart {
    value: DateOrDatetime,
    params: DateTimeParams,
}

impl TryFrom<&[u8]> for DateTimeStart {
    type Error = crate::ast::parser::ParseError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let colon = crate::properties::value_start(v)?;
        let params = DateTimeParams::try_from(&v[..colon])?;
        let value = DateOrDatetime::try_from(&v[colon + 1..])?
            .resolve_tzid(params.tz_identifier.as_ref());
        Ok(Self { value, params })
    }
}

impl DateTimeStart {
    /// The parsed `DTSTART` value — used by component builders to
    /// cross-check its value type (DATE vs DATE-TIME) against a sibling
    /// `RRULE`'s `UNTIL` (RFC 5545 §3.3.10).
    pub(crate) fn value(&self) -> &DateOrDatetime {
        &self.value
    }

    /// This `DTSTART`'s `TZID` parameter, if any — used by component
    /// builders to cross-check it against `EXDATE`/`RDATE`'s own `TZID`
    /// (RFC 5545 §3.8.5.1/§3.8.5.2).
    pub(crate) fn tzid(&self) -> Option<Tz> {
        self.params
            .tz_identifier
            .as_ref()
            .map(TimeZoneIdentifier::tz)
    }

    /// RFC 5545 §3.8.2.2/§3.8.2.3: `DTEND`'s and `DUE`'s value type "MUST be
    /// the same value type as the 'DTSTART' property". `name` is used only
    /// for the error message, so this doubles as the implementation of
    /// [`Self::cmp_until`]. Used by both the parser's component builders
    /// (`crate::ast`) and the public, client-facing ones
    /// (`crate::components`).
    pub(crate) fn cmp_value_type(
        &self,
        other: Option<&DateOrDatetime>,
        name: &'static str,
    ) -> Result<(), ComponentError> {
        let Some(other) = other else {
            return Ok(());
        };
        let matches_type = matches!(
            (self.value(), other),
            (DateOrDatetime::Date(_), DateOrDatetime::Date(_))
                | (DateOrDatetime::DateTime(_), DateOrDatetime::DateTime(_))
        );
        if matches_type {
            Ok(())
        } else {
            Err(ComponentError::MismatchedValueType(name, "DTSTART"))
        }
    }

    /// RFC 5545 §3.3.10: `RRULE`'s `UNTIL` rule part "MUST have the same
    /// value type as the 'DTSTART' property". Checked here, once both are
    /// known, rather than in `Recur::try_from` — which parses `RRULE` on its
    /// own and has no access to the sibling `DTSTART`.
    pub(crate) fn cmp_until(
        &self,
        rrule: Option<&RRule>,
    ) -> Result<(), ComponentError> {
        let Some(rrule) = rrule else {
            return Ok(());
        };
        let Some(until) = rrule.recur().until() else {
            return Ok(());
        };
        self.cmp_value_type(Some(until), "RRULE's UNTIL")
    }

    /// RFC 5545 §3.8.5.1: "The value type of this property MUST be the same
    /// as the value type of the 'DTSTART' property" — checked once per
    /// `EXDATE` value listed across every `EXDATE` property on the
    /// component (`EXDATE` takes a comma-separated list, and a component
    /// MAY repeat the property).
    pub(crate) fn cmp_exdate(
        &self,
        exdate: &[ExceptionDateTimes],
    ) -> Result<(), ComponentError> {
        let matches_type = exdate
            .iter()
            .flat_map(ExceptionDateTimes::value)
            .all(|value| {
                matches!(
                    (self.value(), value),
                    (DateOrDatetime::Date(_), DateOrDatetime::Date(_))
                        | (
                            DateOrDatetime::DateTime(_),
                            DateOrDatetime::DateTime(_)
                        )
                )
            });
        if matches_type {
            Ok(())
        } else {
            Err(ComponentError::MismatchedValueType("EXDATE", "DTSTART"))
        }
    }

    /// RFC 5545 §3.8.5.2: "The value type of the 'RDATE' property, if
    /// specified, MUST be the same as the 'DTSTART' property, or its value
    /// type must be PERIOD" — a `PERIOD` value is always allowed regardless
    /// of `DTSTART`'s value type, unlike `EXDATE`, which has no `PERIOD`
    /// alternative.
    pub(crate) fn cmp_rdate(
        &self,
        rdate: &[RecurrenceDateTimes],
    ) -> Result<(), ComponentError> {
        let matches_type = rdate
            .iter()
            .flat_map(RecurrenceDateTimes::value)
            .all(|value| {
                matches!(
                    (self.value(), value),
                    (DateOrDatetime::Date(_), DateTimePeriod::Date(_))
                        | (
                            DateOrDatetime::DateTime(_),
                            DateTimePeriod::DateTime(_)
                        )
                        | (_, DateTimePeriod::Period(_))
                )
            });
        if matches_type {
            Ok(())
        } else {
            Err(ComponentError::MismatchedValueType("RDATE", "DTSTART"))
        }
    }

    /// RFC 5545 §3.8.5.1 requires `EXDATE`'s value type to match
    /// `DTSTART`'s; real-world producers extend that to expecting the same
    /// `TZID` too (see issue #6) — a `DTSTART;TZID=America/New_York` paired
    /// with an `EXDATE;TZID=Europe/London` value (or one specifying no
    /// `TZID` at all) names a different wall-clock instant than intended,
    /// even though both are DATE-TIME. Checked once per `EXDATE` property
    /// occurrence (the `TZID` parameter applies once to the whole
    /// comma-separated value list).
    pub(crate) fn cmp_exdate_tzid(
        &self,
        exdate: &[ExceptionDateTimes],
    ) -> Result<(), ComponentError> {
        let matches_tzid = exdate.iter().all(|e| e.tzid() == self.tzid());
        if matches_tzid {
            Ok(())
        } else {
            Err(ComponentError::MismatchedTzid("EXDATE", "DTSTART"))
        }
    }

    /// RFC 5545 §3.8.5.2 requires `RDATE`'s value type to match `DTSTART`'s
    /// (or be `PERIOD`); real-world producers extend that to expecting the
    /// same `TZID` too when both are DATE-TIME (see [`Self::cmp_exdate_tzid`]).
    pub(crate) fn cmp_rdate_tzid(
        &self,
        rdate: &[RecurrenceDateTimes],
    ) -> Result<(), ComponentError> {
        let matches_tzid = rdate.iter().all(|r| r.tzid() == self.tzid());
        if matches_tzid {
            Ok(())
        } else {
            Err(ComponentError::MismatchedTzid("RDATE", "DTSTART"))
        }
    }
}

impl std::fmt::Display for DateTimeStart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DTSTART{}:{}", self.params, self.value)
    }
}

/// Adds a `<$builder>` type for a `DATE`-or-`DATE-TIME`-valued property
/// whose params are exactly [`DateTimeParams`] (`VALUE` + `TZID`) — i.e.
/// `DTSTART`, `DTEND`, `DUE`.
macro_rules! impl_date_or_datetime_builder {
    ($builder:ident, $prop:ident) => {
        /// Builder for the property this macro was invoked for.
        #[derive(Debug)]
        pub struct $builder {
            value: DateOrDatetime,
            tzid: Option<TimeZoneIdentifier>,
        }

        impl $builder {
            /// Starts a new builder from the property's required value.
            pub fn new(value: DateOrDatetime) -> Self {
                Self { value, tzid: None }
            }

            /// Sets the `TZID` parameter, resolving a floating
            /// `DATE-TIME` value against it (RFC 5545 §3.3.5). Has no
            /// effect on a `DATE` value.
            pub fn tzid(mut self, tzid: TimeZoneIdentifier) -> Self {
                self.tzid = Some(tzid);
                self
            }

            /// Finishes the builder, producing the property. A `DATE`
            /// value automatically gets `VALUE=DATE`.
            pub fn build(self) -> $prop {
                let value = self.value.resolve_tzid(self.tzid.as_ref());
                let value_data_type = matches!(value, DateOrDatetime::Date(_))
                    .then_some(ValueDataType::Date);
                $prop {
                    value,
                    params: DateTimeParams {
                        shared: SharedParams::default(),
                        value_data_type,
                        tz_identifier: self.tzid,
                    },
                }
            }
        }
    };
}

impl_date_or_datetime_builder!(DateTimeStartBuilder, DateTimeStart);
impl_date_or_datetime_builder!(DateTimeEndBuilder, DateTimeEnd);
impl_date_or_datetime_builder!(DateTimeDueBuilder, DateTimeDue);

/// This property specifies a positive duration of time.
///
/// Example:
///
/// > DURATION:PT1H0M0S
///
/// [Section 3.8.2.5](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.5)
#[derive(Debug)]
pub struct Duration {
    value: DurationV,
    params: SharedParams,
}

impl_try_from_bytes!(Duration, DurationV);
impl_simple_property!(Duration, DurationV);

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DURATION{}:{}", self.params, self.value)
    }
}

/// This property defines one or more free or busy time intervals.
///
/// Example:
///
/// > FREEBUSY;FBTYPE=BUSY-UNAVAILABLE:19970308T160000Z/PT8H30M
/// >
/// > FREEBUSY;FBTYPE=FREE:19970308T160000Z/PT3H,19970308T200000Z/PT1H
///
/// [Section 3.8.2.6](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.6)
#[derive(Debug)]
pub struct FreeBusyTime {
    value: Vec<Period>,
    params: FreeBusyTimeParams,
}

impl_try_from_bytes_list!(FreeBusyTime, Period, FreeBusyTimeParams);

impl FreeBusyTime {
    /// Constructs a new `FREEBUSY` property from its value and `FBTYPE`
    /// parameter.
    pub fn new(value: Vec<Period>, fb_time_type: Fbtype) -> Self {
        Self {
            value,
            params: FreeBusyTimeParams {
                shared: SharedParams::default(),
                fb_time_type,
            },
        }
    }
}

impl std::fmt::Display for FreeBusyTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FREEBUSY{}:", self.params)?;
        crate::properties::fmt_comma_list(f, &self.value)
    }
}

#[derive(Debug, Default)]
struct FreeBusyTimeParams {
    shared: SharedParams,
    fb_time_type: Fbtype,
}

impl TryFrom<&[u8]> for FreeBusyTimeParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"FBTYPE" => {
                    params.fb_time_type =
                        param_value(segment)?.as_slice().try_into()?
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

impl std::fmt::Display for FreeBusyTimeParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ";FBTYPE={}", self.fb_time_type)?;
        write!(f, "{}", self.shared)
    }
}

/// This property defines whether or not an event is transparent to busy time
/// searches.
///
/// Example:
///
/// > TRANSP:TRANSPARENT
///
/// [Section 3.8.2.7](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.7)
#[derive(Debug)]
pub struct TimeTransparency {
    value: TranspValue,
    params: SharedParams,
}

impl_try_from_bytes!(TimeTransparency, TranspValue);
impl_simple_property!(TimeTransparency, TranspValue);

impl std::fmt::Display for TimeTransparency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TRANSP{}:{}", self.params, self.value)
    }
}

/// Time transparency value for [`TimeTransparency`].
#[derive(Debug, Default)]
pub enum TranspValue {
    /// Event blocks busy-time searches. Default.
    #[default]
    Opaque,
    /// Event does not block busy-time searches.
    Transparent,
}

impl TryFrom<&[u8]> for TranspValue {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        match v {
            b"OPAQUE" => Ok(Self::Opaque),
            b"TRANSPARENT" => Ok(Self::Transparent),
            _ => Err(ValueError::Malformed {
                expected: "OPAQUE or TRANSPARENT".into(),
                received: std::str::from_utf8(v).ok().map(|s| s.into()),
            }),
        }
    }
}

impl std::fmt::Display for TranspValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Opaque => "OPAQUE",
            Self::Transparent => "TRANSPARENT",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transp_value_tokens() {
        assert!(matches!(
            TranspValue::try_from(b"OPAQUE".as_slice()),
            Ok(TranspValue::Opaque)
        ));
        assert!(matches!(
            TranspValue::try_from(b"TRANSPARENT".as_slice()),
            Ok(TranspValue::Transparent)
        ));
    }

    #[test]
    fn transp_value_rejects_unknown() {
        assert!(TranspValue::try_from(b"BOGUS".as_slice()).is_err());
    }

    #[test]
    fn dtstart_display_round_trips_a_utc_value_with_no_params() {
        let dtstart =
            DateTimeStart::try_from(b":19980118T073000Z".as_slice()).unwrap();
        assert_eq!(dtstart.to_string(), "DTSTART:19980118T073000Z");
    }

    #[test]
    fn dtstart_display_round_trips_the_tzid_param() {
        let dtstart = DateTimeStart::try_from(
            b";TZID=America/New_York:19980119T020000".as_slice(),
        )
        .unwrap();
        assert_eq!(
            dtstart.to_string(),
            "DTSTART;TZID=America/New_York:19980119T070000Z"
        );
    }

    #[test]
    fn duration_property_display_round_trips() {
        let duration = Duration::try_from(b":PT1H0M0S".as_slice()).unwrap();
        assert_eq!(duration.to_string(), "DURATION:PT1H");
    }

    #[test]
    fn dtstart_builder_round_trips_a_utc_value_with_no_params() {
        let dt = DateTime::try_from(b"19980118T073000Z".as_slice()).unwrap();
        let dtstart =
            DateTimeStartBuilder::new(DateOrDatetime::DateTime(dt)).build();
        assert_eq!(dtstart.to_string(), "DTSTART:19980118T073000Z");
    }

    #[test]
    fn dtstart_builder_resolves_a_floating_value_against_tzid() {
        let dt = DateTime::try_from(b"19980119T020000".as_slice()).unwrap();
        let tzid: TimeZoneIdentifier =
            b"America/New_York".as_slice().try_into().unwrap();
        let dtstart = DateTimeStartBuilder::new(DateOrDatetime::DateTime(dt))
            .tzid(tzid)
            .build();
        assert_eq!(
            dtstart.to_string(),
            "DTSTART;TZID=America/New_York:19980119T070000Z"
        );
    }

    #[test]
    fn dtend_builder_sets_value_date_for_a_date_value() {
        let date =
            crate::values::Date::try_from(b"19980704".as_slice()).unwrap();
        let dtend = DateTimeEndBuilder::new(DateOrDatetime::Date(date)).build();
        assert_eq!(dtend.to_string(), "DTEND;VALUE=DATE:19980704");
    }

    #[test]
    fn due_builder_round_trips() {
        let dt = DateTime::try_from(b"19980430T000000Z".as_slice()).unwrap();
        let due = DateTimeDueBuilder::new(DateOrDatetime::DateTime(dt)).build();
        assert_eq!(due.to_string(), "DUE:19980430T000000Z");
    }

    #[test]
    fn free_busy_time_new_round_trips() {
        let start = DateTime::try_from(b"19970308T160000Z".as_slice()).unwrap();
        let duration = crate::values::Duration::new(
            chrono::Duration::hours(8) + chrono::Duration::minutes(30),
        );
        let ps = vec![Period::Duration { start, duration }];
        let freebusy = FreeBusyTime::new(ps, Fbtype::BusyUnavailable);
        assert_eq!(
            freebusy.to_string(),
            "FREEBUSY;FBTYPE=BUSY-UNAVAILABLE:19970308T160000Z/PT8H30M"
        );
    }
}
