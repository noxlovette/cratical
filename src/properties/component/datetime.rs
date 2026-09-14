use crate::{
    params::{Fbtype, TimeZoneIdentifier, ValueDataType},
    properties::{
        ParameterError, SharedParams, param_name, param_segments, param_value,
    },
    values::{
        DateOrDatetime, DateTime, Duration as DurationV, Period, ValueError,
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
                        Some(param_value(segment)?.try_into()?)
                }
                b"TZID" => {
                    params.tz_identifier =
                        Some(param_value(segment)?.try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
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
}

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

/// This property defines one or more free or busy time intervals.
///
/// Example:
///
/// > FREEBUSY;FBTYPE=BUSY-UNAVAILABLE:19970308T160000Z/PT8H30M
///
/// [Section 3.8.2.6](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.2.6)
#[derive(Debug)]
pub struct FreeBusyTime {
    value: Period,
    params: FreeBusyTimeParams,
}

impl_try_from_bytes!(FreeBusyTime, Period, FreeBusyTimeParams);

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
                    params.fb_time_type = param_value(segment)?.try_into()?
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
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
}
