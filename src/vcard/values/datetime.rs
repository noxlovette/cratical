//! `date`, `time`, `date-time`, `date-and-or-time`, `timestamp` and
//! `utc-offset` (RFC 6350 §4.3, §4.7).
//!
//! Only the basic ISO 8601 format is supported, with the reduced-accuracy
//! and truncated forms the RFC permits, so none of these can be a `chrono`
//! type: `--0415` has no year, `T-2200` has no hour.

use super::{ValueError, malformed};
use std::fmt;

/// `2DIGIT` as a number.
fn two_digits(v: &[u8]) -> Option<u8> {
    match v {
        [a @ b'0'..=b'9', b @ b'0'..=b'9'] => {
            Some((a - b'0') * 10 + (b - b'0'))
        }
        _ => None,
    }
}

/// `4DIGIT` as a number.
fn four_digits(v: &[u8]) -> Option<u16> {
    if v.len() != 4 {
        return None;
    }
    Some(
        u16::from(two_digits(&v[..2])?) * 100 + u16::from(two_digits(&v[2..])?),
    )
}

/// The `utc-offset` value type specifies that the property value is a signed
/// offset from UTC. This value type can be specified in the TZ property.
///
/// The value type is an offset from Coordinated Universal Time (UTC). It is
/// specified as a positive or negative difference in units of hours and
/// minutes (e.g., +hhmm). The time is specified as a 24-hour clock. Hour
/// values are from 00 to 23, and minute values are from 00 to 59. Hour and
/// minutes are 2 digits with high-order zeroes required to maintain digit
/// count. The basic format for ISO 8601 UTC offsets MUST be used.
///
/// The minutes are optional (`utc-offset = sign hour [minute]`), and whether
/// they were written is kept, so `-08` and `-0800` each round-trip as they
/// came.
///
/// Example:
///
/// > -0500
/// >
/// > +01
///
/// [Section 4.7](https://datatracker.ietf.org/doc/html/rfc6350#section-4.7)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UtcOffset {
    negative: bool,
    hour: u8,
    minute: Option<u8>,
}

impl UtcOffset {
    /// Builds an offset, refusing an hour over 23 or a minute over 59.
    pub fn new(
        negative: bool,
        hour: u8,
        minute: Option<u8>,
    ) -> Result<Self, ValueError> {
        if hour > 23 || minute.is_some_and(|m| m > 59) {
            return Err(ValueError::Malformed {
                expected: "utc-offset within 00-23 hours and 00-59 minutes",
                received: format!("{hour}:{minute:?}"),
            });
        }
        Ok(Self {
            negative,
            hour,
            minute,
        })
    }

    /// Whether the offset is west of UTC.
    pub fn is_negative(&self) -> bool {
        self.negative
    }

    /// The hours, 0 to 23.
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// The minutes if they were written, 0 to 59.
    pub fn minute(&self) -> Option<u8> {
        self.minute
    }

    /// The offset in seconds east of UTC.
    pub fn seconds_east(&self) -> i32 {
        let seconds = i32::from(self.hour) * 3600
            + i32::from(self.minute.unwrap_or(0)) * 60;
        if self.negative { -seconds } else { seconds }
    }
}

impl TryFrom<&[u8]> for UtcOffset {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let err = || malformed("utc-offset (+hh or +hhmm)", v);
        let (negative, digits) = match v.split_first() {
            Some((b'+', rest)) => (false, rest),
            Some((b'-', rest)) => (true, rest),
            _ => return Err(err()),
        };
        let hour =
            two_digits(digits.get(..2).ok_or_else(err)?).ok_or_else(err)?;
        let minute = match &digits[2..] {
            [] => None,
            m => Some(two_digits(m).ok_or_else(err)?),
        };
        Self::new(negative, hour, minute).map_err(|_| err())
    }
}

impl fmt::Display for UtcOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{:02}",
            if self.negative { '-' } else { '+' },
            self.hour
        )?;
        if let Some(minute) = self.minute {
            write!(f, "{minute:02}")?;
        }
        Ok(())
    }
}

/// `zone = utc-designator / utc-offset`, where the `utc-designator` is an
/// uppercase "Z".
///
/// Example:
///
/// > Z
/// >
/// > -0800
///
/// [Section 4.3](https://datatracker.ietf.org/doc/html/rfc6350#section-4.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    /// `Z`.
    Utc,
    /// A signed offset from UTC.
    Offset(UtcOffset),
}

impl TryFrom<&[u8]> for Zone {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        match v {
            b"Z" => Ok(Self::Utc),
            other => UtcOffset::try_from(other).map(Self::Offset),
        }
    }
}

impl fmt::Display for Zone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Utc => f.write_str("Z"),
            Self::Offset(offset) => offset.fmt(f),
        }
    }
}

fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4)
        && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

/// The days in `month`. Without a year February is given its leap-year 29,
/// since `--0229` is a date that exists.
fn days_in_month(month: u8, year: Option<u16>) -> u8 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if year.is_some_and(|y| !is_leap_year(y)) => 28,
        2 => 29,
        _ => 31,
    }
}

/// A calendar date as specified in ISO 8601, Section 4.1.2.
///
/// Reduced accuracy, as specified in ISO 8601, Sections 4.1.2.3 a) and b),
/// but not c), is permitted.
///
/// Expanded representation, as specified in ISO 8601, Section 4.1.4, is
/// forbidden.
///
/// Truncated representation, as specified in ISO 8601:2000, Sections 5.2.1.3
/// d), e), and f), is permitted.
///
/// Note the use of YYYY-MM in the second example below. YYYYMM is disallowed
/// to prevent confusion with YYMMDD. Note also that YYYY-MM-DD is disallowed
/// since we are using the basic format instead of the extended format.
///
/// ```text
/// date          = year    [month  day]
///               / year "-" month
///               / "--"     month [day]
///               / "--"      "-"   day
/// date-noreduc  = year     month  day
///               / "--"     month  day
///               / "--"      "-"   day
/// date-complete = year     month  day
/// ```
///
/// Example:
///
/// > 19850412
/// >
/// > 1985-04
/// >
/// > 1985
/// >
/// > --0412
/// >
/// > ---12
///
/// [Section 4.3.1](https://datatracker.ietf.org/doc/html/rfc6350#section-4.3.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    year: Option<u16>,
    month: Option<u8>,
    day: Option<u8>,
}

impl Date {
    /// Builds a date from the parts it has. Only the combinations the
    /// grammar has are legal: year; year and month; all three; month; month
    /// and day; or day alone. A month is 1 to 12, a day exists in its month
    /// (a February 29th needs a leap year, or no year), and a year has 4
    /// digits.
    pub fn new(
        year: Option<u16>,
        month: Option<u8>,
        day: Option<u8>,
    ) -> Result<Self, ValueError> {
        let shape_ok = matches!(
            (year, month, day),
            (Some(_), None, None)
                | (Some(_), Some(_), None)
                | (Some(_), Some(_), Some(_))
                | (None, Some(_), None)
                | (None, Some(_), Some(_))
                | (None, None, Some(_))
        );
        let in_range = year.is_none_or(|y| y <= 9999)
            && month.is_none_or(|m| (1..=12).contains(&m))
            && day.is_none_or(|d| {
                d >= 1 && d <= month.map_or(31, |m| days_in_month(m, year))
            });
        if shape_ok && in_range {
            Ok(Self { year, month, day })
        } else {
            Err(ValueError::Malformed {
                expected: "date",
                received: format!("{year:?}-{month:?}-{day:?}"),
            })
        }
    }

    /// The year, if the date has one.
    pub fn year(&self) -> Option<u16> {
        self.year
    }

    /// The month, 1 to 12, if the date has one.
    pub fn month(&self) -> Option<u8> {
        self.month
    }

    /// The day of the month, if the date has one.
    pub fn day(&self) -> Option<u8> {
        self.day
    }

    /// Whether it's a `date-noreduc`: a day is there, so nothing is reduced
    /// (a missing year or month is truncation, which that production
    /// allows).
    pub fn is_noreduc(&self) -> bool {
        self.day.is_some()
    }

    /// Whether it's a `date-complete`: year, month and day.
    pub fn is_complete(&self) -> bool {
        self.year.is_some() && self.month.is_some() && self.day.is_some()
    }
}

impl TryFrom<&[u8]> for Date {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let err = || malformed("date", v);
        let (year, month, day) = if let Some(day) = v.strip_prefix(b"---") {
            (None, None, Some(two_digits(day).ok_or_else(err)?))
        } else if let Some(rest) = v.strip_prefix(b"--") {
            match rest.len() {
                2 => (None, Some(two_digits(rest).ok_or_else(err)?), None),
                4 => (
                    None,
                    Some(two_digits(&rest[..2]).ok_or_else(err)?),
                    Some(two_digits(&rest[2..]).ok_or_else(err)?),
                ),
                _ => return Err(err()),
            }
        } else {
            let year =
                Some(four_digits(v.get(..4).ok_or_else(err)?).ok_or_else(err)?);
            match &v[4..] {
                [] => (year, None, None),
                [b'-', m @ ..] => {
                    (year, Some(two_digits(m).ok_or_else(err)?), None)
                }
                m_d if m_d.len() == 4 => (
                    year,
                    Some(two_digits(&m_d[..2]).ok_or_else(err)?),
                    Some(two_digits(&m_d[2..]).ok_or_else(err)?),
                ),
                _ => return Err(err()),
            }
        };
        Self::new(year, month, day).map_err(|_| err())
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.year, self.month, self.day) {
            (Some(y), None, None) => write!(f, "{y:04}"),
            (Some(y), Some(m), None) => write!(f, "{y:04}-{m:02}"),
            (Some(y), Some(m), Some(d)) => write!(f, "{y:04}{m:02}{d:02}"),
            (None, Some(m), None) => write!(f, "--{m:02}"),
            (None, Some(m), Some(d)) => write!(f, "--{m:02}{d:02}"),
            (None, None, Some(d)) => write!(f, "---{d:02}"),
            // `new` allows nothing else.
            _ => Ok(()),
        }
    }
}

/// A time of day as specified in ISO 8601, Section 4.2.
///
/// Reduced accuracy, as specified in ISO 8601, Section 4.2.2.3, is
/// permitted.
///
/// Representation with decimal fraction, as specified in ISO 8601, Section
/// 4.2.2.4, is forbidden.
///
/// The midnight hour is always represented by 00, never 24 (see ISO 8601,
/// Section 4.2.3).
///
/// Truncated representation, as specified in ISO 8601:2000, Sections 5.3.1.4
/// a), b), and c), is permitted.
///
/// ```text
/// time          = hour [minute [second]] [zone]
///               /  "-"  minute [second]  [zone]
///               /  "-"   "-"    second   [zone]
/// time-notrunc  = hour [minute [second]] [zone]
/// time-complete = hour  minute  second   [zone]
/// ```
///
/// Example:
///
/// > 102200
/// >
/// > 1022
/// >
/// > 10
/// >
/// > -2200
/// >
/// > --00
/// >
/// > 102200Z
/// >
/// > 102200-0800
///
/// [Section 4.3.2](https://datatracker.ietf.org/doc/html/rfc6350#section-4.3.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time {
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>,
    zone: Option<Zone>,
}

impl Time {
    /// Builds a time from the parts it has. Only the combinations the
    /// grammar has are legal: hour; hour and minute; all three; minute;
    /// minute and second; or second alone. An hour is 0 to 23, a minute 0 to
    /// 59, and a second 0 to 60 (a leap second).
    pub fn new(
        hour: Option<u8>,
        minute: Option<u8>,
        second: Option<u8>,
        zone: Option<Zone>,
    ) -> Result<Self, ValueError> {
        let shape_ok = matches!(
            (hour, minute, second),
            (Some(_), None, None)
                | (Some(_), Some(_), None)
                | (Some(_), Some(_), Some(_))
                | (None, Some(_), None)
                | (None, Some(_), Some(_))
                | (None, None, Some(_))
        );
        let in_range = hour.is_none_or(|h| h <= 23)
            && minute.is_none_or(|m| m <= 59)
            && second.is_none_or(|s| s <= 60);
        if shape_ok && in_range {
            Ok(Self {
                hour,
                minute,
                second,
                zone,
            })
        } else {
            Err(ValueError::Malformed {
                expected: "time",
                received: format!("{hour:?}:{minute:?}:{second:?}"),
            })
        }
    }

    /// The hour, 0 to 23, if the time has one.
    pub fn hour(&self) -> Option<u8> {
        self.hour
    }

    /// The minute, 0 to 59, if the time has one.
    pub fn minute(&self) -> Option<u8> {
        self.minute
    }

    /// The second, 0 to 60, if the time has one.
    pub fn second(&self) -> Option<u8> {
        self.second
    }

    /// The zone, if the time has one.
    pub fn zone(&self) -> Option<Zone> {
        self.zone
    }

    /// Whether it's a `time-notrunc`: it starts at the hour, though it may
    /// stop early.
    pub fn is_notrunc(&self) -> bool {
        self.hour.is_some()
    }

    /// Whether it's a `time-complete`: hour, minute and second.
    pub fn is_complete(&self) -> bool {
        self.hour.is_some() && self.minute.is_some() && self.second.is_some()
    }
}

impl TryFrom<&[u8]> for Time {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let err = || malformed("time", v);
        let dashes = v.iter().take_while(|b| **b == b'-').count();
        let rest = &v[dashes..];
        let digits = rest.iter().take_while(|b| b.is_ascii_digit()).count();
        let (digits, zone) = rest.split_at(digits);
        let zone = match zone {
            [] => None,
            z => Some(Zone::try_from(z).map_err(|_| err())?),
        };
        let pair = |i: usize| two_digits(&digits[i..i + 2]);
        let (hour, minute, second) = match (dashes, digits.len()) {
            (0, 2) => (pair(0), None, None),
            (0, 4) => (pair(0), pair(2), None),
            (0, 6) => (pair(0), pair(2), pair(4)),
            (1, 2) => (None, pair(0), None),
            (1, 4) => (None, pair(0), pair(2)),
            (2, 2) => (None, None, pair(0)),
            _ => return Err(err()),
        };
        Self::new(hour, minute, second, zone).map_err(|_| err())
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.hour, self.minute, self.second) {
            (Some(h), m, s) => {
                write!(f, "{h:02}")?;
                if let Some(m) = m {
                    write!(f, "{m:02}")?;
                }
                if let Some(s) = s {
                    write!(f, "{s:02}")?;
                }
            }
            (None, Some(m), s) => {
                write!(f, "-{m:02}")?;
                if let Some(s) = s {
                    write!(f, "{s:02}")?;
                }
            }
            (None, None, Some(s)) => write!(f, "--{s:02}")?,
            // `new` allows nothing else.
            (None, None, None) => {}
        }
        if let Some(zone) = self.zone {
            zone.fmt(f)?;
        }
        Ok(())
    }
}

/// Splits `v` at its first uppercase "T", the `time-designator`.
fn split_designator(v: &[u8]) -> Option<(&[u8], &[u8])> {
    crate::ast::split_once(v, b'T')
}

/// A date and time of day combination as specified in ISO 8601, Section 4.3.
///
/// Truncation of the date part, as specified in ISO 8601:2000, Section 5.4.2
/// c), is permitted.
///
/// ```text
/// date-time = date-noreduc  time-designator time-notrunc
/// ```
///
/// The date part is a `date-noreduc` (no reduced accuracy) and the time part
/// a `time-notrunc` (no truncation at the front).
///
/// Example:
///
/// > 19961022T140000
/// >
/// > --1022T1400
/// >
/// > ---22T14
///
/// [Section 4.3.3](https://datatracker.ietf.org/doc/html/rfc6350#section-4.3.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    date: Date,
    time: Time,
}

impl DateTime {
    /// Combines a date and a time, refusing a `date` that isn't a
    /// `date-noreduc` or a `time` that isn't a `time-notrunc`.
    pub fn new(date: Date, time: Time) -> Result<Self, ValueError> {
        if date.is_noreduc() && time.is_notrunc() {
            Ok(Self { date, time })
        } else {
            Err(ValueError::Malformed {
                expected: "date-noreduc and time-notrunc",
                received: format!("{date}T{time}"),
            })
        }
    }

    /// The date part.
    pub fn date(&self) -> Date {
        self.date
    }

    /// The time part.
    pub fn time(&self) -> Time {
        self.time
    }
}

impl TryFrom<&[u8]> for DateTime {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let err = || malformed("date-time", v);
        let (date, time) = split_designator(v).ok_or_else(err)?;
        Self::new(
            Date::try_from(date).map_err(|_| err())?,
            Time::try_from(time).map_err(|_| err())?,
        )
        .map_err(|_| err())
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}T{}", self.date, self.time)
    }
}

/// A complete date and time of day combination as specified in ISO 8601,
/// Section 4.3.2.
///
/// ```text
/// timestamp = date-complete time-designator time-complete
/// ```
///
/// Example:
///
/// > 19961022T140000
/// >
/// > 19961022T140000Z
/// >
/// > 19961022T140000-05
/// >
/// > 19961022T140000-0500
///
/// [Section 4.3.5](https://datatracker.ietf.org/doc/html/rfc6350#section-4.3.5)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp {
    date: Date,
    time: Time,
}

impl Timestamp {
    /// Combines a date and a time, refusing a `date` that isn't a
    /// `date-complete` or a `time` that isn't a `time-complete`.
    pub fn new(date: Date, time: Time) -> Result<Self, ValueError> {
        if date.is_complete() && time.is_complete() {
            Ok(Self { date, time })
        } else {
            Err(ValueError::Malformed {
                expected: "date-complete and time-complete",
                received: format!("{date}T{time}"),
            })
        }
    }

    /// The date part.
    pub fn date(&self) -> Date {
        self.date
    }

    /// The time part.
    pub fn time(&self) -> Time {
        self.time
    }
}

impl TryFrom<&[u8]> for Timestamp {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let err = || malformed("timestamp", v);
        let (date, time) = split_designator(v).ok_or_else(err)?;
        Self::new(
            Date::try_from(date).map_err(|_| err())?,
            Time::try_from(time).map_err(|_| err())?,
        )
        .map_err(|_| err())
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}T{}", self.date, self.time)
    }
}

/// Either a DATE-TIME, a DATE, or a TIME value. To allow unambiguous
/// interpretation, a stand-alone TIME value is always preceded by a "T".
///
/// ```text
/// date-and-or-time = date-time / date / time-designator time
/// ```
///
/// Example:
///
/// > 19961022T140000
/// >
/// > --1022T1400
/// >
/// > ---22T14
/// >
/// > 19850412
/// >
/// > 1985-04
/// >
/// > 1985
/// >
/// > --0412
/// >
/// > ---12
/// >
/// > T102200
/// >
/// > T1022
/// >
/// > T10
/// >
/// > T-2200
/// >
/// > T--00
/// >
/// > T102200Z
/// >
/// > T102200-0800
///
/// [Section 4.3.4](https://datatracker.ietf.org/doc/html/rfc6350#section-4.3.4)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateAndOrTime {
    /// A date and a time.
    DateTime(DateTime),
    /// A date alone.
    Date(Date),
    /// A time alone, written after a "T".
    Time(Time),
}

impl TryFrom<&[u8]> for DateAndOrTime {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        if let Some(time) = v.strip_prefix(b"T") {
            Time::try_from(time).map(Self::Time)
        } else if v.contains(&b'T') {
            DateTime::try_from(v).map(Self::DateTime)
        } else {
            Date::try_from(v).map(Self::Date)
        }
    }
}

impl fmt::Display for DateAndOrTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DateTime(v) => v.fmt(f),
            Self::Date(v) => v.fmt(f),
            Self::Time(v) => write!(f, "T{v}"),
        }
    }
}
