//! vCard property parameters (issue #47): RFC 6350 §5.
#![cfg(feature = "rfc-6350")]

use cratical::vcard::{
    ParseError, VCard,
    params::{
        CalScale, ParamError, Parameters, Pid, RelatedType, TelType, TypeValue,
        Tz, ValueDataType,
    },
    properties::Property,
};

fn parse(wire: &str) -> Parameters {
    Parameters::try_from(wire.as_bytes())
        .unwrap_or_else(|e| panic!("{wire:?} should parse: {e}"))
}

fn rejected(wire: &str) -> ParamError {
    Parameters::try_from(wire.as_bytes())
        .expect_err(&format!("{wire:?} should be rejected"))
}

/// Parses each of `inputs`, and checks that it is written as `written`.
fn writes_as(written: &str, inputs: &[&str]) {
    for input in inputs {
        assert_eq!(parse(input).to_string(), written, "from {input:?}");
    }
}

// ---- the general shape of §5 ----

#[test]
fn no_parameters_is_empty() {
    assert!(parse("").is_empty());
    assert!(Parameters::default().is_empty());
    assert_eq!(parse("").to_string(), "");
    assert!(!parse(";PREF=1").is_empty());
}

#[test]
fn names_are_case_insensitive() {
    let params = parse(";pref=1;Type=work;language=en;VALUE=text;AltId=x");
    assert_eq!(params.pref().unwrap().value(), 1);
    assert_eq!(params.types(), [TypeValue::Work]);
    assert_eq!(params.language().unwrap().as_str(), "en");
    assert_eq!(params.value(), Some(&ValueDataType::Text));
    assert_eq!(params.altid().unwrap().as_str(), "x");
}

#[test]
fn a_parameter_needs_a_name_and_an_equals_sign() {
    for wire in [
        ";WORK",
        ";TYPE",
        ";=1",
        ";X FOO=1",
        ";TY.PE=1",
        ";TYPE work",
    ] {
        rejected(wire);
    }
}

#[test]
fn a_parameter_that_can_be_given_once_is_an_error_twice() {
    for wire in [
        ";PREF=1;PREF=2",
        ";LANGUAGE=en;LANGUAGE=fr",
        ";VALUE=text;VALUE=uri",
        ";ALTID=1;ALTID=2",
        ";MEDIATYPE=image/png;MEDIATYPE=image/gif",
        ";CALSCALE=gregorian;CALSCALE=gregorian",
        ";SORT-AS=a;SORT-AS=b",
        ";GEO=\"geo:1,2\";GEO=\"geo:3,4\"",
        ";TZ=-0500;TZ=-0600",
        ";LABEL=a;LABEL=b",
    ] {
        assert!(matches!(rejected(wire), ParamError::Duplicate(_)), "{wire}");
    }
}

#[test]
fn a_dquote_only_delimits_a_whole_value() {
    // "Property parameter values MUST NOT contain the DQUOTE character."
    for wire in [
        ";ALTID=a\"b",
        ";LABEL=\"abc",
        ";LABEL=abc\"",
        ";TYPE=\"a\"b",
    ] {
        rejected(wire);
    }
}

#[test]
fn a_quoted_value_may_hold_the_separators() {
    // "elements that contain the COLON, SEMICOLON, or COMMA ... MUST be
    // specified as quoted-string"
    let params = parse(";ALTID=\"a:b;c,d\";LABEL=\"x;y\"");
    assert_eq!(params.altid().unwrap().as_str(), "a:b;c,d");
    assert_eq!(params.label().unwrap().as_str(), "x;y");
    assert_eq!(params.to_string(), ";ALTID=\"a:b;c,d\";LABEL=\"x;y\"");
}

#[test]
fn text_that_is_not_utf8_is_an_error() {
    assert!(Parameters::try_from(&b";LABEL=\xff"[..]).is_err());
    assert!(Parameters::try_from(&b";X-A=\xff"[..]).is_err());
}

// ---- TYPE: the three spellings ----

#[test]
fn type_comma_separated_quoted_and_repeated_are_the_same() {
    let comma = parse(";TYPE=work,voice");
    let quoted = parse(";TYPE=\"work,voice\"");
    let repeated = parse(";TYPE=work;TYPE=voice");

    assert_eq!(comma, quoted);
    assert_eq!(comma, repeated);
    assert_eq!(
        comma.types(),
        [TypeValue::Work, TypeValue::Tel(TelType::Voice)]
    );
    // ...and are all written the one way.
    writes_as(
        ";TYPE=work,voice",
        &[
            ";TYPE=work,voice",
            ";TYPE=\"work,voice\"",
            ";TYPE=work;TYPE=voice",
            ";type=WORK,Voice",
        ],
    );
}

#[test]
fn type_repeated_and_listed_can_be_mixed_and_keep_their_order() {
    let params = parse(";TYPE=home,text;TYPE=\"fax,cell\";TYPE=pager");
    assert_eq!(
        params.types(),
        [
            TypeValue::Home,
            TypeValue::Tel(TelType::Text),
            TypeValue::Tel(TelType::Fax),
            TypeValue::Tel(TelType::Cell),
            TypeValue::Tel(TelType::Pager),
        ]
    );
    // Separated by other parameters, still one list.
    let params = parse(";TYPE=work;PREF=1;TYPE=voice");
    assert_eq!(params.types().len(), 2);
    assert_eq!(params.to_string(), ";PREF=1;TYPE=work,voice");
}

#[test]
fn type_values_of_tel_are_all_known() {
    for (token, expected) in [
        ("text", TelType::Text),
        ("voice", TelType::Voice),
        ("fax", TelType::Fax),
        ("cell", TelType::Cell),
        ("video", TelType::Video),
        ("pager", TelType::Pager),
        ("textphone", TelType::Textphone),
    ] {
        let params = parse(&format!(";TYPE={token}"));
        assert_eq!(params.types(), [TypeValue::Tel(expected)], "{token}");
        assert_eq!(params.to_string(), format!(";TYPE={token}"));
    }
}

#[test]
fn type_values_of_related_are_all_known() {
    for (token, expected) in [
        ("contact", RelatedType::Contact),
        ("acquaintance", RelatedType::Acquaintance),
        ("friend", RelatedType::Friend),
        ("met", RelatedType::Met),
        ("co-worker", RelatedType::CoWorker),
        ("colleague", RelatedType::Colleague),
        ("co-resident", RelatedType::CoResident),
        ("neighbor", RelatedType::Neighbor),
        ("child", RelatedType::Child),
        ("parent", RelatedType::Parent),
        ("sibling", RelatedType::Sibling),
        ("spouse", RelatedType::Spouse),
        ("kin", RelatedType::Kin),
        ("muse", RelatedType::Muse),
        ("crush", RelatedType::Crush),
        ("date", RelatedType::Date),
        ("sweetheart", RelatedType::Sweetheart),
        ("me", RelatedType::Me),
        ("agent", RelatedType::Agent),
        ("emergency", RelatedType::Emergency),
    ] {
        let params = parse(&format!(";TYPE={token}"));
        assert_eq!(params.types(), [TypeValue::Related(expected)], "{token}");
        assert_eq!(params.to_string(), format!(";TYPE={token}"));
    }
}

#[test]
fn type_known_values_are_case_insensitive_and_written_lower_case() {
    for wire in [";TYPE=WORK", ";TYPE=Work", ";TYPE=work"] {
        assert_eq!(parse(wire).types(), [TypeValue::Work]);
    }
    assert_eq!(parse(";TYPE=CELL").types(), [TypeValue::Tel(TelType::Cell)]);
    assert_eq!(parse(";TYPE=CELL").to_string(), ";TYPE=cell");
    assert_eq!(
        parse(";TYPE=Co-Worker").types(),
        [TypeValue::Related(RelatedType::CoWorker)]
    );
}

#[test]
fn type_unknown_values_are_preserved_as_written() {
    // Real data: 3.0's `TYPE=pref` and `TYPE=INTERNET`, private values, and
    // whatever else producers make up.
    for token in ["pref", "INTERNET", "X-Custom", "x-lower", "Home Fax", "ie"] {
        let params = parse(&format!(";TYPE={token}"));
        match params.types() {
            [TypeValue::Other(value)] => assert_eq!(value.as_str(), token),
            other => panic!("{token}: expected one Other, got {other:?}"),
        }
        assert_eq!(params.to_string(), format!(";TYPE={token}"), "{token}");
    }
}

#[test]
fn type_known_and_unknown_values_mix() {
    let params = parse(";TYPE=work,INTERNET,pref");
    assert_eq!(params.types().len(), 3);
    assert_eq!(params.types()[0], TypeValue::Work);
    assert_eq!(params.to_string(), ";TYPE=work,INTERNET,pref");
}

// ---- PREF ----

#[test]
fn pref_is_an_integer_from_1_to_100() {
    for n in [1, 2, 50, 99, 100] {
        let params = parse(&format!(";PREF={n}"));
        assert_eq!(params.pref().unwrap().value(), n);
        assert_eq!(params.to_string(), format!(";PREF={n}"));
    }
    // `1*2DIGIT`: a leading zero is two digits.
    assert_eq!(parse(";PREF=01").pref().unwrap().value(), 1);
}

#[test]
fn pref_out_of_range_is_an_error() {
    for wire in [
        ";PREF=0",
        ";PREF=00",
        ";PREF=101",
        ";PREF=200",
        ";PREF=1000",
        ";PREF=-1",
        ";PREF=+1",
        ";PREF=",
        ";PREF=abc",
        ";PREF=1.5",
        ";PREF=1,2",
        ";PREF= 1",
        ";PREF=0100",
        ";PREF=99999999999",
    ] {
        rejected(wire);
    }
}

// ---- PID ----

#[test]
fn pid_is_one_or_two_integers_separated_by_a_dot() {
    let params = parse(";PID=1");
    assert_eq!(params.pid(), [Pid::new(1, None)]);
    let params = parse(";PID=1.1");
    assert_eq!(params.pid(), [Pid::new(1, Some(1))]);
    let params = parse(";PID=12.34");
    assert_eq!(
        (params.pid()[0].first(), params.pid()[0].second()),
        (12, Some(34))
    );
}

#[test]
fn pid_lists_and_repeats_are_one_list() {
    // "It MAY appear more than once in a given property", and "Multiple
    // values may be encoded in a single PID parameter by separating the
    // values with a comma".
    let listed = parse(";PID=1.1,2.1");
    let repeated = parse(";PID=1.1;PID=2.1");
    assert_eq!(listed, repeated);
    assert_eq!(listed.pid(), [Pid::new(1, Some(1)), Pid::new(2, Some(1))]);
    assert_eq!(repeated.to_string(), ";PID=1.1,2.1");
}

#[test]
fn pid_rejects_what_is_not_digits_and_a_dot() {
    for wire in [
        ";PID=",
        ";PID=.",
        ";PID=1.",
        ";PID=.1",
        ";PID=a",
        ";PID=1.a",
        ";PID=1.2.3",
        ";PID=-1",
        ";PID=+1",
        ";PID=1,",
        ";PID=,1",
        ";PID=1..2",
        ";PID=99999999999",
    ] {
        rejected(wire);
    }
}

// ---- LANGUAGE ----

#[test]
fn language_is_a_language_tag() {
    let params = parse(";LANGUAGE=tr");
    assert_eq!(params.language().unwrap().as_str(), "tr");
    writes_as(";LANGUAGE=en-US", &[";LANGUAGE=en-US", ";language=en-US"]);
    // The case it was written in is kept.
    assert_eq!(parse(";LANGUAGE=EN-us").to_string(), ";LANGUAGE=EN-us");
}

#[test]
fn language_rejects_what_is_not_a_tag() {
    for wire in [
        ";LANGUAGE=",
        ";LANGUAGE=en_US",
        ";LANGUAGE=e",
        ";LANGUAGE=-en",
        ";LANGUAGE=en,fr",
    ] {
        rejected(wire);
    }
}

// ---- VALUE ----

#[test]
fn value_knows_every_type_of_section_5_2() {
    for (token, expected) in [
        ("text", ValueDataType::Text),
        ("uri", ValueDataType::Uri),
        ("date", ValueDataType::Date),
        ("time", ValueDataType::Time),
        ("date-time", ValueDataType::DateTime),
        ("date-and-or-time", ValueDataType::DateAndOrTime),
        ("timestamp", ValueDataType::Timestamp),
        ("boolean", ValueDataType::Boolean),
        ("integer", ValueDataType::Integer),
        ("float", ValueDataType::Float),
        ("utc-offset", ValueDataType::UtcOffset),
        ("language-tag", ValueDataType::LanguageTag),
    ] {
        let params = parse(&format!(";VALUE={token}"));
        assert_eq!(params.value(), Some(&expected), "{token}");
        assert_eq!(params.to_string(), format!(";VALUE={token}"));
        // Case-insensitive.
        let upper = parse(&format!(";VALUE={}", token.to_uppercase()));
        assert_eq!(upper.value(), Some(&expected), "{token}");
    }
}

#[test]
fn value_other_tokens_are_kept() {
    // `iana-token / x-name`: 3.0 has `binary`, `vcard`, `phone-number`.
    for token in ["binary", "vcard", "phone-number", "X-Thing"] {
        let params = parse(&format!(";VALUE={token}"));
        assert_eq!(
            params.value(),
            Some(&ValueDataType::Other(token.into())),
            "{token}"
        );
        assert_eq!(params.to_string(), format!(";VALUE={token}"));
    }
}

#[test]
fn value_rejects_what_is_not_a_token() {
    for wire in [
        ";VALUE=",
        ";VALUE=a b",
        ";VALUE=a.b",
        ";VALUE=text,uri",
        ";VALUE=\"text\",\"uri\"",
    ] {
        rejected(wire);
    }
}

// ---- ALTID ----

#[test]
fn altid_is_an_opaque_string() {
    for value in ["1", "a", "first one", "x-y_z"] {
        let params = parse(&format!(";ALTID={value}"));
        assert_eq!(params.altid().unwrap().as_str(), value);
    }
    // Empty is a `*SAFE-CHAR` of none.
    assert_eq!(parse(";ALTID=").altid().unwrap().as_str(), "");
}

// ---- MEDIATYPE ----

#[test]
fn mediatype_is_a_type_and_a_subtype() {
    let params = parse(";MEDIATYPE=image/jpeg");
    let mediatype = params.mediatype().unwrap();
    assert_eq!(mediatype.media_type(), "image");
    assert_eq!(mediatype.subtype(), "jpeg");
    assert_eq!(params.to_string(), ";MEDIATYPE=image/jpeg");
}

#[test]
fn mediatype_with_attributes_must_be_quoted() {
    // `mediatype = type-name "/" subtype-name *( ";" attribute "=" value )`,
    // and a `;` in a parameter value has to be in a quoted-string.
    let params = parse(";MEDIATYPE=\"text/plain;charset=utf-8\"");
    assert_eq!(params.mediatype().unwrap().media_type(), "text");
    assert_eq!(
        params.to_string(),
        ";MEDIATYPE=\"text/plain;charset=utf-8\""
    );
}

#[test]
fn mediatype_needs_a_type_and_a_subtype() {
    for wire in [
        ";MEDIATYPE=jpeg",
        ";MEDIATYPE=",
        ";MEDIATYPE=image/jpeg,image/png",
    ] {
        rejected(wire);
    }
}

// ---- CALSCALE ----

#[test]
fn calscale_gregorian_and_other_tokens() {
    assert_eq!(
        parse(";CALSCALE=gregorian").calscale(),
        Some(&CalScale::Gregorian)
    );
    assert_eq!(
        parse(";CALSCALE=GREGORIAN").calscale(),
        Some(&CalScale::Gregorian)
    );
    assert_eq!(
        parse(";CALSCALE=GREGORIAN").to_string(),
        ";CALSCALE=gregorian"
    );
    assert_eq!(
        parse(";CALSCALE=x-lunar").calscale(),
        Some(&CalScale::Other("x-lunar".into()))
    );
    assert_eq!(parse(";CALSCALE=x-lunar").to_string(), ";CALSCALE=x-lunar");
}

#[test]
fn calscale_rejects_what_is_not_a_token() {
    for wire in [";CALSCALE=", ";CALSCALE=a b", ";CALSCALE=a,b"] {
        rejected(wire);
    }
}

// ---- SORT-AS ----

fn sort_as(params: &Parameters) -> Vec<&str> {
    params
        .sort_as()
        .unwrap()
        .values()
        .iter()
        .map(|v| v.as_str())
        .collect()
}

#[test]
fn sort_as_is_a_comma_separated_list_quoted_or_not() {
    // The examples of §5.9.
    assert_eq!(
        sort_as(&parse(";SORT-AS=\"Harten,Rene\"")),
        ["Harten", "Rene"]
    );
    assert_eq!(
        sort_as(&parse(";SORT-AS=\"Pau Shou Chang,Robert\"")),
        ["Pau Shou Chang", "Robert"]
    );
    assert_eq!(
        sort_as(&parse(";SORT-AS=\"Koura,Osamu\"")),
        ["Koura", "Osamu"]
    );
    // `sort-as-value = param-value *("," param-value)`
    assert_eq!(
        parse(";SORT-AS=Harten,Rene"),
        parse(";SORT-AS=\"Harten,Rene\"")
    );
    assert_eq!(sort_as(&parse(";SORT-AS=Koura")), ["Koura"]);
}

#[test]
fn sort_as_is_case_sensitive() {
    // "This parameter's value is case-sensitive."
    assert_eq!(
        sort_as(&parse(";SORT-AS=\"van der Harten,Rene\"")),
        ["van der Harten", "Rene"]
    );
    assert_ne!(parse(";SORT-AS=koura"), parse(";SORT-AS=Koura"));
    assert_eq!(parse(";SORT-AS=koura").to_string(), ";SORT-AS=koura");
}

// ---- GEO ----

#[test]
fn geo_is_a_uri_in_a_quoted_string() {
    let params = parse(";GEO=\"geo:37.386013,-122.082932\"");
    assert_eq!(
        params.geo().unwrap().uri().to_string(),
        "geo:37.386013,-122.082932"
    );
    assert_eq!(params.to_string(), ";GEO=\"geo:37.386013,-122.082932\"");
}

#[test]
fn geo_must_be_quoted_and_a_uri() {
    // `geo-parameter = "GEO=" DQUOTE URI DQUOTE`
    for wire in [
        ";GEO=geo:37.386013,-122.082932",
        ";GEO=geo:37.386013",
        ";GEO=\"\"",
        ";GEO=\"not a uri\"",
        ";GEO=\"geo:1,2\",\"geo:3,4\"",
    ] {
        rejected(wire);
    }
}

// ---- TZ ----

#[test]
fn tz_is_text_a_uri_or_an_offset() {
    assert!(
        matches!(parse(";TZ=-0500").tz(), Some(Tz::UtcOffset(o)) if o.seconds_east() == -18000)
    );
    assert!(matches!(parse(";TZ=+01").tz(), Some(Tz::UtcOffset(_))));
    assert!(matches!(
        parse(";TZ=America/New_York").tz(),
        Some(Tz::Text(t)) if t.as_str() == "America/New_York"
    ));
    assert!(matches!(
        parse(";TZ=Raleigh/North America").tz(),
        Some(Tz::Text(t)) if t.as_str() == "Raleigh/North America"
    ));
    assert!(matches!(
        parse(";TZ=\"http://example.com/tz/NY\"").tz(),
        Some(Tz::Uri(u)) if u.to_string() == "http://example.com/tz/NY"
    ));
}

#[test]
fn tz_is_written_the_way_it_is_read() {
    writes_as(";TZ=-0500", &[";TZ=-0500", ";TZ=\"-0500\""]);
    writes_as(";TZ=America/New_York", &[";TZ=America/New_York"]);
    writes_as(
        ";TZ=\"http://example.com/tz/NY\"",
        &[";TZ=\"http://example.com/tz/NY\""],
    );
}

#[test]
fn tz_takes_one_value() {
    rejected(";TZ=a,b");
}

// ---- LABEL and the RFC 6868 caret-encoding ----

#[test]
fn label_decodes_the_caret_encoding() {
    let params = parse(
        ";LABEL=\"Mr. John Q. Public, Esq.^nNew York, NY 10001^nU.S.A.\"",
    );
    assert_eq!(
        params.label().unwrap().as_str(),
        "Mr. John Q. Public, Esq.\nNew York, NY 10001\nU.S.A."
    );
    // ...and encodes it again, quoted for its commas.
    assert_eq!(
        params.to_string(),
        ";LABEL=\"Mr. John Q. Public, Esq.^nNew York, NY 10001^nU.S.A.\""
    );
}

#[test]
fn caret_encoded_dquote_does_not_end_a_quoted_string() {
    // The `^'` has to be decoded after the quotes are read, not before.
    let params = parse(";LABEL=\"say ^'hi^', then leave\"");
    assert_eq!(params.label().unwrap().as_str(), "say \"hi\", then leave");
    let written = params.to_string();
    assert_eq!(written, ";LABEL=\"say ^'hi^', then leave\"");
    assert_eq!(parse(&written), params);
}

#[test]
fn caret_encoding_applies_in_every_value() {
    assert_eq!(parse(";ALTID=a^^b").altid().unwrap().as_str(), "a^b");
    assert_eq!(
        parse(";X-NOTE=one^ntwo").extensions()[0].values()[0].as_str(),
        "one\ntwo"
    );
    assert_eq!(sort_as(&parse(";SORT-AS=^'x^'")), ["\"x\""]);
    // An undefined sequence is left as it is (RFC 6868 §3.2).
    assert_eq!(parse(";ALTID=a^zb").altid().unwrap().as_str(), "a^zb");
}

// ---- extensions ----

#[test]
fn unknown_parameters_are_kept_in_order_with_their_names_as_written() {
    let params = parse(";X-Foo=bar,baz;ENCODING=b;x-lower=1;CHARSET=utf-8");
    let names: Vec<&str> =
        params.extensions().iter().map(|e| e.name()).collect();
    assert_eq!(names, ["X-Foo", "ENCODING", "x-lower", "CHARSET"]);
    let foo = &params.extensions()[0];
    assert_eq!(
        foo.values().iter().map(|v| v.as_str()).collect::<Vec<_>>(),
        ["bar", "baz"]
    );
    assert!(foo.is_experimental());
    assert!(params.extensions()[2].is_experimental());
    assert!(!params.extensions()[1].is_experimental());
    assert_eq!(
        params.to_string(),
        ";X-Foo=bar,baz;ENCODING=b;x-lower=1;CHARSET=utf-8"
    );
}

#[test]
fn unknown_parameters_keep_a_quoted_value_whole() {
    let params = parse(";X-A=\"b,c\",d");
    let values: Vec<&str> = params.extensions()[0]
        .values()
        .iter()
        .map(|v| v.as_str())
        .collect();
    assert_eq!(values, ["b,c", "d"]);
    assert_eq!(params.to_string(), ";X-A=\"b,c\",d");
}

#[test]
fn known_and_unknown_parameters_mix() {
    let params = parse(";X-A=1;TYPE=work;ENCODING=b;PREF=2");
    assert_eq!(params.extensions().len(), 2);
    assert_eq!(params.pref().unwrap().value(), 2);
    assert_eq!(params.types(), [TypeValue::Work]);
}

// ---- writing ----

#[test]
fn every_parameter_survives_being_written_and_read_again() {
    let wire = ";LANGUAGE=fr;VALUE=text;PREF=1;ALTID=2;PID=1.1,2.1;TYPE=work,voice;\
        MEDIATYPE=image/png;CALSCALE=gregorian;SORT-AS=\"Harten,Rene\";\
        GEO=\"geo:1,2\";TZ=-0500;LABEL=\"a, b^nc\";X-K=v";
    let params = parse(wire);
    assert_eq!(params.to_string(), wire);
    assert_eq!(parse(&params.to_string()), params);
}

// ---- through a whole vCard ----

fn card_with(line: &str) -> VCard {
    let wire = format!(
        "BEGIN:VCARD\r\nVERSION:4.0\r\n{line}\r\nFN:Test\r\nEND:VCARD\r\n"
    );
    VCard::parse(wire.as_bytes()).unwrap_or_else(|e| panic!("{line}: {e}"))
}

#[test]
fn a_property_carries_its_typed_parameters() {
    let card = card_with(
        "TEL;TYPE=\"work,voice\";PREF=1;PID=1.1:tel:+1-555-555-5555\r\nCLIENTPIDMAP:1;urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b",
    );
    let Property::Telephone(tel) = &card.properties()[0] else {
        panic!("expected TEL");
    };
    let params = tel.params();
    assert_eq!(params.pref().unwrap().value(), 1);
    assert_eq!(params.pid(), [Pid::new(1, Some(1))]);
    assert_eq!(
        params.types(),
        [TypeValue::Work, TypeValue::Tel(TelType::Voice)]
    );
}

#[test]
fn a_card_is_written_with_its_parameters_in_the_one_spelling() {
    let card = card_with("TEL;TYPE=work;TYPE=voice;pref=1:+1-555");
    assert_eq!(
        card.properties()[0].to_string(),
        "TEL;PREF=1;TYPE=work,voice:+1-555"
    );
}

#[test]
fn a_bad_parameter_fails_the_card() {
    for line in [
        "TEL;PREF=0:+1",
        "TEL;PREF=101:+1",
        "TEL;PID=a:+1",
        "TEL;LANGUAGE=en_US:+1",
        "TEL;WORK:+1",
        "TEL;PREF=1;PREF=2:+1",
    ] {
        let wire = format!(
            "BEGIN:VCARD\r\nVERSION:4.0\r\n{line}\r\nFN:Test\r\nEND:VCARD\r\n"
        );
        assert!(
            matches!(VCard::parse(wire.as_bytes()), Err(ParseError::Param(_))),
            "{line}"
        );
    }
}

#[test]
fn a_quoted_colon_in_a_parameter_does_not_start_the_value() {
    let card = card_with("ADR;LABEL=\"a: b\":;;street");
    let Property::Address(adr) = &card.properties()[0] else {
        panic!("expected ADR");
    };
    assert_eq!(adr.params().label().unwrap().as_str(), "a: b");
    assert_eq!(adr.value().street()[0].as_str(), "street");
}
