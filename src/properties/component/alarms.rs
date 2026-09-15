use crate::{
    params::{AlarmTriggerRelationship, TimeZoneIdentifier, ValueDataType},
    properties::{
        ParameterError, SharedParams, param_name, param_segments, param_value,
    },
    values::{DateTimeDuration, Integer, Text, ValueError},
};

/// This property defines the action to be invoked when an alarm is triggered.
///
/// Example:
///
/// > ACTION:AUDIO
///
/// [Section 3.8.6.1](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.6.1)
#[derive(Debug)]
pub struct Action {
    value: ActionEnum,
    params: SharedParams,
}

/// Possible alarm actions for [`Action`].
#[derive(Debug)]
pub enum ActionEnum {
    /// Play an audio clip.
    Audio,
    /// Display a text message.
    Display,
    /// Send an email message.
    Email,
    /// An IANA-registered action.
    Iana(Text),
    /// A non-standard `X-` prefixed action.
    XName(Text),
}

impl_try_from_bytes!(Action, ActionEnum);

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ACTION{}:{}", self.params, self.value)
    }
}

impl std::fmt::Display for ActionEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Audio => f.write_str("AUDIO"),
            Self::Display => f.write_str("DISPLAY"),
            Self::Email => f.write_str("EMAIL"),
            Self::Iana(t) | Self::XName(t) => f.write_str(t.as_str()),
        }
    }
}

impl Action {
    /// Which `audioprop`/`dispprop`/`emailprop` alternative (RFC 5545
    /// §3.6.6) this action selects — used by `AlarmBuilder::build` to check
    /// the alternative-specific requirements once `ACTION` is known.
    pub(crate) fn kind(&self) -> &ActionEnum {
        &self.value
    }
}

impl TryFrom<&[u8]> for ActionEnum {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let r = match v {
            b"AUDIO" => Self::Audio,
            b"DISPLAY" => Self::Display,
            b"EMAIL" => Self::Email,
            x => {
                if x.to_ascii_uppercase().starts_with(b"X-") {
                    Self::XName(x.try_into()?)
                } else {
                    Self::Iana(x.try_into()?)
                }
            }
        };
        Ok(r)
    }
}

/// This property defines the number of times the alarm should be repeated,
/// after the initial trigger.
///
/// Example:
///
/// > REPEAT:4
///
/// [Section 3.8.6.2](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.6.2)
#[derive(Debug)]
pub struct Repeat {
    value: Integer,
    params: SharedParams,
}

impl_try_from_bytes!(Repeat, Integer);

impl std::fmt::Display for Repeat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "REPEAT{}:{}", self.params, self.value)
    }
}

/// This property specifies when an alarm will trigger.
///
/// Example:
///
/// > TRIGGER:-PT15M
/// >
/// > TRIGGER;RELATED=END:PT5M
///
/// [Section 3.8.6.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.6.3)
#[derive(Debug)]
pub struct Trigger {
    value: DateTimeDuration,
    params: TriggerParams,
}

impl_try_from_bytes!(Trigger, DateTimeDuration, TriggerParams);

impl std::fmt::Display for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TRIGGER{}:{}", self.params, self.value)
    }
}

#[derive(Debug, Default)]
struct TriggerParams {
    shared: SharedParams,
    value_data_type: Option<ValueDataType>,
    tz_identifier: Option<TimeZoneIdentifier>,
    trigger_relationship: Option<AlarmTriggerRelationship>,
}

impl TryFrom<&[u8]> for TriggerParams {
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
                b"RELATED" => {
                    params.trigger_relationship =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

impl std::fmt::Display for TriggerParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.value_data_type {
            write!(f, ";VALUE={v}")?;
        }
        if let Some(v) = &self.tz_identifier {
            write!(f, ";TZID={v}")?;
        }
        if let Some(v) = &self.trigger_relationship {
            write!(f, ";RELATED={v}")?;
        }
        write!(f, "{}", self.shared)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_fixed_tokens() {
        assert!(matches!(
            ActionEnum::try_from(b"AUDIO".as_slice()),
            Ok(ActionEnum::Audio)
        ));
        assert!(matches!(
            ActionEnum::try_from(b"DISPLAY".as_slice()),
            Ok(ActionEnum::Display)
        ));
        assert!(matches!(
            ActionEnum::try_from(b"EMAIL".as_slice()),
            Ok(ActionEnum::Email)
        ));
    }

    #[test]
    fn action_x_name_and_iana() {
        assert!(matches!(
            ActionEnum::try_from(b"X-CUSTOM".as_slice()),
            Ok(ActionEnum::XName(_))
        ));
        assert!(matches!(
            ActionEnum::try_from(b"PROCEDURE".as_slice()),
            Ok(ActionEnum::Iana(_))
        ));
    }
}
