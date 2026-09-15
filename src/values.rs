use crate::{
    ast::split_once,
    params::TimeZoneIdentifier,
    values::datetime::{
        ICAL_DATE_FMT, ICAL_DATETIME_FMT, ICAL_DATETIME_UTC_FMT,
    },
};
use base64::Engine;
use chrono::{
    DateTime as ChronoDateTime, Duration as ChronoDuration, FixedOffset,
    MappedLocalTime, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone,
    Utc,
};
use chrono_tz::Tz;
pub use recurrence::Recur;
use std::{ops::Deref, str::from_utf8};
use thiserror::Error;
use url::Url;
/// Date/time formatting constants and the [`datetime::DateTimeExt`] helper
/// trait.
pub mod datetime;
/// The RFC 5545's helper
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateOrDatetime {
    /// A calendar date without a time component.
    Date(Date),
    /// A precise calendar date and time of day.
    DateTime(DateTime),
}

impl TryFrom<&[u8]> for DateOrDatetime {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        // DATE is 8 digits (YYYYMMDD); DATE-TIME always contains the "T"
        // time designator. The two are unambiguous by shape alone.
        if v.contains(&b'T') {
            Ok(Self::DateTime(v.try_into()?))
        } else {
            Ok(Self::Date(v.try_into()?))
        }
    }
}

impl DateOrDatetime {
    /// Resolves a `DateTime` value against `tzid`'s real offset rules (RFC
    /// 5545 §3.3.5) when a `TZID` parameter is present. A `Date` value, or
    /// a `DateTime` with no `TZID`, is returned unchanged — `TZID` "MUST
    /// NOT be applied" to `DATE` properties (RFC 5545 §3.2.19).
    pub(crate) fn resolve_tzid(
        self,
        tzid: Option<&TimeZoneIdentifier>,
    ) -> Self {
        match (self, tzid) {
            (Self::DateTime(dt), Some(tzid)) => {
                Self::DateTime(dt.resolve_tz(tzid.tz()))
            }
            (value, _) => value,
        }
    }
}

/// Convenience union of [`Date`], [`DateTime`], and [`Period`] used by
/// properties that accept any of those three value types (e.g., `FREEBUSY`).
#[derive(Debug)]
pub enum DateTimePeriod {
    /// A calendar date without a time component.
    Date(Date),
    /// A precise calendar date and time of day.
    DateTime(DateTime),
    /// A span of time defined by start/end or start/duration.
    Period(Period),
}

impl TryFrom<&[u8]> for DateTimePeriod {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        // PERIOD always contains a "/" separator; DATE-TIME contains "T";
        // DATE is bare digits. Unambiguous by shape alone.
        if v.contains(&b'/') {
            Ok(Self::Period(v.try_into()?))
        } else if v.contains(&b'T') {
            Ok(Self::DateTime(v.try_into()?))
        } else {
            Ok(Self::Date(v.try_into()?))
        }
    }
}

impl DateTimePeriod {
    /// Resolves any `DateTime` component (bare, or as a `Period`'s start/
    /// end) against `tzid`'s real offset rules (RFC 5545 §3.3.5) when a
    /// `TZID` parameter is present. A `Date` value is returned unchanged.
    pub(crate) fn resolve_tzid(
        self,
        tzid: Option<&TimeZoneIdentifier>,
    ) -> Self {
        match (self, tzid) {
            (Self::DateTime(dt), Some(tzid)) => {
                Self::DateTime(dt.resolve_tz(tzid.tz()))
            }
            (Self::Period(period), Some(tzid)) => {
                Self::Period(period.resolve_tzid(tzid))
            }
            (value, _) => value,
        }
    }
}

/// Convenience union of [`Duration`] and [`DateTime`] used by properties
/// that accept either value type (e.g., `TRIGGER`).
#[derive(Debug)]
pub enum DateTimeDuration {
    /// A positive span of time.
    Duration(Duration),
    /// A precise calendar date and time of day.
    DateTime(DateTime),
}

impl TryFrom<&[u8]> for DateTimeDuration {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        if is_duration_shaped(v) {
            Ok(Self::Duration(v.try_into()?))
        } else {
            Ok(Self::DateTime(v.try_into()?))
        }
    }
}

/// If the property permits, multiple "duration" values are
/// specified by a COMMA-separated list of values.  The format is
/// based on the [ISO.8601.2004] complete representation basic format
/// with designators for the duration of time.  The format can
/// represent nominal durations (weeks and days) and accurate
/// durations (hours, minutes, and seconds).  Note that unlike
/// [ISO.8601.2004], this value type doesn't support the "Y" and "M"
/// designators to specify durations in terms of years and months.
///
/// The duration of a week or a day depends on its position in the
/// calendar.  In the case of discontinuities in the time scale, such
/// as the change from standard time to daylight time and back, the
/// computation of the exact duration requires the subtraction or
/// addition of the change of duration of the discontinuity.  Leap
/// seconds MUST NOT be considered when computing an exact duration.
/// When computing an exact duration, the greatest order time
/// components MUST be added first, that is, the number of days MUST
/// be added first, followed by the number of hours, number of
/// minutes, and number of seconds.
/// Negative durations are typically used to schedule an alarm to
/// trigger before an associated time (see Section 3.8.6.3).
///
/// Example:  A duration of 15 days, 5 hours, and 20 seconds would be:
///
/// > P15DT5H0M20S
///
/// A duration of 7 weeks would be:
///
/// > P7W
///
/// [Section 3.3.6](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.6)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration(ChronoDuration);

impl Deref for Duration {
    type Target = ChronoDuration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&[u8]> for Duration {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let str = from_utf8(v)?;
        let (negative, str) = match str.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, str.strip_prefix('+').unwrap_or(str)),
        };
        let str = str.strip_prefix('P').ok_or(ValueError::Duration)?;

        let total = if let Some(weeks) = str.strip_suffix('W') {
            ChronoDuration::try_weeks(
                weeks.parse().map_err(|_| ValueError::Duration)?,
            )
            .ok_or(ValueError::Duration)?
        } else {
            let (date_part, time_part) = match str.split_once('T') {
                Some((d, t)) => (d, Some(t)),
                None => (str, None),
            };

            let mut total = ChronoDuration::seconds(0);
            let mut rest = date_part;
            if let Some(idx) = rest.find('D') {
                let days = ChronoDuration::try_days(
                    rest[..idx].parse().map_err(|_| ValueError::Duration)?,
                )
                .ok_or(ValueError::Duration)?;
                total = total.checked_add(&days).ok_or(ValueError::Duration)?;
                rest = &rest[idx + 1..];
            }
            if !rest.is_empty() {
                return Err(ValueError::Duration);
            }

            if let Some(mut rest) = time_part {
                if let Some(idx) = rest.find('H') {
                    let hours = ChronoDuration::try_hours(
                        rest[..idx]
                            .parse()
                            .map_err(|_| ValueError::Duration)?,
                    )
                    .ok_or(ValueError::Duration)?;
                    total = total
                        .checked_add(&hours)
                        .ok_or(ValueError::Duration)?;
                    rest = &rest[idx + 1..];
                }
                if let Some(idx) = rest.find('M') {
                    let minutes = ChronoDuration::try_minutes(
                        rest[..idx]
                            .parse()
                            .map_err(|_| ValueError::Duration)?,
                    )
                    .ok_or(ValueError::Duration)?;
                    total = total
                        .checked_add(&minutes)
                        .ok_or(ValueError::Duration)?;
                    rest = &rest[idx + 1..];
                }
                if let Some(idx) = rest.find('S') {
                    let seconds = ChronoDuration::try_seconds(
                        rest[..idx]
                            .parse()
                            .map_err(|_| ValueError::Duration)?,
                    )
                    .ok_or(ValueError::Duration)?;
                    total = total
                        .checked_add(&seconds)
                        .ok_or(ValueError::Duration)?;
                    rest = &rest[idx + 1..];
                }
                if !rest.is_empty() {
                    return Err(ValueError::Duration);
                }
            }

            total
        };

        Ok(Self(if negative { -total } else { total }))
    }
}

/// If the property permits, multiple "DATE-TIME" values
/// are specified as a COMMA-separated list of values.  No additional
/// content value encoding (i.e., BACKSLASH character encoding, see
/// Section 3.3.11) is defined for this value type.
///
/// The "DATE-TIME" value type is used to identify values that contain
/// a precise calendar date and time of day.  The format is based on
/// the [ISO.8601.2004] complete representation, basic format for a
/// calendar date and time of day.  The text format is a concatenation
/// of the "date", followed by the LATIN CAPITAL LETTER T character,
/// the time designator, followed by the "time" format.
///
/// The "DATE-TIME" value type expresses time values in three forms:
/// The form of date and time with UTC offset MUST NOT be used.  For
/// example, the following is not valid for a DATE-TIME value:
///
/// > 19980119T230000-0800       ;Invalid time format
///
/// ## FORM #1: DATE WITH LOCAL TIME
///
/// The date with local time form is simply a DATE-TIME value that
/// does not contain the UTC designator nor does it reference a time
/// zone.  For example, the following represents January 18, 1998, at
/// 11 PM:
///
/// > 19980118T230000
///
/// DATE-TIME values of this type are said to be "floating" and are
/// not bound to any time zone in particular.  They are used to
/// represent the same hour, minute, and second value regardless of
/// which time zone is currently being observed.  For example, an
/// event can be defined that indicates that an individual will be
/// busy from 11:00 AM to 1:00 PM every day, no matter which time zone
/// the person is in.  In these cases, a local time can be specified.
/// The recipient of an iCalendar object with a property value
/// consisting of a local time, without any relative time zone
/// information, SHOULD interpret the value as being fixed to whatever
/// time zone the "ATTENDEE" is in at any given moment.  This means
/// that two "Attendees", in different time zones, receiving the same
/// event definition as a floating time, may be participating in the
/// event at different actual times.  Floating time SHOULD only be
/// used where that is the reasonable behavior.
///
/// In most cases, a fixed time is desired.  To properly communicate a
/// fixed time in a property value, either UTC time or local time with
/// time zone reference MUST be specified.
///
/// The use of local time in a DATE-TIME value without the "TZID"
/// property parameter is to be interpreted as floating time,
/// regardless of the existence of "VTIMEZONE" calendar components in
/// the iCalendar object.
///
/// ## FORM #2: DATE WITH UTC TIME
///
/// The date with UTC time, or absolute time, is identified by a LATIN
/// CAPITAL LETTER Z suffix character, the UTC designator, appended to
/// the time value.  For example, the following represents January 19,
/// 1998, at 0700 UTC:
///
/// > 19980119T070000Z
///
/// The "TZID" property parameter MUST NOT be applied to DATE-TIME
/// properties whose time values are specified in UTC.
/// FORM #3: DATE WITH LOCAL TIME AND TIME ZONE REFERENCE
/// The date and local time with reference to time zone information is
/// identified by the use the "TZID" property parameter to reference
/// the appropriate time zone definition.  "TZID" is discussed in
/// detail in Section 3.2.19.  For example, the following represents
/// 2:00 A.M. in New York on January 19, 1998:
///
/// > TZID=America/New_York:19980119T020000
///
/// If, based on the definition of the referenced time zone, the local
/// time described occurs more than once (when changing from daylight
/// to standard time), the DATE-TIME value refers to the first
/// occurrence of the referenced time.  Thus, TZID=America/
/// New_York:20071104T013000 indicates November 4, 2007 at 1:30 A.M.
/// EDT (UTC-04:00).  If the local time described does not occur (when
/// changing from standard to daylight time), the DATE-TIME value is
/// interpreted using the UTC offset before the gap in local times.
/// Thus, TZID=America/New_York:20070311T023000 indicates March 11,
/// 2007 at 3:30 A.M. EDT (UTC-04:00), one hour after 1:30 A.M. EST
/// (UTC-05:00).
///
/// A time value MUST only specify the second 60 when specifying a
/// positive leap second.  For example:
///
/// > 19970630T235960Z
///
/// Implementations that do not support leap seconds SHOULD interpret
/// the second 60 as equivalent to the second 59.
///
/// Example:  The following represents July 14, 1997, at 1:30 PM in New
/// York City in each of the three time formats, using the "DTSTART"
/// property.
///
/// > DTSTART:19970714T133000                   ; Local time
/// > DTSTART:19970714T173000Z                  ; UTC time
/// > DTSTART;TZID=America/New_York:19970714T133000 ; Local date and time ; zone
/// > reference
///
/// [Section 3.3.5](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.5)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTime {
    /// An absolute instant: either given directly in UTC (form #2, trailing
    /// `Z`), or a local time already resolved against its `TZID`'s real
    /// offset rules via [`resolve_tz`](DateTime::resolve_tz) (form #3).
    Utc(ChronoDateTime<Utc>),
    /// A "floating" local time (form #1): no `TZID`, no trailing `Z`. Kept
    /// as wall-clock text rather than resolved through any implicit time
    /// zone — in particular, never the host machine's, which is what
    /// floating time is defined to be independent of.
    Floating(NaiveDateTime),
}

impl TryFrom<&[u8]> for DateTime {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let str = from_utf8(v)?;
        if let Some(stripped) = str.strip_suffix('Z') {
            let naive =
                NaiveDateTime::parse_from_str(stripped, ICAL_DATETIME_FMT)?;
            return Ok(Self::Utc(Utc.from_utc_datetime(&naive)));
        }

        let naive = NaiveDateTime::parse_from_str(str, ICAL_DATETIME_FMT)?;
        Ok(Self::Floating(naive))
    }
}

impl DateTime {
    /// Resolves a floating local time against `tz`'s real offset rules
    /// (RFC 5545 §3.3.5, form #3). A value already anchored to UTC is
    /// returned unchanged — the `TZID` parameter "MUST NOT be applied" to
    /// a `DATE-TIME` whose value is already UTC (RFC 5545 §3.2.19).
    ///
    /// Per the RFC: if the local time is ambiguous (a DST fall-back fold),
    /// it "refers to the first occurrence" — the earlier of the two
    /// instants. If the local time does not exist (a DST spring-forward
    /// gap), it's "interpreted using the UTC offset before the gap".
    pub(crate) fn resolve_tz(self, tz: Tz) -> Self {
        let Self::Floating(naive) = self else {
            return self;
        };
        let utc = match tz.from_local_datetime(&naive) {
            MappedLocalTime::Single(dt) => dt.to_utc(),
            MappedLocalTime::Ambiguous(earliest, _latest) => earliest.to_utc(),
            MappedLocalTime::None => {
                let offset = offset_before_gap(tz, naive);
                offset
                    .from_local_datetime(&naive)
                    .single()
                    .expect("a fixed offset always resolves a local time")
                    .to_utc()
            }
        };
        Self::Utc(utc)
    }
}

/// The UTC offset in effect immediately before a DST "spring-forward" gap
/// that swallowed `naive` in `tz` — i.e. the offset RFC 5545 §3.3.5 says to
/// use for a local time that doesn't exist. Real-world DST gaps are at
/// most a couple of hours and transitions are at minimum months apart, so
/// probing a day earlier lands safely before the gap and before any other
/// transition.
fn offset_before_gap(tz: Tz, naive: NaiveDateTime) -> FixedOffset {
    let probe = naive - ChronoDuration::days(1);
    match tz.offset_from_local_datetime(&probe) {
        MappedLocalTime::Single(offset) => offset.fix(),
        // Pathological: fall back to a fixed zero offset rather than loop
        // or panic on adversarial input (e.g. the fuzz corpus).
        _ => FixedOffset::east_opt(0).expect("zero is a valid UTC offset"),
    }
}

/// If the property permits, multiple "date" values are
/// specified as a COMMA-separated list of values.  The format for the
/// value type is based on the [ISO.8601.2004] complete
/// representation, basic format for a calendar date.  The textual
/// format specifies a four-digit year, two-digit month, and two-digit
/// day of the month.  There are no separator characters between the
/// year, month, and day component text.
///
/// Example:  The following represents July 14, 1997:
///
/// > 19970714
///
/// [Section 3.3.4](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.4)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date(NaiveDate);

impl Deref for Date {
    type Target = NaiveDate;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&[u8]> for Date {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let str = from_utf8(v)?;
        Ok(Self(NaiveDate::parse_from_str(str, ICAL_DATE_FMT)?))
    }
}

/// The PLUS SIGN character MUST be specified for positive
/// UTC offsets (i.e., ahead of UTC).  The HYPHEN-MINUS character MUST
/// be specified for negative UTC offsets (i.e., behind of UTC).  The
/// value of "-0000" and "-000000" are not allowed.  The time-second,
/// if present, MUST NOT be 60; if absent, it defaults to zero.
/// No additional content value encoding (i.e., BACKSLASH character
/// encoding, see Section 3.3.11) is defined for this value type.
/// Example:  The following UTC offsets are given for standard time for
/// New York (five hours behind UTC) and Geneva (one hour ahead of
/// UTC):
///
/// > -0500
/// >
/// > +0100
///
/// [Section 3.3.14](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.14)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UtcOffset(FixedOffset);

impl Deref for UtcOffset {
    type Target = FixedOffset;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&[u8]> for UtcOffset {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let str = from_utf8(v)?;
        let (sign, rest) = match str.as_bytes().first() {
            Some(b'+') => (1, &str[1..]),
            Some(b'-') => (-1, &str[1..]),
            _ => return Err(ValueError::UtcOffset),
        };
        if rest.len() != 4 && rest.len() != 6 {
            return Err(ValueError::UtcOffset);
        }
        if !rest.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ValueError::UtcOffset);
        }
        let hour: i32 =
            rest[0..2].parse().map_err(|_| ValueError::UtcOffset)?;
        let minute: i32 =
            rest[2..4].parse().map_err(|_| ValueError::UtcOffset)?;
        let second: i32 = if rest.len() == 6 {
            rest[4..6].parse().map_err(|_| ValueError::UtcOffset)?
        } else {
            0
        };
        let total = sign * (hour * 3600 + minute * 60 + second);
        // "-0000" and "-000000" are not allowed.
        if sign == -1 && total == 0 {
            return Err(ValueError::UtcOffset);
        }
        FixedOffset::east_opt(total)
            .map(Self)
            .ok_or(ValueError::UtcOffset)
    }
}

/// If the property permits, multiple "period" values are
/// specified by a COMMA-separated list of values.  There are two
/// forms of a period of time.  First, a period of time is identified
/// by its start and its end.  This format is based on the
/// [ISO.8601.2004](https://datatracker.ietf.org/doc/html/rfc5545#ref-ISO.8601.2004) complete representation, basic format for "DATE-
/// TIME" start of the period, followed by a SOLIDUS character
/// followed by the "DATE-TIME" of the end of the period.  The start
/// of the period MUST be before the end of the period.  Second, a
/// period of time can also be defined by a start and a positive
/// duration of time.  The format is based on the [ISO.8601.2004](https://datatracker.ietf.org/doc/html/rfc5545#ref-ISO.8601.2004)
/// complete representation, basic format for the "DATE-TIME" start of
/// the period, followed by a SOLIDUS character, followed by the
/// [ISO.8601.2004](https://datatracker.ietf.org/doc/html/rfc5545#ref-ISO.8601.2004) basic format for "DURATION" of the period.
///
/// [Section 3.3.9](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.9)
#[derive(Debug)]
pub enum Period {
    /// Period defined by an explicit start and end date-time.
    StartEnd {
        /// Inclusive start of the period.
        start: DateTime,
        /// Exclusive end of the period; MUST be after `start`.
        end: DateTime,
    },
    /// Period defined by a start date-time and a positive duration.
    Duration {
        /// Start of the period.
        start: DateTime,
        /// Length of the period.
        duration: Duration,
    },
}

impl TryFrom<&[u8]> for Period {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let (start_b, rest) =
            split_once(v, b'/').ok_or_else(|| ValueError::Malformed {
                expected: "start/end or start/duration".into(),
                received: std::str::from_utf8(v).ok().map(Into::into),
            })?;
        let start = DateTime::try_from(start_b)?;

        if is_duration_shaped(rest) {
            Ok(Self::Duration {
                start,
                duration: rest.try_into()?,
            })
        } else {
            Ok(Self::StartEnd {
                start,
                end: rest.try_into()?,
            })
        }
    }
}

impl Period {
    /// Resolves this period's `start`/`end` (or `start`) against `tzid`'s
    /// real offset rules (RFC 5545 §3.3.5).
    pub(crate) fn resolve_tzid(self, tzid: &TimeZoneIdentifier) -> Self {
        let tz = tzid.tz();
        match self {
            Self::StartEnd { start, end } => Self::StartEnd {
                start: start.resolve_tz(tz),
                end: end.resolve_tz(tz),
            },
            Self::Duration { start, duration } => Self::Duration {
                start: start.resolve_tz(tz),
                duration,
            },
        }
    }
}

/// Whether `v` looks like a `dur-value` (optionally signed, `P`-prefixed)
/// rather than a `date-time`.
fn is_duration_shaped(v: &[u8]) -> bool {
    match v.first() {
        Some(b'P') => true,
        Some(b'+') | Some(b'-') => v.get(1) == Some(&b'P'),
        _ => false,
    }
}

/// If the property permits, multiple "time" values are
/// specified by a COMMA-separated list of values.  No additional
/// content value encoding (i.e., BACKSLASH character encoding, see
/// Section 3.3.11) is defined for this value type.
///
/// The "TIME" value type is used to identify values that contain a
/// time of day.  The format is based on the [ISO.8601.2004] complete
/// representation, basic format for a time of day.  The text format
/// consists of a two-digit, 24-hour of the day (i.e., values 00-23),
/// two-digit minute in the hour (i.e., values 00-59), and two-digit
/// seconds in the minute (i.e., values 00-60).  The seconds value of
/// 60 MUST only be used to account for positive "leap" seconds.
/// Fractions of a second are not supported by this format.
///
/// ### In parallel to the "DATE-TIME" definition above, the "TIME" value
/// type expresses time values in three forms:
///
/// The form of time with UTC offset MUST NOT be used.  For example,
/// the following is not valid for a time value:
///
/// > 230000-0800        ;Invalid time format
///
/// ##   FORM #1 LOCAL TIME
///
/// The local time form is simply a time value that does not contain
/// the UTC designator nor does it reference a time zone.  For
/// example, 11:00 PM:
///
/// > 230000
///
/// Time values of this type are said to be "floating" and are not
/// bound to any time zone in particular.  They are used to represent
/// the same hour, minute, and second value regardless of which time
/// zone is currently being observed.  For example, an event can be
/// defined that indicates that an individual will be busy from 11:00
/// AM to 1:00 PM every day, no matter which time zone the person is
/// in.  In these cases, a local time can be specified.  The recipient
/// of an iCalendar object with a property value consisting of a local
/// time, without any relative time zone information, SHOULD interpret
/// the value as being fixed to whatever time zone the "ATTENDEE" is
/// in at any given moment.  This means that two "Attendees", may
/// participate in the same event at different UTC times; floating
/// time SHOULD only be used where that is reasonable behavior.
///
/// In most cases, a fixed time is desired.  To properly communicate
/// a fixed time in a property value, either UTC time or local time
/// with time zone reference MUST be specified.
///
/// The use of local time in a TIME value without the "TZID"
/// property parameter is to be interpreted as floating time,
/// regardless of the existence of "VTIMEZONE" calendar components
/// in the iCalendar object.
///
/// ## FORM #2: UTC TIME
///
/// UTC time, or absolute time, is identified by a LATIN CAPITAL
/// LETTER Z suffix character, the UTC designator, appended to the
/// time value.  For example, the following represents 07:00 AM UTC:
///
/// > 070000Z
///
/// The "TZID" property parameter MUST NOT be applied to TIME
/// properties whose time values are specified in UTC.
///
/// ## FORM #3: LOCAL TIME AND TIME ZONE REFERENCE
///
/// The local time with reference to time zone information form is
/// identified by the use the "TZID" property parameter to reference
/// the appropriate time zone definition.  "TZID" is discussed in
/// detail in Section 3.2.19.
///
/// Example:  The following represents 8:30 AM in New York in winter,
/// five hours behind UTC, in each of the three formats:
///
/// > 083000
/// >
/// > 133000Z
/// >
/// > TZID=America/New_York:083000
///
/// [Section 3.3.12](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.12)
#[derive(Debug)]
pub enum Time {
    /// Local (floating) time — not bound to any time zone.
    Floating(NaiveTime),
    /// Local time anchored to a specific time zone via a TZID reference.
    Zoned {
        /// The clock time.
        time: NaiveTime,
        /// The VTIMEZONE identifier that gives the offset context.
        tzid: TimeZoneIdentifier,
    },
}

/// Description:  Property values with this value type MUST also include
/// the inline encoding parameter sequence of ";ENCODING=BASE64".
/// That is, all inline binary data MUST first be character encoded
/// using the "BASE64" encoding method defined in [RFC2045](https://datatracker.ietf.org/doc/html/rfc2045).  No
/// additional content value encoding (i.e., BACKSLASH character
/// encoding, see Section [3.3.11](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.11)) is defined for this value type.
/// Example: The following is an example of a "BASE64" encoded binary
/// value data:
/// > ATTACH;FMTTYPE=image/vnd.microsoft.icon;ENCODING=BASE64;VALUE
/// > =BINARY:AAABAAEAEBAQAAEABAAoAQAAFgAAACgAAAAQAAAAIAAAAAEABAAA
/// > AAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAgIAAAICAgADAwMAA////AAAA
/// > AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
/// > AAAAAAAAAAAAAAAAAAAAAAMwAAAAAAABNEMQAAAAAAAkQgAAAAAAJEREQgAA
/// > ACECQ0QgEgAAQxQzM0E0AABERCRCREQAADRDJEJEQwAAAhA0QwEQAAAAAERE
/// > AAAAAAAAREQAAAAAAAAkQgAAAAAAAAMgAAAAAAAAAAAAAAAAAAAAAAAAAAAA
/// > AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
/// > AAAAAAAAAAAA
///
/// [Section 3.3.1](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.1)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binary(Vec<u8>);

impl Deref for Binary {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&[u8]> for Binary {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(base64::engine::general_purpose::STANDARD.decode(v)?))
    }
}

/// These values are case-insensitive text.  No additional
/// content value encoding (i.e., BACKSLASH character encoding, see
/// [Section 3.3.11](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.11))
/// is defined for this value type.
///
/// [Section 3.3.2](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.2)
#[derive(Debug)]
pub struct Boolean(bool);

/// If the property permits, multiple TEXT values are
/// specified by a COMMA-separated list of values.
///
/// The language in which the text is represented can be controlled by
/// the "LANGUAGE" property parameter.
///
/// An intentional formatted text line break MUST only be included in
/// a "TEXT" property value by representing the line break with the
/// character sequence of BACKSLASH, followed by a LATIN SMALL LETTER
/// N or a LATIN CAPITAL LETTER N, that is "\n" or "\N".
///
/// The "TEXT" property values may also contain special characters
/// that are used to signify delimiters, such as a COMMA character for
/// lists of values or a SEMICOLON character for structured values.
/// In order to support the inclusion of these special characters in
/// "TEXT" property values, they MUST be escaped with a BACKSLASH
/// character.  A BACKSLASH character in a "TEXT" property value MUST
/// be escaped with another BACKSLASH character.  A COMMA character in
/// a "TEXT" property value MUST be escaped with a BACKSLASH
/// character.  A SEMICOLON character in a "TEXT" property value MUST
/// be escaped with a BACKSLASH character.  However, a COLON character
/// in a "TEXT" property value SHALL NOT be escaped with a BACKSLASH
/// character.
///
/// Example:  A multiple line value of:
///
/// > Project XYZ Final Review
/// >
/// > Conference Room - 3B
/// >
/// > Come Prepared.
///
/// would be represented as:
///
/// > Project XYZ Final Review\nConference Room - 3B\nCome Prepared.
///
/// [Section 3.3.11](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.11)
#[derive(Debug, Default)]
pub struct Text(String);

/// This value type might be used to reference binary
/// information, for values that are large, or otherwise undesirable
/// to include directly in the iCalendar object.
/// Property values with this value type MUST follow the generic URI
/// syntax defined in [RFC3986](https://datatracker.ietf.org/doc/html/rfc3986).
///
/// When a property parameter value is a URI value type, the URI MUST
/// be specified as a quoted-string value.
///
/// No additional content value encoding (i.e., BACKSLASH character
/// encoding, see [Section 3.3.11](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.11))
/// is defined for this value type.
///
/// Example:  The following is a URI for a network file:
/// > http://example.com/my-report.txt
///
/// [Section 3.3.13](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.13)
#[derive(Debug)]
pub struct Uri(Url);

/// If the property permits, multiple "integer" values are
/// specified by a COMMA-separated list of values.  The valid range
/// for "integer" is -2147483648 to 2147483647.  If the sign is not
/// specified, then the value is assumed to be positive.
///
/// [Section 3.3.8](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.8)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Integer(i32);

impl Deref for Integer {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&[u8]> for Integer {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(from_utf8(v)?.parse()?))
    }
}

/// If the property permits, multiple "float" values are
/// specified by a COMMA-separated list of values.
///
/// [Section 3.3.7](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.7)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Float(f64);

impl Deref for Float {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&[u8]> for Float {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(from_utf8(v)?.parse()?))
    }
}
/// The value is a URI as defined by [RFC3986] or any other
/// IANA-registered form for a URI.  When used to address an Internet
/// email transport address for a calendar user, the value MUST be a
/// mailto URI, as defined by [RFC2368].  No additional content value
/// encoding (i.e., BACKSLASH character encoding, see Section 3.3.11)
/// is defined for this value type.
///
/// Example:
///
/// > mailto:jane_doe@example.com
///
/// [Section 3.3.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.3)
#[derive(Debug)]
pub struct CalendarUserAddress(Uri);

mod recurrence {
    use crate::{params::ParamError, values::ValueError};

    use super::DateOrDatetime;
    use std::str::from_utf8;

    /// Enforces 0 to 60
    #[derive(Debug, Clone)]
    struct Seconds(u8);
    /// 0 to 59
    #[derive(Debug, Clone)]
    struct Minutes(u8);
    /// 0 to 23
    #[derive(Debug, Clone)]
    struct Hour(u8);
    #[derive(Debug, Clone)]
    struct WeekNum(i8);

    #[derive(Debug, Clone)]
    struct WeekdayNum {
        ordinal: Option<i8>,
        weekday: Weekday,
    }
    /// 1 to 12
    #[derive(Debug, Clone)]
    struct MonthNum(u8);
    #[derive(Debug, Clone)]
    struct MonthDayNum(i8);
    #[derive(Debug, Clone)]
    struct YearDayNum(i16);
    type SetPosDay = YearDayNum;

    /// Day of the week
    #[derive(Debug, Clone, Default)]
    enum Weekday {
        Su,
        #[default]
        Mo,
        Tu,
        We,
        Th,
        Fr,
        Sa,
    }

    /// This value type is a structured value consisting of a
    /// list of one or more recurrence grammar parts.  Each rule part is
    /// defined by a NAME=VALUE pair.  The rule parts are separated from
    /// each other by the SEMICOLON character.  The rule parts are not
    /// ordered in any particular sequence.  Individual rule parts MUST
    /// only be specified once.  Compliant applications MUST accept rule
    /// parts ordered in any sequence, but to ensure backward
    /// compatibility with applications that pre-date this revision of
    /// iCalendar the FREQ rule part MUST be the first rule part specified
    /// in a RECUR value.
    ///
    /// Recurrence rules may generate recurrence instances with an invalid
    /// date (e.g., February 30) or nonexistent local time (e.g., 1:30 AM
    /// on a day where the local time is moved forward by an hour at 1:00
    /// AM).  Such recurrence instances MUST be ignored and MUST NOT be
    /// counted as part of the recurrence set.
    ///
    /// Information, not contained in the rule, necessary to determine the
    /// various recurrence instance start time and dates are derived from
    /// the Start Time ("DTSTART") component attribute.  For example,
    /// "FREQ=YEARLY;BYMONTH=1" doesn't specify a specific day within the
    /// month or a time.  This information would be the same as what is
    /// specified for "DTSTART".
    ///
    /// [More](https://datatracker.ietf.org/doc/html/rfc5545#autoid-42)
    #[derive(Debug, Clone, Default)]
    pub struct Recur {
        /// The FREQ rule part identifies the type of recurrence rule. This
        /// rule part MUST be specified in the recurrence rule.  Valid values
        /// include SECONDLY, to specify repeating events based on an interval
        /// of a second or more; MINUTELY, to specify repeating events based
        /// on an interval of a minute or more; HOURLY, to specify repeating
        /// events based on an interval of an hour or more; DAILY, to specify
        /// repeating events based on an interval of a day or more; WEEKLY, to
        /// specify repeating events based on an interval of a week or more;
        /// MONTHLY, to specify repeating events based on an interval of a
        /// month or more; and YEARLY, to specify repeating events based on an
        /// interval of a year or more.
        freq: Frequency,
        /// The UNTIL rule part defines a DATE or DATE-TIME value that bounds
        /// the recurrence rule in an inclusive manner.  If the value
        /// specified by UNTIL is synchronized with the specified recurrence,
        /// this DATE or DATE-TIME becomes the last instance of the
        /// recurrence.  The value of the UNTIL rule part MUST have the same
        /// value type as the "DTSTART" property.  Furthermore, if the
        /// "DTSTART" property is specified as a date with local time, then
        /// the UNTIL rule part MUST also be specified as a date with local
        /// time.  If the "DTSTART" property is specified as a date with UTC
        /// time or a date with local time and time zone reference, then the
        /// UNTIL rule part MUST be specified as a date with UTC time.  In the
        /// case of the "STANDARD" and "DAYLIGHT" sub-components the UNTIL
        /// rule part MUST always be specified as a date with UTC time.  If
        /// specified as a DATE-TIME value, then it MUST be specified in a UTC
        /// time format.  If not present, and the COUNT rule part is also not
        /// present, the "RRULE" is considered to repeat forever.
        until: Option<DateOrDatetime>,
        /// The COUNT rule part defines the number of occurrences at which to
        /// range-bound the recurrence.  The "DTSTART" property value always
        /// counts as the first occurrence.
        count: Option<i32>,
        /// The INTERVAL rule part contains a positive integer representing at
        /// which intervals the recurrence rule repeats.  The default value is
        /// "1", meaning every second for a SECONDLY rule, every minute for a
        /// MINUTELY rule, every hour for an HOURLY rule, every day for a
        /// DAILY rule, every week for a WEEKLY rule, every month for a
        /// MONTHLY rule, and every year for a YEARLY rule.  For example,
        /// within a DAILY rule, a value of "8" means every eight days.
        interval: Option<i32>,
        /// The BYSECOND rule part specifies a COMMA-separated list of seconds
        /// a minute.  Valid values are 0 to 60.  The BYMINUTE rule
        /// specifies a COMMA-separated list of minutes within an hour.
        /// values are 0 to 59.  The BYHOUR rule part specifies a COMMA-
        /// list of hours of the day.  Valid values are 0 to 23.
        /// BYSECOND, BYMINUTE and BYHOUR rule parts MUST NOT be specified
        /// the associated "DTSTART" property has a DATE value type.
        /// rule parts MUST be ignored in RECUR value that violate the
        /// requirement (e.g., generated by applications that pre-date
        /// revision of iCalendar).
        by_second: Vec<Seconds>,
        by_minute: Vec<Minutes>,
        by_hour: Vec<Hour>,
        /// The BYDAY rule part specifies a COMMA-separated list of days of
        /// week; SU indicates Sunday; MO indicates Monday; TU indicates
        /// Tuesday; WE indicates Wednesday; TH indicates Thursday; FR
        /// Friday; and SA indicates Saturday.
        ///
        /// Each BYDAY value can also be preceded by a positive (+n) or
        /// negative (-n) integer.  If present, this indicates the nth
        /// occurrence of a specific day within the MONTHLY or YEARLY "RRULE".
        ///
        /// For example, within a MONTHLY rule, +1MO (or simply 1MO)
        /// represents the first Monday within the month, whereas -1MO
        /// represents the last Monday of the month.  The numeric value in a
        /// BYDAY rule part with the FREQ rule part set to YEARLY corresponds
        /// to an offset within the month when the BYMONTH rule part is
        /// present, and corresponds to an offset within the year when the
        /// BYWEEKNO or BYMONTH rule parts are present.  If an integer
        /// modifier is not present, it means all days of this type within the
        /// specified frequency.  For example, within a MONTHLY rule, MO
        /// represents all Mondays within the month.  The BYDAY rule part MUST
        /// NOT be specified with a numeric value when the FREQ rule part is
        /// not set to MONTHLY or YEARLY.  Furthermore, the BYDAY rule part
        /// MUST NOT be specified with a numeric value with the FREQ rule part
        /// set to YEARLY when the BYWEEKNO rule part is specified.
        by_day: Vec<WeekdayNum>,
        /// The BYMONTHDAY rule part specifies a COMMA-separated list of days
        /// of the month.  Valid values are 1 to 31 or -31 to -1.  For
        /// example, -10 represents the tenth to the last day of the month.
        /// The BYMONTHDAY rule part MUST NOT be specified when the FREQ rule
        /// part is set to WEEKLY.
        by_month_day: Vec<MonthDayNum>,
        /// The BYYEARDAY rule part specifies a COMMA-separated list of days
        /// of the year.  Valid values are 1 to 366 or -366 to -1.  For
        /// example, -1 represents the last day of the year (December 31st)
        /// and -306 represents the 306th to the last day of the year (March
        /// 1st).  The BYYEARDAY rule part MUST NOT be specified when the FREQ
        /// rule part is set to DAILY, WEEKLY, or MONTHLY.
        by_year_day: Vec<YearDayNum>,
        /// The BYWEEKNO rule part specifies a COMMA-separated list of
        /// ordinals specifying weeks of the year.  Valid values are 1 to 53
        /// or -53 to -1.  This corresponds to weeks according to week
        /// numbering as defined in [ISO.8601.2004].  A week is defined as a
        /// seven day period, starting on the day of the week defined to be
        /// the week start (see WKST).  Week number one of the calendar year
        /// is the first week that contains at least four (4) days in that
        /// calendar year.  This rule part MUST NOT be used when the FREQ rule
        /// part is set to anything other than YEARLY.  For example, 3
        /// represents the third week of the year.
        ///
        /// > Note: Assuming a Monday week start, week 53 can only occur when
        /// > Thursday is January 1 or if it is a leap year and Wednesday is
        /// > January 1.
        by_week_no: Vec<WeekNum>,
        /// The BYMONTH rule part specifies a COMMA-separated list of months
        /// of the year.  Valid values are 1 to 12.
        by_month: Vec<MonthNum>,

        /// The BYSETPOS rule part specifies a COMMA-separated list of values
        /// that corresponds to the nth occurrence within the set of
        /// recurrence instances specified by the rule.  BYSETPOS operates on
        /// a set of recurrence instances in one interval of the recurrence
        /// rule.  For example, in a WEEKLY rule, the interval would be one
        /// week A set of recurrence instances starts at the beginning of the
        /// interval defined by the FREQ rule part.  Valid values are 1 to 366
        /// or -366 to -1.  It MUST only be used in conjunction with another
        /// BYxxx rule part.  For example "the last work day of the month"
        /// could be represented as:
        ///
        /// FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1
        by_set_pos: Vec<SetPosDay>,
        /// The WKST rule part specifies the day on which the workweek starts.
        /// Valid values are MO, TU, WE, TH, FR, SA, and SU.  This is
        /// significant when a WEEKLY "RRULE" has an interval greater
        /// than 1, and a BYDAY rule part is specified. This is also
        /// significant when in a YEARLY "RRULE" when a BYWEEKNO rule
        /// part is specified. The default value is MO.
        wkst: Option<Weekday>,
    }

    impl Recur {
        /// The `UNTIL` rule part, if present — used by component builders to
        /// cross-check its value type (DATE vs DATE-TIME) against the
        /// enclosing property's `DTSTART` (RFC 5545 §3.3.10).
        pub(crate) fn until(&self) -> Option<&DateOrDatetime> {
            self.until.as_ref()
        }
    }

    /// The FREQ rule part identifies the type of recurrence rule. This
    /// rule part MUST be specified in the recurrence rule.  Valid values
    /// include SECONDLY, to specify repeating events based on an interval
    /// of a second or more; MINUTELY, to specify repeating events based
    /// on an interval of a minute or more; HOURLY, to specify repeating
    /// events based on an interval of an hour or more; DAILY, to specify
    /// repeating events based on an interval of a day or more; WEEKLY, to
    /// specify repeating events based on an interval of a week or more;
    /// MONTHLY, to specify repeating events based on an interval of a
    /// month or more; and YEARLY, to specify repeating events based on an
    /// interval of a year or more.
    #[derive(Debug, Clone, Default)]
    enum Frequency {
        Secondly,
        Minutely,
        Hourly,
        /// Every N days.
        Daily,
        /// Every N weeks.
        #[default]
        Weekly,
        /// Every N months.
        Monthly,
        /// Every N years.
        Yearly,
    }

    /// Builds a [`ValueError::Malformed`] for a malformed `RECUR` sub-part.
    fn recur_err(expected: &str, received: &str) -> ValueError {
        ValueError::Malformed {
            expected: expected.into(),
            received: Some(received.into()),
        }
    }

    impl TryFrom<&[u8]> for Frequency {
        type Error = ParamError;

        fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
            let r = match v {
                b"SECONDLY" => Self::Secondly,
                b"MINUTELY" => Self::Minutely,
                b"HOURLY" => Self::Hourly,
                b"DAILY" => Self::Daily,
                b"WEEKLY" => Self::Weekly,
                b"MONTHLY" => Self::Monthly,
                b"YEARLY" => Self::Yearly,
                x => {
                    return Err(ParamError::InvalidFreq(
                        from_utf8(x)
                            .unwrap_or("<invalid utf8>")
                            .to_ascii_uppercase(),
                    ));
                }
            };
            Ok(r)
        }
    }

    impl TryFrom<&str> for Weekday {
        type Error = ParamError;

        fn try_from(s: &str) -> Result<Self, Self::Error> {
            let r = match s {
                "SU" => Self::Su,
                "MO" => Self::Mo,
                "TU" => Self::Tu,
                "WE" => Self::We,
                "TH" => Self::Th,
                "FR" => Self::Fr,
                "SA" => Self::Sa,
                _ => return Err(ParamError::InvalidWeekday(s.into())),
            };
            Ok(r)
        }
    }

    /// Parses a `weekdaynum` (e.g. `"MO"`, `"+1MO"`, `"-1SU"`): an optional
    /// signed ordinal followed by a two-letter weekday code.
    fn parse_weekday_num(tok: &str) -> Result<WeekdayNum, ValueError> {
        if tok.len() < 2 {
            return Err(recur_err("BYDAY", tok));
        }
        let (ord_part, day_part) = tok.split_at(tok.len() - 2);
        let weekday =
            Weekday::try_from(day_part).map_err(|_| recur_err("BYDAY", tok))?;
        let ordinal = if ord_part.is_empty() {
            None
        } else {
            let num = ord_part
                .parse::<i8>()
                .map_err(|_| recur_err("BYDAY ordinal", ord_part))?;
            if num == 0 || num.unsigned_abs() > 53 {
                return Err(recur_err("BYDAY ordinal", ord_part));
            }
            Some(num)
        };
        Ok(WeekdayNum { ordinal, weekday })
    }

    /// Parses a bounded integer sub-part shared by the `BYxxx` list rules,
    /// e.g. `BYSECOND`'s `0 to 60` range. `N` is the tuple struct's inner
    /// integer type; `S` is the tuple struct itself.
    fn parse_bounded<N, S>(
        name: &'static str,
        tok: &str,
        min: i32,
        max: i32,
        wrap: impl Fn(N) -> S,
    ) -> Result<S, ValueError>
    where
        N: TryFrom<i32>,
    {
        let n: i32 = tok.parse().map_err(|_| recur_err(name, tok))?;
        if n < min || n > max {
            return Err(recur_err(name, tok));
        }
        let n: N = n.try_into().map_err(|_| recur_err(name, tok))?;
        Ok(wrap(n))
    }

    impl TryFrom<&str> for Seconds {
        type Error = ValueError;

        fn try_from(s: &str) -> Result<Self, Self::Error> {
            parse_bounded("BYSECOND", s, 0, 60, Self)
        }
    }
    impl TryFrom<&str> for Minutes {
        type Error = ValueError;

        fn try_from(s: &str) -> Result<Self, Self::Error> {
            parse_bounded("BYMINUTE", s, 0, 59, Self)
        }
    }
    impl TryFrom<&str> for Hour {
        type Error = ValueError;

        fn try_from(s: &str) -> Result<Self, Self::Error> {
            parse_bounded("BYHOUR", s, 0, 23, Self)
        }
    }
    impl TryFrom<&str> for WeekNum {
        type Error = ValueError;

        fn try_from(s: &str) -> Result<Self, Self::Error> {
            parse_bounded("BYWEEKNO", s, -53, 53, Self).and_then(
                |WeekNum(n)| {
                    if n == 0 {
                        Err(recur_err("BYWEEKNO", s))
                    } else {
                        Ok(WeekNum(n))
                    }
                },
            )
        }
    }
    impl TryFrom<&str> for MonthNum {
        type Error = ValueError;

        fn try_from(s: &str) -> Result<Self, Self::Error> {
            parse_bounded("BYMONTH", s, 1, 12, Self)
        }
    }
    impl TryFrom<&str> for MonthDayNum {
        type Error = ValueError;

        fn try_from(s: &str) -> Result<Self, Self::Error> {
            parse_bounded("BYMONTHDAY", s, -31, 31, Self).and_then(
                |MonthDayNum(n)| {
                    if n == 0 {
                        Err(recur_err("BYMONTHDAY", s))
                    } else {
                        Ok(MonthDayNum(n))
                    }
                },
            )
        }
    }
    impl TryFrom<&str> for YearDayNum {
        type Error = ValueError;

        fn try_from(s: &str) -> Result<Self, Self::Error> {
            parse_bounded("BYYEARDAY", s, -366, 366, Self).and_then(
                |YearDayNum(n)| {
                    if n == 0 {
                        Err(recur_err("BYYEARDAY", s))
                    } else {
                        Ok(YearDayNum(n))
                    }
                },
            )
        }
    }

    /// Parses a COMMA-separated `BYxxx` list into its element type.
    fn parse_list<T, E>(
        value: &str,
        parse_one: impl Fn(&str) -> Result<T, E>,
    ) -> Result<Vec<T>, E> {
        value.split(',').map(parse_one).collect()
    }

    impl TryFrom<&[u8]> for Recur {
        type Error = ValueError;

        fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
            let s = from_utf8(v)?;
            let mut recur = Self::default();
            let mut freq_seen = false;

            for part in s.split(';') {
                let (name, value) = part
                    .split_once('=')
                    .ok_or_else(|| recur_err("NAME=VALUE", part))?;

                match name.to_ascii_uppercase().as_str() {
                    "FREQ" => {
                        recur.freq = value
                            .as_bytes()
                            .try_into()
                            .map_err(|_| recur_err("FREQ", value))?;
                        freq_seen = true;
                    }
                    "UNTIL" => {
                        recur.until = Some(value.as_bytes().try_into()?);
                    }
                    "COUNT" => {
                        let count: i32 = value.parse()?;
                        recur.count = Some(count);
                    }
                    "INTERVAL" => {
                        let interval: i32 = value.parse()?;
                        if interval < 1 {
                            return Err(recur_err("INTERVAL", value));
                        }
                        recur.interval = Some(interval);
                    }
                    "BYSECOND" => {
                        recur.by_second =
                            parse_list(value, |s| Seconds::try_from(s))?;
                    }
                    "BYMINUTE" => {
                        recur.by_minute =
                            parse_list(value, |s| Minutes::try_from(s))?;
                    }
                    "BYHOUR" => {
                        recur.by_hour =
                            parse_list(value, |s| Hour::try_from(s))?;
                    }
                    "BYDAY" => {
                        recur.by_day = parse_list(value, parse_weekday_num)?;
                    }
                    "BYMONTHDAY" => {
                        recur.by_month_day =
                            parse_list(value, |s| MonthDayNum::try_from(s))?;
                    }
                    "BYYEARDAY" => {
                        recur.by_year_day =
                            parse_list(value, |s| YearDayNum::try_from(s))?;
                    }
                    "BYWEEKNO" => {
                        recur.by_week_no =
                            parse_list(value, |s| WeekNum::try_from(s))?;
                    }
                    "BYMONTH" => {
                        recur.by_month =
                            parse_list(value, |s| MonthNum::try_from(s))?;
                    }
                    "BYSETPOS" => {
                        recur.by_set_pos =
                            parse_list(value, |s| YearDayNum::try_from(s))?;
                    }
                    "WKST" => {
                        recur.wkst = Some(
                            Weekday::try_from(value)
                                .map_err(|_| recur_err("WKST", value))?,
                        );
                    }
                    _ => return Err(recur_err("recur-rule-part", name)),
                }
            }

            if !freq_seen {
                return Err(recur_err("FREQ (required)", s));
            }
            if recur.until.is_some() && recur.count.is_some() {
                // UNTIL and COUNT MUST NOT occur in the same recur.
                return Err(recur_err("UNTIL or COUNT, not both", s));
            }

            Ok(recur)
        }
    }

    /// Renders a `BYxxx` rule part (e.g. `BYSECOND=1,2,3`) if `items` isn't
    /// empty; a no-op otherwise, since an empty `BYxxx` list was never
    /// specified at all.
    fn write_by_list<T: std::fmt::Display>(
        f: &mut std::fmt::Formatter<'_>,
        name: &str,
        items: &[T],
    ) -> std::fmt::Result {
        if items.is_empty() {
            return Ok(());
        }
        write!(f, ";{name}=")?;
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "{item}")?;
        }
        Ok(())
    }

    impl std::fmt::Display for Recur {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "FREQ={}", self.freq)?;
            if let Some(until) = &self.until {
                write!(f, ";UNTIL={until}")?;
            }
            if let Some(count) = self.count {
                write!(f, ";COUNT={count}")?;
            }
            if let Some(interval) = self.interval {
                write!(f, ";INTERVAL={interval}")?;
            }
            write_by_list(f, "BYSECOND", &self.by_second)?;
            write_by_list(f, "BYMINUTE", &self.by_minute)?;
            write_by_list(f, "BYHOUR", &self.by_hour)?;
            write_by_list(f, "BYDAY", &self.by_day)?;
            write_by_list(f, "BYMONTHDAY", &self.by_month_day)?;
            write_by_list(f, "BYYEARDAY", &self.by_year_day)?;
            write_by_list(f, "BYWEEKNO", &self.by_week_no)?;
            write_by_list(f, "BYMONTH", &self.by_month)?;
            write_by_list(f, "BYSETPOS", &self.by_set_pos)?;
            if let Some(wkst) = &self.wkst {
                write!(f, ";WKST={wkst}")?;
            }
            Ok(())
        }
    }

    impl std::fmt::Display for Frequency {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                Self::Secondly => "SECONDLY",
                Self::Minutely => "MINUTELY",
                Self::Hourly => "HOURLY",
                Self::Daily => "DAILY",
                Self::Weekly => "WEEKLY",
                Self::Monthly => "MONTHLY",
                Self::Yearly => "YEARLY",
            })
        }
    }

    impl std::fmt::Display for Weekday {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                Self::Su => "SU",
                Self::Mo => "MO",
                Self::Tu => "TU",
                Self::We => "WE",
                Self::Th => "TH",
                Self::Fr => "FR",
                Self::Sa => "SA",
            })
        }
    }

    impl std::fmt::Display for Seconds {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::fmt::Display for Minutes {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::fmt::Display for Hour {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::fmt::Display for WeekNum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::fmt::Display for WeekdayNum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if let Some(ordinal) = self.ordinal {
                write!(f, "{ordinal}")?;
            }
            write!(f, "{}", self.weekday)
        }
    }

    impl std::fmt::Display for MonthNum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::fmt::Display for MonthDayNum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::fmt::Display for YearDayNum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}

/// [RFC 4288](https://datatracker.ietf.org/doc/html/rfc4288#section-4.2)
///
/// As used in this crate, we only validate that there is a type and subtype,
/// separated by a slash
#[derive(Default, Debug)]
pub struct MediaType {
    media_type: Text,
    subtype: Text,
}

impl From<&str> for Text {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}
impl TryFrom<&[u8]> for MediaType {
    type Error = ValueError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let (t, s) = split_once(value, b'/').ok_or(ValueError::MediaType)?;
        Ok(Self {
            media_type: t.try_into()?,
            subtype: s.try_into()?,
        })
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.media_type.as_str(), self.subtype.as_str())
    }
}

impl TryFrom<&[u8]> for Text {
    type Error = ValueError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(unescape_text(std::str::from_utf8(value)?)))
    }
}

/// Decodes the BACKSLASH-escape sequences defined for `TEXT` values by
/// [Section 3.3.11](https://datatracker.ietf.org/doc/html/rfc5545#section-3.3.11):
/// `\\` -> `\`, `\;` -> `;`, `\,` -> `,`, and `\N`/`\n` -> a newline. A
/// backslash followed by anything else isn't a defined escape, so it's left
/// as-is (the backslash retained) rather than silently dropped.
fn unescape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            Some('\\') => out.push('\\'),
            Some(';') => out.push(';'),
            Some(',') => out.push(','),
            Some('n') | Some('N') => out.push('\n'),
            _ => {
                out.push('\\');
                continue;
            }
        }
        chars.next();
    }
    out
}

impl TryFrom<&[u8]> for Uri {
    type Error = ValueError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(url::Url::parse(str::from_utf8(value)?)?))
    }
}

impl TryFrom<&[u8]> for CalendarUserAddress {
    type Error = ValueError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let uri: Uri = value.try_into()?;
        if uri.scheme() != "mailto" {
            Err(ValueError::CalUserAddress)
        } else {
            Ok(Self(uri))
        }
    }
}

impl std::fmt::Display for CalendarUserAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self.0)
    }
}

impl Deref for Uri {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Text {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Boolean {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&[u8]> for Boolean {
    type Error = ValueError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let r = match value {
            b"TRUE" => Self(true),
            b"FALSE" => Self(false),
            _ => return Err(ValueError::Boolean),
        };

        Ok(r)
    }
}

/// An error parsing a raw property value into one of this crate's typed value
/// representations.
#[derive(Debug, Error)]
pub enum ValueError {
    /// [CalendarUserAddress] Parsing Error
    #[error("Malformed CalenderUserAddress")]
    CalUserAddress,

    /// [MediaType] Parsing Error
    #[error("Malformed MediaType")]
    MediaType,

    /// `BOOLEAN` Parsing Error
    #[error("Malformed Boolean")]
    Boolean,

    /// \[[Integer](crate::values::Integer)\] parsing error
    #[error(transparent)]
    Integer(#[from] std::num::ParseIntError),

    /// \[[Float](crate::values::Float)\] parsing error
    #[error(transparent)]
    Float(#[from] std::num::ParseFloatError),

    /// URL parsing error
    #[error(transparent)]
    URL(#[from] url::ParseError),

    /// \[[UtcOffset](crate::values::UtcOffset)\] parsing error
    #[error("Malformed UTC offset")]
    UtcOffset,

    /// \[[Duration](crate::values::Duration)\] parsing error
    #[error("Malformed Duration")]
    Duration,

    /// \[[Binary](crate::values::Binary)\] decoding error
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),

    /// Encoding error surfaced while decoding a value's raw bytes as UTF-8.
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),

    /// Shared by every value type backed by a `chrono` parser (`DATE`,
    /// `DATE-TIME`).
    #[error(transparent)]
    ChronoParse(#[from] chrono::ParseError),

    /// Catch-all for a value (or value sub-part, e.g. a `RECUR` rule-part)
    /// that doesn't match its expected shape.
    #[error("Value parsing failed. Expected {expected}, got {received:?}")]
    Malformed {
        /// What the value is supposed to be
        expected: String,
        /// What we actually received
        received: Option<String>,
    },
}

impl std::fmt::Display for DateOrDatetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Date(d) => write!(f, "{d}"),
            Self::DateTime(dt) => write!(f, "{dt}"),
        }
    }
}

impl std::fmt::Display for DateTimePeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Date(d) => write!(f, "{d}"),
            Self::DateTime(dt) => write!(f, "{dt}"),
            Self::Period(p) => write!(f, "{p}"),
        }
    }
}

impl std::fmt::Display for DateTimeDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Duration(d) => write!(f, "{d}"),
            Self::DateTime(dt) => write!(f, "{dt}"),
        }
    }
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let negative = self.0 < ChronoDuration::zero();
        let abs = if negative { -self.0 } else { self.0 };
        if negative {
            f.write_str("-")?;
        }
        f.write_str("P")?;

        let total_secs = abs.num_seconds();
        const WEEK: i64 = 7 * 24 * 3600;
        if total_secs != 0 && total_secs % WEEK == 0 {
            return write!(f, "{}W", total_secs / WEEK);
        }

        let days = total_secs / 86400;
        let rem = total_secs % 86400;
        let hours = rem / 3600;
        let minutes = (rem % 3600) / 60;
        let seconds = rem % 60;

        if days != 0 {
            write!(f, "{days}D")?;
        }
        if hours != 0 || minutes != 0 || seconds != 0 || days == 0 {
            f.write_str("T")?;
            if hours != 0 {
                write!(f, "{hours}H")?;
            }
            if minutes != 0 {
                write!(f, "{minutes}M")?;
            }
            if seconds != 0 || (hours == 0 && minutes == 0) {
                write!(f, "{seconds}S")?;
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Utc(dt) => write!(f, "{}", dt.format(ICAL_DATETIME_UTC_FMT)),
            Self::Floating(dt) => {
                write!(f, "{}", dt.format(ICAL_DATETIME_FMT))
            }
        }
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format(ICAL_DATE_FMT))
    }
}

impl std::fmt::Display for UtcOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total = self.0.local_minus_utc();
        let sign = if total < 0 { '-' } else { '+' };
        let abs = total.unsigned_abs();
        let hours = abs / 3600;
        let minutes = (abs % 3600) / 60;
        let seconds = abs % 60;
        if seconds == 0 {
            write!(f, "{sign}{hours:02}{minutes:02}")
        } else {
            write!(f, "{sign}{hours:02}{minutes:02}{seconds:02}")
        }
    }
}

impl std::fmt::Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StartEnd { start, end } => write!(f, "{start}/{end}"),
            Self::Duration { start, duration } => {
                write!(f, "{start}/{duration}")
            }
        }
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time = match self {
            Self::Floating(t) => t,
            Self::Zoned { time, .. } => time,
        };
        write!(f, "{}", time.format("%H%M%S"))
    }
}

impl std::fmt::Display for Binary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&base64::engine::general_purpose::STANDARD.encode(&self.0))
    }
}

impl std::fmt::Display for Boolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.0 { "TRUE" } else { "FALSE" })
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self.0.chars() {
            match c {
                '\\' => f.write_str("\\\\")?,
                ';' => f.write_str("\\;")?,
                ',' => f.write_str("\\,")?,
                '\n' => f.write_str("\\n")?,
                _ => write!(f, "{c}")?,
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Float {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_parses_ymd() {
        let date = Date::try_from(b"19970714".as_slice()).unwrap();
        assert_eq!(*date, NaiveDate::from_ymd_opt(1997, 7, 14).unwrap());
    }

    #[test]
    fn date_rejects_datetime_shape() {
        assert!(Date::try_from(b"19970714T133000".as_slice()).is_err());
    }

    #[test]
    fn date_or_datetime_dispatches_on_t() {
        assert!(matches!(
            DateOrDatetime::try_from(b"19970714".as_slice()),
            Ok(DateOrDatetime::Date(_))
        ));
        assert!(matches!(
            DateOrDatetime::try_from(b"19970714T133000Z".as_slice()),
            Ok(DateOrDatetime::DateTime(_))
        ));
    }

    #[test]
    fn date_time_with_trailing_z_is_utc() {
        let dt = DateTime::try_from(b"19980119T070000Z".as_slice()).unwrap();
        assert_eq!(
            dt,
            DateTime::Utc(Utc.with_ymd_and_hms(1998, 1, 19, 7, 0, 0).unwrap())
        );
    }

    #[test]
    fn date_time_with_no_z_and_no_tzid_stays_floating() {
        // RFC 5545 §3.3.5 form #1: a local time with no TZID is "floating"
        // and must not be resolved through any implicit time zone —
        // in particular, never the host machine's (issue #11).
        let dt = DateTime::try_from(b"19980118T230000".as_slice()).unwrap();
        assert_eq!(
            dt,
            DateTime::Floating(
                NaiveDate::from_ymd_opt(1998, 1, 18)
                    .unwrap()
                    .and_hms_opt(23, 0, 0)
                    .unwrap()
            )
        );
    }

    #[test]
    fn resolve_tz_uses_the_tzid_zone_not_the_host_zone() {
        // The RFC 5545 §3.3.5 form #3 worked example: TZID=America/
        // New_York:19970714T133000 is 1:30 PM in New York, i.e.
        // 17:30 UTC (EDT, UTC-04:00) — regardless of the host's own zone.
        let tz: Tz = "America/New_York".parse().unwrap();
        let dt = DateTime::try_from(b"19970714T133000".as_slice())
            .unwrap()
            .resolve_tz(tz);
        assert_eq!(
            dt,
            DateTime::Utc(
                Utc.with_ymd_and_hms(1997, 7, 14, 17, 30, 0).unwrap()
            )
        );
    }

    #[test]
    fn resolve_tz_is_a_no_op_on_an_already_utc_value() {
        // RFC 5545 §3.2.19: TZID "MUST NOT be applied" to a DATE-TIME
        // that's already UTC.
        let tz: Tz = "America/New_York".parse().unwrap();
        let dt = DateTime::try_from(b"19980119T070000Z".as_slice())
            .unwrap()
            .resolve_tz(tz);
        assert_eq!(
            dt,
            DateTime::Utc(Utc.with_ymd_and_hms(1998, 1, 19, 7, 0, 0).unwrap())
        );
    }

    #[test]
    fn resolve_tz_ambiguous_local_time_uses_the_first_occurrence() {
        // RFC 5545 §3.3.5's own worked example: on the fall-back DST
        // transition, "TZID=America/New_York:20071104T013000 indicates
        // November 4, 2007 at 1:30 A.M. EDT (UTC-04:00)" — the *first* of
        // the two instants this local time could mean, not the host's.
        let tz: Tz = "America/New_York".parse().unwrap();
        let dt = DateTime::try_from(b"20071104T013000".as_slice())
            .unwrap()
            .resolve_tz(tz);
        assert_eq!(
            dt,
            DateTime::Utc(Utc.with_ymd_and_hms(2007, 11, 4, 5, 30, 0).unwrap())
        );
    }

    #[test]
    fn resolve_tz_nonexistent_local_time_uses_the_offset_before_the_gap() {
        // RFC 5545 §3.3.5's own worked example: on the spring-forward DST
        // transition, "TZID=America/New_York:20070311T023000 indicates
        // March 11, 2007 at 3:30 A.M. EDT (UTC-04:00), one hour after
        // 1:30 A.M. EST (UTC-05:00)" — i.e. resolved using EST (the offset
        // before the gap), not left as an error.
        let tz: Tz = "America/New_York".parse().unwrap();
        let dt = DateTime::try_from(b"20070311T023000".as_slice())
            .unwrap()
            .resolve_tz(tz);
        assert_eq!(
            dt,
            DateTime::Utc(Utc.with_ymd_and_hms(2007, 3, 11, 7, 30, 0).unwrap())
        );
    }

    #[test]
    fn date_time_period_dispatches_on_shape() {
        assert!(matches!(
            DateTimePeriod::try_from(b"19970714".as_slice()),
            Ok(DateTimePeriod::Date(_))
        ));
        assert!(matches!(
            DateTimePeriod::try_from(b"19970714T133000Z".as_slice()),
            Ok(DateTimePeriod::DateTime(_))
        ));
        assert!(matches!(
            DateTimePeriod::try_from(
                b"19970101T180000Z/19970102T070000Z".as_slice()
            ),
            Ok(DateTimePeriod::Period(_))
        ));
    }

    #[test]
    fn period_start_end_form() {
        let period =
            Period::try_from(b"19970101T180000Z/19970102T070000Z".as_slice())
                .unwrap();
        assert!(matches!(period, Period::StartEnd { .. }));
    }

    #[test]
    fn period_start_duration_form() {
        let period =
            Period::try_from(b"19970308T160000Z/PT8H30M".as_slice()).unwrap();
        assert!(matches!(period, Period::Duration { .. }));
    }

    #[test]
    fn date_time_duration_dispatches_on_leading_p() {
        assert!(matches!(
            DateTimeDuration::try_from(b"-PT15M".as_slice()),
            Ok(DateTimeDuration::Duration(_))
        ));
        assert!(matches!(
            DateTimeDuration::try_from(b"19980101T050000Z".as_slice()),
            Ok(DateTimeDuration::DateTime(_))
        ));
    }

    #[test]
    fn duration_examples_from_rfc() {
        let d = Duration::try_from(b"P15DT5H0M20S".as_slice()).unwrap();
        assert_eq!(
            *d,
            ChronoDuration::days(15)
                + ChronoDuration::hours(5)
                + ChronoDuration::seconds(20)
        );

        let d = Duration::try_from(b"P7W".as_slice()).unwrap();
        assert_eq!(*d, ChronoDuration::weeks(7));

        let d = Duration::try_from(b"-PT15M".as_slice()).unwrap();
        assert_eq!(*d, -ChronoDuration::minutes(15));

        let d = Duration::try_from(b"PT1H0M0S".as_slice()).unwrap();
        assert_eq!(*d, ChronoDuration::hours(1));
    }

    #[test]
    fn duration_rejects_missing_p() {
        assert!(Duration::try_from(b"15D".as_slice()).is_err());
    }

    #[test]
    fn duration_rejects_overflowing_week_count_without_panicking() {
        // `P999999999999999999W` overflows every representable
        // `chrono::TimeDelta` (i64 milliseconds). Must be a `ValueError`,
        // never a panic from the underlying `TimeDelta` arithmetic.
        assert!(matches!(
            Duration::try_from(b"P999999999999999999W".as_slice()),
            Err(ValueError::Duration)
        ));
    }

    #[test]
    fn utc_offset_examples_from_rfc() {
        let off = UtcOffset::try_from(b"-0500".as_slice()).unwrap();
        assert_eq!(*off, FixedOffset::west_opt(5 * 3600).unwrap());

        let off = UtcOffset::try_from(b"+0100".as_slice()).unwrap();
        assert_eq!(*off, FixedOffset::east_opt(3600).unwrap());
    }

    #[test]
    fn utc_offset_rejects_negative_zero() {
        assert!(UtcOffset::try_from(b"-0000".as_slice()).is_err());
        assert!(UtcOffset::try_from(b"-000000".as_slice()).is_err());
    }

    #[test]
    fn utc_offset_parses_seconds_precision() {
        // tests/fixtures/collective-icalendar/timezones/
        // issue_55_parse_error_on_utc_offset_with_seconds.ics has
        // `TZOFFSETFROM:-075258` — the 6-digit HHMMSS branch is
        // structurally supported but was never exercised by a test.
        let off = UtcOffset::try_from(b"-075258".as_slice()).unwrap();
        assert_eq!(
            *off,
            FixedOffset::west_opt(7 * 3600 + 52 * 60 + 58).unwrap()
        );
    }

    #[test]
    fn integer_and_float_parse() {
        assert_eq!(*Integer::try_from(b"39".as_slice()).unwrap(), 39);
        assert_eq!(*Integer::try_from(b"-5".as_slice()).unwrap(), -5);
        assert_eq!(
            *Float::try_from(b"37.386013".as_slice()).unwrap(),
            37.386013
        );
    }

    #[test]
    fn binary_decodes_base64() {
        let bin = Binary::try_from(b"aGVsbG8=".as_slice()).unwrap();
        assert_eq!(&*bin, b"hello");
    }

    #[test]
    fn recur_requires_freq() {
        assert!(Recur::try_from(b"BYMONTH=1".as_slice()).is_err());
    }

    #[test]
    fn recur_rejects_until_and_count_together() {
        assert!(
            Recur::try_from(
                b"FREQ=DAILY;COUNT=5;UNTIL=19971224T000000Z".as_slice()
            )
            .is_err()
        );
    }

    #[test]
    fn recur_parses_freq_and_bymonth() {
        assert!(Recur::try_from(b"FREQ=YEARLY;BYMONTH=1".as_slice()).is_ok());
    }

    #[test]
    fn recur_parses_byday_with_ordinal() {
        assert!(Recur::try_from(b"FREQ=MONTHLY;BYDAY=-1MO".as_slice()).is_ok());
    }

    #[test]
    fn recur_rejects_out_of_range_bysecond() {
        assert!(
            Recur::try_from(b"FREQ=SECONDLY;BYSECOND=61".as_slice()).is_err()
        );
    }

    #[test]
    fn text_unescapes_backslash_sequences() {
        // RFC 5545 §3.3.11: \\ -> \, \; -> ;, \, -> ,, \n and \N -> newline.
        let text =
            Text::try_from(b"a\\\\b,c\\;d\\,e\\nf\\Ng".as_slice()).unwrap();
        assert_eq!(text.as_str(), "a\\b,c;d,e\nf\ng");
    }

    #[test]
    fn text_leaves_undefined_escapes_untouched() {
        // No escape is defined for e.g. \a, so the backslash is kept as-is
        // rather than silently dropped.
        let text = Text::try_from(b"C:\\at".as_slice()).unwrap();
        assert_eq!(text.as_str(), "C:\\at");
    }

    #[test]
    fn text_display_escapes_backslash_semicolon_comma_and_newline() {
        let text =
            Text::try_from(b"a\\\\b,c\\;d\\,e\\nf\\Ng".as_slice()).unwrap();
        assert_eq!(text.to_string(), "a\\\\b\\,c\\;d\\,e\\nf\\ng");
    }

    #[test]
    fn date_display_matches_ical_wire_format() {
        let date = Date::try_from(b"19970714".as_slice()).unwrap();
        assert_eq!(date.to_string(), "19970714");
    }

    #[test]
    fn date_time_display_round_trips_utc_and_floating_forms() {
        let utc = DateTime::try_from(b"19980119T070000Z".as_slice()).unwrap();
        assert_eq!(utc.to_string(), "19980119T070000Z");

        let floating =
            DateTime::try_from(b"19980118T230000".as_slice()).unwrap();
        assert_eq!(floating.to_string(), "19980118T230000");
    }

    #[test]
    fn utc_offset_display_examples_from_rfc() {
        let west = UtcOffset::try_from(b"-0500".as_slice()).unwrap();
        assert_eq!(west.to_string(), "-0500");

        let east = UtcOffset::try_from(b"+0100".as_slice()).unwrap();
        assert_eq!(east.to_string(), "+0100");

        let secs = UtcOffset::try_from(b"-075258".as_slice()).unwrap();
        assert_eq!(secs.to_string(), "-075258");
    }

    #[test]
    fn duration_display_round_trips_rfc_examples() {
        let d = Duration::try_from(b"P15DT5H0M20S".as_slice()).unwrap();
        assert_eq!(d.to_string(), "P15DT5H20S");

        let d = Duration::try_from(b"P7W".as_slice()).unwrap();
        assert_eq!(d.to_string(), "P7W");

        let d = Duration::try_from(b"-PT15M".as_slice()).unwrap();
        assert_eq!(d.to_string(), "-PT15M");
    }

    #[test]
    fn recur_display_round_trips_freq_and_byday() {
        let recur =
            Recur::try_from(b"FREQ=MONTHLY;BYDAY=-1MO;COUNT=5".as_slice())
                .unwrap();
        assert_eq!(recur.to_string(), "FREQ=MONTHLY;COUNT=5;BYDAY=-1MO");
    }
}
