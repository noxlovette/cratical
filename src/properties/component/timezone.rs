use crate::{
    params::Language,
    properties::{
        ParameterError, SharedParams, param_name, param_segments, param_value,
    },
    values::{Text, Uri, UtcOffset},
};

/// This property specifies the text value that uniquely identifies the
/// "VTIMEZONE" calendar component in the scope of an iCalendar object.
///
/// Example:
///
/// > TZID:America/New_York
///
/// [Section 3.8.3.1](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.3.1)
#[derive(Debug)]
pub struct TimeZoneIdentifier {
    value: Text,
    params: SharedParams,
}

impl_try_from_bytes!(TimeZoneIdentifier);

impl TimeZoneIdentifier {
    /// The `TZID` text — used by the calendar-wide check that every `TZID`
    /// parameter used elsewhere in the `VCALENDAR` matches a `VTIMEZONE`
    /// component defined by this property (RFC 5545 §3.6.5).
    pub(crate) fn as_str(&self) -> &str {
        &self.value
    }
}

/// This property specifies the customary designation for a time zone
/// description.
///
/// Example:
///
/// > TZNAME:EST
///
/// [Section 3.8.3.2](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.3.2)
#[derive(Debug)]
pub struct TimeZoneName {
    value: Text,
    params: TZNameParams,
}

impl_try_from_bytes!(TimeZoneName, Text, TZNameParams);

#[derive(Debug, Default)]
struct TZNameParams {
    shared: SharedParams,
    // LANGUAGE is OPTIONAL on TZNAME per RFC 5545 §3.8.3.2 (every other
    // LANGUAGE-bearing params struct in this crate models it the same way).
    language: Option<Language>,
}

impl TryFrom<&[u8]> for TZNameParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"LANGUAGE" => {
                    params.language =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

/// This property specifies the offset that is in use prior to this time zone
/// observance.
///
/// Example:
///
/// > TZOFFSETFROM:-0500
///
/// [Section 3.8.3.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.3.3)
#[derive(Debug)]
pub struct TimeZoneOffsetFrom {
    value: UtcOffset,
    params: SharedParams,
}

impl_try_from_bytes!(TimeZoneOffsetFrom, UtcOffset);

/// This property specifies the UTC offset that is in use in this time zone
/// observance.
///
/// Example:
///
/// > TZOFFSETTO:-0400
///
/// [Section 3.8.3.4](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.3.4)
#[derive(Debug)]
pub struct TimeZoneOffsetTo {
    value: UtcOffset,
    params: SharedParams,
}

impl_try_from_bytes!(TimeZoneOffsetTo, UtcOffset);

/// This property provides a means for a VTIMEZONE component to point to a
/// network location that can be used to retrieve an up-to-date version of
/// itself.
///
/// Example:
///
/// > TZURL:http://timezones.example.org/tz/America-Los_Angeles.ics
///
/// [Section 3.8.3.5](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.3.5)
#[derive(Debug)]
pub struct TimeZoneUrl {
    value: Uri,
    params: SharedParams,
}

impl_try_from_bytes!(TimeZoneUrl, Uri);
