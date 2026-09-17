//! The iCalendar Transport-Independent Interoperability Protocol (iTIP)
//! layers scheduling semantics on top of RFC 5545's `METHOD` property (RFC
//! 5545 itself defines no methods — see [`crate::properties::Method`]'s
//! doc). This module recognizes RFC 5546's 8 defined methods
//! ([`crate::itip::Method`]) and, whenever one of them is used, validates the
//! `VCALENDAR` against that method's property/component restriction table
//! (RFC 5546 §3) as part of [`crate::Calendar::parse`]'s normal build step.
//!
//! **Scope.** RFC 5546 mixes two kinds of rule: restrictions checkable from
//! a single iCalendar object in isolation (a property's presence/count, a
//! `STATUS` value being in some allowed set, every component in a
//! multi-component message sharing one `UID`), and restrictions that are
//! inherently stateful — "MUST match the `SEQUENCE` of the original
//! request", "MUST be the address of the Attendee replying" — which
//! require comparing against a *different*, earlier message this crate has
//! no way to know about. This module validates the former; the latter are
//! documented on the relevant [`crate::itip::ItipError`] variants but left
//! to applications that actually hold both messages.
//!
//! A `METHOD` value that isn't one of RFC 5546's 8 tokens (an `iana-token`
//! or `x-name` registered by some other specification, or simply absent)
//! isn't validated here at all — RFC 5545 §3.7.2 explicitly leaves `METHOD`
//! open to other specifications.

use crate::{
    calendar::Component as CalComponent,
    components::{
        event::Event, free_busy::FreeBusy, journal::Journal, todo::Todo,
    },
    properties::{Method as MethodProperty, Status, StatusValue},
};

/// The iCalendar object method associated with a scheduling operation.
/// iTIP methods are always originated by either the "Organizer" (`PUBLISH`,
/// `REQUEST`, `ADD`, `CANCEL`, `DECLINECOUNTER`) or an "Attendee" (`REPLY`,
/// `REFRESH`, `COUNTER`, and `REQUEST` only when delegating).
///
/// Note that for some calendar component types, the allowable methods are
/// a subset of the full set. In addition, apart from "VTIMEZONE" iCalendar
/// components, only one component type is allowed in a single iTIP
/// message.
///
/// Example:
///
/// > METHOD:REQUEST
///
/// [Section 1.4](https://datatracker.ietf.org/doc/html/rfc5546#section-1.4)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Used to publish an iCalendar object to one or more Calendar Users,
    /// with no interactivity between the publisher and any other Calendar
    /// User (e.g. a baseball team publishing its schedule to the public).
    Publish,
    /// Used to schedule an iCalendar object with other Calendar Users.
    /// Requests are interactive in that they require the receiver to
    /// respond using the reply methods. Also used by the Organizer to
    /// update the status of an iCalendar object.
    Request,
    /// Used in response to a request to convey Attendee status to the
    /// Organizer.
    Reply,
    /// Adds one or more new instances to an existing recurring iCalendar
    /// object.
    Add,
    /// Cancels one or more instances of an existing iCalendar object.
    Cancel,
    /// Used by an Attendee to request the latest version of an iCalendar
    /// object.
    Refresh,
    /// Used by an Attendee to negotiate a change in an iCalendar object
    /// (e.g. proposing a different event time).
    Counter,
    /// Used by the Organizer to decline a proposed counter proposal.
    DeclineCounter,
}

impl Method {
    /// Recognizes `method`'s raw token as one of RFC 5546's 8 defined
    /// methods, or `None` if it names something else (an `iana-token` or
    /// `x-name` from another specification, or simply not one of these 8
    /// exact tokens).
    pub fn recognized(method: &MethodProperty) -> Option<Self> {
        match method.value().as_str() {
            "PUBLISH" => Some(Self::Publish),
            "REQUEST" => Some(Self::Request),
            "REPLY" => Some(Self::Reply),
            "ADD" => Some(Self::Add),
            "CANCEL" => Some(Self::Cancel),
            "REFRESH" => Some(Self::Refresh),
            "COUNTER" => Some(Self::Counter),
            "DECLINECOUNTER" => Some(Self::DeclineCounter),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Publish => "PUBLISH",
            Self::Request => "REQUEST",
            Self::Reply => "REPLY",
            Self::Add => "ADD",
            Self::Cancel => "CANCEL",
            Self::Refresh => "REFRESH",
            Self::Counter => "COUNTER",
            Self::DeclineCounter => "DECLINECOUNTER",
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A `REQUEST-STATUS` status code, conveying the outcome of a `REPLY`,
/// `COUNTER`, or `DECLINECOUNTER` iTIP message. If `REQUEST-STATUS` isn't
/// present in one of these messages, a status code of `2.0` (success) MUST
/// be assumed.
///
/// Within any one component, the "top-level" numeric value MUST be the
/// same for all `REQUEST-STATUS` properties present (no mixing e.g. 2.x and
/// 5.x within one component). Across components in one iTIP message: if
/// any component has a 5.x code, every component must either have a code
/// in that range or omit `REQUEST-STATUS` entirely; otherwise the same
/// rule applies to 3.x codes; 2.x and 4.x codes may otherwise vary freely
/// between components.
///
/// Example:
///
/// > REQUEST-STATUS:2.0;Success
///
/// [Section 3.6](https://datatracker.ietf.org/doc/html/rfc5546#section-3.6)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    /// iTIP operation succeeded.
    Success,
    /// iTIP operation succeeded with fallback on one or more property
    /// values.
    SuccessFallback,
    /// iTIP operation succeeded but a property was ignored.
    SuccessPropertyIgnored,
    /// iTIP operation succeeded but a property parameter was ignored
    /// because it was invalid.
    SuccessParameterIgnored,
    /// iTIP operation succeeded but a non-standard property was ignored
    /// because it was unknown.
    SuccessUnknownPropertyIgnored,
    /// iTIP operation succeeded but a property was ignored because its
    /// non-standard value was unknown.
    SuccessUnknownValueIgnored,
    /// iTIP operation succeeded but an invalid calendar component was
    /// ignored.
    SuccessComponentIgnored,
    /// iTIP operation succeeded; the request was forwarded to the Calendar
    /// User.
    SuccessForwarded,
    /// iTIP operation succeeded; a repeating event was ignored and
    /// scheduled as a single component.
    SuccessRepeatingEventIgnored,
    /// iTIP operation succeeded; the end date-time was truncated to a date
    /// boundary.
    SuccessTruncatedEndDateTime,
    /// iTIP operation succeeded; a repeating `VTODO` was ignored and
    /// scheduled as a single `VTODO`.
    SuccessRepeatingTodoIgnored,
    /// iTIP operation succeeded; an unbounded `RRULE` was clipped at some
    /// finite number of instances.
    SuccessRruleClipped,
    /// Invalid property name.
    InvalidPropertyName,
    /// Invalid property value.
    InvalidPropertyValue,
    /// Invalid property parameter.
    InvalidPropertyParameter,
    /// Invalid property parameter value.
    InvalidPropertyParameterValue,
    /// Invalid calendar component sequence.
    InvalidComponentSequence,
    /// Invalid date or time.
    InvalidDateTime,
    /// Invalid rule.
    InvalidRule,
    /// Invalid Calendar User.
    InvalidCalendarUser,
    /// No authority.
    NoAuthority,
    /// Unsupported version.
    UnsupportedVersion,
    /// Request entity too large.
    RequestEntityTooLarge,
    /// Required component or property missing.
    RequiredComponentOrPropertyMissing,
    /// Unknown component or property found.
    UnknownComponentOrProperty,
    /// Unsupported component or property found.
    UnsupportedComponentOrProperty,
    /// Unsupported capability.
    UnsupportedCapability,
    /// Event conflict; date/time is busy.
    EventConflict,
    /// Request not supported.
    RequestNotSupported,
    /// Service unavailable.
    ServiceUnavailable,
    /// Invalid calendar service.
    InvalidCalendarService,
    /// No scheduling support for user.
    NoSchedulingSupport,
}

impl StatusCode {
    /// The numeric `(major, minor)` pair this code renders as, e.g. `(2,
    /// 0)` for `2.0`.
    pub fn code(self) -> (u8, u8) {
        match self {
            Self::Success => (2, 0),
            Self::SuccessFallback => (2, 1),
            Self::SuccessPropertyIgnored => (2, 2),
            Self::SuccessParameterIgnored => (2, 3),
            Self::SuccessUnknownPropertyIgnored => (2, 4),
            Self::SuccessUnknownValueIgnored => (2, 5),
            Self::SuccessComponentIgnored => (2, 6),
            Self::SuccessForwarded => (2, 7),
            Self::SuccessRepeatingEventIgnored => (2, 8),
            Self::SuccessTruncatedEndDateTime => (2, 9),
            Self::SuccessRepeatingTodoIgnored => (2, 10),
            Self::SuccessRruleClipped => (2, 11),
            Self::InvalidPropertyName => (3, 0),
            Self::InvalidPropertyValue => (3, 1),
            Self::InvalidPropertyParameter => (3, 2),
            Self::InvalidPropertyParameterValue => (3, 3),
            Self::InvalidComponentSequence => (3, 4),
            Self::InvalidDateTime => (3, 5),
            Self::InvalidRule => (3, 6),
            Self::InvalidCalendarUser => (3, 7),
            Self::NoAuthority => (3, 8),
            Self::UnsupportedVersion => (3, 9),
            Self::RequestEntityTooLarge => (3, 10),
            Self::RequiredComponentOrPropertyMissing => (3, 11),
            Self::UnknownComponentOrProperty => (3, 12),
            Self::UnsupportedComponentOrProperty => (3, 13),
            Self::UnsupportedCapability => (3, 14),
            Self::EventConflict => (4, 0),
            Self::RequestNotSupported => (5, 0),
            Self::ServiceUnavailable => (5, 1),
            Self::InvalidCalendarService => (5, 2),
            Self::NoSchedulingSupport => (5, 3),
        }
    }

    /// The RFC 5546 §3.6 "Status Description" text for this code.
    pub fn description(self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::SuccessFallback => {
                "Success, but fallback taken on one or more property values"
            }
            Self::SuccessPropertyIgnored => "Success; invalid property ignored",
            Self::SuccessParameterIgnored => {
                "Success; invalid property parameter ignored"
            }
            Self::SuccessUnknownPropertyIgnored => {
                "Success; unknown, non-standard property ignored"
            }
            Self::SuccessUnknownValueIgnored => {
                "Success; unknown, non-standard property value ignored"
            }
            Self::SuccessComponentIgnored => {
                "Success; invalid calendar component ignored"
            }
            Self::SuccessForwarded => {
                "Success; request forwarded to Calendar User"
            }
            Self::SuccessRepeatingEventIgnored => {
                "Success; repeating event ignored. Scheduled as a single \
                 component"
            }
            Self::SuccessTruncatedEndDateTime => {
                "Success; truncated end date time to date boundary"
            }
            Self::SuccessRepeatingTodoIgnored => {
                "Success; repeating VTODO ignored. Scheduled as a single VTODO"
            }
            Self::SuccessRruleClipped => {
                "Success; unbounded RRULE clipped at some finite number of \
                 instances"
            }
            Self::InvalidPropertyName => "Invalid property name",
            Self::InvalidPropertyValue => "Invalid property value",
            Self::InvalidPropertyParameter => "Invalid property parameter",
            Self::InvalidPropertyParameterValue => {
                "Invalid property parameter value"
            }
            Self::InvalidComponentSequence => {
                "Invalid calendar component sequence"
            }
            Self::InvalidDateTime => "Invalid date or time",
            Self::InvalidRule => "Invalid rule",
            Self::InvalidCalendarUser => "Invalid Calendar User",
            Self::NoAuthority => "No authority",
            Self::UnsupportedVersion => "Unsupported version",
            Self::RequestEntityTooLarge => "Request entity too large",
            Self::RequiredComponentOrPropertyMissing => {
                "Required component or property missing"
            }
            Self::UnknownComponentOrProperty => {
                "Unknown component or property found"
            }
            Self::UnsupportedComponentOrProperty => {
                "Unsupported component or property found"
            }
            Self::UnsupportedCapability => "Unsupported capability",
            Self::EventConflict => "Event conflict. Date/time is busy",
            Self::RequestNotSupported => "Request not supported",
            Self::ServiceUnavailable => "Service unavailable",
            Self::InvalidCalendarService => "Invalid calendar service",
            Self::NoSchedulingSupport => "No scheduling support for user",
        }
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (major, minor) = self.code();
        write!(f, "{major}.{minor}")
    }
}

/// An RFC 5546 constraint was violated by a `VCALENDAR` whose `METHOD`
/// names one of the 8 recognized iTIP methods ([`Method::recognized`]).
///
/// This only covers restrictions checkable from a single iCalendar object
/// — see this module's doc for why some of RFC 5546's rules (anything
/// requiring comparison against an earlier message) aren't checked here at
/// all.
#[derive(Debug, thiserror::Error)]
pub enum ItipError {
    /// A property RFC 5546 requires for this `METHOD` wasn't present.
    #[error("{component} MUST include {property} for METHOD:{method}")]
    MissingProperty {
        /// The iTIP method in effect.
        method: &'static str,
        /// The component the property is missing from.
        component: &'static str,
        /// The missing property's name.
        property: &'static str,
    },

    /// A property RFC 5546 prohibits for this `METHOD` was present.
    #[error("{component} MUST NOT include {property} for METHOD:{method}")]
    ProhibitedProperty {
        /// The iTIP method in effect.
        method: &'static str,
        /// The component the property was found on.
        component: &'static str,
        /// The prohibited property's name.
        property: &'static str,
    },

    /// A property's cardinality didn't match what RFC 5546 requires for
    /// this `METHOD` (e.g. `ATTENDEE` MUST occur exactly once on a
    /// `REPLY`).
    #[error("{component}'s {property} {expected} for METHOD:{method}")]
    WrongCardinality {
        /// The iTIP method in effect.
        method: &'static str,
        /// The component the property is on.
        component: &'static str,
        /// The property whose count is wrong.
        property: &'static str,
        /// A human-readable description of the required count.
        expected: &'static str,
    },

    /// The number of primary components (`VEVENT`/`VTODO`/`VJOURNAL`/
    /// `VFREEBUSY`) didn't match what RFC 5546 requires for this `METHOD`
    /// (e.g. exactly one `VEVENT` for `ADD`, but one or more for
    /// `REQUEST`).
    #[error("METHOD:{method} requires {expected} {component} component(s)")]
    ComponentCount {
        /// The iTIP method in effect.
        method: &'static str,
        /// The component type being counted.
        component: &'static str,
        /// A human-readable description of the required count.
        expected: &'static str,
    },

    /// `SEQUENCE` was present but not greater than 0, where RFC 5546
    /// requires it to be (`ADD`).
    #[error(
        "{component}'s SEQUENCE MUST be greater than 0 for METHOD:{method}"
    )]
    SequenceMustBePositive {
        /// The iTIP method in effect.
        method: &'static str,
        /// The component whose `SEQUENCE` must be positive.
        component: &'static str,
    },

    /// `STATUS`'s value wasn't among the values RFC 5546 allows for this
    /// `METHOD`.
    #[error(
        "{component}'s STATUS MUST be one of {allowed} for METHOD:{method}"
    )]
    InvalidStatus {
        /// The iTIP method in effect.
        method: &'static str,
        /// The component whose `STATUS` is invalid.
        component: &'static str,
        /// A human-readable description of the allowed values.
        allowed: &'static str,
    },

    /// This `METHOD` has no semantics defined for this component type
    /// (e.g. `REFRESH` is only defined for `VEVENT`/`VTODO`, not
    /// `VJOURNAL`).
    #[error("METHOD:{method} is not defined for {component}")]
    UnsupportedMethod {
        /// The iTIP method in effect.
        method: &'static str,
        /// The component type the method doesn't apply to.
        component: &'static str,
    },

    /// A `VCALENDAR` mixed components of more than one primary type under
    /// one `METHOD` — apart from `VTIMEZONE`, iTIP restricts a message to
    /// a single component type (RFC 5546 §1.4).
    #[error("METHOD:{0} MUST NOT mix component types in one VCALENDAR")]
    MixedComponentTypes(&'static str),

    /// Several components sharing one scheduling operation didn't all
    /// carry the same `UID`, where RFC 5546 requires them to (e.g. the
    /// several `VEVENT`s of a `REQUEST` describing one recurring series).
    #[error(
        "all {component} components MUST share the same UID for \
         METHOD:{method}"
    )]
    InconsistentUid {
        /// The iTIP method in effect.
        method: &'static str,
        /// The component type that must share a `UID`.
        component: &'static str,
    },
}

fn require_present<T>(
    value: Option<&T>,
    method: &'static str,
    component: &'static str,
    property: &'static str,
) -> Result<(), ItipError> {
    if value.is_none() {
        return Err(ItipError::MissingProperty {
            method,
            component,
            property,
        });
    }
    Ok(())
}

fn require_absent<T>(
    value: Option<&T>,
    method: &'static str,
    component: &'static str,
    property: &'static str,
) -> Result<(), ItipError> {
    if value.is_some() {
        return Err(ItipError::ProhibitedProperty {
            method,
            component,
            property,
        });
    }
    Ok(())
}

fn require_empty<T>(
    values: &[T],
    method: &'static str,
    component: &'static str,
    property: &'static str,
) -> Result<(), ItipError> {
    if !values.is_empty() {
        return Err(ItipError::ProhibitedProperty {
            method,
            component,
            property,
        });
    }
    Ok(())
}

fn require_nonempty<T>(
    values: &[T],
    method: &'static str,
    component: &'static str,
    property: &'static str,
) -> Result<(), ItipError> {
    if values.is_empty() {
        return Err(ItipError::MissingProperty {
            method,
            component,
            property,
        });
    }
    Ok(())
}

fn require_exactly_one<T>(
    values: &[T],
    method: &'static str,
    component: &'static str,
    property: &'static str,
) -> Result<(), ItipError> {
    if values.len() != 1 {
        return Err(ItipError::WrongCardinality {
            method,
            component,
            property,
            expected: "MUST occur exactly once",
        });
    }
    Ok(())
}

fn require_status_in(
    status: Option<&Status>,
    allowed: &[fn(&StatusValue) -> bool],
    allowed_desc: &'static str,
    method: &'static str,
    component: &'static str,
) -> Result<(), ItipError> {
    if let Some(status) = status
        && !allowed.iter().any(|pred| pred(status.kind()))
    {
        return Err(ItipError::InvalidStatus {
            method,
            component,
            allowed: allowed_desc,
        });
    }
    Ok(())
}

fn same_uid<'a>(
    mut uids: impl Iterator<Item = &'a str>,
    method: &'static str,
    component: &'static str,
) -> Result<(), ItipError> {
    if let Some(first) = uids.next()
        && uids.any(|uid| uid != first)
    {
        return Err(ItipError::InconsistentUid { method, component });
    }
    Ok(())
}

/// Validates `components` against RFC 5546's restriction tables for
/// `method`, if `method_prop`'s value names one of the 8 recognized iTIP
/// methods (a no-op otherwise). Called once from `CalendarBuilder::build`
/// after every component has already been individually built (so RFC 5545
/// core validation has already run).
pub(crate) fn validate(
    method_prop: &MethodProperty,
    components: &[CalComponent],
) -> Result<(), ItipError> {
    let Some(method) = Method::recognized(method_prop) else {
        return Ok(());
    };

    // RFC 5546 predates RFC 7953's VAVAILABILITY and says nothing about
    // it; skip entirely rather than guess at semantics the RFC doesn't
    // define.
    #[cfg(feature = "rfc-7953")]
    if components
        .iter()
        .any(|c| matches!(c, CalComponent::Availability(_)))
    {
        return Ok(());
    }

    let events: Vec<&Event> = components
        .iter()
        .filter_map(|c| match c {
            CalComponent::Event(e) => Some(e),
            _ => None,
        })
        .collect();
    let todos: Vec<&Todo> = components
        .iter()
        .filter_map(|c| match c {
            CalComponent::Todo(t) => Some(t),
            _ => None,
        })
        .collect();
    let journals: Vec<&Journal> = components
        .iter()
        .filter_map(|c| match c {
            CalComponent::Journal(j) => Some(j),
            _ => None,
        })
        .collect();
    let freebusys: Vec<&FreeBusy> = components
        .iter()
        .filter_map(|c| match c {
            CalComponent::FreeBusy(f) => Some(f),
            _ => None,
        })
        .collect();

    let kinds_present = [
        !events.is_empty(),
        !todos.is_empty(),
        !journals.is_empty(),
        !freebusys.is_empty(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if kinds_present > 1 {
        return Err(ItipError::MixedComponentTypes(method.as_str()));
    }

    if !events.is_empty() {
        validate_vevent(method, &events)
    } else if !todos.is_empty() {
        validate_vtodo(method, &todos)
    } else if !journals.is_empty() {
        validate_vjournal(method, &journals)
    } else if !freebusys.is_empty() {
        validate_vfreebusy(method, &freebusys)
    } else {
        // Nothing to validate against (e.g. a VTIMEZONE-only object).
        Ok(())
    }
}

fn tentative(v: &StatusValue) -> bool {
    matches!(v, StatusValue::Tentative)
}
fn confirmed(v: &StatusValue) -> bool {
    matches!(v, StatusValue::Confirmed)
}
fn cancelled(v: &StatusValue) -> bool {
    matches!(v, StatusValue::Cancelled)
}

/// RFC 5546 §3.2: `VEVENT` supports all 8 methods, so no
/// [`ItipError::UnsupportedMethod`] case exists here.
fn validate_vevent(method: Method, events: &[&Event]) -> Result<(), ItipError> {
    const C: &str = "VEVENT";
    let m = method.as_str();

    match method {
        Method::Publish => {
            for e in events {
                require_present(e.dtstart(), m, C, "DTSTART")?;
                require_present(e.organizer(), m, C, "ORGANIZER")?;
                require_present(e.summary(), m, C, "SUMMARY")?;
                require_empty(e.attendee(), m, C, "ATTENDEE")?;
                require_empty(e.rstatus(), m, C, "REQUEST-STATUS")?;
                require_status_in(
                    e.status(),
                    &[tentative, confirmed, cancelled],
                    "TENTATIVE/CONFIRMED/CANCELLED",
                    m,
                    C,
                )?;
            }
        }
        Method::Request => {
            for e in events {
                require_nonempty(e.attendee(), m, C, "ATTENDEE")?;
                require_present(e.dtstart(), m, C, "DTSTART")?;
                require_present(e.organizer(), m, C, "ORGANIZER")?;
                require_present(e.summary(), m, C, "SUMMARY")?;
                require_empty(e.rstatus(), m, C, "REQUEST-STATUS")?;
                require_status_in(
                    e.status(),
                    &[tentative, confirmed],
                    "TENTATIVE/CONFIRMED",
                    m,
                    C,
                )?;
            }
            same_uid(events.iter().map(|e| e.uid().as_str()), m, C)?;
        }
        Method::Reply => {
            for e in events {
                require_exactly_one(e.attendee(), m, C, "ATTENDEE")?;
                require_present(e.organizer(), m, C, "ORGANIZER")?;
                require_empty(e.alarms(), m, C, "VALARM")?;
            }
            same_uid(events.iter().map(|e| e.uid().as_str()), m, C)?;
        }
        Method::Add => {
            if events.len() != 1 {
                return Err(ItipError::ComponentCount {
                    method: m,
                    component: C,
                    expected: "exactly one",
                });
            }
            let e = events[0];
            require_present(e.dtstart(), m, C, "DTSTART")?;
            require_present(e.organizer(), m, C, "ORGANIZER")?;
            match e.seq() {
                None => {
                    return Err(ItipError::MissingProperty {
                        method: m,
                        component: C,
                        property: "SEQUENCE",
                    });
                }
                Some(seq) if **seq.value() <= 0 => {
                    return Err(ItipError::SequenceMustBePositive {
                        method: m,
                        component: C,
                    });
                }
                _ => {}
            }
            require_present(e.summary(), m, C, "SUMMARY")?;
            require_empty(e.exdate(), m, C, "EXDATE")?;
            require_absent(e.recurid(), m, C, "RECURRENCE-ID")?;
            require_empty(e.rstatus(), m, C, "REQUEST-STATUS")?;
            require_empty(e.rdate(), m, C, "RDATE")?;
            require_absent(e.rrule(), m, C, "RRULE")?;
            require_status_in(
                e.status(),
                &[tentative, confirmed],
                "TENTATIVE/CONFIRMED",
                m,
                C,
            )?;
        }
        Method::Cancel => {
            for e in events {
                require_present(e.organizer(), m, C, "ORGANIZER")?;
                require_present(e.seq(), m, C, "SEQUENCE")?;
                require_empty(e.rstatus(), m, C, "REQUEST-STATUS")?;
                require_empty(e.alarms(), m, C, "VALARM")?;
            }
            same_uid(events.iter().map(|e| e.uid().as_str()), m, C)?;
        }
        Method::Refresh => {
            if events.len() != 1 {
                return Err(ItipError::ComponentCount {
                    method: m,
                    component: C,
                    expected: "exactly one",
                });
            }
            let e = events[0];
            require_exactly_one(e.attendee(), m, C, "ATTENDEE")?;
            require_present(e.organizer(), m, C, "ORGANIZER")?;
            require_empty(e.attach(), m, C, "ATTACH")?;
            require_empty(e.categories(), m, C, "CATEGORIES")?;
            require_absent(e.class(), m, C, "CLASS")?;
            require_empty(e.contact(), m, C, "CONTACT")?;
            require_absent(e.created(), m, C, "CREATED")?;
            require_absent(e.description(), m, C, "DESCRIPTION")?;
            require_absent(e.dtend(), m, C, "DTEND")?;
            require_absent(e.dtstart(), m, C, "DTSTART")?;
            require_absent(e.duration(), m, C, "DURATION")?;
            require_empty(e.exdate(), m, C, "EXDATE")?;
            require_absent(e.geo(), m, C, "GEO")?;
            require_absent(e.last_mod(), m, C, "LAST-MODIFIED")?;
            require_absent(e.location(), m, C, "LOCATION")?;
            require_absent(e.priority(), m, C, "PRIORITY")?;
            require_empty(e.rdate(), m, C, "RDATE")?;
            require_empty(e.related(), m, C, "RELATED-TO")?;
            require_empty(e.rstatus(), m, C, "REQUEST-STATUS")?;
            require_empty(e.resources(), m, C, "RESOURCES")?;
            require_absent(e.rrule(), m, C, "RRULE")?;
            require_absent(e.seq(), m, C, "SEQUENCE")?;
            require_absent(e.status(), m, C, "STATUS")?;
            require_absent(e.summary(), m, C, "SUMMARY")?;
            require_absent(e.transp(), m, C, "TRANSP")?;
            require_absent(e.url(), m, C, "URL")?;
            require_empty(e.alarms(), m, C, "VALARM")?;
        }
        Method::Counter => {
            if events.len() != 1 {
                return Err(ItipError::ComponentCount {
                    method: m,
                    component: C,
                    expected: "exactly one",
                });
            }
            let e = events[0];
            require_present(e.dtstart(), m, C, "DTSTART")?;
            require_present(e.organizer(), m, C, "ORGANIZER")?;
            require_present(e.seq(), m, C, "SEQUENCE")?;
            require_present(e.summary(), m, C, "SUMMARY")?;
            require_status_in(
                e.status(),
                &[confirmed, tentative, cancelled],
                "CONFIRMED/TENTATIVE/CANCELLED",
                m,
                C,
            )?;
        }
        Method::DeclineCounter => {
            for e in events {
                require_nonempty(e.attendee(), m, C, "ATTENDEE")?;
                require_present(e.organizer(), m, C, "ORGANIZER")?;
                require_present(e.seq(), m, C, "SEQUENCE")?;
                require_empty(e.alarms(), m, C, "VALARM")?;
                require_status_in(
                    e.status(),
                    &[tentative, confirmed],
                    "TENTATIVE/CONFIRMED",
                    m,
                    C,
                )?;
            }
            same_uid(events.iter().map(|e| e.uid().as_str()), m, C)?;
        }
    }
    Ok(())
}

fn validate_vtodo(method: Method, todos: &[&Todo]) -> Result<(), ItipError> {
    const C: &str = "VTODO";
    let m = method.as_str();

    match method {
        Method::Publish => {
            for t in todos {
                require_present(t.dtstart(), m, C, "DTSTART")?;
                require_present(t.organizer(), m, C, "ORGANIZER")?;
                require_present(t.priority(), m, C, "PRIORITY")?;
                require_present(t.summary(), m, C, "SUMMARY")?;
                require_empty(t.attendee(), m, C, "ATTENDEE")?;
                require_empty(t.rstatus(), m, C, "REQUEST-STATUS")?;
                require_status_in(
                    t.status(),
                    &[
                        |v| matches!(v, StatusValue::Completed),
                        |v| matches!(v, StatusValue::NeedsAction),
                        |v| matches!(v, StatusValue::InProcess),
                        cancelled,
                    ],
                    "COMPLETED/NEEDS-ACTION/IN-PROCESS/CANCELLED",
                    m,
                    C,
                )?;
            }
        }
        Method::Request => {
            for t in todos {
                require_nonempty(t.attendee(), m, C, "ATTENDEE")?;
                require_present(t.dtstart(), m, C, "DTSTART")?;
                require_present(t.organizer(), m, C, "ORGANIZER")?;
                require_present(t.priority(), m, C, "PRIORITY")?;
                require_present(t.summary(), m, C, "SUMMARY")?;
                require_empty(t.rstatus(), m, C, "REQUEST-STATUS")?;
                require_status_in(
                    t.status(),
                    &[
                        |v| matches!(v, StatusValue::Completed),
                        |v| matches!(v, StatusValue::NeedsAction),
                        |v| matches!(v, StatusValue::InProcess),
                    ],
                    "COMPLETED/NEEDS-ACTION/IN-PROCESS",
                    m,
                    C,
                )?;
            }
            same_uid(todos.iter().map(|t| t.uid().as_str()), m, C)?;
        }
        Method::Reply => {
            for t in todos {
                require_exactly_one(t.attendee(), m, C, "ATTENDEE")?;
                require_present(t.organizer(), m, C, "ORGANIZER")?;
                require_empty(t.alarms(), m, C, "VALARM")?;
            }
            same_uid(todos.iter().map(|t| t.uid().as_str()), m, C)?;
        }
        Method::Add => {
            if todos.len() != 1 {
                return Err(ItipError::ComponentCount {
                    method: m,
                    component: C,
                    expected: "exactly one",
                });
            }
            let t = todos[0];
            require_present(t.organizer(), m, C, "ORGANIZER")?;
            require_present(t.priority(), m, C, "PRIORITY")?;
            match t.seq() {
                None => {
                    return Err(ItipError::MissingProperty {
                        method: m,
                        component: C,
                        property: "SEQUENCE",
                    });
                }
                Some(seq) if **seq.value() <= 0 => {
                    return Err(ItipError::SequenceMustBePositive {
                        method: m,
                        component: C,
                    });
                }
                _ => {}
            }
            require_present(t.summary(), m, C, "SUMMARY")?;
            require_empty(t.exdate(), m, C, "EXDATE")?;
            require_absent(t.recur_id(), m, C, "RECURRENCE-ID")?;
            require_empty(t.rstatus(), m, C, "REQUEST-STATUS")?;
            require_empty(t.rdate(), m, C, "RDATE")?;
            require_absent(t.rrule(), m, C, "RRULE")?;
            require_status_in(
                t.status(),
                &[
                    |v| matches!(v, StatusValue::Completed),
                    |v| matches!(v, StatusValue::NeedsAction),
                    |v| matches!(v, StatusValue::InProcess),
                ],
                "COMPLETED/NEEDS-ACTION/IN-PROCESS",
                m,
                C,
            )?;
        }
        Method::Cancel => {
            for t in todos {
                require_present(t.organizer(), m, C, "ORGANIZER")?;
                require_present(t.seq(), m, C, "SEQUENCE")?;
                require_empty(t.rstatus(), m, C, "REQUEST-STATUS")?;
                require_empty(t.alarms(), m, C, "VALARM")?;
            }
        }
        Method::Refresh => {
            if todos.len() != 1 {
                return Err(ItipError::ComponentCount {
                    method: m,
                    component: C,
                    expected: "exactly one",
                });
            }
            let t = todos[0];
            require_exactly_one(t.attendee(), m, C, "ATTENDEE")?;
            require_absent(t.organizer(), m, C, "ORGANIZER")?;
            require_empty(t.attach(), m, C, "ATTACH")?;
            require_empty(t.categories(), m, C, "CATEGORIES")?;
            require_absent(t.class(), m, C, "CLASS")?;
            require_empty(t.comment(), m, C, "COMMENT")?;
            require_absent(t.completed(), m, C, "COMPLETED")?;
            require_empty(t.contact(), m, C, "CONTACT")?;
            require_absent(t.created(), m, C, "CREATED")?;
            require_absent(t.description(), m, C, "DESCRIPTION")?;
            require_absent(t.dtstart(), m, C, "DTSTART")?;
            require_absent(t.due(), m, C, "DUE")?;
            require_absent(t.duration(), m, C, "DURATION")?;
            require_absent(t.geo(), m, C, "GEO")?;
            require_absent(t.last_mod(), m, C, "LAST-MODIFIED")?;
            require_absent(t.location(), m, C, "LOCATION")?;
            require_absent(t.percent(), m, C, "PERCENT-COMPLETE")?;
            require_absent(t.priority(), m, C, "PRIORITY")?;
            require_empty(t.related(), m, C, "RELATED-TO")?;
            require_empty(t.rstatus(), m, C, "REQUEST-STATUS")?;
            require_empty(t.resources(), m, C, "RESOURCES")?;
            require_absent(t.rrule(), m, C, "RRULE")?;
            require_absent(t.seq(), m, C, "SEQUENCE")?;
            require_absent(t.status(), m, C, "STATUS")?;
            require_absent(t.url(), m, C, "URL")?;
            require_empty(t.alarms(), m, C, "VALARM")?;
        }
        Method::Counter => {
            if todos.len() != 1 {
                return Err(ItipError::ComponentCount {
                    method: m,
                    component: C,
                    expected: "exactly one",
                });
            }
            let t = todos[0];
            require_nonempty(t.attendee(), m, C, "ATTENDEE")?;
            require_present(t.organizer(), m, C, "ORGANIZER")?;
            require_present(t.priority(), m, C, "PRIORITY")?;
            require_present(t.summary(), m, C, "SUMMARY")?;
            require_status_in(
                t.status(),
                &[
                    |v| matches!(v, StatusValue::Completed),
                    |v| matches!(v, StatusValue::NeedsAction),
                    |v| matches!(v, StatusValue::InProcess),
                    cancelled,
                ],
                "COMPLETED/NEEDS-ACTION/IN-PROCESS/CANCELLED",
                m,
                C,
            )?;
        }
        Method::DeclineCounter => {
            if todos.len() != 1 {
                return Err(ItipError::ComponentCount {
                    method: m,
                    component: C,
                    expected: "exactly one",
                });
            }
            let t = todos[0];
            require_nonempty(t.attendee(), m, C, "ATTENDEE")?;
            require_present(t.organizer(), m, C, "ORGANIZER")?;
            require_present(t.seq(), m, C, "SEQUENCE")?;
            require_empty(t.alarms(), m, C, "VALARM")?;
            require_status_in(
                t.status(),
                &[
                    |v| matches!(v, StatusValue::Completed),
                    |v| matches!(v, StatusValue::NeedsAction),
                    |v| matches!(v, StatusValue::InProcess),
                ],
                "COMPLETED/NEEDS-ACTION/IN-PROCESS",
                m,
                C,
            )?;
        }
    }
    Ok(())
}

/// RFC 5546 §3.5: `VJOURNAL` only defines `PUBLISH`/`ADD`/`CANCEL` — every
/// other method has no meaning for it.
fn validate_vjournal(
    method: Method,
    journals: &[&Journal],
) -> Result<(), ItipError> {
    const C: &str = "VJOURNAL";
    let m = method.as_str();

    match method {
        Method::Publish => {
            for j in journals {
                require_exactly_one(j.description(), m, C, "DESCRIPTION")?;
                require_present(j.dtstart(), m, C, "DTSTART")?;
                require_present(j.organizer(), m, C, "ORGANIZER")?;
                require_empty(j.attendee(), m, C, "ATTENDEE")?;
                require_empty(j.rstatus(), m, C, "REQUEST-STATUS")?;
                require_status_in(
                    j.status(),
                    &[
                        |v| matches!(v, StatusValue::Draft),
                        |v| matches!(v, StatusValue::Final),
                        cancelled,
                    ],
                    "DRAFT/FINAL/CANCELLED",
                    m,
                    C,
                )?;
            }
        }
        Method::Add => {
            if journals.len() != 1 {
                return Err(ItipError::ComponentCount {
                    method: m,
                    component: C,
                    expected: "exactly one",
                });
            }
            let j = journals[0];
            require_exactly_one(j.description(), m, C, "DESCRIPTION")?;
            require_present(j.dtstart(), m, C, "DTSTART")?;
            require_present(j.organizer(), m, C, "ORGANIZER")?;
            match j.seq() {
                None => {
                    return Err(ItipError::MissingProperty {
                        method: m,
                        component: C,
                        property: "SEQUENCE",
                    });
                }
                Some(seq) if **seq.value() <= 0 => {
                    return Err(ItipError::SequenceMustBePositive {
                        method: m,
                        component: C,
                    });
                }
                _ => {}
            }
            require_empty(j.attendee(), m, C, "ATTENDEE")?;
            require_empty(j.exdate(), m, C, "EXDATE")?;
            require_absent(j.recurid(), m, C, "RECURRENCE-ID")?;
            require_empty(j.rstatus(), m, C, "REQUEST-STATUS")?;
            require_empty(j.rdate(), m, C, "RDATE")?;
            require_absent(j.rrule(), m, C, "RRULE")?;
            require_status_in(
                j.status(),
                &[
                    |v| matches!(v, StatusValue::Draft),
                    |v| matches!(v, StatusValue::Final),
                    cancelled,
                ],
                "DRAFT/FINAL/CANCELLED",
                m,
                C,
            )?;
        }
        Method::Cancel => {
            for j in journals {
                require_present(j.organizer(), m, C, "ORGANIZER")?;
                require_present(j.seq(), m, C, "SEQUENCE")?;
                require_empty(j.attendee(), m, C, "ATTENDEE")?;
                require_empty(j.rstatus(), m, C, "REQUEST-STATUS")?;
                require_status_in(j.status(), &[cancelled], "CANCELLED", m, C)?;
            }
            same_uid(journals.iter().map(|j| j.uid().as_str()), m, C)?;
        }
        Method::Request
        | Method::Reply
        | Method::Refresh
        | Method::Counter
        | Method::DeclineCounter => {
            return Err(ItipError::UnsupportedMethod {
                method: m,
                component: C,
            });
        }
    }
    Ok(())
}

/// RFC 5546 §3.3: `VFREEBUSY` only defines `PUBLISH`/`REQUEST`/`REPLY`.
fn validate_vfreebusy(
    method: Method,
    freebusys: &[&FreeBusy],
) -> Result<(), ItipError> {
    const C: &str = "VFREEBUSY";
    let m = method.as_str();

    match method {
        Method::Publish => {
            for f in freebusys {
                require_present(f.dtstamp(), m, C, "DTSTAMP")?;
                require_present(f.dtstart(), m, C, "DTSTART")?;
                require_present(f.dtend(), m, C, "DTEND")?;
                require_present(f.organizer(), m, C, "ORGANIZER")?;
                require_present(f.uid(), m, C, "UID")?;
                require_empty(f.attendee(), m, C, "ATTENDEE")?;
                require_empty(f.rstatus(), m, C, "REQUEST-STATUS")?;
            }
        }
        Method::Request => {
            if freebusys.len() != 1 {
                return Err(ItipError::ComponentCount {
                    method: m,
                    component: C,
                    expected: "exactly one",
                });
            }
            let f = freebusys[0];
            require_nonempty(f.attendee(), m, C, "ATTENDEE")?;
            require_present(f.dtend(), m, C, "DTEND")?;
            require_present(f.dtstamp(), m, C, "DTSTAMP")?;
            require_present(f.dtstart(), m, C, "DTSTART")?;
            require_present(f.organizer(), m, C, "ORGANIZER")?;
            require_present(f.uid(), m, C, "UID")?;
            require_empty(f.freebusy(), m, C, "FREEBUSY")?;
            require_empty(f.rstatus(), m, C, "REQUEST-STATUS")?;
            require_absent(f.url(), m, C, "URL")?;
        }
        Method::Reply => {
            if freebusys.len() != 1 {
                return Err(ItipError::ComponentCount {
                    method: m,
                    component: C,
                    expected: "exactly one",
                });
            }
            let f = freebusys[0];
            require_exactly_one(f.attendee(), m, C, "ATTENDEE")?;
            require_present(f.dtstamp(), m, C, "DTSTAMP")?;
            require_present(f.dtend(), m, C, "DTEND")?;
            require_present(f.dtstart(), m, C, "DTSTART")?;
            require_present(f.organizer(), m, C, "ORGANIZER")?;
            require_present(f.uid(), m, C, "UID")?;
        }
        Method::Add
        | Method::Cancel
        | Method::Refresh
        | Method::Counter
        | Method::DeclineCounter => {
            return Err(ItipError::UnsupportedMethod {
                method: m,
                component: C,
            });
        }
    }
    Ok(())
}
