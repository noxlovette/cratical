//! **Scope decision (issue #7):** this crate targets RFC 5545 core, plus
//! `VAVAILABILITY` (RFC 7953, added under the `rfc-7953` feature — see
//! issue #16) and `VLOCATION` ([RFC
//! 9073](https://datatracker.ietf.org/doc/html/rfc9073), nested inside
//! `VALARM`) plus the [RFC
//! 9074](https://datatracker.ietf.org/doc/html/rfc9074) `VALARM` extensions
//! (`UID`, `RELATED-TO`, `ACKNOWLEDGED`, `PROXIMITY`), added under the
//! `rfc-9074` feature — see issue #17. Fixture files for RFC 9074 that are
//! bare excerpts too incomplete to form a valid `icalobject` on their own
//! exist under `tests/fixtures/collective-icalendar/` (see
//! `tests/out_of_scope.rs`); real coverage lives in `tests/rfc_9074.rs`.

/// The `VALARM` calendar component and its builder.
pub mod alarm;
/// The `VAVAILABILITY` calendar component (RFC 7953) and its `AVAILABLE`
/// sub-component, and their builders.
#[cfg(feature = "rfc-7953")]
pub mod availability;
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
/// The `VLOCATION` sub-component (RFC 9073 §7.2), nested only inside
/// `VALARM`, and its builder.
#[cfg(feature = "rfc-9074")]
pub mod vlocation;

/// Writes each item in `items`, each followed by CRLF — used by every
/// component's `Display` to render a `Vec<Property>` field (RFC 5545 §3.1
/// content lines are CRLF-terminated).
pub(crate) fn write_lines<T: std::fmt::Display>(
    f: &mut std::fmt::Formatter<'_>,
    items: &[T],
) -> std::fmt::Result {
    for item in items {
        write!(f, "{item}\r\n")?;
    }
    Ok(())
}

/// Writes each nested sub-component in `items` as-is — a sub-component's own
/// `Display` already ends with its CRLF-terminated `END:...` line, so no
/// extra separator is added here.
pub(crate) fn write_components<T: std::fmt::Display>(
    f: &mut std::fmt::Formatter<'_>,
    items: &[T],
) -> std::fmt::Result {
    for item in items {
        write!(f, "{item}")?;
    }
    Ok(())
}
