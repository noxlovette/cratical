use crate::{
    Pair,
    params::{
        Altrep, Encoding, Feature, Fmttype, ImageDisplay, Label, Language,
        ValueDataType,
    },
    properties::{
        AltrepLanguageParams, ParameterError, PropertyError, SharedParams,
        param_name, param_segments, param_value,
    },
    values::{
        Binary, Duration as DurationV, Float, Integer, Text, Uri, ValueError,
    },
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
                expected: "ATTACH value shape consistent with its \
                           ENCODING/VALUE params"
                    .into(),
                received: std::str::from_utf8(&v[colon + 1..])
                    .ok()
                    .map(Into::into),
            }
            .into());
        }

        Ok(Self { value, params })
    }
}

/// The `ATTACH` value.
#[derive(Debug)]
pub enum AttachmentValue {
    /// A URI pointing to the resource.
    Uri(Uri),
    /// The resource's content, inlined and BASE64-decoded.
    Binary(Binary),
}

impl Attachment {
    /// Constructs a new `ATTACH` property pointing to a URI, with no
    /// parameters set beyond `FMTTYPE`.
    pub fn from_uri(uri: Uri, fmttype: Option<Fmttype>) -> Self {
        Self {
            value: AttachmentValue::Uri(uri),
            params: AttachmentParams {
                shared: SharedParams::default(),
                encoding: None,
                value_data_type: None,
                fmttype,
            },
        }
    }

    /// Constructs a new `ATTACH` property with inline BASE64-encoded
    /// content, setting `ENCODING=BASE64;VALUE=BINARY` as required by RFC
    /// 5545 §3.8.1.1 for this form.
    pub fn from_binary(data: Binary, fmttype: Option<Fmttype>) -> Self {
        Self {
            value: AttachmentValue::Binary(data),
            params: AttachmentParams {
                shared: SharedParams::default(),
                encoding: Some(Encoding::Base64),
                value_data_type: Some(ValueDataType::Binary),
                fmttype,
            },
        }
    }
}

impl std::fmt::Display for Attachment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ATTACH{}:{}", self.params, self.value)
    }
}

impl std::fmt::Display for AttachmentValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uri(u) => write!(f, "{u}"),
            Self::Binary(b) => write!(f, "{b}"),
        }
    }
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
                    params.encoding =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                b"VALUE" => {
                    params.value_data_type =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                b"FMTTYPE" => {
                    params.fmttype =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

impl std::fmt::Display for AttachmentParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.encoding {
            write!(f, ";ENCODING={v}")?;
        }
        if let Some(v) = &self.value_data_type {
            write!(f, ";VALUE={v}")?;
        }
        if let Some(v) = &self.fmttype {
            write!(f, ";FMTTYPE={v}")?;
        }
        write!(f, "{}", self.shared)
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

/// Builder for [`Categories`].
#[derive(Debug, Default)]
pub struct CategoriesBuilder {
    value: Vec<Text>,
    language: Option<Language>,
}

impl CategoriesBuilder {
    /// Starts a new builder from the property's required list of
    /// categories.
    pub fn new(value: Vec<Text>) -> Self {
        Self {
            value,
            language: None,
        }
    }

    /// Sets the `LANGUAGE` parameter.
    pub fn language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    /// Finishes the builder, producing a [`Categories`].
    pub fn build(self) -> Categories {
        Categories {
            value: self.value,
            params: CategoriesParams {
                shared: SharedParams::default(),
                language: self.language,
            },
        }
    }
}

impl std::fmt::Display for Categories {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CATEGORIES{}:", self.params)?;
        crate::properties::fmt_comma_list(f, &self.value)
    }
}

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
                    params.language =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

impl std::fmt::Display for CategoriesParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.language {
            write!(f, ";LANGUAGE={v}")?;
        }
        write!(f, "{}", self.shared)
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
impl_simple_property!(Classification, ClassificationEnum);

impl std::fmt::Display for Classification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CLASS{}:{}", self.params, self.value)
    }
}

/// The `CLASS` value.
#[derive(Debug)]
pub enum ClassificationEnum {
    /// Publicly visible.
    Public,
    /// Private.
    Private,
    /// Confidential.
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

impl std::fmt::Display for ClassificationEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => f.write_str("PUBLIC"),
            Self::Private => f.write_str("PRIVATE"),
            Self::Confidential => f.write_str("CONFIDENTIAL"),
            Self::Iana(t) | Self::XName(t) => f.write_str(t.as_str()),
        }
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
impl_altrep_language_builder!(CommentBuilder, Comment, Text);

impl std::fmt::Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "COMMENT{}:{}", self.params, self.value)
    }
}

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
impl_altrep_language_builder!(DescriptionBuilder, Description, Text);

impl std::fmt::Display for Description {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DESCRIPTION{}:{}", self.params, self.value)
    }
}

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

impl Geo {
    /// Constructs a new `GEO` property, validating that the latitude is in
    /// `-90.0..=90.0`.
    pub fn new(value: Pair<Float>) -> Result<Self, PropertyError> {
        if *value.0 > 90.0 || *value.0 < -90.0 {
            return Err(PropertyError::InvalidGeo);
        }
        Ok(Self {
            value,
            params: SharedParams::default(),
        })
    }
}

impl std::fmt::Display for Geo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GEO{}:{}", self.params, self.value)
    }
}

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
impl_altrep_language_builder!(LocationBuilder, Location, Text);

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LOCATION{}:{}", self.params, self.value)
    }
}

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

impl PercentComplete {
    /// Constructs a new `PERCENT-COMPLETE` property, validating that the
    /// value is in `0..=100`.
    pub fn new(value: Integer) -> Result<Self, PropertyError> {
        if (0..=100).contains(&*value) {
            Ok(Self {
                value,
                params: SharedParams::default(),
            })
        } else {
            Err(PropertyError::InvalidPercentComplete)
        }
    }
}

impl std::fmt::Display for PercentComplete {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PERCENT-COMPLETE{}:{}", self.params, self.value)
    }
}

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

impl Priority {
    /// Constructs a new `PRIORITY` property, validating that the value is
    /// in `0..=9`.
    pub fn new(value: Integer) -> Result<Self, PropertyError> {
        if (0..=9).contains(&*value) {
            Ok(Self {
                value,
                params: SharedParams::default(),
            })
        } else {
            Err(PropertyError::InvalidPriority)
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PRIORITY{}:{}", self.params, self.value)
    }
}

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
impl_altrep_language_builder!(ResourcesBuilder, Resources, Text);

impl std::fmt::Display for Resources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RESOURCES{}:{}", self.params, self.value)
    }
}

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
impl_simple_property!(Status, StatusValue);

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "STATUS{}:{}", self.params, self.value)
    }
}

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
pub enum StatusValue {
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

impl std::fmt::Display for StatusValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Tentative => "TENTATIVE",
            Self::Confirmed => "CONFIRMED",
            Self::Cancelled => "CANCELLED",
            Self::NeedsAction => "NEEDS-ACTION",
            Self::Completed => "COMPLETED",
            Self::InProcess => "IN-PROCESS",
            Self::Draft => "DRAFT",
            Self::Final => "FINAL",
        })
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
impl_altrep_language_builder!(SummaryBuilder, Summary, Text);

impl std::fmt::Display for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SUMMARY{}:{}", self.params, self.value)
    }
}

/// This property specifies a color used for displaying the calendar,
/// event, to-do, or journal data.
///
/// This is a new property defined by \[RFC7986\], which updates RFC 5545 —
/// this crate treats it, and the rest of RFC 7986's new properties, as core
/// (always available, not behind a feature flag; see the crate-level docs).
/// It can be specified once in a `VCALENDAR` object, or once in a
/// `VEVENT`/`VTODO`/`VJOURNAL` component. The value SHOULD be one of the
/// CSS3 extended color keyword names, but this crate stores it as opaque
/// text rather than validating it against that list.
///
/// Example:
///
/// > COLOR:turquoise
///
/// [Section 5.9](https://datatracker.ietf.org/doc/html/rfc7986#section-5.9)
#[derive(Debug)]
pub struct Color {
    value: Text,
    params: SharedParams,
}

impl_try_from_bytes!(Color);
impl_simple_property!(Color, Text);

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "COLOR{}:{}", self.params, self.value)
    }
}

/// This property specifies an image associated with the calendar or a
/// calendar component. The value MUST be data with a media type of
/// "image" or refer to such data.
///
/// A new property defined by \[RFC7986\] (core in this crate — see the
/// crate-level docs). It can be specified multiple times in a `VCALENDAR`
/// object or in `VEVENT`/`VTODO`/`VJOURNAL` components — calendar
/// applications SHOULD select one to display (e.g. by resolution or
/// format) rather than showing all of them. The `DISPLAY` parameter
/// suggests how it's meant to be presented; `ALTREP` may point at a
/// clickable target for it.
///
/// Example:
///
/// > IMAGE;VALUE=URI;DISPLAY=BADGE;FMTTYPE=image/png:http://example.com/images/party.png
///
/// [Section 5.10](https://datatracker.ietf.org/doc/html/rfc7986#section-5.10)
#[derive(Debug)]
pub struct Image {
    value: AttachmentValue,
    params: ImageParams,
}

impl TryFrom<&[u8]> for Image {
    type Error = crate::ast::parser::ParseError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let colon = crate::properties::value_start(v)?;
        let params = ImageParams::try_from(&v[..colon])?;
        let value = AttachmentValue::try_from(&v[colon + 1..])?;

        // Same URI/BINARY-vs-ENCODING/VALUE cross-check as ATTACH (see
        // that type's own `TryFrom`) — IMAGE reuses its value shape
        // wholesale.
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
                expected: "IMAGE value shape consistent with its \
                           ENCODING/VALUE params"
                    .into(),
                received: std::str::from_utf8(&v[colon + 1..])
                    .ok()
                    .map(Into::into),
            }
            .into());
        }

        Ok(Self { value, params })
    }
}

impl Image {
    /// Constructs a new `IMAGE` property pointing to a URI, with no
    /// parameters set beyond `FMTTYPE`/`ALTREP`/`DISPLAY`.
    pub fn from_uri(
        uri: Uri,
        fmttype: Option<Fmttype>,
        altrep: Option<Altrep>,
        display: Option<ImageDisplay>,
    ) -> Self {
        Self {
            value: AttachmentValue::Uri(uri),
            params: ImageParams {
                shared: SharedParams::default(),
                encoding: None,
                value_data_type: None,
                fmttype,
                altrep,
                display,
            },
        }
    }

    /// Constructs a new `IMAGE` property with inline BASE64-encoded
    /// content, setting `ENCODING=BASE64;VALUE=BINARY` as required by RFC
    /// 7986 §5.10 for this form.
    pub fn from_binary(
        data: Binary,
        fmttype: Option<Fmttype>,
        display: Option<ImageDisplay>,
    ) -> Self {
        Self {
            value: AttachmentValue::Binary(data),
            params: ImageParams {
                shared: SharedParams::default(),
                encoding: Some(Encoding::Base64),
                value_data_type: Some(ValueDataType::Binary),
                fmttype,
                altrep: None,
                display,
            },
        }
    }
}

impl std::fmt::Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IMAGE{}:{}", self.params, self.value)
    }
}

#[derive(Default, Debug)]
struct ImageParams {
    shared: SharedParams,
    encoding: Option<Encoding>,
    value_data_type: Option<ValueDataType>,
    fmttype: Option<Fmttype>,
    altrep: Option<Altrep>,
    display: Option<ImageDisplay>,
}

impl TryFrom<&[u8]> for ImageParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"ENCODING" => {
                    params.encoding =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                b"VALUE" => {
                    params.value_data_type =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                b"FMTTYPE" => {
                    params.fmttype =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                b"ALTREP" => {
                    params.altrep =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                b"DISPLAY" => {
                    params.display =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                _ => params.shared.absorb(segment)?,
            }
        }
        Ok(params)
    }
}

impl std::fmt::Display for ImageParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.encoding {
            write!(f, ";ENCODING={v}")?;
        }
        if let Some(v) = &self.value_data_type {
            write!(f, ";VALUE={v}")?;
        }
        if let Some(v) = &self.fmttype {
            write!(f, ";FMTTYPE={v}")?;
        }
        if let Some(v) = &self.altrep {
            write!(f, ";ALTREP={v}")?;
        }
        if let Some(v) = &self.display {
            write!(f, ";DISPLAY={v}")?;
        }
        write!(f, "{}", self.shared)
    }
}

/// This property specifies information for accessing a conferencing
/// system, e.g. a dial-in number or a video call URI.
///
/// A new property defined by \[RFC7986\] (core in this crate — see the
/// crate-level docs). It can be specified multiple times in `VEVENT` or
/// `VTODO` components, one per access method (phone, video, chat, ...).
/// The `FEATURE` parameter describes what kind of access the URI provides;
/// `LABEL` gives a human-readable description of it.
///
/// Example:
///
/// > CONFERENCE;VALUE=URI;FEATURE=PHONE,MODERATOR;LABEL=Moderator
/// > dial-in:tel:+1-412-555-0123,,,654321
///
/// [Section 5.11](https://datatracker.ietf.org/doc/html/rfc7986#section-5.11)
#[derive(Debug)]
pub struct Conference {
    value: Uri,
    params: ConferenceParams,
}

impl_try_from_bytes!(Conference, Uri, ConferenceParams);

impl std::fmt::Display for Conference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CONFERENCE{}:{}", self.params, self.value)
    }
}

/// Builder for [`Conference`].
#[derive(Debug)]
pub struct ConferenceBuilder {
    value: Uri,
    feature: Option<Feature>,
    label: Option<Label>,
    language: Option<Language>,
}

impl ConferenceBuilder {
    /// Starts building a `CONFERENCE` property from its URI.
    pub fn new(value: Uri) -> Self {
        Self {
            value,
            feature: None,
            label: None,
            language: None,
        }
    }

    /// Sets `FEATURE`.
    pub fn feature(mut self, feature: Feature) -> Self {
        self.feature = Some(feature);
        self
    }

    /// Sets `LABEL`.
    pub fn label(mut self, label: Label) -> Self {
        self.label = Some(label);
        self
    }

    /// Sets `LANGUAGE`.
    pub fn language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    /// Finishes the builder, producing the property.
    pub fn build(self) -> Conference {
        Conference {
            value: self.value,
            params: ConferenceParams {
                shared: SharedParams::default(),
                feature: self.feature,
                label: self.label,
                language: self.language,
            },
        }
    }
}

#[derive(Default, Debug)]
struct ConferenceParams {
    shared: SharedParams,
    feature: Option<Feature>,
    label: Option<Label>,
    language: Option<Language>,
}

impl TryFrom<&[u8]> for ConferenceParams {
    type Error = ParameterError;

    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        let mut params = Self::default();
        for segment in param_segments(v) {
            match param_name(segment)?.to_ascii_uppercase().as_slice() {
                b"FEATURE" => {
                    params.feature =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
                b"LABEL" => {
                    params.label =
                        Some(param_value(segment)?.as_slice().try_into()?)
                }
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

impl std::fmt::Display for ConferenceParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.feature {
            write!(f, ";FEATURE={v}")?;
        }
        if let Some(v) = &self.label {
            write!(f, ";LABEL={v}")?;
        }
        if let Some(v) = &self.language {
            write!(f, ";LANGUAGE={v}")?;
        }
        write!(f, "{}", self.shared)
    }
}

/// This property specifies a suggested minimum interval for polling for
/// changes of the calendar data from the original source of that data.
///
/// A new property defined by \[RFC7986\] (core in this crate — see the
/// crate-level docs). It can be specified once in a `VCALENDAR` object.
/// RFC 7986's own grammar requires a `VALUE=DURATION` parameter on every
/// instance of this property; this crate doesn't enforce that on parse
/// (the value's own `dur-value` syntax is unambiguous without it) but
/// still accepts it as any other parameter would be.
///
/// Example:
///
/// > REFRESH-INTERVAL;VALUE=DURATION:PT1H
///
/// [Section 5.7](https://datatracker.ietf.org/doc/html/rfc7986#section-5.7)
#[derive(Debug)]
pub struct RefreshInterval {
    value: DurationV,
    params: SharedParams,
}

impl_try_from_bytes!(RefreshInterval, DurationV);
impl_simple_property!(RefreshInterval, DurationV);

impl std::fmt::Display for RefreshInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "REFRESH-INTERVAL{}:{}", self.params, self.value)
    }
}

/// This property identifies a source URI where calendar data can be
/// refreshed from, e.g. by a `REFRESH-INTERVAL`-driven poll.
///
/// A new property defined by \[RFC7986\] (core in this crate — see the
/// crate-level docs). It can be specified once in a `VCALENDAR` object.
///
/// Example:
///
/// > SOURCE:https://example.com/holidays.ics
///
/// [Section 5.8](https://datatracker.ietf.org/doc/html/rfc7986#section-5.8)
#[derive(Debug)]
pub struct Source {
    value: Uri,
    params: SharedParams,
}

impl_try_from_bytes!(Source, Uri);
impl_simple_property!(Source, Uri);

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SOURCE{}:{}", self.params, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_source_refresh_interval_new_match_the_parsed_equivalent() {
        assert_eq!(
            Color::new("turquoise".into()).to_string(),
            "COLOR:turquoise"
        );
        assert_eq!(
            Source::new(
                Uri::parse("https://example.com/holidays.ics").unwrap()
            )
            .to_string(),
            "SOURCE:https://example.com/holidays.ics"
        );
        assert_eq!(
            RefreshInterval::new(DurationV::new(chrono::Duration::hours(1)))
                .to_string(),
            "REFRESH-INTERVAL:PT1H"
        );
    }

    #[test]
    fn conference_builder_round_trips() {
        let conference = ConferenceBuilder::new(
            Uri::parse("tel:+1-412-555-0123,,,654321").unwrap(),
        )
        .feature(Feature::new(vec![crate::params::FeatureValue::Phone]))
        .build();
        assert_eq!(
            conference.to_string(),
            "CONFERENCE;FEATURE=PHONE:tel:+1-412-555-0123,,,654321"
        );
    }

    #[test]
    fn image_from_uri_and_from_binary_round_trip() {
        let image = Image::from_uri(
            Uri::parse("http://example.com/images/party.png").unwrap(),
            Some(Fmttype::new(crate::values::MediaType::new("image", "png"))),
            None,
            None,
        );
        assert_eq!(
            image.to_string(),
            "IMAGE;FMTTYPE=image/png:http://example.com/images/party.png"
        );

        let binary = Image::from_binary(
            Binary::try_from(b"aGVsbG8=".as_slice()).unwrap(),
            None,
            None,
        );
        assert_eq!(
            binary.to_string(),
            "IMAGE;ENCODING=BASE64;VALUE=BINARY:aGVsbG8="
        );
    }

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

    #[test]
    fn summary_display_round_trips_the_content_line() {
        let summary =
            Summary::try_from(b":Department Party".as_slice()).unwrap();
        assert_eq!(summary.to_string(), "SUMMARY:Department Party");
    }

    #[test]
    fn categories_display_round_trips_the_comma_separated_list() {
        let categories = Categories::try_from(
            b":Meeting\\, John,Work\\, Sarah,Project".as_slice(),
        )
        .unwrap();
        assert_eq!(
            categories.to_string(),
            "CATEGORIES:Meeting\\, John,Work\\, Sarah,Project"
        );
    }

    #[test]
    fn geo_display_round_trips_the_lat_lon_pair() {
        let geo = Geo::try_from(b":37.386013;-122.082932".as_slice()).unwrap();
        assert_eq!(geo.to_string(), "GEO:37.386013;-122.082932");
    }

    #[test]
    fn geo_new_matches_the_parsed_equivalent() {
        let geo =
            Geo::new(Pair::new(Float::new(37.386013), Float::new(-122.082932)))
                .unwrap();
        assert_eq!(geo.to_string(), "GEO:37.386013;-122.082932");
    }

    #[test]
    fn geo_new_rejects_out_of_range_latitude() {
        assert!(
            Geo::new(Pair::new(Float::new(91.0), Float::new(0.0))).is_err()
        );
    }

    #[test]
    fn percent_complete_new_rejects_out_of_range() {
        assert!(PercentComplete::new(Integer::new(101)).is_err());
        assert!(PercentComplete::new(Integer::new(39)).is_ok());
    }

    #[test]
    fn priority_new_rejects_out_of_range() {
        assert!(Priority::new(Integer::new(10)).is_err());
        assert!(Priority::new(Integer::new(1)).is_ok());
    }

    #[test]
    fn altrep_language_builders_round_trip() {
        let summary = SummaryBuilder::new("Department Party".into()).build();
        assert_eq!(summary.to_string(), "SUMMARY:Department Party");

        let comment = CommentBuilder::new("Hi".into())
            .altrep(crate::params::Altrep::new(
                crate::values::Uri::parse("cid:part1").unwrap(),
            ))
            .build();
        assert_eq!(comment.to_string(), "COMMENT;ALTREP=\"cid:part1\":Hi");
    }

    #[test]
    fn categories_builder_round_trips_the_comma_separated_list() {
        let categories = CategoriesBuilder::new(vec![
            "Meeting, John".into(),
            "Work, Sarah".into(),
            "Project".into(),
        ])
        .build();
        assert_eq!(
            categories.to_string(),
            "CATEGORIES:Meeting\\, John,Work\\, Sarah,Project"
        );
    }

    #[test]
    fn attachment_from_uri_round_trips() {
        let attachment = Attachment::from_uri(
            crate::values::Uri::parse("ftp://example.com/pub/docs/agenda.doc")
                .unwrap(),
            None,
        );
        assert_eq!(
            attachment.to_string(),
            "ATTACH:ftp://example.com/pub/docs/agenda.doc"
        );
    }

    #[test]
    fn attachment_from_binary_sets_encoding_and_value_params() {
        let attachment = Attachment::from_binary(
            crate::values::Binary::new(b"hello".to_vec()),
            None,
        );
        assert_eq!(
            attachment.to_string(),
            "ATTACH;ENCODING=BASE64;VALUE=BINARY:aGVsbG8="
        );
    }
}
