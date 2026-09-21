//! vCard parsing layer (issue #45): what RFC 6350 §3.2/§3.3 and §6.1/§6.7.9
//! require of `BEGIN:VCARD`, `VERSION`, groups, folding and streams.
#![cfg(feature = "rfc-6350")]

use cratical::vcard::{
    ParseError, VCard, properties::Property, values::Version,
};

const MINIMAL: &[u8] =
    b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Simon Perreault\r\nEND:VCARD\r\n";

fn group_and_line(p: &Property) -> (Option<&str>, String) {
    (p.group().map(|g| g.as_str()), p.to_string())
}

#[test]
fn parses_a_minimal_vcard() {
    let card = VCard::parse(MINIMAL).unwrap();
    assert_eq!(card.version().value(), Version::V4_0);
    assert_eq!(card.properties().len(), 1);
}

#[test]
fn version_is_kept_out_of_the_other_properties() {
    let card = VCard::parse(MINIMAL).unwrap();
    assert!(
        card.properties()
            .iter()
            .all(|p| !matches!(p, Property::Version(_)))
    );
}

// ---- BEGIN / END ----

#[test]
fn begin_end_and_property_names_are_case_insensitive() {
    // §6.1.1/§6.1.2: "The value is case-insensitive." §3.3: names too.
    let card = VCard::parse(
        b"begin:vcard\r\nversion:4.0\r\nfn:Simon\r\nend:vcard\r\n",
    )
    .unwrap();
    assert_eq!(card.version().value(), Version::V4_0);
}

#[test]
fn the_final_end_needs_no_trailing_crlf() {
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Simon\r\nEND:VCARD";
    assert!(VCard::parse(src).is_ok());
}

#[test]
fn a_bare_lf_is_accepted_as_the_line_break() {
    let src = b"BEGIN:VCARD\nVERSION:4.0\nFN:Simon\nEND:VCARD\n";
    assert!(VCard::parse(src).is_ok());
}

#[test]
fn a_missing_end_errors() {
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Simon\r\n";
    assert!(matches!(VCard::parse(src), Err(ParseError::UnexpectedEof)));
}

#[test]
fn a_mismatched_end_errors() {
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Simon\r\nEND:VCALENDAR\r\n";
    assert!(matches!(
        VCard::parse(src),
        Err(ParseError::MismatchedEnd { .. })
    ));
}

#[test]
fn a_begin_that_is_not_vcard_errors() {
    let src = b"BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n";
    assert!(matches!(
        VCard::parse(src),
        Err(ParseError::NotVCard { .. })
    ));
}

#[test]
fn junk_before_begin_errors() {
    let src = b"FN:Simon\r\nBEGIN:VCARD\r\nVERSION:4.0\r\nEND:VCARD\r\n";
    assert!(matches!(
        VCard::parse(src),
        Err(ParseError::UnexpectedToken { .. })
    ));
}

#[test]
fn empty_input_is_not_a_vcard() {
    assert!(VCard::parse(b"").is_err());
}

#[test]
fn a_nested_begin_errors_instead_of_being_accepted() {
    // The only legal nesting is 3.0's AGENT (RFC 2426 §3.5.1), which is
    // #51's to model; it must not silently parse in the meantime.
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nBEGIN:VCARD\r\nVERSION:4.0\r\nEND:VCARD\r\nEND:VCARD\r\n";
    assert!(matches!(
        VCard::parse(src),
        Err(ParseError::NestedComponent { line: 2 })
    ));
}

// ---- VERSION ----

#[test]
fn a_vcard_4_0_without_version_errors() {
    let src = b"BEGIN:VCARD\r\nFN:Simon\r\nEND:VCARD\r\n";
    assert!(matches!(VCard::parse(src), Err(ParseError::MissingVersion)));
}

#[test]
fn version_4_0_must_come_immediately_after_begin() {
    // §6.7.9: "it must appear immediately after BEGIN:VCARD".
    let src = b"BEGIN:VCARD\r\nFN:Simon\r\nVERSION:4.0\r\nEND:VCARD\r\n";
    assert!(matches!(
        VCard::parse(src),
        Err(ParseError::VersionNotFirst)
    ));
}

#[test]
fn a_second_version_errors() {
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nVERSION:4.0\r\nEND:VCARD\r\n";
    assert!(matches!(
        VCard::parse(src),
        Err(ParseError::DuplicateVersion)
    ));
}

#[test]
fn an_unknown_version_errors() {
    for version in ["2.1", "5.0", "4", ""] {
        let src = format!(
            "BEGIN:VCARD\r\nVERSION:{version}\r\nFN:Simon\r\nEND:VCARD\r\n"
        );
        assert!(
            matches!(
                VCard::parse(src.as_bytes()),
                Err(ParseError::UnsupportedVersion(_))
            ),
            "VERSION:{version}"
        );
    }
}

#[cfg(feature = "rfc-2426")]
mod v3 {
    use super::*;

    #[test]
    fn parses_version_3_0() {
        let src = b"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Simon\r\nN:Perreault;Simon;;;\r\nEND:VCARD\r\n";
        let card = VCard::parse(src).unwrap();
        assert_eq!(card.version().value(), Version::V3_0);
    }

    #[test]
    fn version_3_0_may_appear_anywhere() {
        // RFC 2426 §3.6.9 only says the property MUST be present;
        // RFC 6350 §6.7.9: "earlier versions of vCard allowed this
        // property to be placed anywhere in the vCard object".
        let src = b"BEGIN:VCARD\r\nFN:Simon\r\nVERSION:3.0\r\nN:Perreault;Simon;;;\r\nEND:VCARD\r\n";
        let card = VCard::parse(src).unwrap();
        assert_eq!(card.version().value(), Version::V3_0);
        assert_eq!(card.properties().len(), 2);
    }

    #[test]
    fn a_vcard_3_0_without_version_still_errors() {
        let src =
            b"BEGIN:VCARD\r\nFN:Simon\r\nN:Perreault;Simon;;;\r\nEND:VCARD\r\n";
        assert!(matches!(VCard::parse(src), Err(ParseError::MissingVersion)));
    }
}

#[cfg(not(feature = "rfc-2426"))]
#[test]
fn version_3_0_is_unsupported_without_rfc_2426() {
    let src = b"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Simon\r\nEND:VCARD\r\n";
    assert!(matches!(
        VCard::parse(src),
        Err(ParseError::UnsupportedVersion(_))
    ));
}

// ---- groups (§3.3) ----

#[test]
fn a_group_is_kept_on_the_property_and_round_trips() {
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nitem1.TEL;TYPE=cell:+1 555\r\nitem1.X-ABLabel:Mobile\r\nEND:VCARD\r\n";
    let card = VCard::parse(src).unwrap();
    let props: Vec<_> = card.properties().iter().map(group_and_line).collect();
    assert_eq!(
        props,
        [
            (Some("item1"), "item1.TEL;TYPE=cell:+1 555".to_owned()),
            (Some("item1"), "item1.X-ABLABEL:Mobile".to_owned()),
        ]
    );
}

#[test]
fn a_property_without_a_group_has_none() {
    let card = VCard::parse(MINIMAL).unwrap();
    assert_eq!(card.properties()[0].group(), None);
}

#[test]
fn the_group_keeps_its_case() {
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nItEm1.FN:Simon\r\nEND:VCARD\r\n";
    let card = VCard::parse(src).unwrap();
    assert_eq!(card.properties()[0].group().unwrap().as_str(), "ItEm1");
}

#[test]
fn version_may_carry_a_group() {
    let src = b"BEGIN:VCARD\r\ng.VERSION:4.0\r\nFN:Simon\r\nEND:VCARD\r\n";
    let card = VCard::parse(src).unwrap();
    assert_eq!(card.version().group().unwrap().as_str(), "g");
}

#[test]
fn begin_and_end_cannot_carry_a_group() {
    let src = b"g.BEGIN:VCARD\r\nVERSION:4.0\r\nEND:VCARD\r\n";
    assert!(matches!(VCard::parse(src), Err(ParseError::Lexer(_))));
}

// ---- content lines ----

#[test]
fn an_empty_value_is_kept() {
    // `1*contentline` with `value = *VALUE-CHAR`: empty is fine.
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:\r\nEND:VCARD\r\n";
    let card = VCard::parse(src).unwrap();
    assert_eq!(card.properties()[0].to_string(), "NOTE:");
}

#[test]
fn a_colon_inside_a_quoted_param_does_not_start_the_value() {
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nADR;LABEL=\"a:b;c,d\":;;x\r\nEND:VCARD\r\n";
    let card = VCard::parse(src).unwrap();
    assert_eq!(
        card.properties()[0].to_string(),
        "ADR;LABEL=\"a:b;c,d\":;;x"
    );
}

#[test]
fn a_property_line_without_a_colon_errors() {
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN Simon\r\nEND:VCARD\r\n";
    assert!(VCard::parse(src).is_err());
}

#[test]
fn a_param_without_equals_errors() {
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nTEL;CELL:+1\r\nEND:VCARD\r\n";
    assert!(VCard::parse(src).is_err());
}

#[test]
fn unknown_and_x_properties_are_never_an_error() {
    // §6.10: extension properties are open-ended.
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nX-CUSTOM;X-P=1:v\r\nSOME-FUTURE-PROP:v\r\nEND:VCARD\r\n";
    let card = VCard::parse(src).unwrap();
    assert!(matches!(card.properties()[0], Property::Xprop(_)));
    assert!(matches!(card.properties()[1], Property::Iana(_)));
}

#[test]
fn parameters_round_trip_untouched() {
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nX-A;TYPE=\"work,voice\";X-B=^'q^':v\r\nEND:VCARD\r\n";
    let card = VCard::parse(src).unwrap();
    assert_eq!(
        card.properties()[0].to_string(),
        "X-A;TYPE=\"work,voice\";X-B=^'q^':v"
    );
}

// ---- folding (§3.2) ----

#[test]
fn folded_lines_are_unfolded_exactly() {
    // The single WSP after the CRLF is removed, and nothing else.
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:This is a long descrip\r\n tion that exists o\r\n n a long line.\r\nEND:VCARD\r\n";
    let card = VCard::parse(src).unwrap();
    assert_eq!(
        card.properties()[0].to_string(),
        "NOTE:This is a long description that exists on a long line."
    );
}

#[test]
fn a_fold_may_use_a_tab() {
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:ab\r\n\tcd\r\nEND:VCARD\r\n";
    let card = VCard::parse(src).unwrap();
    assert_eq!(card.properties()[0].to_string(), "NOTE:abcd");
}

#[test]
fn only_one_whitespace_char_is_removed_per_fold() {
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:ab\r\n  cd\r\nEND:VCARD\r\n";
    let card = VCard::parse(src).unwrap();
    assert_eq!(card.properties()[0].to_string(), "NOTE:ab cd");
}

#[test]
fn a_multibyte_char_split_across_a_fold_is_restored() {
    // §3.2: implementations SHOULD unfold so a split multi-octet sequence
    // is properly restored. "é" is C3 A9.
    let src =
        b"BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:caf\xC3\r\n \xA9\r\nEND:VCARD\r\n";
    let card = VCard::parse(src).unwrap();
    assert_eq!(card.properties()[0].to_string(), "NOTE:caf\u{e9}");
}

#[test]
fn a_fold_may_split_the_begin_line() {
    let src = b"BEGIN:VC\r\n ARD\r\nVERSION:4.0\r\nEND:VCARD\r\n";
    assert!(VCard::parse(src).is_ok());
}

#[test]
fn a_leading_utf8_bom_is_skipped() {
    let mut src = b"\xEF\xBB\xBF".to_vec();
    src.extend_from_slice(MINIMAL);
    assert!(VCard::parse(&src).is_ok());
}

#[test]
fn non_utf8_content_errors_instead_of_panicking() {
    let src = b"BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:\xFF\xFE\r\nEND:VCARD\r\n";
    assert!(matches!(VCard::parse(src), Err(ParseError::Utf8(_))));
}

// ---- streams ----

#[test]
fn a_stream_holds_several_cards_in_order() {
    let mut src = MINIMAL.to_vec();
    src.extend_from_slice(
        b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Second\r\nEND:VCARD\r\n",
    );
    let cards = VCard::parse_stream(&src).unwrap();
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].properties()[0].to_string(), "FN:Simon Perreault");
    assert_eq!(cards[1].properties()[0].to_string(), "FN:Second");
}

#[test]
fn a_stream_may_mix_versions() {
    #[cfg(feature = "rfc-2426")]
    {
        let mut src = MINIMAL.to_vec();
        src.extend_from_slice(
            b"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Second\r\nEND:VCARD\r\n",
        );
        let cards = VCard::parse_stream(&src).unwrap();
        assert_eq!(cards[0].version().value(), Version::V4_0);
        assert_eq!(cards[1].version().value(), Version::V3_0);
    }
}

#[test]
fn a_stream_of_one_is_fine_and_an_empty_stream_is_empty() {
    assert_eq!(VCard::parse_stream(MINIMAL).unwrap().len(), 1);
    assert!(VCard::parse_stream(b"").unwrap().is_empty());
}

#[test]
fn a_stream_ending_without_a_final_crlf_parses() {
    let mut src = MINIMAL.to_vec();
    src.extend_from_slice(b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:B\r\nEND:VCARD");
    assert_eq!(VCard::parse_stream(&src).unwrap().len(), 2);
}

#[test]
fn a_bad_card_in_a_stream_fails_the_stream() {
    let mut src = MINIMAL.to_vec();
    src.extend_from_slice(b"BEGIN:VCARD\r\nFN:no version\r\nEND:VCARD\r\n");
    assert!(VCard::parse_stream(&src).is_err());
}

#[test]
fn parse_rejects_a_second_card() {
    let mut src = MINIMAL.to_vec();
    src.extend_from_slice(MINIMAL);
    assert!(matches!(VCard::parse(&src), Err(ParseError::TrailingData)));
}

#[test]
fn parse_rejects_junk_after_the_card() {
    let mut src = MINIMAL.to_vec();
    src.extend_from_slice(b"FN:stray\r\n");
    assert!(VCard::parse(&src).is_err());
}
