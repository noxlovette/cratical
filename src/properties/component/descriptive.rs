use crate::{
    Pair,
    params::{Encoding, Fmttype, Language, ValueDataType},
    properties::{
        AltrepLanguageParams, ParameterError, PropertyError, SharedParams,
        param_name, param_segments, param_value,
    },
    values::{Binary, Float, Integer, Text, Uri, ValueError},
};

/// This property is used in "VEVENT", "VTODO", and "VJOURNAL" calendar
/// components to associate a resource (e.g., document) with the calendar
/// component.  This property is used in "VALARM" calendar components to
/// specify an audio sound resource or an email message attachment.  This
/// property can be specified as a URI pointing to a resource or as inline
/// binary encoded content.
///
/// When this property is specified as inline binary encoded content,
/// calendar applications MAY attempt to guess the media type of the resource
/// via inspection of its content if and only if the media type of the
/// resource is not given by the "FMTTYPE" parameter.  If the media type
/// remains unknown, calendar applications SHOULD treat it as type
/// "application/octet-stream".
///
/// [Section 3.8.1.1](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.1.1)
#[derive(Debug)]
pub struct Attachment {
    value: AttachmentValue,
    params: AttachmentParams,
}

impl TryFrom<&[u8]> for Attachment {
    type Error = crate::ast::parser::ParseError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let colon = crate::properties::value_start(v)?;
        let params = AttachmentParams::try_from(&v[..colon])?;
        let value = AttachmentValue::try_from(&v[colon + 1..])?;

        // ENCODING/VALUE select BASE64 inline content; anything else
        // implies a URI. The value's shape was inferred without seeing
        // these params (see `AttachmentValue::try_from`), so cross-check
        // them now that both are available.
        let declared_binary = matches!(params.encoding, Some(Encoding::Base64))
            || matches!(params.value_data_type, Some(ValueDataType::Binary));
        let declared_uri =
            matches!(params.value_data_type, Some(ValueDataType::Uri));
        let mismatch = match &value {
            AttachmentValue::Uri(_) => declared_binary,
            AttachmentValue::Binary(_) => declared_uri,
        };
        if mismatch {
            return Err(ValueError::Malformed {
                expected: "ATTACH value shape consistent with its ENCODING/VALUE params".into(),
                received: std::str::from_utf8(&v[colon + 1..]).ok().map(Into::into),
            }
            .into());
        }

        Ok(Self { value, params })
    }
}

#[derive(Debug)]
enum AttachmentValue {
    /// A URI pointing to the resource.
    Uri(Uri),
    /// The resource's content, inlined and BASE64-decoded.
    Binary(Binary),
}

impl TryFrom<&[u8]> for AttachmentValue {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        // RFC 5545 selects between these via the ENCODING/VALUE params, but
        // this `TryFrom` only sees the value bytes — `Attachment::try_from`
        // cross-checks the params against whichever shape is inferred here
        // once both are available. A valid URI always has a "scheme:"
        // prefix that inline BASE64 content cannot produce (BASE64's
        // alphabet has no ':'), so the shapes don't collide.
        if let Ok(uri) = Uri::try_from(v) {
            Ok(Self::Uri(uri))
        } else {
            Ok(Self::Binary(v.try_into()?))
        }
    }
}

#[derive(Default, Debug)]
struct AttachmentParams {
    shared: SharedParams,
    encoding: Option<Encoding>,
    value_data_type: Option<ValueDataType>,
    fmttype: Option<Fmttype>,
}

impl TryFrom<&[u8]> for AttachmentParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"ENCODING" => {
                    params.encoding = Some(param_value(segment)?.try_into()?)
                }
                b"VALUE" => {
                    params.value_data_type =
                        Some(param_value(segment)?.try_into()?)
                }
                b"FMTTYPE" => {
                    params.fmttype = Some(param_value(segment)?.try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

/// This property is used to specify categories or subtypes of the calendar
/// component.  The categories are useful in searching for a calendar
/// component of a particular type and category.  Within the "VEVENT",
/// "VTODO", or "VJOURNAL" calendar components, more than one category can
/// be specified as a COMMA-separated list of categories.
///
/// Example:
///
/// > CATEGORIES:APPOINTMENT,EDUCATION
///
/// [Section 3.8.1.2](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.1.2)
#[derive(Debug)]
pub struct Categories {
    value: Vec<Text>,
    params: CategoriesParams,
}

impl_try_from_bytes_list!(Categories, Text, CategoriesParams);

#[derive(Debug, Default)]
struct CategoriesParams {
    shared: SharedParams,
    language: Option<Language>,
}

impl TryFrom<&[u8]> for CategoriesParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"LANGUAGE" => {
                    params.language = Some(param_value(segment)?.try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

/// An access classification is only one component of the general security
/// system within a calendar application.  It provides a method of capturing
/// the scope of the access the calendar owner intends for information within
/// an individual calendar entry.  The access classification of an individual
/// iCalendar component is useful when measured along with the other security
/// components of a calendar system (e.g., calendar user authentication,
/// authorization, access rights, access role, etc.).
///
/// Hence, the semantics of the individual access classifications cannot be
/// completely defined by this memo alone.  Additionally, due to the "blind"
/// nature of most exchange processes using this memo, these access
/// classifications cannot serve as an enforcement statement for a system
/// receiving an iCalendar object.  Rather, they provide a method for
/// capturing the intention of the calendar owner for the access to the
/// calendar component.  If not specified in a component that allows this
/// property, the default value is PUBLIC.  Applications MUST treat x-name
/// and iana-token values they don't recognize the same way as they would the
/// PRIVATE value.
///
/// Example:
///
/// > CLASS:PUBLIC
///
/// [Section 3.8.1.3](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.1.3)
#[derive(Debug)]
pub struct Classification {
    value: ClassificationEnum,
    params: SharedParams,
}

impl_try_from_bytes!(Classification, ClassificationEnum);

#[derive(Debug)]
enum ClassificationEnum {
    Public,
    Private,
    Confidential,
    /// An IANA-registered classification.
    Iana(Text),
    /// A non-standard `X-` prefixed classification.
    XName(Text),
}

impl TryFrom<&[u8]> for ClassificationEnum {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let r = match v {
            b"PUBLIC" => Self::Public,
            b"PRIVATE" => Self::Private,
            b"CONFIDENTIAL" => Self::Confidential,
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

/// This property is used to specify a comment to the calendar user.
///
/// Example:
///
/// > COMMENT:The meeting really needs to include both the director and the
/// > vice-
/// > president of the division.
///
/// [Section 3.8.1.4](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.1.4)
#[derive(Debug)]
pub struct Comment {
    value: Text,
    params: AltrepLanguageParams,
}

impl_try_from_bytes!(Comment, Text, AltrepLanguageParams);

/// This property is used in the "VEVENT" and "VTODO" to capture lengthy
/// textual descriptions associated with the activity.
///
/// This property is used in the "VJOURNAL" calendar component to capture one
/// or more textual journal entries.
///
/// This property is used in the "VALARM" calendar component to capture the
/// display text for a DISPLAY category of alarm, and to capture the body
/// text for an EMAIL category of alarm.
///
/// [Section 3.8.1.5](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.1.5)
#[derive(Debug)]
pub struct Description {
    value: Text,
    params: AltrepLanguageParams,
}

impl_try_from_bytes!(Description, Text, AltrepLanguageParams);

/// This property value specifies latitude and longitude, in that order
/// (i.e., "LAT LON" ordering).  The longitude represents the location east
/// or west of the prime meridian as a positive or negative real number,
/// respectively.  The longitude and latitude values MAY be specified up to
/// six decimal places, which will allow for accuracy to within one meter of
/// geographical position.  Receiving applications MUST accept values of this
/// precision and MAY truncate values of greater precision.
///
/// Values for latitude and longitude shall be expressed as decimal fractions
/// of degrees.  Latitudes north of the equator and longitudes east of the
/// prime meridian are positive; south and west are negative.
///
/// Example:
///
/// > GEO:37.386013;-122.082932
///
/// [Section 3.8.1.6](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.1.6)
#[derive(Debug)]
pub struct Geo {
    value: Pair<Float>,
    params: SharedParams,
}

impl_try_from_bytes!(Geo, Pair<Float>, SharedParams, |f: &Pair<Float>| {
    if *f.0 > 90.0 || *f.0 < -90.0 {
        return Err(PropertyError::InvalidGeo.into());
    }

    Ok(())
});

/// Specific venues such as conference or meeting rooms may be explicitly
/// specified using this property.  An alternate representation may be
/// specified that is a URI that points to directory information with more
/// structured specification of the location.  For example, the alternate
/// representation may specify either an LDAP URL [RFC4516] pointing to an
/// LDAP server entry or a CID URL [RFC2392] pointing to a MIME body part
/// containing a Virtual-Information Card (vCard) [RFC2426] for the location.
///
/// Example:
///
/// > LOCATION:Conference Room - F123\, Bldg. 002
///
/// [Section 3.8.1.7](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.1.7)
#[derive(Debug)]
pub struct Location {
    value: Text,
    params: AltrepLanguageParams,
}

impl_try_from_bytes!(Location, Text, AltrepLanguageParams);

/// The property value is a positive integer between 0 and 100.  A value of
/// "0" indicates the to-do has not yet been started.  A value of "100"
/// indicates that the to-do has been completed.  Integer values in between
/// indicate the percent partially complete.
///
/// When a to-do is assigned to multiple individuals, the property value
/// indicates the percent complete for that portion of the to-do assigned to
/// the assignee or delegatee.
///
/// Example:
///
/// > PERCENT-COMPLETE:39
///
/// [Section 3.8.1.8](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.1.8)
#[derive(Debug)]
pub struct PercentComplete {
    value: Integer,
    params: SharedParams,
}

impl_try_from_bytes!(PercentComplete, Integer, SharedParams, |v: &Integer| {
    if (0..=100).contains(&**v) {
        Ok(())
    } else {
        Err(crate::ast::parser::ParseError::Parameter {
            expected: "PERCENT-COMPLETE value in 0..=100".into(),
            received: Some((**v).to_string()),
        })
    }
});

/// This priority is specified as an integer in the range 0 to 9.  A value
/// of 0 specifies an undefined priority.  A value of 1 is the highest
/// priority.  A value of 2 is the second highest priority.  Subsequent
/// numbers specify a decreasing ordinal priority.  A value of 9 is the
/// lowest priority.
///
/// A CUA with a three-level priority scheme of "HIGH", "MEDIUM", and "LOW"
/// is mapped into this property such that a property value in the range of
/// 1 to 4 specifies "HIGH" priority.  A value of 5 is the normal or
/// "MEDIUM" priority.  A value in the range of 6 to 9 is "LOW" priority.
///
/// Within a "VEVENT" calendar component, this property specifies a priority
/// for the event.  Within a "VTODO" calendar component, this property
/// specifies a priority for the to-do.
///
/// Example:
///
/// > PRIORITY:1
///
/// [Section 3.8.1.9](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.1.9)
#[derive(Debug)]
pub struct Priority {
    value: Integer,
    params: SharedParams,
}

impl_try_from_bytes!(Priority, Integer, SharedParams, |v: &Integer| {
    if (0..=9).contains(&**v) {
        Ok(())
    } else {
        Err(crate::ast::parser::ParseError::Parameter {
            expected: "PRIORITY value in 0..=9".to_string(),
            received: Some((**v).to_string()),
        })
    }
});

/// The property value is an arbitrary text.  More than one resource can be
/// specified as a COMMA-separated list of resources.
///
/// Example:
///
/// > RESOURCES:EASEL,PROJECTOR,VCR
///
/// [Section 3.8.1.10](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.1.10)
#[derive(Debug)]
pub struct Resources {
    value: Text,
    params: AltrepLanguageParams,
}

impl_try_from_bytes!(Resources, Text, AltrepLanguageParams);

/// In a group-scheduled calendar component, the property is used by the
/// "Organizer" to provide a confirmation of the event to the "Attendees".
/// For example in a "VEVENT" calendar component, the "Organizer" can
/// indicate that a meeting is tentative, confirmed, or cancelled.  In a
/// "VTODO" calendar component, the "Organizer" can indicate that an action
/// item needs action, is completed, is in process or being worked on, or has
/// been cancelled.  In a "VJOURNAL" calendar component, the "Organizer" can
/// indicate that a journal entry is draft, final, or has been cancelled or
/// removed.
///
/// Example:
///
/// > STATUS:TENTATIVE
///
/// [Section 3.8.1.11](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.1.11)
#[derive(Debug)]
pub struct Status {
    value: StatusValue,
    params: SharedParams,
}

impl_try_from_bytes!(Status, StatusValue);

/// The full set of `STATUS` wire tokens across `VEVENT`, `VTODO`, and
/// `VJOURNAL`. The raw property text alone doesn't say which component a
/// `STATUS` belongs to (and `CANCELLED` is valid for all three), so parsing
/// can't select a component-scoped variant the way [`Status`]'s doc implies
/// per RFC 5545 §3.8.1.11 — this flat enum carries the union of tokens
/// instead. Whether a given variant is valid for the component the
/// `STATUS` is attached to (e.g. `NEEDS-ACTION` is only valid on a
/// `VTODO`) is a validation concern for whoever builds the component, not
/// this parse step.
#[derive(Debug)]
enum StatusValue {
    /// `VEVENT`: tentatively scheduled.
    Tentative,
    /// `VEVENT`: confirmed.
    Confirmed,
    /// `VEVENT`/`VTODO`/`VJOURNAL`: cancelled.
    Cancelled,
    /// `VTODO`: not yet started.
    NeedsAction,
    /// `VTODO`: complete.
    Completed,
    /// `VTODO`: currently in process.
    InProcess,
    /// `VJOURNAL`: a draft.
    Draft,
    /// `VJOURNAL`: final.
    Final,
}

impl TryFrom<&[u8]> for StatusValue {
    type Error = ValueError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        match v {
            b"TENTATIVE" => Ok(Self::Tentative),
            b"CONFIRMED" => Ok(Self::Confirmed),
            b"CANCELLED" => Ok(Self::Cancelled),
            b"NEEDS-ACTION" => Ok(Self::NeedsAction),
            b"COMPLETED" => Ok(Self::Completed),
            b"IN-PROCESS" => Ok(Self::InProcess),
            b"DRAFT" => Ok(Self::Draft),
            b"FINAL" => Ok(Self::Final),
            _ => Err(ValueError::Malformed {
                expected: "a valid STATUS token".into(),
                received: std::str::from_utf8(v).ok().map(|s| s.into()),
            }),
        }
    }
}

/// This property is used in the "VEVENT", "VTODO", and "VJOURNAL" calendar
/// components to capture a short, one-line summary about the activity or
/// journal entry.
///
/// This property is used in the "VALARM" calendar component to capture the
/// subject of an EMAIL category of alarm.
///
/// Example:
///
/// > SUMMARY:Department Party
///
/// [Section 3.8.1.12](https://datatracker.ietf.org/doc/html/rfc5545#section-3.8.1.12)
#[derive(Debug)]
pub struct Summary {
    value: Text,
    params: AltrepLanguageParams,
}

impl_try_from_bytes!(Summary, Text, AltrepLanguageParams);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_fixed_tokens() {
        assert!(matches!(
            ClassificationEnum::try_from(b"PUBLIC".as_slice()),
            Ok(ClassificationEnum::Public)
        ));
        assert!(matches!(
            ClassificationEnum::try_from(b"PRIVATE".as_slice()),
            Ok(ClassificationEnum::Private)
        ));
        assert!(matches!(
            ClassificationEnum::try_from(b"CONFIDENTIAL".as_slice()),
            Ok(ClassificationEnum::Confidential)
        ));
    }

    #[test]
    fn classification_x_name_and_iana() {
        assert!(matches!(
            ClassificationEnum::try_from(b"X-COMPANY-INTERNAL".as_slice()),
            Ok(ClassificationEnum::XName(_))
        ));
        assert!(matches!(
            ClassificationEnum::try_from(b"SOME-IANA-TOKEN".as_slice()),
            Ok(ClassificationEnum::Iana(_))
        ));
    }

    #[test]
    fn attachment_value_uri() {
        assert!(matches!(
            AttachmentValue::try_from(
                b"ftp://example.com/pub/docs/agenda.doc".as_slice()
            ),
            Ok(AttachmentValue::Uri(_))
        ));
    }

    #[test]
    fn attachment_value_binary() {
        assert!(matches!(
            AttachmentValue::try_from(b"aGVsbG8=".as_slice()),
            Ok(AttachmentValue::Binary(_))
        ));
    }

    #[test]
    fn attachment_rejects_encoding_base64_on_a_uri_shaped_value() {
        assert!(
            Attachment::try_from(
                b";ENCODING=BASE64:ftp://example.com/pub/docs/agenda.doc"
                    .as_slice()
            )
            .is_err()
        );
    }

    #[test]
    fn attachment_rejects_value_binary_on_a_uri_shaped_value() {
        assert!(
            Attachment::try_from(
                b";VALUE=BINARY:ftp://example.com/pub/docs/agenda.doc"
                    .as_slice()
            )
            .is_err()
        );
    }

    #[test]
    fn attachment_rejects_value_uri_on_a_binary_shaped_value() {
        assert!(
            Attachment::try_from(b";VALUE=URI:aGVsbG8=".as_slice()).is_err()
        );
    }

    #[test]
    fn attachment_accepts_consistent_binary_params() {
        assert!(
            Attachment::try_from(
                b";ENCODING=BASE64;VALUE=BINARY:aGVsbG8=".as_slice()
            )
            .is_ok()
        );
    }

    #[test]
    fn attachment_accepts_a_plain_uri_with_no_params() {
        assert!(
            Attachment::try_from(
                b":ftp://example.com/pub/docs/agenda.doc".as_slice()
            )
            .is_ok()
        );
    }

    #[test]
    fn status_value_shared_cancelled() {
        assert!(matches!(
            StatusValue::try_from(b"CANCELLED".as_slice()),
            Ok(StatusValue::Cancelled)
        ));
    }

    #[test]
    fn status_value_all_tokens() {
        for (tok, matches_variant) in [
            ("TENTATIVE", "Tentative"),
            ("CONFIRMED", "Confirmed"),
            ("NEEDS-ACTION", "NeedsAction"),
            ("COMPLETED", "Completed"),
            ("IN-PROCESS", "InProcess"),
            ("DRAFT", "Draft"),
            ("FINAL", "Final"),
        ] {
            let parsed = StatusValue::try_from(tok.as_bytes());
            assert!(
                parsed.is_ok(),
                "{tok} ({matches_variant}) failed to parse"
            );
        }
    }

    #[test]
    fn status_value_rejects_unknown_token() {
        assert!(StatusValue::try_from(b"BOGUS".as_slice()).is_err());
    }

    #[test]
    fn geo_property_keeps_the_internal_semicolon_in_the_value() {
        // Regression test: the property-level macro used to split on the
        // *first* ';' in the whole buffer, which would truncate GEO's
        // "lat;lon" value at the latitude. It must split on the colon
        // instead.
        let geo = Geo::try_from(b":37.386013;-122.082932".as_slice()).unwrap();
        let Pair(lat, lon) = geo.value;
        assert_eq!(*lat, 37.386013);
        assert_eq!(*lon, -122.082932);
    }

    #[test]
    fn geo_property_with_params() {
        let geo = Geo::try_from(b";X-FOO=bar:37.386013;-122.082932".as_slice())
            .unwrap();
        assert_eq!(geo.params.xname.len(), 1);
    }

    #[test]
    fn categories_splits_on_unescaped_commas_only() {
        // Regression test for issue #2: a naive byte-level `,` split would
        // cut "Meeting\, John" into two elements instead of treating the
        // escaped comma as part of a single category.
        let categories = Categories::try_from(
            b":Meeting\\, John,Work\\, Sarah,Project".as_slice(),
        )
        .unwrap();
        let values: Vec<&str> =
            categories.value.iter().map(|t| t.as_str()).collect();
        assert_eq!(values, ["Meeting, John", "Work, Sarah", "Project"]);
    }
}
