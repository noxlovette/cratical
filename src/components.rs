//! **Scope decision (issue #7):** this crate targets RFC 5545 core only.
//! `VAVAILABILITY` ([RFC 7953](https://datatracker.ietf.org/doc/html/rfc7953),
//! with its `AVAILABLE` sub-component) and `VLOCATION` ([RFC
//! 9073](https://datatracker.ietf.org/doc/html/rfc9073), nested inside
//! `VALARM`) plus the [RFC 9074](https://datatracker.ietf.org/doc/html/rfc9074)
//! `VALARM` extensions (`UID`, `PROXIMITY`, `ACKNOWLEDGED`) are deliberately
//! not implemented — a documented scope decision, not an oversight. Fixture
//! files for these RFCs exist under `tests/fixtures/collective-icalendar/`
//! (see `tests/out_of_scope.rs`) but aren't coverage of anything this crate
//! implements; revisit if/when the crate extends past RFC 5545 core.

/// The `VALARM` calendar component and its builder.
pub mod alarm;
/// The `VEVENT` calendar component and its builder.
pub mod event;
/// The `VFREEBUSY` calendar component and its builder.
pub mod free_busy;
/// The `VJOURNAL` calendar component and its builder.
pub mod journal;
/// The `VTIMEZONE` calendar component and its builder.
pub mod timezone;
/// The `VTODO` calendar component and its builder.
pub mod todo;
/// An unrecognized top-level calendar component (`iana-comp`/`x-comp`).
pub mod unknown;
