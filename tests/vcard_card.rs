//! The vCard as a whole (issue #50): what RFC 6350 requires of a card that
//! needs all its properties, how one is built from code, and how it is
//! written back.
#![cfg(feature = "rfc-6350")]

use cratical::vcard::{
    ParseError, VCard, ValidationError,
    params::Parameters,
    properties::{
        self, Birthday, Email, FormattedName, Group, Iana, Member, Note,
        Property, Telephone, Uid, Xprop,
    },
    values::{self, DateAndOrTimeOrText, Text, TextOrUri, Uri},
};

/// A vCard 4.0 with `lines` after `VERSION`, as written on the wire.
fn wire(lines: &str) -> String {
    format!("BEGIN:VCARD\r\nVERSION:4.0\r\n{lines}\r\nEND:VCARD\r\n")
}

/// The same, with an `FN` so it only fails for what a test is about.
fn parse(lines: &str) -> Result<VCard, ParseError> {
    VCard::parse(wire(&format!("{lines}\r\nFN:Test")).as_bytes())
}

fn invalid(lines: &str) -> ValidationError {
    match parse(lines) {
        Err(ParseError::Invalid(e)) => e,
        Err(e) => panic!("{lines:?} should be invalid, but is: {e}"),
        Ok(_) => panic!("{lines:?} should be invalid, but parses"),
    }
}

fn valid(lines: &str) -> VCard {
    parse(lines).unwrap_or_else(|e| panic!("{lines:?} should parse: {e}"))
}

fn text(s: &str) -> Text {
    Text::from(s)
}

fn uri(s: &str) -> Uri {
    Uri::try_from(s.as_bytes()).unwrap()
}

fn params(wire: &str) -> Parameters {
    Parameters::try_from(wire.as_bytes()).unwrap()
}

// ---- §6.2.1: FN is required ----

#[test]
fn a_card_without_fn_is_invalid() {
    let src = wire("N:Perreault;Simon;;;");
    assert!(matches!(
        VCard::parse(src.as_bytes()),
        Err(ParseError::Invalid(ValidationError::MissingFormattedName))
    ));
}

#[test]
fn a_card_can_have_several_fn() {
    // "Cardinality: 1*"
    let src = wire("FN:Simon Perreault\r\nFN:Perreault Simon");
    assert_eq!(
        VCard::parse(src.as_bytes())
            .unwrap()
            .formatted_names()
            .count(),
        2
    );
}

#[test]
fn fn_alternatives_with_one_altid_are_valid() {
    // §5.4's own use: translations of one property.
    let card = VCard::parse(
        wire("FN;ALTID=1;LANGUAGE=en:Boss\r\nFN;ALTID=1;LANGUAGE=fr:Patron")
            .as_bytes(),
    )
    .unwrap();
    assert_eq!(card.formatted_names().count(), 2);
}

// ---- §6: cardinality *1 ----

const AT_MOST_ONE: &[(&str, &str)] = &[
    ("KIND", "individual"),
    ("N", "Perreault;Simon;;;"),
    ("BDAY", "--0203"),
    ("ANNIVERSARY", "20090808T1430-0500"),
    ("GENDER", "M"),
    ("PRODID", "-//ONLINE DIRECTORY//EN"),
    ("REV", "19951031T222710Z"),
    ("UID", "urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6"),
];

#[test]
fn a_property_with_cardinality_one_at_most_cannot_occur_twice() {
    for (name, value) in AT_MOST_ONE {
        let lines = format!("{name}:{value}\r\n{name}:{value}");
        assert_eq!(
            invalid(&lines),
            ValidationError::TooMany(name),
            "{name} twice"
        );
    }
}

#[test]
fn a_property_with_cardinality_one_at_most_can_occur_once() {
    for (name, value) in AT_MOST_ONE {
        valid(&format!("{name}:{value}"));
    }
}

#[test]
fn instances_with_one_altid_count_as_one() {
    // §5.4: "N;ALTID=1;LANGUAGE=jp ... N;ALTID=1;LANGUAGE=en" is legal.
    valid(
        "N;ALTID=1;LANGUAGE=jp:Yamada;Taro;;;\r\nN;ALTID=1;LANGUAGE=en:Yamada;Taro;;;",
    );
}

#[test]
fn instances_with_different_altids_count_as_two() {
    // §5.4: "The last line should probably have ALTID=2. But that would be
    // illegal because N has cardinality *1."
    assert_eq!(
        invalid("N;ALTID=1:A;;;;\r\nN;ALTID=2:B;;;;"),
        ValidationError::TooMany("N")
    );
}

#[test]
fn an_instance_without_altid_is_no_alternative_of_another() {
    // §5.4: "Property instances without the ALTID parameter MUST NOT be
    // considered an alternative representation of any other property
    // instance."
    assert_eq!(
        invalid("N;ALTID=1:A;;;;\r\nN:B;;;;"),
        ValidationError::TooMany("N")
    );
    assert_eq!(
        invalid("BDAY:--0203\r\nBDAY:--0203"),
        ValidationError::TooMany("BDAY")
    );
}

#[test]
fn an_altid_belongs_to_its_property_name() {
    // §5.4: "Values for the ALTID parameter are not globally unique: they
    // MAY be reused for different property names."
    valid("N;ALTID=1:A;;;;\r\nBDAY;ALTID=1:--0203\r\nGENDER;ALTID=1:M");
}

#[test]
fn properties_with_cardinality_any_can_repeat() {
    valid(
        "TEL:1\r\nTEL:2\r\nEMAIL:a@example.com\r\nEMAIL:b@example.com\r\nTITLE:A\r\nTITLE:B\r\nNOTE:x\r\nNOTE:y\r\nURL:http://a.example/\r\nURL:http://b.example/\r\nX-A:1\r\nX-A:2",
    );
}

// ---- §6.6.5: MEMBER ----

#[test]
fn member_needs_kind_group() {
    assert_eq!(
        invalid("MEMBER:urn:uuid:03a0e51f-d1aa-4385-8a53-e29025acd8af"),
        ValidationError::MemberWithoutGroup
    );
}

#[test]
fn member_is_not_allowed_on_the_other_kinds() {
    for kind in ["individual", "org", "location", "x-thing"] {
        assert_eq!(
            invalid(&format!("KIND:{kind}\r\nMEMBER:mailto:a@example.com")),
            ValidationError::MemberWithoutGroup,
            "KIND:{kind}"
        );
    }
}

#[test]
fn member_is_allowed_on_a_group_whatever_its_case_or_place() {
    valid("KIND:group\r\nMEMBER:mailto:a@example.com");
    valid("KIND:GROUP\r\nMEMBER:mailto:a@example.com");
    valid("MEMBER:mailto:a@example.com\r\nKIND:group");
}

#[test]
fn a_group_card_without_members_is_fine() {
    valid("KIND:group");
}

// ---- §6.7.7: CLIENTPIDMAP ----

#[test]
fn a_pid_source_needs_its_clientpidmap() {
    assert_eq!(
        invalid("TEL;PID=1.1:1"),
        ValidationError::UnmappedPidSource(1)
    );
    assert_eq!(
        invalid(
            "TEL;PID=1.1:1\r\nCLIENTPIDMAP:2;urn:uuid:d89c9c7a-2e1b-4832-82de-7e992d95faa5"
        ),
        ValidationError::UnmappedPidSource(1)
    );
}

#[test]
fn every_distinct_pid_source_needs_its_own() {
    // The example of §6.7.7: sources 1 and 2, both mapped.
    valid(
        "TEL;PID=3.1,4.2:1\r\nEMAIL;PID=4.1,5.2:a@example.com\r\nCLIENTPIDMAP:1;urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b\r\nCLIENTPIDMAP:2;urn:uuid:d89c9c7a-2e1b-4832-82de-7e992d95faa5",
    );
    assert_eq!(
        invalid(
            "TEL;PID=3.1,4.2:1\r\nCLIENTPIDMAP:1;urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b"
        ),
        ValidationError::UnmappedPidSource(2)
    );
}

#[test]
fn a_pid_with_no_source_needs_no_clientpidmap() {
    // §5.5: the second field is optional ("PID=1").
    valid("TEL;PID=1,2:1");
}

// ---- VERSION ----

#[test]
fn version_is_written_right_after_begin() {
    let card = valid("KIND:individual");
    assert!(
        card.to_string()
            .starts_with("BEGIN:VCARD\r\nVERSION:4.0\r\nKIND:individual\r\n")
    );
}

// ---- accessors ----

#[test]
fn kind_defaults_to_individual() {
    // §6.1.4: "If this property is absent, "individual" MUST be assumed as
    // the default."
    assert_eq!(valid("N:A;;;;").kind(), values::Kind::Individual);
    assert_eq!(valid("KIND:org").kind(), values::Kind::Org);
}

#[test]
fn uid_is_there_when_the_card_has_one() {
    let card = valid("UID:urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6");
    let uid = card.uid().expect("a UID");
    assert!(matches!(uid.value(), values::Uid::Uri(_)));
    // RFC 6350 doesn't require one, so a card without is not an error.
    assert!(valid("N:A;;;;").uid().is_none());
}

// ---- §3.2: folding ----

fn lines(card: &VCard) -> Vec<String> {
    let out = card.to_string();
    assert!(out.ends_with("\r\n"), "ends with CRLF");
    out.strip_suffix("\r\n")
        .unwrap()
        .split("\r\n")
        .map(str::to_owned)
        .collect()
}

fn unfold(s: &str) -> String {
    s.replace("\r\n ", "")
}

#[test]
fn no_line_is_longer_than_75_octets() {
    let long = "a".repeat(400);
    let card = valid(&format!("NOTE:{long}\r\nX-LONG;X-PARAM=1:{long}"));
    for line in lines(&card) {
        assert!(line.len() <= 75, "{} octets: {line:?}", line.len());
    }
}

#[test]
fn a_long_line_is_folded_with_a_space_and_unfolds_to_itself() {
    let long = "abcdefghij".repeat(30);
    let card = valid(&format!("NOTE:{long}"));
    let out = card.to_string();
    assert!(out.contains("\r\n "), "folded: {out:?}");
    assert_eq!(
        unfold(&out),
        format!(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:{long}\r\nFN:Test\r\nEND:VCARD\r\n"
        )
    );
}

#[test]
fn a_fold_makes_the_first_line_75_octets_and_the_rest_74_and_the_space() {
    // The space that starts a continuation counts toward its 75 octets.
    let card = valid(&format!("NOTE:{}", "x".repeat(200)));
    let all = lines(&card);
    let note: Vec<_> = all
        .iter()
        .skip_while(|l| !l.starts_with("NOTE:"))
        .take_while(|l| l.starts_with("NOTE:") || l.starts_with(' '))
        .collect();
    assert_eq!(note[0].len(), 75);
    for cont in &note[1..note.len() - 1] {
        assert_eq!(cont.len(), 75);
        assert!(cont.starts_with(' ') && !cont.starts_with("  "));
    }
}

#[test]
fn a_line_of_exactly_75_octets_is_not_folded() {
    let card = valid(&format!("NOTE:{}", "x".repeat(70)));
    assert!(lines(&card).contains(&format!("NOTE:{}", "x".repeat(70))));
    let folded = valid(&format!("NOTE:{}", "x".repeat(71)));
    assert!(folded.to_string().contains("\r\n "));
}

#[test]
fn folding_never_splits_a_utf8_character() {
    // "é" is two octets; at every offset the 75th octet can fall inside one.
    for pad in 0..4 {
        let value = format!("{}{}", "x".repeat(pad), "é".repeat(120));
        let card = valid(&format!("NOTE:{value}"));
        let out = card.to_string();
        for line in out.split("\r\n") {
            assert!(line.len() <= 75, "{} octets", line.len());
            // Each physical line is UTF-8 on its own: `split` on a `&str`
            // couldn't have produced it otherwise, so check the characters
            // survive the unfold too.
        }
        let back = VCard::parse(out.as_bytes()).unwrap();
        assert_eq!(back, card, "pad {pad}");
        assert_eq!(unfold(&out).matches('é').count(), 120);
    }
}

#[test]
fn folding_handles_four_octet_characters() {
    let value = "😀".repeat(60);
    let card = valid(&format!("NOTE:{value}"));
    let out = card.to_string();
    assert!(out.split("\r\n").all(|l| l.len() <= 75));
    assert_eq!(VCard::parse(out.as_bytes()).unwrap(), card);
}

// ---- round trip ----

fn round_trips(card: &VCard) {
    let written = card.to_string();
    let read = VCard::parse(written.as_bytes())
        .unwrap_or_else(|e| panic!("{written:?} should parse: {e}"));
    assert_eq!(&read, card, "{written:?}");
    assert_eq!(read.to_string(), written);
}

// §8 of the RFC.
const AUTHOR: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nKIND:individual\r\nFN:Simon Perreault\r\nN:Perreault;Simon;;;ing. jr,M.Sc.\r\nBDAY:--0203\r\nANNIVERSARY:20090808T1430-0500\r\nGENDER:M\r\nLANG;PREF=1:fr\r\nLANG;PREF=2:en\r\nORG;TYPE=work:Viagenie\r\nADR;TYPE=work:;Suite D2-630;2875 Laurier;\r\n Quebec;QC;G1V 2M2;Canada\r\nTEL;VALUE=uri;TYPE=\"work,voice\";PREF=1:tel:+1-418-656-9254;ext=102\r\nTEL;VALUE=uri;TYPE=\"work,cell,voice,video,text\":tel:+1-418-262-6501\r\nEMAIL;TYPE=work:simon.perreault@viagenie.ca\r\nGEO;TYPE=work:geo:46.772673,-71.282945\r\nKEY;TYPE=work;VALUE=uri:\r\n http://www.viagenie.ca/simon.perreault/simon.asc\r\nTZ:-0500\r\nURL;TYPE=home:http://nomis80.org\r\nEND:VCARD\r\n";

#[test]
fn the_rfc_author_card_round_trips() {
    round_trips(&VCard::parse(AUTHOR.as_bytes()).unwrap());
}

#[test]
fn what_is_written_keeps_to_75_octets_and_reads_as_the_same_card() {
    let card = VCard::parse(AUTHOR.as_bytes()).unwrap();
    for line in lines(&card) {
        assert!(line.len() <= 75, "{line:?}");
    }
}

#[test]
fn groups_extensions_and_parameter_order_survive() {
    let card = valid(
        "item1.TEL;TYPE=cell;PREF=1:+1 555\r\nitem1.X-ABLabel:Mobile\r\nX-CUSTOM;X-P=1:v\r\nSOME-FUTURE-PROP:v\r\nNOTE;LANGUAGE=en;ALTID=1:x",
    );
    round_trips(&card);
    let out = card.to_string();
    assert!(out.contains("\r\nitem1.TEL;"));
    assert!(out.contains("\r\nitem1.X-ABLABEL:Mobile\r\n"));
}

#[test]
fn escapes_survive_a_round_trip() {
    round_trips(&valid(
        "NOTE:a\\, b\\; c\\nnext \\\\ end\r\nN:Doe\\, Jr.;John;;;\r\nCATEGORIES:a\\,b,c",
    ));
}

#[test]
fn a_stream_of_cards_round_trips_one_by_one() {
    let src = format!("{AUTHOR}{AUTHOR}");
    let cards = VCard::parse_stream(src.as_bytes()).unwrap();
    assert_eq!(cards.len(), 2);
    for card in &cards {
        round_trips(card);
    }
}

// ---- building from code ----

fn simon() -> FormattedName {
    FormattedName::new(text("Simon Perreault"))
}

#[test]
fn a_built_card_is_vcard_4_and_has_its_fn() {
    let card = VCard::builder(simon()).build().unwrap();
    assert_eq!(
        card.to_string(),
        "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Simon Perreault\r\nEND:VCARD\r\n"
    );
    assert_eq!(card.version().value(), values::Version::V4_0);
}

#[test]
fn a_built_card_reads_back_as_itself() {
    let card = VCard::builder(simon())
        .property(Email::new(text("simon.perreault@viagenie.ca")))
        .property(Telephone::new(TextOrUri::Uri(uri("tel:+1-418-656-9254"))))
        .property(Telephone::new(TextOrUri::Text(text("+1 (418) 262-6501"))))
        .property(Birthday::new(DateAndOrTimeOrText::Text(text("circa 1800"))))
        .property(Note::new(text("Line one\nLine two, with; punctuation\\")))
        .property(Uid::new(values::Uid::Uri(uri(
            "urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
        ))))
        .property(Xprop::new("X-ABLabel", "Home").unwrap())
        .property(Iana::new("EXAMPLE-IANA-PROPERTY", "value").unwrap())
        .build()
        .unwrap();
    round_trips(&card);
}

#[test]
fn a_value_that_needs_its_value_type_gets_it() {
    // TEL is text unless `VALUE=uri`; RELATED and KEY are URIs unless
    // `VALUE=text`; a UID that is text but looks like a URI needs
    // `VALUE=text` to stay text.
    assert_eq!(
        Telephone::new(TextOrUri::Uri(uri("tel:+1-555"))).to_string(),
        "TEL;VALUE=uri:tel:+1-555"
    );
    assert_eq!(
        Telephone::new(TextOrUri::Text(text("+1 555"))).to_string(),
        "TEL:+1 555"
    );
    assert_eq!(
        Uid::new(values::Uid::Text(text("urn:not:really"))).to_string(),
        "UID;VALUE=text:urn:not:really"
    );
    assert_eq!(
        Uid::new(values::Uid::Text(text("1234-ABCD"))).to_string(),
        "UID:1234-ABCD"
    );
    assert_eq!(
        Birthday::new(DateAndOrTimeOrText::Text(text("circa 1800")))
            .to_string(),
        "BDAY;VALUE=text:circa 1800"
    );
}

#[test]
fn a_property_takes_a_group_and_parameters() {
    let tel = Telephone::new(TextOrUri::Uri(uri("tel:+1-555")))
        .in_group(Group::try_from(b"item1".as_slice()).unwrap())
        .with_params(params(";TYPE=\"work,voice\";PREF=1"))
        .unwrap();
    // The `VALUE=uri` the value needs stays although the parameters given
    // had none.
    assert_eq!(
        tel.to_string(),
        "item1.TEL;VALUE=uri;PREF=1;TYPE=work,voice:tel:+1-555"
    );
    round_trips(&VCard::builder(simon()).property(tel).build().unwrap());
}

#[test]
fn a_property_refuses_parameters_it_cannot_have() {
    // §5.6: type-param-tel "MUST NOT be used with a property other than TEL".
    assert!(matches!(
        Email::new(text("a@example.com")).with_params(params(";TYPE=voice")),
        Err(ParseError::TypeValue { .. })
    ));
    // `FN` is text only.
    assert!(matches!(
        FormattedName::new(text("A")).with_params(params(";VALUE=uri")),
        Err(ParseError::ValueType { .. })
    ));
}

#[test]
fn parameters_that_would_change_the_value_are_refused() {
    // The value is text, so `VALUE=uri` would have it read as a URI.
    assert!(
        Telephone::new(TextOrUri::Text(text("+1 555")))
            .with_params(params(";VALUE=uri"))
            .is_err()
    );
}

#[test]
fn an_extension_property_has_a_name_and_a_one_line_value() {
    let x = Xprop::new("x-favourite-color", "green").unwrap();
    assert_eq!(x.to_string(), "X-FAVOURITE-COLOR:green");
    assert!(Xprop::new("FAVOURITE", "green").is_err());
    assert!(Xprop::new("X-A B", "v").is_err());
    assert!(Xprop::new("X-A", "one\r\nTEL:injected").is_err());
    assert!(Iana::new("EXAMPLE", "a\nb").is_err());
}

#[test]
fn an_iana_property_cannot_take_the_name_of_a_typed_one() {
    assert!(Iana::new("TEL", "1").is_err());
    assert!(Iana::new("X-A", "1").is_err());
    assert!(Iana::new("EXAMPLE", "1").is_ok());
}

#[test]
fn the_builder_applies_the_same_checks_as_parsing() {
    let member = Member::new(uri("mailto:a@example.com"));
    assert_eq!(
        VCard::builder(simon())
            .property(member.clone())
            .build()
            .unwrap_err(),
        ValidationError::MemberWithoutGroup
    );
    let kind = |s: &str| {
        Property::Kind(properties::Kind::new(
            values::Kind::try_from(s.as_bytes()).unwrap(),
        ))
    };
    let card = VCard::builder(simon())
        .property(kind("group"))
        .property(member)
        .build()
        .unwrap();
    assert_eq!(card.kind(), values::Kind::Group);

    let birthday = |s| Birthday::new(DateAndOrTimeOrText::Text(text(s)));
    assert_eq!(
        VCard::builder(simon())
            .property(birthday("a"))
            .property(birthday("b"))
            .build()
            .unwrap_err(),
        ValidationError::TooMany("BDAY")
    );
}

#[test]
fn the_builder_writes_version_first_however_it_is_given() {
    let card = VCard::builder(simon())
        .property(Note::new(text("x")))
        .property(properties::Version::new(values::Version::V4_0))
        .build()
        .unwrap();
    assert!(
        card.to_string()
            .starts_with("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:")
    );
    assert!(
        !card
            .properties()
            .iter()
            .any(|p| matches!(p, Property::Version(_)))
    );
}

#[test]
fn a_built_card_has_a_uid_when_given_one() {
    let card = VCard::builder(simon())
        .property(Uid::new(values::Uid::Text(text("1234-ABCD"))))
        .build()
        .unwrap();
    assert!(matches!(card.uid().unwrap().value(), values::Uid::Text(_)));
}
