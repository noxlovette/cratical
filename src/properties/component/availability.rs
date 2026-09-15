use crate::{
    properties::SharedParams,
    values::{Text, ValueError},
};

/// This property is used to specify the default busy time type.  The
/// values correspond to those used by the "FBTYPE" parameter used on a
/// "FREEBUSY" property, with the exception that the "FREE" value is not
/// used in this property.  If not specified on a component that allows
/// this property, the default is "BUSY-UNAVAILABLE".
///
/// Example:
///
/// > BUSYTYPE:BUSY
///
/// [Section 3.2](https://datatracker.ietf.org/doc/html/rfc7953#section-3.2)
#[derive(Debug)]
pub struct BusyType {
    value: BusyTypeEnum,
    params: SharedParams,
}

impl_try_from_bytes!(BusyType, BusyTypeEnum);

impl std::fmt::Display for BusyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BUSYTYPE{}:{}", self.params, self.value)
    }
}

#[derive(Debug)]
enum BusyTypeEnum {
    Busy,
    BusyUnavailable,
    BusyTentative,
    /// An IANA-registered busy type.
    Iana(Text),
    /// A non-standard `X-` prefixed busy type.
    XName(Text),
}

impl TryFrom<&[u8]> for BusyTypeEnum {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let r = match v {
            b"BUSY" => Self::Busy,
            b"BUSY-UNAVAILABLE" => Self::BusyUnavailable,
            b"BUSY-TENTATIVE" => Self::BusyTentative,
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

impl std::fmt::Display for BusyTypeEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Busy => f.write_str("BUSY"),
            Self::BusyUnavailable => f.write_str("BUSY-UNAVAILABLE"),
            Self::BusyTentative => f.write_str("BUSY-TENTATIVE"),
            Self::Iana(t) | Self::XName(t) => f.write_str(t.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_displays_the_known_values() {
        for raw in ["BUSY", "BUSY-UNAVAILABLE", "BUSY-TENTATIVE"] {
            let line = format!("BUSYTYPE:{raw}");
            let busytype =
                BusyType::try_from(line.as_bytes()[8..].as_ref()).unwrap();
            assert_eq!(busytype.to_string(), line);
        }
    }

    #[test]
    fn falls_back_to_xname_and_iana() {
        let xname = BusyType::try_from(b":X-CUSTOM".as_slice()).unwrap();
        assert_eq!(xname.to_string(), "BUSYTYPE:X-CUSTOM");

        let iana = BusyType::try_from(b":SOME-IANA-VALUE".as_slice()).unwrap();
        assert_eq!(iana.to_string(), "BUSYTYPE:SOME-IANA-VALUE");
    }
}
