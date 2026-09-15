/// Parses a type from its raw iCalendar wire-format bytes.
pub trait DateTimeExt {
    /// Parses `b`, the raw bytes of a `DATE`/`DATE-TIME` value, into `Self`.
    fn from_ical(b: &[u8]) -> Self;
}
/// `chrono` format string for a `DATE` value, e.g. `20260909`.
pub const ICAL_DATE_FMT: &str = "%Y%m%d"; // 20260909
/// `chrono` format string for a floating/local `DATE-TIME` value, e.g. `20260909T153000`.
pub const ICAL_DATETIME_FMT: &str = "%Y%m%dT%H%M%S"; // 20260909T153000  (floating/local)
/// `chrono` format string for a UTC `DATE-TIME` value, e.g. `20260909T153000Z`.
pub const ICAL_DATETIME_UTC_FMT: &str = "%Y%m%dT%H%M%SZ"; // 20260909T153000Z (UTC)
