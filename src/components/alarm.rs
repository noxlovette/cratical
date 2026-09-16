use crate::{
    ast::ComponentError,
    components::write_lines,
    properties::{
        Action, ActionEnum, Attachment, Attendee, Description, Duration, Iana,
        Repeat, Summary, Trigger, Xprop,
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

/// Fields shared by every [`Alarm`] builder regardless of `ACTION`
/// alternative.
#[derive(Debug, Default)]
struct AlarmCommon {
    duration: Option<Duration>,
    repeat: Option<Repeat>,
    #[cfg(feature = "rfc_9074")]
    uid: Option<Uid>,
    #[cfg(feature = "rfc_9074")]
    related: Vec<RelatedTo>,
    #[cfg(feature = "rfc_9074")]
    acknowledged: Option<Acknowledged>,
    #[cfg(feature = "rfc_9074")]
    proximity: Option<Proximity>,
    #[cfg(feature = "rfc_9074")]
    locations: Vec<VLocation>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
}

impl AlarmCommon {
    /// Runs the one cross-field check common to every `ACTION`
    /// alternative (RFC 9074 §8: nested `VLOCATION`s are only legal
    /// alongside `PROXIMITY`) and assembles the finished [`Alarm`].
    /// `DURATION`/`REPEAT` occurring together, and every alternative's own
    /// required/forbidden property set, are guaranteed by construction —
    /// see each builder's own doc comment — so there's nothing left to
    /// check for those here.
    fn finish(
        self,
        action: Action,
        trigger: Trigger,
        description: Option<Description>,
        summary: Option<Summary>,
        attendee: Vec<Attendee>,
        attach: Vec<Attachment>,
    ) -> Result<Alarm, ComponentError> {
        #[cfg(feature = "rfc_9074")]
        if !self.locations.is_empty() && self.proximity.is_none() {
            return Err(ComponentError::Requires("VLOCATION", "PROXIMITY"));
        }
        Ok(Alarm {
            action,
            trigger,
            duration: self.duration,
            repeat: self.repeat,
            description,
            summary,
            attendee,
            attach,
            #[cfg(feature = "rfc_9074")]
            uid: self.uid,
            #[cfg(feature = "rfc_9074")]
            related: self.related,
            #[cfg(feature = "rfc_9074")]
            acknowledged: self.acknowledged,
            #[cfg(feature = "rfc_9074")]
            proximity: self.proximity,
            #[cfg(feature = "rfc_9074")]
            locations: self.locations,
            xprop: self.xprop,
            iana: self.iana,
        })
    }
}

/// Setters shared by every [`Alarm`] builder, generated once per concrete
/// builder type rather than hand-written four times.
macro_rules! impl_alarm_common_setters {
    ($ty:ident) => {
        impl $ty {
            /// Sets `DURATION` and `REPEAT` together. RFC 5545 §3.6.6
            /// requires them to occur together or not at all — taking
            /// both in one call makes setting just one impossible to
            /// express, so there's nothing to check for this at
            /// `build()` time.
            pub fn duration_and_repeat(
                mut self,
                duration: Duration,
                repeat: Repeat,
            ) -> Self {
                self.common.duration = Some(duration);
                self.common.repeat = Some(repeat);
                self
            }

            /// Sets `UID` (RFC 9074 §4).
            #[cfg(feature = "rfc_9074")]
            pub fn uid(mut self, v: Uid) -> Self {
                self.common.uid = Some(v);
                self
            }

            /// Adds a `RELATED-TO` property (RFC 9074 §5).
            #[cfg(feature = "rfc_9074")]
            pub fn related(mut self, v: RelatedTo) -> Self {
                self.common.related.push(v);
                self
            }

            /// Sets `ACKNOWLEDGED` (RFC 9074 §6.1).
            #[cfg(feature = "rfc_9074")]
            pub fn acknowledged(mut self, v: Acknowledged) -> Self {
                self.common.acknowledged = Some(v);
                self
            }

            /// Sets `PROXIMITY` (RFC 9074 §8.1).
            #[cfg(feature = "rfc_9074")]
            pub fn proximity(mut self, v: Proximity) -> Self {
                self.common.proximity = Some(v);
                self
            }

            /// Adds a nested `VLOCATION` sub-component, already built via
            /// [`crate::components::vlocation::VLocationBuilder`]. Legal
            /// only alongside `PROXIMITY` (RFC 9074 §8) — checked in
            /// `build()`, since it depends on whether `.proximity()` was
            /// also called.
            #[cfg(feature = "rfc_9074")]
            pub fn location(mut self, v: VLocation) -> Self {
                self.common.locations.push(v);
                self
            }

            /// Adds a non-standard (`X-`) property.
            pub fn xprop(mut self, v: Xprop) -> Self {
                self.common.xprop.push(v);
                self
            }

            /// Adds an IANA-registered property this crate doesn't
            /// otherwise model.
            pub fn iana(mut self, v: Iana) -> Self {
                self.common.iana.push(v);
                self
            }
        }
    };
}

/// Entry points for [`Alarm`]'s builder. RFC 5545 §3.6.6 splits `VALARM`
/// into three alternative property sets (`audioprop`/`dispprop`/
/// `emailprop`), selected by `ACTION`, each with a different required/
/// forbidden set for `DESCRIPTION`/`SUMMARY`/`ATTENDEE`/`ATTACH`. Rather
/// than one builder accepting all of them and validating at runtime
/// (RFC 5545's own grammar shape, and what the parser's internal builder
/// does), each alternative gets its own concrete builder type that only
/// exposes the setters — and, where possible, only accepts the
/// constructor arguments — its own grammar allows.
#[derive(Debug)]
pub struct AlarmBuilder;

impl AlarmBuilder {
    /// Starts building a `VALARM` with `ACTION:AUDIO`. `DESCRIPTION`,
    /// `SUMMARY`, and `ATTENDEE` aren't allowed for this alternative, so
    /// [`AudioAlarmBuilder`] has no setters for them; at most one `ATTACH`
    /// is allowed, checked in [`AudioAlarmBuilder::build`].
    pub fn audio(trigger: Trigger) -> AudioAlarmBuilder {
        AudioAlarmBuilder {
            trigger,
            attach: Vec::new(),
            common: AlarmCommon::default(),
        }
    }

    /// Starts building a `VALARM` with `ACTION:DISPLAY` from its required
    /// `DESCRIPTION`. `SUMMARY`/`ATTENDEE`/`ATTACH` aren't allowed for
    /// this alternative, so [`DisplayAlarmBuilder`] has no setters for
    /// them.
    pub fn display(
        trigger: Trigger,
        description: Description,
    ) -> DisplayAlarmBuilder {
        DisplayAlarmBuilder {
            trigger,
            description,
            common: AlarmCommon::default(),
        }
    }

    /// Starts building a `VALARM` with `ACTION:EMAIL` from its required
    /// `DESCRIPTION`, `SUMMARY`, and first `ATTENDEE` (RFC 5545 §3.6.6
    /// requires at least one) — further `ATTENDEE`s can be added via
    /// [`EmailAlarmBuilder::attendee`].
    pub fn email(
        trigger: Trigger,
        description: Description,
        summary: Summary,
        first_attendee: Attendee,
    ) -> EmailAlarmBuilder {
        EmailAlarmBuilder {
            trigger,
            description,
            summary,
            attendee: vec![first_attendee],
            attach: Vec::new(),
            common: AlarmCommon::default(),
        }
    }

    /// Starts building a `VALARM` with a non-standard (`X-`) or
    /// IANA-registered `ACTION`, for which RFC 5545 defines no
    /// alternative-specific property-set restriction beyond `ACTION` and
    /// `TRIGGER` both being required.
    pub fn custom(action: Action, trigger: Trigger) -> CustomAlarmBuilder {
        CustomAlarmBuilder {
            action,
            trigger,
            description: None,
            summary: None,
            attendee: Vec::new(),
            attach: Vec::new(),
            common: AlarmCommon::default(),
        }
    }
}

/// Builder for an `ACTION:AUDIO` [`Alarm`] — see [`AlarmBuilder::audio`].
#[derive(Debug)]
pub struct AudioAlarmBuilder {
    trigger: Trigger,
    attach: Vec<Attachment>,
    common: AlarmCommon,
}

impl_alarm_common_setters!(AudioAlarmBuilder);

impl AudioAlarmBuilder {
    /// Adds an `ATTACH` property. RFC 5545 §3.6.6 allows at most one for
    /// `ACTION:AUDIO` — checked in [`Self::build`], since a `Vec` can't
    /// express "at most one" at the type level the way a plain field can.
    pub fn attach(mut self, v: Attachment) -> Self {
        self.attach.push(v);
        self
    }

    /// Assembles the finished [`Alarm`].
    pub fn build(self) -> Result<Alarm, ComponentError> {
        if self.attach.len() > 1 {
            return Err(ComponentError::NotAllowed(
                "more than one ATTACH",
                "ACTION is AUDIO",
            ));
        }
        self.common.finish(
            Action::new(ActionEnum::Audio),
            self.trigger,
            None,
            None,
            Vec::new(),
            self.attach,
        )
    }
}

/// Builder for an `ACTION:DISPLAY` [`Alarm`] — see
/// [`AlarmBuilder::display`].
#[derive(Debug)]
pub struct DisplayAlarmBuilder {
    trigger: Trigger,
    description: Description,
    common: AlarmCommon,
}

impl_alarm_common_setters!(DisplayAlarmBuilder);

impl DisplayAlarmBuilder {
    /// Assembles the finished [`Alarm`].
    pub fn build(self) -> Result<Alarm, ComponentError> {
        self.common.finish(
            Action::new(ActionEnum::Display),
            self.trigger,
            Some(self.description),
            None,
            Vec::new(),
            Vec::new(),
        )
    }
}

/// Builder for an `ACTION:EMAIL` [`Alarm`] — see [`AlarmBuilder::email`].
#[derive(Debug)]
pub struct EmailAlarmBuilder {
    trigger: Trigger,
    description: Description,
    summary: Summary,
    attendee: Vec<Attendee>,
    attach: Vec<Attachment>,
    common: AlarmCommon,
}

impl_alarm_common_setters!(EmailAlarmBuilder);

impl EmailAlarmBuilder {
    /// Adds another `ATTENDEE` property, beyond the one
    /// [`AlarmBuilder::email`] already requires.
    pub fn attendee(mut self, v: Attendee) -> Self {
        self.attendee.push(v);
        self
    }

    /// Adds an `ATTACH` property. RFC 5545 §3.6.6 allows any number of
    /// these for `ACTION:EMAIL`.
    pub fn attach(mut self, v: Attachment) -> Self {
        self.attach.push(v);
        self
    }

    /// Assembles the finished [`Alarm`].
    pub fn build(self) -> Result<Alarm, ComponentError> {
        self.common.finish(
            Action::new(ActionEnum::Email),
            self.trigger,
            Some(self.description),
            Some(self.summary),
            self.attendee,
            self.attach,
        )
    }
}

/// Builder for a non-standard/IANA-registered-`ACTION` [`Alarm`] — see
/// [`AlarmBuilder::custom`].
#[derive(Debug)]
pub struct CustomAlarmBuilder {
    action: Action,
    trigger: Trigger,
    description: Option<Description>,
    summary: Option<Summary>,
    attendee: Vec<Attendee>,
    attach: Vec<Attachment>,
    common: AlarmCommon,
}

impl_alarm_common_setters!(CustomAlarmBuilder);

impl CustomAlarmBuilder {
    /// Sets `DESCRIPTION`.
    pub fn description(mut self, v: Description) -> Self {
        self.description = Some(v);
        self
    }

    /// Sets `SUMMARY`.
    pub fn summary(mut self, v: Summary) -> Self {
        self.summary = Some(v);
        self
    }

    /// Adds an `ATTENDEE` property.
    pub fn attendee(mut self, v: Attendee) -> Self {
        self.attendee.push(v);
        self
    }

    /// Adds an `ATTACH` property.
    pub fn attach(mut self, v: Attachment) -> Self {
        self.attach.push(v);
        self
    }

    /// Assembles the finished [`Alarm`].
    pub fn build(self) -> Result<Alarm, ComponentError> {
        self.common.finish(
            self.action,
            self.trigger,
            self.description,
            self.summary,
            self.attendee,
            self.attach,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::{DateTimeDuration, Duration as DurationV};

    fn trigger() -> Trigger {
        crate::properties::TriggerBuilder::new(DateTimeDuration::Duration(
            DurationV::new(chrono::Duration::minutes(-15)),
        ))
        .build()
    }

    #[test]
    fn audio_alarm_builder_round_trips() {
        let alarm = AlarmBuilder::audio(trigger()).build().unwrap();
        assert_eq!(
            alarm.to_string(),
            "BEGIN:VALARM\r\nACTION:AUDIO\r\nTRIGGER:-PT15M\r\nEND:VALARM\r\n"
        );
    }

    #[test]
    fn audio_alarm_builder_rejects_more_than_one_attach() {
        let attach = |uri: &str| {
            Attachment::from_uri(crate::values::Uri::parse(uri).unwrap(), None)
        };
        let result = AlarmBuilder::audio(trigger())
            .attach(attach("ftp://example.com/a.aud"))
            .attach(attach("ftp://example.com/b.aud"))
            .build();
        assert!(matches!(
            result,
            Err(ComponentError::NotAllowed(
                "more than one ATTACH",
                "ACTION is AUDIO"
            ))
        ));
    }

    #[test]
    fn display_alarm_builder_round_trips() {
        let alarm = AlarmBuilder::display(
            trigger(),
            crate::properties::DescriptionBuilder::new(
                "Meeting reminder".into(),
            )
            .build(),
        )
        .build()
        .unwrap();
        assert_eq!(
            alarm.to_string(),
            "BEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-PT15M\r\nDESCRIPTION:\
             Meeting reminder\r\nEND:VALARM\r\n"
        );
    }

    #[test]
    fn email_alarm_builder_requires_an_attendee_by_construction() {
        let attendee = crate::properties::AttendeeBuilder::new(
            crate::values::CalendarUserAddress::new(
                crate::values::Uri::parse("mailto:jsmith@example.com").unwrap(),
            )
            .unwrap(),
        )
        .build();
        let alarm = AlarmBuilder::email(
            trigger(),
            crate::properties::DescriptionBuilder::new("A reminder".into())
                .build(),
            crate::properties::SummaryBuilder::new("Reminder".into()).build(),
            attendee,
        )
        .build()
        .unwrap();
        assert_eq!(alarm.attendee().len(), 1);
    }

    #[cfg(feature = "rfc_9074")]
    #[test]
    fn alarm_builder_rejects_a_location_without_proximity() {
        let location = crate::components::vlocation::VLocationBuilder::new(
            Uid::new("loc@example.com".into()),
        )
        .build();
        let result = AlarmBuilder::audio(trigger()).location(location).build();
        assert!(matches!(
            result,
            Err(ComponentError::Requires("VLOCATION", "PROXIMITY"))
        ));
    }
}
