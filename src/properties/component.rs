/// `VALARM` extension properties (RFC 9074): `ACKNOWLEDGED`, `PROXIMITY`.
#[cfg(feature = "rfc_9074")]
mod alarm_ext;
/// Alarm properties (Section 3.8.6): `ACTION`, `REPEAT`, `TRIGGER`.
mod alarms;
/// Availability properties (RFC 7953 Section 3.1): `BUSYTYPE`.
#[cfg(feature = "rfc_7953")]
mod availability;
/// Change-management properties (Section 3.8.7): `CREATED`, `DTSTAMP`,
/// `LAST-MODIFIED`, `SEQUENCE`.
mod change;
/// Date and time properties (Section 3.8.2): `COMPLETED`, `DTEND`, `DUE`,
/// `DTSTART`, `DURATION`, `FREEBUSY`, `TRANSP`.
mod datetime;
/// Descriptive properties (Section 3.8.1): `ATTACH`, `CATEGORIES`, `CLASS`,
/// `COMMENT`, `DESCRIPTION`, `GEO`, `LOCATION`, `PERCENT-COMPLETE`, `PRIORITY`,
/// `RESOURCES`, `STATUS`, `SUMMARY`.
mod descriptive;
/// `VLOCATION` properties (RFC 9073 Section 6.1, RFC 7986 Section 5.1):
/// `NAME`, `LOCATION-TYPE`.
#[cfg(feature = "rfc_9074")]
mod location;
/// Miscellaneous properties (Section 3.8.8): `REQUEST-STATUS`.
mod misc;
/// Recurrence properties (Section 3.8.5): `EXDATE`, `RDATE`, `RRULE`.
mod recurrence;
/// Relationship properties (Section 3.8.4): `ATTENDEE`, `CONTACT`, `ORGANIZER`,
/// `RECURRENCE-ID`, `RELATED-TO`, `URL`, `UID`.
mod relationship;
/// Time zone properties (Section 3.8.3): `TZID`, `TZNAME`, `TZOFFSETFROM`,
/// `TZOFFSETTO`, `TZURL`.
mod timezone;

#[cfg(feature = "rfc_9074")]
pub use alarm_ext::*;
pub use alarms::*;
#[cfg(feature = "rfc_7953")]
pub use availability::*;
pub use change::*;
pub use datetime::*;
pub use descriptive::*;
#[cfg(feature = "rfc_9074")]
pub use location::*;
pub use misc::*;
pub use recurrence::*;
pub use relationship::*;
pub use timezone::*;
