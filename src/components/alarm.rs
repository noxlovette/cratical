use crate::{
    components::write_lines,
    properties::{
        Action, Attachment, Attendee, Description, Duration, Iana, Repeat,
        Summary, Trigger, Xprop,
    },
};
#[cfg(feature = "rfc_9074")]
use crate::{
    components::{vlocation::VLocation, write_components},
    properties::{Acknowledged, Proximity, RelatedTo, Uid},
};

/// A "VALARM" calendar component is a grouping of component
/// properties that is a reminder or alarm for an event or a to-do.
/// For example, it may be used to define a reminder for a pending
/// event or an overdue to-do.
///
/// RFC 5545 actually splits this into three alternative property sets
/// (`audioprop`/`dispprop`/`emailprop`, selected by [`Action`]'s value)
/// each with its own required/optional/cardinality rules for
/// `DESCRIPTION`, `SUMMARY`, `ATTENDEE`, and `ATTACH`. This type holds the
/// union of what's structurally possible across all three — which
/// alternative applies, and whether a given `Alarm` satisfies it, is
/// checked at build time once `ACTION` is known.
///
/// Example:
///
/// > BEGIN:VALARM
/// >
/// > TRIGGER;VALUE=DATE-TIME:19970317T133000Z
/// >
/// > REPEAT:4
/// >
/// > DURATION:PT15M
/// >
/// > ACTION:AUDIO
/// >
/// > ATTACH;FMTTYPE=audio/basic:ftp://example.com/pub/sounds/bell-01.aud
/// >
/// > END:VALARM
/// >
///
/// Under the `rfc_9074` feature, this type also carries the [RFC
/// 9074](https://datatracker.ietf.org/doc/html/rfc9074) `VALARM` extensions
/// — `UID`, `RELATED-TO`, `ACKNOWLEDGED`, `PROXIMITY` — and nested
/// `VLOCATION` sub-components ([RFC
/// 9073](https://datatracker.ietf.org/doc/html/rfc9073) §7.2), which are
/// only legal alongside a `PROXIMITY` property (checked at build time).
///
/// [Section 3.6.6](https://datatracker.ietf.org/doc/html/rfc5545#section-3.6.6)
#[derive(Debug)]
pub struct Alarm {
    pub(crate) action: Action,
    pub(crate) trigger: Trigger,
    // DURATION and REPEAT are optional but MUST appear together (RFC 5545
    // §3.6.6) — enforced at build time.
    pub(crate) duration: Option<Duration>,
    pub(crate) repeat: Option<Repeat>,
    pub(crate) description: Option<Description>,
    pub(crate) summary: Option<Summary>,
    pub(crate) attendee: Vec<Attendee>,
    pub(crate) attach: Vec<Attachment>,
    #[cfg(feature = "rfc_9074")]
    pub(crate) uid: Option<Uid>,
    #[cfg(feature = "rfc_9074")]
    pub(crate) related: Vec<RelatedTo>,
    #[cfg(feature = "rfc_9074")]
    pub(crate) acknowledged: Option<Acknowledged>,
    #[cfg(feature = "rfc_9074")]
    pub(crate) proximity: Option<Proximity>,
    #[cfg(feature = "rfc_9074")]
    pub(crate) locations: Vec<VLocation>,
    pub(crate) xprop: Vec<Xprop>,
    pub(crate) iana: Vec<Iana>,
}

impl Alarm {
    /// The `ACTION` property.
    pub fn action(&self) -> &Action {
        &self.action
    }

    /// The `TRIGGER` property.
    pub fn trigger(&self) -> &Trigger {
        &self.trigger
    }

    /// The `DURATION` property, if present.
    pub fn duration(&self) -> Option<&Duration> {
        self.duration.as_ref()
    }

    /// The `REPEAT` property, if present.
    pub fn repeat(&self) -> Option<&Repeat> {
        self.repeat.as_ref()
    }

    /// The `DESCRIPTION` property, if present.
    pub fn description(&self) -> Option<&Description> {
        self.description.as_ref()
    }

    /// The `SUMMARY` property, if present.
    pub fn summary(&self) -> Option<&Summary> {
        self.summary.as_ref()
    }

    /// The `ATTENDEE` properties.
    pub fn attendee(&self) -> &[Attendee] {
        &self.attendee
    }

    /// The `ATTACH` properties.
    pub fn attach(&self) -> &[Attachment] {
        &self.attach
    }

    /// The `UID` property (RFC 9074 §4), if present.
    #[cfg(feature = "rfc_9074")]
    pub fn uid(&self) -> Option<&Uid> {
        self.uid.as_ref()
    }

    /// The `RELATED-TO` properties (RFC 9074 §5).
    #[cfg(feature = "rfc_9074")]
    pub fn related(&self) -> &[RelatedTo] {
        &self.related
    }

    /// The `ACKNOWLEDGED` property (RFC 9074 §6.1), if present.
    #[cfg(feature = "rfc_9074")]
    pub fn acknowledged(&self) -> Option<&Acknowledged> {
        self.acknowledged.as_ref()
    }

    /// The `PROXIMITY` property (RFC 9074 §8.1), if present.
    #[cfg(feature = "rfc_9074")]
    pub fn proximity(&self) -> Option<&Proximity> {
        self.proximity.as_ref()
    }

    /// The nested `VLOCATION` sub-components (RFC 9073 §7.2).
    #[cfg(feature = "rfc_9074")]
    pub fn locations(&self) -> &[VLocation] {
        &self.locations
    }

    /// The non-standard (`X-`) properties.
    pub fn xprop(&self) -> &[Xprop] {
        &self.xprop
    }

    /// The IANA-registered properties this crate doesn't otherwise model.
    pub fn iana(&self) -> &[Iana] {
        &self.iana
    }
}

impl std::fmt::Display for Alarm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BEGIN:VALARM\r\n")?;
        write!(f, "{}\r\n", self.action)?;
        write!(f, "{}\r\n", self.trigger)?;
        if let Some(v) = &self.duration {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.repeat {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.description {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.summary {
            write!(f, "{v}\r\n")?;
        }
        write_lines(f, &self.attendee)?;
        write_lines(f, &self.attach)?;
        #[cfg(feature = "rfc_9074")]
        {
            if let Some(v) = &self.uid {
                write!(f, "{v}\r\n")?;
            }
            write_lines(f, &self.related)?;
            if let Some(v) = &self.acknowledged {
                write!(f, "{v}\r\n")?;
            }
            if let Some(v) = &self.proximity {
                write!(f, "{v}\r\n")?;
            }
        }
        write_lines(f, &self.xprop)?;
        write_lines(f, &self.iana)?;
        #[cfg(feature = "rfc_9074")]
        write_components(f, &self.locations)?;
        write!(f, "END:VALARM\r\n")
    }
}
