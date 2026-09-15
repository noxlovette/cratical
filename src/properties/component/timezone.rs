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
impl_simple_property!(TimeZoneIdentifier, Text);

impl TimeZoneIdentifier {
    /// The `TZID` text — used by the calendar-wide check that every `TZID`
    /// parameter used elsewhere in the `VCALENDAR` matches a `VTIMEZONE`
    /// component defined by this property (RFC 5545 §3.6.5).
    pub(crate) fn as_str(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for TimeZoneIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TZID{}:{}", self.params, self.value)
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

impl TimeZoneName {
    /// Constructs a new `TZNAME` property from its value and an optional
    /// `LANGUAGE` parameter.
    pub fn new(value: Text, language: Option<Language>) -> Self {
        Self {
            value,
            params: TZNameParams {
                shared: SharedParams::default(),
                language,
            },
        }
    }
}

impl std::fmt::Display for TimeZoneName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TZNAME{}:{}", self.params, self.value)
    }
}

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

impl std::fmt::Display for TZNameParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.language {
            write!(f, ";LANGUAGE={v}")?;
        }
        write!(f, "{}", self.shared)
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
impl_simple_property!(TimeZoneOffsetFrom, UtcOffset);

impl std::fmt::Display for TimeZoneOffsetFrom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TZOFFSETFROM{}:{}", self.params, self.value)
    }
}

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
impl_simple_property!(TimeZoneOffsetTo, UtcOffset);

impl std::fmt::Display for TimeZoneOffsetTo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TZOFFSETTO{}:{}", self.params, self.value)
    }
}

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
impl_simple_property!(TimeZoneUrl, Uri);

impl std::fmt::Display for TimeZoneUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TZURL{}:{}", self.params, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_constructors_match_their_parsed_equivalent() {
        assert_eq!(
            TimeZoneIdentifier::new("America/New_York".into()).to_string(),
            "TZID:America/New_York"
        );
        assert_eq!(
            TimeZoneName::new("EST".into(), None).to_string(),
            "TZNAME:EST"
        );
        let offset: crate::values::UtcOffset =
            b"-0500".as_slice().try_into().unwrap();
        assert_eq!(
            TimeZoneOffsetFrom::new(offset).to_string(),
            "TZOFFSETFROM:-0500"
        );
        assert_eq!(
            TimeZoneOffsetTo::new(offset).to_string(),
            "TZOFFSETTO:-0500"
        );
        assert_eq!(
            TimeZoneUrl::new(
                Uri::parse(
                    "http://timezones.example.org/tz/America-Los_Angeles.ics"
                )
                .unwrap()
            )
            .to_string(),
            "TZURL:http://timezones.example.org/tz/America-Los_Angeles.ics"
        );
    }
}
