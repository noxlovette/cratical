use crate::{
    ast::ComponentError,
    components::write_lines,
    properties::{
        Comment, DateTimeStart, Iana, LastModified, RRule, RecurrenceDateTimes,
        TimeZoneIdentifier, TimeZoneName, TimeZoneOffsetFrom, TimeZoneOffsetTo,
        TimeZoneUrl, Xprop,
    },
};

/// A time zone is unambiguously defined by the set of time
/// measurement rules determined by the governing body for a given
/// geographic area.  These rules describe, at a minimum, the base
/// offset from UTC for the time zone, often referred to as the
/// Standard Time offset.  Many locations adjust their Standard Time
/// forward or backward by one hour, in order to accommodate seasonal
/// changes in number of daylight hours, often referred to as Daylight
/// Saving Time.  Some locations adjust their time by a fraction of an
/// hour.  Standard Time is also known as Winter Time.  Daylight
/// Saving Time is also known as Advanced Time, Summer Time, or Legal
/// Time in certain countries.
///
/// Interoperability between two calendaring and scheduling
/// applications, especially for recurring events, to-dos or journal
/// entries, is dependent on the ability to capture and convey date
/// and time information in an unambiguous format.  The specification
/// of current time zone information is integral to this behavior.
///
/// If present, the "VTIMEZONE" calendar component defines the set of
/// Standard Time and Daylight Saving Time observances (or rules) for
/// a particular time zone for a given interval of time.  The
/// "VTIMEZONE" calendar component cannot be nested within other
/// calendar components.  Multiple "VTIMEZONE" calendar components can
/// exist in an iCalendar object.  In this situation, each "VTIMEZONE"
/// MUST represent a unique time zone definition.  This is necessary
/// for some classes of events, such as airline flights, that start in
/// one time zone and end in another.
///
/// The "VTIMEZONE" calendar component MUST include the "TZID"
/// property and at least one definition of a "STANDARD" or "DAYLIGHT"
/// sub-component.  The "STANDARD" or "DAYLIGHT" sub-component MUST
/// include the "DTSTART", "TZOFFSETFROM", and "TZOFFSETTO"
/// properties.
///
/// An individual "VTIMEZONE" calendar component MUST be specified for
/// each unique "TZID" parameter value specified in the iCalendar
/// object.  In addition, a "VTIMEZONE" calendar component, referred
/// to by a recurring calendar component, MUST provide valid time zone
/// information for all recurrence instances.
///
/// Example:  This is a simple example showing the current time zone
/// rules for New York City using only the "DTSTART" property, suitable
/// for a recurring event that starts on or later than March 11, 2007
/// at 03:00:00 EDT and ends no later than March 9, 2008 at 01:59:59
/// EST.
///
/// > BEGIN:VTIMEZONE
/// >
/// > TZID:America/New_York
/// >
/// > LAST-MODIFIED:20050809T050000Z
/// >
/// > BEGIN:STANDARD
/// >
/// > DTSTART:20071104T020000
/// >
/// > TZOFFSETFROM:-0400
/// >
/// > TZOFFSETTO:-0500
/// >
/// > TZNAME:EST
/// >
/// > END:STANDARD
/// >
/// > BEGIN:DAYLIGHT
/// >
/// > DTSTART:20070311T020000
/// >
/// > TZOFFSETFROM:-0500
/// >
/// > TZOFFSETTO:-0400
/// >
/// > TZNAME:EDT
/// >
/// > END:DAYLIGHT
/// >
/// > END:VTIMEZONE
/// >
///
/// [Section 3.6.5](https://datatracker.ietf.org/doc/html/rfc5545#section-3.6.5)
#[derive(Debug)]
pub struct Timezone {
    pub(crate) tzid: TimeZoneIdentifier,
    pub(crate) last_mod: Option<LastModified>,
    pub(crate) tz_url: Option<TimeZoneUrl>,
    /// At least one of `standardc`/`daylightc` MUST be non-empty (RFC 5545
    /// §3.6.5) — enforced at build time, since neither list alone can be
    /// required by the type.
    pub(crate) standardc: Vec<TzProp>,
    pub(crate) daylightc: Vec<TzProp>,
    pub(crate) xprop: Vec<Xprop>,
    pub(crate) iana: Vec<Iana>,
}

impl Timezone {
    /// The `TZID` property.
    pub fn tzid(&self) -> &TimeZoneIdentifier {
        &self.tzid
    }

    /// The `LAST-MODIFIED` property, if present.
    pub fn last_mod(&self) -> Option<&LastModified> {
        self.last_mod.as_ref()
    }

    /// The `TZURL` property, if present.
    pub fn tz_url(&self) -> Option<&TimeZoneUrl> {
        self.tz_url.as_ref()
    }

    /// The `STANDARD` sub-components.
    pub fn standardc(&self) -> &[TzProp] {
        &self.standardc
    }

    /// The `DAYLIGHT` sub-components.
    pub fn daylightc(&self) -> &[TzProp] {
        &self.daylightc
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

/// Builder for [`Timezone`]. RFC 5545 §3.6.5 requires at least one
/// `STANDARD` or `DAYLIGHT` sub-component — a "list" rule that doesn't map
/// cleanly onto the type-state pattern the way a mutually-exclusive pair
/// of optional fields does (see [`crate::components::event::EventBuilder`]
/// for that shape), so it's a runtime check in [`Self::build`] instead,
/// same as the parser's own internal builder.
#[derive(Debug)]
pub struct TimezoneBuilder {
    tzid: TimeZoneIdentifier,
    last_mod: Option<LastModified>,
    tz_url: Option<TimeZoneUrl>,
    standardc: Vec<TzProp>,
    daylightc: Vec<TzProp>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
}

impl TimezoneBuilder {
    /// Starts building a `VTIMEZONE` from its one required property,
    /// `TZID` (RFC 5545 §3.6.5).
    pub fn new(tzid: TimeZoneIdentifier) -> Self {
        Self {
            tzid,
            last_mod: None,
            tz_url: None,
            standardc: Vec::new(),
            daylightc: Vec::new(),
            xprop: Vec::new(),
            iana: Vec::new(),
        }
    }

    /// Sets `LAST-MODIFIED`.
    pub fn last_mod(mut self, v: LastModified) -> Self {
        self.last_mod = Some(v);
        self
    }

    /// Sets `TZURL`.
    pub fn tz_url(mut self, v: TimeZoneUrl) -> Self {
        self.tz_url = Some(v);
        self
    }

    /// Adds a `STANDARD` sub-component, already built via
    /// [`TzPropBuilder`].
    pub fn standard(mut self, v: TzProp) -> Self {
        self.standardc.push(v);
        self
    }

    /// Adds a `DAYLIGHT` sub-component, already built via
    /// [`TzPropBuilder`].
    pub fn daylight(mut self, v: TzProp) -> Self {
        self.daylightc.push(v);
        self
    }

    /// Adds a non-standard (`X-`) property.
    pub fn xprop(mut self, v: Xprop) -> Self {
        self.xprop.push(v);
        self
    }

    /// Adds an IANA-registered property this crate doesn't otherwise model.
    pub fn iana(mut self, v: Iana) -> Self {
        self.iana.push(v);
        self
    }

    /// Validates the cross-field rule RFC 5545 §3.6.5 places on
    /// `VTIMEZONE` and assembles the finished [`Timezone`].
    pub fn build(self) -> Result<Timezone, ComponentError> {
        if self.standardc.is_empty() && self.daylightc.is_empty() {
            return Err(ComponentError::RequiresAtLeastOne(
                "VTIMEZONE",
                "STANDARD or DAYLIGHT",
            ));
        }
        Ok(Timezone {
            tzid: self.tzid,
            last_mod: self.last_mod,
            tz_url: self.tz_url,
            standardc: self.standardc,
            daylightc: self.daylightc,
            xprop: self.xprop,
            iana: self.iana,
        })
    }
}

impl std::fmt::Display for Timezone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BEGIN:VTIMEZONE\r\n")?;
        write!(f, "{}\r\n", self.tzid)?;
        if let Some(v) = &self.last_mod {
            write!(f, "{v}\r\n")?;
        }
        if let Some(v) = &self.tz_url {
            write!(f, "{v}\r\n")?;
        }
        for p in &self.standardc {
            fmt_tz_observance(f, "STANDARD", p)?;
        }
        for p in &self.daylightc {
            fmt_tz_observance(f, "DAYLIGHT", p)?;
        }
        write_lines(f, &self.xprop)?;
        write_lines(f, &self.iana)?;
        write!(f, "END:VTIMEZONE\r\n")
    }
}

/// Renders one `STANDARD`/`DAYLIGHT` sub-component (RFC 5545 §3.6.5) — its
/// `tzprop` grammar is shared by both, so which sub-component name applies
/// comes from which of `Timezone`'s two lists a given `TzProp` is iterated
/// out of, not from `TzProp` itself.
fn fmt_tz_observance(
    f: &mut std::fmt::Formatter<'_>,
    name: &str,
    p: &TzProp,
) -> std::fmt::Result {
    write!(f, "BEGIN:{name}\r\n")?;
    write!(f, "{}\r\n", p.dtstart)?;
    write!(f, "{}\r\n", p.tz_offset_to)?;
    write!(f, "{}\r\n", p.tz_offset_from)?;
    if let Some(v) = &p.rrule {
        write!(f, "{v}\r\n")?;
    }
    write_lines(f, &p.comment)?;
    write_lines(f, &p.rdate)?;
    write_lines(f, &p.tzname)?;
    write_lines(f, &p.xprop)?;
    write_lines(f, &p.iana)?;
    write!(f, "END:{name}\r\n")
}

/// The `tzprop` grammar shared by `STANDARD`/`DAYLIGHT` sub-components (RFC
/// 5545 §3.6.5).
#[derive(Debug)]
pub struct TzProp {
    pub(crate) dtstart: DateTimeStart,
    pub(crate) tz_offset_to: TimeZoneOffsetTo,
    pub(crate) tz_offset_from: TimeZoneOffsetFrom,
    pub(crate) rrule: Option<RRule>,
    pub(crate) comment: Vec<Comment>,
    pub(crate) rdate: Vec<RecurrenceDateTimes>,
    pub(crate) tzname: Vec<TimeZoneName>,
    pub(crate) xprop: Vec<Xprop>,
    pub(crate) iana: Vec<Iana>,
}

impl TzProp {
    /// The `DTSTART` property.
    pub fn dtstart(&self) -> &DateTimeStart {
        &self.dtstart
    }

    /// The `TZOFFSETTO` property.
    pub fn tz_offset_to(&self) -> &TimeZoneOffsetTo {
        &self.tz_offset_to
    }

    /// The `TZOFFSETFROM` property.
    pub fn tz_offset_from(&self) -> &TimeZoneOffsetFrom {
        &self.tz_offset_from
    }

    /// The `RRULE` property, if present.
    pub fn rrule(&self) -> Option<&RRule> {
        self.rrule.as_ref()
    }

    /// The `COMMENT` properties.
    pub fn comment(&self) -> &[Comment] {
        &self.comment
    }

    /// The `RDATE` properties.
    pub fn rdate(&self) -> &[RecurrenceDateTimes] {
        &self.rdate
    }

    /// The `TZNAME` properties.
    pub fn tzname(&self) -> &[TimeZoneName] {
        &self.tzname
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

/// Builder for [`TzProp`] (the `tzprop` grammar shared by `STANDARD`/
/// `DAYLIGHT` — see [`TimezoneBuilder::standard`]/[`TimezoneBuilder::daylight`]
/// for which sub-component name a given `TzProp` ends up under). The
/// value-type/`TZID` matching between `DTSTART` and its siblings can't be
/// resolved until the actual property values are known, so it stays a
/// runtime check in [`Self::build`].
#[derive(Debug)]
pub struct TzPropBuilder {
    dtstart: DateTimeStart,
    tz_offset_to: TimeZoneOffsetTo,
    tz_offset_from: TimeZoneOffsetFrom,
    rrule: Option<RRule>,
    comment: Vec<Comment>,
    rdate: Vec<RecurrenceDateTimes>,
    tzname: Vec<TimeZoneName>,
    xprop: Vec<Xprop>,
    iana: Vec<Iana>,
}

impl TzPropBuilder {
    /// Starts building a `STANDARD`/`DAYLIGHT` sub-component from its
    /// three required properties, `DTSTART`, `TZOFFSETTO`, and
    /// `TZOFFSETFROM` (RFC 5545 §3.6.5).
    pub fn new(
        dtstart: DateTimeStart,
        tz_offset_to: TimeZoneOffsetTo,
        tz_offset_from: TimeZoneOffsetFrom,
    ) -> Self {
        Self {
            dtstart,
            tz_offset_to,
            tz_offset_from,
            rrule: None,
            comment: Vec::new(),
            rdate: Vec::new(),
            tzname: Vec::new(),
            xprop: Vec::new(),
            iana: Vec::new(),
        }
    }

    /// Sets `RRULE`.
    pub fn rrule(mut self, v: RRule) -> Self {
        self.rrule = Some(v);
        self
    }

    /// Adds a `COMMENT` property.
    pub fn comment(mut self, v: Comment) -> Self {
        self.comment.push(v);
        self
    }

    /// Adds an `RDATE` property.
    pub fn rdate(mut self, v: RecurrenceDateTimes) -> Self {
        self.rdate.push(v);
        self
    }

    /// Adds a `TZNAME` property.
    pub fn tzname(mut self, v: TimeZoneName) -> Self {
        self.tzname.push(v);
        self
    }

    /// Adds a non-standard (`X-`) property.
    pub fn xprop(mut self, v: Xprop) -> Self {
        self.xprop.push(v);
        self
    }

    /// Adds an IANA-registered property this crate doesn't otherwise model.
    pub fn iana(mut self, v: Iana) -> Self {
        self.iana.push(v);
        self
    }

    /// Validates the `tzprop` grammar's cross-field rules (RFC 5545
    /// §3.6.5) and assembles the finished [`TzProp`].
    pub fn build(self) -> Result<TzProp, ComponentError> {
        self.dtstart.cmp_until(self.rrule.as_ref())?;
        self.dtstart.cmp_rdate(&self.rdate)?;
        self.dtstart.cmp_rdate_tzid(&self.rdate)?;

        Ok(TzProp {
            dtstart: self.dtstart,
            tz_offset_to: self.tz_offset_to,
            tz_offset_from: self.tz_offset_from,
            rrule: self.rrule,
            comment: self.comment,
            rdate: self.rdate,
            tzname: self.tzname,
            xprop: self.xprop,
            iana: self.iana,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::{DateOrDatetime, DateTime, UtcOffset};
    use chrono::FixedOffset;

    #[test]
    // rustfmt's `format_strings` wrapping of this literal corrupts its
    // runtime value (it splits inside a `\r\n` escape pair, not between
    // whole escapes) — skip it here rather than let a formatting pass
    // silently reintroduce that bug.
    #[rustfmt::skip]
    fn timezone_builder_round_trips_a_minimal_timezone() {
        let dtstart = crate::properties::DateTimeStartBuilder::new(
            DateOrDatetime::DateTime(DateTime::Floating(
                chrono::NaiveDate::from_ymd_opt(2007, 11, 4)
                    .unwrap()
                    .and_hms_opt(2, 0, 0)
                    .unwrap(),
            )),
        )
        .build();
        let standard = TzPropBuilder::new(
            dtstart,
            TimeZoneOffsetTo::new(UtcOffset::new(
                FixedOffset::west_opt(5 * 3600).unwrap(),
            )),
            TimeZoneOffsetFrom::new(UtcOffset::new(
                FixedOffset::west_opt(4 * 3600).unwrap(),
            )),
        )
        .build()
        .unwrap();

        let timezone = TimezoneBuilder::new(TimeZoneIdentifier::new(
            "America/New_York".into(),
        ))
        .standard(standard)
        .build()
        .unwrap();

        assert_eq!(
            timezone.to_string(),
            "BEGIN:VTIMEZONE\r\nTZID:America/New_York\r\nBEGIN:STANDARD\r\nDTSTART:20071104T020000\r\nTZOFFSETTO:-0500\r\nTZOFFSETFROM:-0400\r\nEND:STANDARD\r\nEND:VTIMEZONE\r\n"
        );
    }

    #[test]
    fn timezone_builder_rejects_no_standard_or_daylight() {
        let result = TimezoneBuilder::new(TimeZoneIdentifier::new(
            "America/New_York".into(),
        ))
        .build();
        assert!(matches!(
            result,
            Err(ComponentError::RequiresAtLeastOne(
                "VTIMEZONE",
                "STANDARD or DAYLIGHT"
            ))
        ));
    }
}
