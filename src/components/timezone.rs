use crate::{
    components::write_lines,
    properties::{
        Comment, DateTimeStart, Iana, LastModified, RRule,
        RecurrenceDateTimes, TimeZoneIdentifier, TimeZoneName,
        TimeZoneOffsetFrom, TimeZoneOffsetTo, TimeZoneUrl, Xprop,
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
