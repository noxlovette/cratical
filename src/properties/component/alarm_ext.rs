use crate::{
    properties::SharedParams,
    values::{DateTime, Text, ValueError},
};

/// This property specifies the UTC date and time at which the
/// corresponding alarm was last sent or acknowledged.
///
/// This property is used to specify when an alarm was last sent or
/// acknowledged.  This allows clients to determine when a pending alarm
/// has been acknowledged by a calendar user so that any alerts can be
/// dismissed across multiple devices.  It also allows clients to track
/// repeating alarms or alarms on recurring events or to-dos to ensure
/// that the right number of missed alarms can be tracked.
///
/// Example:
///
/// > ACKNOWLEDGED:20090604T084500Z
///
/// [Section 6.1](https://datatracker.ietf.org/doc/html/rfc9074#section-6.1)
#[derive(Debug)]
pub struct Acknowledged {
    value: DateTime,
    params: SharedParams,
}

impl_try_from_bytes!(Acknowledged, DateTime);

impl std::fmt::Display for Acknowledged {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ACKNOWLEDGED{}:{}", self.params, self.value)
    }
}

/// This property indicates that a location-based trigger is applied to
/// an alarm.
///
/// This property is used to indicate that an alarm has a location-based
/// trigger.  Its value identifies the action that will trigger the
/// alarm.
///
/// When the property value is set to "ARRIVE", the alarm is triggered
/// when the calendar user agent arrives in the vicinity of one or more
/// locations.  When set to "DEPART", the alarm is triggered when the
/// calendar user agent departs from the vicinity of one or more
/// locations.  Each location MUST be specified with a "VLOCATION"
/// component.  Note that the meaning of "vicinity" in this context is
/// implementation defined.
///
/// When the property value is set to "CONNECT", the alarm is triggered
/// when the calendar user agent connects to an automobile to which it
/// has been paired via Bluetooth.  When set to "DISCONNECT", the alarm
/// is triggered when the calendar user agent disconnects from an
/// automobile to which it has been paired via Bluetooth.
///
/// Example:
///
/// > PROXIMITY:DEPART
///
/// [Section 8.1](https://datatracker.ietf.org/doc/html/rfc9074#section-8.1)
#[derive(Debug)]
pub struct Proximity {
    value: ProximityEnum,
    params: SharedParams,
}

/// Possible trigger actions for [`Proximity`].
#[derive(Debug)]
pub enum ProximityEnum {
    /// Triggers on arrival in the vicinity of a location.
    Arrive,
    /// Triggers on departure from the vicinity of a location.
    Depart,
    /// Triggers on connecting to a paired automobile.
    Connect,
    /// Triggers on disconnecting from a paired automobile.
    Disconnect,
    /// An IANA-registered proximity value.
    Iana(Text),
    /// A non-standard `X-` prefixed proximity value.
    XName(Text),
}

impl_try_from_bytes!(Proximity, ProximityEnum);

impl std::fmt::Display for Proximity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PROXIMITY{}:{}", self.params, self.value)
    }
}

impl std::fmt::Display for ProximityEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arrive => f.write_str("ARRIVE"),
            Self::Depart => f.write_str("DEPART"),
            Self::Connect => f.write_str("CONNECT"),
            Self::Disconnect => f.write_str("DISCONNECT"),
            Self::Iana(t) | Self::XName(t) => f.write_str(t.as_str()),
        }
    }
}

impl TryFrom<&[u8]> for ProximityEnum {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let r = match v {
            b"ARRIVE" => Self::Arrive,
            b"DEPART" => Self::Depart,
            b"CONNECT" => Self::Connect,
            b"DISCONNECT" => Self::Disconnect,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acknowledged_parses_and_displays() {
        let line = "ACKNOWLEDGED:20090604T084500Z";
        let acknowledged =
            Acknowledged::try_from(line.as_bytes()[12..].as_ref()).unwrap();
        assert_eq!(acknowledged.to_string(), line);
    }

    #[test]
    fn proximity_parses_and_displays_the_known_values() {
        for raw in ["ARRIVE", "DEPART", "CONNECT", "DISCONNECT"] {
            let line = format!("PROXIMITY:{raw}");
            let proximity =
                Proximity::try_from(line.as_bytes()[9..].as_ref()).unwrap();
            assert_eq!(proximity.to_string(), line);
        }
    }

    #[test]
    fn proximity_falls_back_to_xname_and_iana() {
        let xname = Proximity::try_from(b":X-CUSTOM".as_slice()).unwrap();
        assert_eq!(xname.to_string(), "PROXIMITY:X-CUSTOM");

        let iana = Proximity::try_from(b":SOME-IANA-VALUE".as_slice()).unwrap();
        assert_eq!(iana.to_string(), "PROXIMITY:SOME-IANA-VALUE");
    }
}
