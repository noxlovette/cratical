use std::fmt;

/// The longest a content line SHOULD be, in octets, excluding the line
/// break.
const MAX_OCTETS: usize = 75;

/// Writes `line` and its CRLF, folding it as RFC 6350 §3.2 describes.
///
/// Lines of text SHOULD be folded to no more than 75 octets excluding the
/// line break. A long line is split by inserting a CRLF immediately followed
/// by a single space, which [`unfold`](super::unfold::unfold) removes again.
/// The space counts toward the 75 octets of the line it starts, so the first
/// line holds 75 octets of `line` and each continuation 74.
///
/// The RFC says to avoid folding inside a UTF-8 multi-octet sequence: a line
/// is cut earlier, on a character boundary, when 75 octets would end inside
/// one.
///
/// `line` must not contain a line break; the values of a property have them
/// escaped already.
///
/// # Example
///
/// > NOTE:This is a long description that exists on a long line.
///
/// folds, at 75 octets, into:
///
/// > NOTE:This is a long description that exists on a long line. This is a lon
/// > g description that exists on a long line.
///
/// [Section 3.2](https://datatracker.ietf.org/doc/html/rfc6350#section-3.2)
pub(crate) fn write_folded(
    out: &mut impl fmt::Write,
    line: &str,
) -> fmt::Result {
    let mut rest = line;
    let mut room = MAX_OCTETS;
    while rest.len() > room {
        let mut cut = room;
        while !rest.is_char_boundary(cut) {
            cut -= 1;
        }
        let (head, tail) = rest.split_at(cut);
        out.write_str(head)?;
        out.write_str("\r\n ")?;
        rest = tail;
        // The leading space of a continuation line takes one octet.
        room = MAX_OCTETS - 1;
    }
    out.write_str(rest)?;
    out.write_str("\r\n")
}
