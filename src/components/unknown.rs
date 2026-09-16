/// An unrecognized top-level calendar component — an `iana-comp` or
/// `x-comp` whose name isn't one of the five component types RFC 5545
/// §3.6 defines directly under `VCALENDAR` (`VEVENT`, `VTODO`, `VJOURNAL`,
/// `VFREEBUSY`, `VTIMEZONE`).
///
/// Applications MUST ignore x-comp and iana-comp values they don't
/// recognize.  Applications that support importing iCalendar objects
/// SHOULD support all of the component types defined in this document,
/// and SHOULD NOT silently drop any components as that can lead to user
/// data loss.
///
/// This crate has no typed model for an arbitrary component's contents,
/// so — per the SHOULD above — its content lines are preserved verbatim
/// rather than parsed or discarded, and any component nested inside it is
/// captured the same opaque way, recursively.
///
/// Example:
///
/// > BEGIN:VCALENDAR
/// >
/// > BEGIN:X-MY-COMPONENT
/// >
/// > X-MY-PROPERTY:some value
/// >
/// > END:X-MY-COMPONENT
/// >
/// > END:VCALENDAR
///
/// [Section 3.6](https://datatracker.ietf.org/doc/html/rfc5545#section-3.6)
#[derive(Debug)]
pub struct UnknownComponent {
    pub(crate) name: String,
    pub(crate) lines: Vec<String>,
    pub(crate) components: Vec<UnknownComponent>,
}

impl UnknownComponent {
    /// Constructs an `UnknownComponent` directly from its raw parts.
    /// RFC 5545 imposes no rules on an unrecognized component's contents
    /// (see this type's own docs), so unlike every other component in
    /// this crate, there's no builder — nothing to validate, and every
    /// field is required.
    pub fn new(
        name: String,
        lines: Vec<String>,
        components: Vec<UnknownComponent>,
    ) -> Self {
        Self {
            name,
            lines,
            components,
        }
    }

    /// The component's name (`iana-token` or `x-name`), exactly as it
    /// appeared after `BEGIN:`/`END:`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// This component's own content lines (RFC 5545 §3.8), verbatim and
    /// unparsed — this crate has no typed model for an unrecognized
    /// component's properties, so they're kept as raw text rather than
    /// dropped.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Any further unrecognized components nested directly inside this
    /// one.
    pub fn components(&self) -> &[UnknownComponent] {
        &self.components
    }
}

impl std::fmt::Display for UnknownComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BEGIN:{}\r\n", self.name)?;
        for line in &self.lines {
            write!(f, "{line}\r\n")?;
        }
        crate::components::write_components(f, &self.components)?;
        write!(f, "END:{}\r\n", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_round_trips_a_minimal_unknown_component() {
        let component = UnknownComponent::new(
            "X-MY-COMPONENT".into(),
            vec!["X-MY-PROPERTY:some value".into()],
            Vec::new(),
        );
        assert_eq!(
            component.to_string(),
            "BEGIN:X-MY-COMPONENT\r\nX-MY-PROPERTY:some \
             value\r\nEND:X-MY-COMPONENT\r\n"
        );
    }
}
