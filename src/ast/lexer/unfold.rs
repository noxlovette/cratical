/// Unfolds RFC 5545 folded content lines.
///
/// Lines of text SHOULD NOT be longer than 75 octets, excluding the line
/// break. Long content lines SHOULD be split into a multiple line
/// representations using a line "folding" technique. That is, a long line
/// can be split between any two characters by inserting a CRLF immediately
/// followed by a single linear white-space character (i.e., SPACE or
/// HTAB). Any sequence of CRLF followed immediately by a single linear
/// white-space character is ignored (i.e., removed) when processing the
/// content type.
///
/// The process of moving from this folded multiple-line representation to
/// its single-line representation is called "unfolding". Unfolding is
/// accomplished by removing the CRLF and the linear white-space character
/// that immediately follows.
///
/// When parsing a content line, folded lines MUST first be unfolded
/// according to the unfolding procedure described above.
///
/// A bare `\n` is also accepted as the line break half of a fold, mirroring
/// the lexer's own leniency toward bare-LF line endings (see
/// [`super::Lexer`]'s handling of `\n` as a line terminator): real-world
/// `.ics` files produced without CRLF normalization fold with `\n` + WSP
/// just as often as `\r\n` + WSP.
///
/// # Example
///
/// > DESCRIPTION:This is a lo
/// > ng description
/// > that exists on a long line.
///
/// unfolds to:
///
/// > DESCRIPTION:This is a long description that exists on a long line.
///
/// [Section 3.1](https://datatracker.ietf.org/doc/html/rfc5545#section-3.1)
pub fn unfold(src: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len());
    let mut i = 0;

    while i < src.len() {
        let crlf_fold = src[i] == b'\r'
            && src.get(i + 1) == Some(&b'\n')
            && matches!(src.get(i + 2), Some(b' ') | Some(b'\t'));
        let lf_fold = src[i] == b'\n'
            && matches!(src.get(i + 1), Some(b' ') | Some(b'\t'));

        if crlf_fold {
            i += 3;
        } else if lf_fold {
            i += 2;
        } else {
            out.push(src[i]);
            i += 1;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unfolds_the_rfc_worked_example() {
        // §3.1's own example: a folded DESCRIPTION collapses back to the
        // single logical line it represents.
        let folded = b"DESCRIPTION:This is a lo\r\n ng description\r\n  that exists on a long line.\r\n";
        let expected =
            b"DESCRIPTION:This is a long description that exists on a long line.\r\n";

        assert_eq!(unfold(folded), expected.to_vec());
    }

    #[test]
    fn bare_crlf_not_followed_by_wsp_is_preserved() {
        // A real line break (not a fold) must survive unfolding untouched.
        let src = b"UID:foo\r\nDTSTAMP:20240102T090000Z\r\n";

        assert_eq!(unfold(src), src.to_vec());
    }

    #[test]
    fn consecutive_folds_all_collapse() {
        // Two short folded segments in a row both get unfolded, not just
        // the first.
        let folded = b"SUMMARY:A\r\n B\r\n C\r\n";
        let expected = b"SUMMARY:ABC\r\n";

        assert_eq!(unfold(folded), expected.to_vec());
    }

    #[test]
    fn input_without_folding_is_unchanged() {
        let src = b"BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n";

        assert_eq!(unfold(src), src.to_vec());
    }

    #[test]
    fn lone_cr_not_followed_by_lf_passes_through() {
        // Malformed CRLF is not this function's concern - it's left
        // untouched for the lexer's own `LexerError::Crlf` to catch.
        let src = b"BEGIN\rX";

        assert_eq!(unfold(src), src.to_vec());
    }

    #[test]
    fn htab_continuation_is_also_unfolded() {
        // The RFC permits either SPACE or HTAB as the fold's leading WSP.
        let folded = b"SUMMARY:A\r\n\tB\r\n";
        let expected = b"SUMMARY:AB\r\n";

        assert_eq!(unfold(folded), expected.to_vec());
    }

    #[test]
    fn bare_lf_fold_is_also_unfolded() {
        // Real-world LF-only `.ics` files fold with `\n` + WSP, not just
        // `\r\n` + WSP (issue #10).
        let folded = b"SUMMARY:A\n B\n C\n";
        let expected = b"SUMMARY:ABC\n";

        assert_eq!(unfold(folded), expected.to_vec());
    }

    #[test]
    fn bare_lf_not_followed_by_wsp_is_preserved() {
        // A real LF line break (not a fold) must survive unfolding
        // untouched, mirroring `bare_crlf_not_followed_by_wsp_is_preserved`.
        let src = b"UID:foo\nDTSTAMP:20240102T090000Z\n";

        assert_eq!(unfold(src), src.to_vec());
    }
}
