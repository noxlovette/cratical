//! vCard properties (issues #48 and #49): every property of RFC 6350 §6.1
//! to §6.10, from the RFC's own examples, and what the ABNF forbids of them.
#![cfg(feature = "rfc-6350")]

use cratical::vcard::{
    ParseError, VCard,
    params::{TelType, TypeValue, ValueDataType},
    properties::Property,
    values::{
        DateAndOrTime, DateAndOrTimeOrText, Kind, Sex, TextOrUri, Timezone,
        Uid, UtcOffset,
    },
};

/// A vCard holding `lines` after `VERSION`, and the `FN` every card needs
/// (§6.2.1). MEMBER is only allowed on a `KIND:group` card (§6.6.5), so a
/// card with one is made a group.
fn wire(lines: &str) -> String {
    let kind = if lines.contains("MEMBER") {
        "\r\nKIND:group"
    } else {
        ""
    };
    format!(
        "BEGIN:VCARD\r\nVERSION:4.0\r\n{lines}{kind}\r\nFN:Test\r\nEND:VCARD\r\n"
    )
}

fn parse(lines: &str) -> Result<VCard, ParseError> {
    VCard::parse(wire(lines).as_bytes())
}

/// The first property after `VERSION`.
fn prop(line: &str) -> Property {
    parse(line)
        .unwrap_or_else(|e| panic!("{line:?} should parse: {e}"))
        .properties()[0]
        .clone()
}

fn error(line: &str) -> ParseError {
    match parse(line) {
        Ok(card) => panic!("{line:?} should fail, got {:?}", card.properties()),
        Err(e) => e,
    }
}

/// The property as the variant `$variant`.
macro_rules! typed {
    ($line:expr, $variant:ident) => {
        match prop($line) {
            Property::$variant(p) => p,
            other => panic!(
                "{:?} should be {}: {other:?}",
                $line,
                stringify!($variant)
            ),
        }
    };
}

/// Parses each line, and checks it is written back as it came.
fn round_trips(lines: &[&str]) {
    for line in lines {
        assert_eq!(prop(line).to_string(), *line, "round trip of {line:?}");
    }
}

// ---- every property has its own type ----

#[test]
fn every_property_name_is_dispatched_to_its_own_variant() {
    let table = [
        ("SOURCE:http://a.example/", "Source"),
        ("KIND:group", "Kind"),
        ("XML:<a xmlns=\"urn:x\"/>", "Xml"),
        ("FN:A", "FormattedName"),
        ("N:A;B;;;", "Name"),
        ("NICKNAME:A", "Nickname"),
        ("PHOTO:http://a.example/a.gif", "Photo"),
        ("BDAY:19960415", "Birthday"),
        ("ANNIVERSARY:19960415", "Anniversary"),
        ("GENDER:M", "Gender"),
        ("ADR:;;street", "Address"),
        ("TEL:+1", "Telephone"),
        ("EMAIL:a@example.com", "Email"),
        ("IMPP:xmpp:a@example.com", "Impp"),
        ("LANG:en", "Lang"),
        ("TZ:Europe/Paris", "Tz"),
        ("GEO:geo:1,2", "Geo"),
        ("TITLE:A", "Title"),
        ("ROLE:A", "Role"),
        ("LOGO:http://a.example/a.jpg", "Logo"),
        ("ORG:A;B", "Organization"),
        ("MEMBER:urn:uuid:1", "Member"),
        ("RELATED:urn:uuid:1", "Related"),
        ("CATEGORIES:A,B", "Categories"),
        ("NOTE:A", "Note"),
        ("PRODID:A", "ProductId"),
        ("REV:19951031T222710Z", "Revision"),
        ("SOUND:http://a.example/a.wav", "Sound"),
        ("UID:urn:uuid:1", "Uid"),
        ("CLIENTPIDMAP:1;urn:uuid:1", "ClientPidMap"),
        ("URL:http://a.example/", "Url"),
        ("KEY:http://a.example/k", "Key"),
        ("FBURL:http://a.example/busy", "FreeBusyUrl"),
        ("CALADRURI:mailto:a@example.com", "CalendarAddressUri"),
        ("CALURI:http://a.example/cal", "CalendarUri"),
        ("X-CUSTOM:A", "Xprop"),
        ("FUTURE:A", "Iana"),
    ];
    for (line, variant) in table {
        let debug = format!("{:?}", prop(line));
        assert!(
            debug.starts_with(&format!("{variant}(")),
            "{line:?} should be {variant}, got {debug}"
        );
    }
}

#[test]
fn property_names_are_case_insensitive() {
    assert!(matches!(prop("fn:A"), Property::FormattedName(_)));
    assert!(matches!(prop("Tel:+1"), Property::Telephone(_)));
    assert_eq!(prop("fn:A").to_string(), "FN:A");
}

#[test]
fn a_group_survives_on_a_typed_property() {
    let tel = typed!("item1.TEL;TYPE=cell:+1 555", Telephone);
    assert_eq!(tel.group().unwrap().as_str(), "item1");
    assert_eq!(tel.to_string(), "item1.TEL;TYPE=cell:+1 555");
}

#[test]
fn parameters_are_reachable_from_the_property_enum() {
    let property = prop("EMAIL;PREF=1;TYPE=work:a@example.com");
    assert_eq!(property.params().pref().unwrap().value(), 1);
    assert_eq!(property.params().types(), [TypeValue::Work]);
}

// ---- §6.1 general ----

#[test]
fn source_is_a_uri() {
    round_trips(&[
        "SOURCE:ldap://ldap.example.com/cn=Babs%20Jensen,%20o=Babsco,%20c=US",
    ]);
    let source = typed!(
        "SOURCE:http://directory.example.com/addressbooks/jdoe/\r\n Jean%20Dupont.vcf",
        Source
    );
    assert_eq!(
        source.value().as_str(),
        "http://directory.example.com/addressbooks/jdoe/Jean%20Dupont.vcf"
    );
}

#[test]
fn source_must_be_a_uri() {
    assert!(matches!(error("SOURCE:not a uri"), ParseError::Value(_)));
}

#[test]
fn kind_takes_the_four_values_case_insensitively() {
    for (written, kind) in [
        ("individual", Kind::Individual),
        ("group", Kind::Group),
        ("org", Kind::Org),
        ("location", Kind::Location),
        ("ORG", Kind::Org),
        ("Location", Kind::Location),
    ] {
        let line = format!("KIND:{written}");
        assert_eq!(typed!(&line, Kind).value(), &kind, "{line}");
    }
    assert_eq!(prop("KIND:ORG").to_string(), "KIND:org");
}

#[test]
fn kind_keeps_an_x_name_or_iana_token() {
    let kind = typed!("KIND:X-Robot", Kind);
    assert_eq!(kind.value(), &Kind::Other("X-Robot".into()));
    assert_eq!(kind.to_string(), "KIND:X-Robot");
    assert_eq!(
        typed!("KIND:thing", Kind).value(),
        &Kind::Other("thing".into())
    );
}

#[test]
fn kind_is_a_token() {
    assert!(matches!(error("KIND:two words"), ParseError::Value(_)));
    assert!(matches!(error("KIND:"), ParseError::Value(_)));
}

#[test]
fn xml_is_text() {
    let xml = typed!("XML:<a xmlns=\"urn:example:a\">x\\ny</a>", Xml);
    assert_eq!(xml.value().as_str(), "<a xmlns=\"urn:example:a\">x\ny</a>");
    assert_eq!(xml.to_string(), "XML:<a xmlns=\"urn:example:a\">x\\ny</a>");
}

// ---- §6.2 identification ----

#[test]
fn fn_is_text() {
    let name = typed!("FN:Mr. John Q. Public\\, Esq.", FormattedName);
    assert_eq!(name.value().as_str(), "Mr. John Q. Public, Esq.");
    round_trips(&["FN:Mr. John Q. Public\\, Esq."]);
}

#[test]
fn n_has_its_five_components() {
    let n = typed!("N:Stevenson;John;Philip,Paul;Dr.;Jr.,M.D.,A.C.P.", Name);
    let name = n.value();
    assert_eq!(name.family()[0].as_str(), "Stevenson");
    assert_eq!(name.given()[0].as_str(), "John");
    assert_eq!(name.additional().len(), 2);
    assert_eq!(name.prefixes()[0].as_str(), "Dr.");
    assert_eq!(name.suffixes().len(), 3);
    round_trips(&[
        "N:Public;John;Quinlan;Mr.;Esq.",
        "N:Stevenson;John;Philip,Paul;Dr.;Jr.,M.D.,A.C.P.",
    ]);
}

#[test]
fn n_with_six_components_errors() {
    assert!(matches!(error("N:a;b;c;d;e;f"), ParseError::Value(_)));
}

#[test]
fn n_takes_sort_as() {
    let n = typed!("N;SORT-AS=\"Stevenson,John\":Stevenson;John;;;", Name);
    assert!(n.params().sort_as().is_some());
}

#[test]
fn nickname_is_a_list() {
    round_trips(&[
        "NICKNAME:Robbie",
        "NICKNAME:Jim,Jimmie",
        "NICKNAME;TYPE=work:Boss",
    ]);
    let nickname = typed!("NICKNAME:Jim,Jimmie", Nickname);
    assert_eq!(nickname.value().items().len(), 2);
}

#[test]
fn photo_is_a_uri_and_a_data_uri_is_one() {
    round_trips(&["PHOTO:http://www.example.com/pub/photos/jqpublic.gif"]);
    let photo = typed!(
        "PHOTO:data:image/jpeg;base64,MIICajCCAdOgAwIBAgICBEUwDQYJKoZIhv\r\n AQEEBQAwdzELMAkGA1UEBhMCVVMxLDAqBgNVBAoTI05ldHNjYXBlIENvbW11bm\r\n ljYXRpb25zIENvcnBvcmF0aW9u",
        Photo
    );
    assert_eq!(photo.value().scheme(), "data");
    assert!(photo.value().path().starts_with("image/jpeg;base64,MIICaj"));
}

#[test]
fn bday_is_a_date_and_or_time() {
    round_trips(&["BDAY:19960415", "BDAY:--0415", "BDAY:19531015T231000Z"]);
    let bday = typed!("BDAY:19531015T231000Z", Birthday);
    assert!(matches!(
        bday.value(),
        DateAndOrTimeOrText::DateAndOrTime(DateAndOrTime::DateTime(_))
    ));
    let bday = typed!("BDAY:--0415", Birthday);
    let DateAndOrTimeOrText::DateAndOrTime(DateAndOrTime::Date(date)) =
        bday.value()
    else {
        panic!("a date");
    };
    assert_eq!(
        (date.year(), date.month(), date.day()),
        (None, Some(4), Some(15))
    );
}

#[test]
fn bday_can_be_reset_to_text() {
    round_trips(&["BDAY;VALUE=text:circa 1800"]);
    let bday = typed!("BDAY;VALUE=text:circa 1800", Birthday);
    let DateAndOrTimeOrText::Text(text) = bday.value() else {
        panic!("text");
    };
    assert_eq!(text.as_str(), "circa 1800");
    assert_eq!(bday.params().value(), Some(&ValueDataType::Text));
}

#[test]
fn bday_can_name_its_value_type_and_calscale() {
    let bday = typed!(
        "BDAY;VALUE=date-and-or-time;CALSCALE=gregorian:19960415",
        Birthday
    );
    assert!(bday.params().calscale().is_some());
}

#[test]
fn bday_that_is_not_a_date_and_not_text_errors() {
    // The 3.0 extended format is not the 4.0 one.
    assert!(matches!(error("BDAY:1996-04-15"), ParseError::Value(_)));
    assert!(matches!(error("BDAY:circa 1800"), ParseError::Value(_)));
    assert!(matches!(
        error("BDAY;VALUE=uri:http://a.example/"),
        ParseError::ValueType {
            property: "BDAY",
            ..
        }
    ));
}

#[test]
fn anniversary_is_the_same_as_bday() {
    round_trips(&["ANNIVERSARY:19960415", "ANNIVERSARY;VALUE=text:June"]);
    assert!(matches!(
        typed!("ANNIVERSARY:19960415", Anniversary).value(),
        DateAndOrTimeOrText::DateAndOrTime(_)
    ));
    assert!(matches!(
        error("ANNIVERSARY;VALUE=uri:http://a.example/"),
        ParseError::ValueType {
            property: "ANNIVERSARY",
            ..
        }
    ));
}

#[test]
fn gender_has_the_rfc_examples() {
    round_trips(&[
        "GENDER:M",
        "GENDER:F",
        "GENDER:M;Fellow",
        "GENDER:F;grrrl",
        "GENDER:O;intersex",
        "GENDER:;it's complicated",
    ]);
    let gender = typed!("GENDER:M;Fellow", Gender);
    assert_eq!(gender.value().sex(), Some(Sex::Male));
    assert_eq!(gender.value().identity().unwrap().as_str(), "Fellow");
    assert_eq!(
        typed!("GENDER:;it's complicated", Gender).value().sex(),
        None
    );
}

#[test]
fn gender_with_a_sex_the_rfc_does_not_have_errors() {
    assert!(matches!(error("GENDER:X"), ParseError::Value(_)));
}

// ---- §6.3 delivery addressing ----

#[test]
fn adr_is_the_rfc_example_with_its_geo_and_label() {
    let adr = typed!(
        "ADR;GEO=\"geo:12.3457,78.910\";LABEL=\"Mr. John Q. Public, Esq.\\n\r\n Mail Drop: TNE QB\\n123 Main Street\\nAny Town, CA  91921-1234\\n\r\n U.S.A.\":;;123 Main Street;Any Town;CA;91921-1234;U.S.A.",
        Address
    );
    let address = adr.value();
    assert!(address.pobox().is_empty());
    assert!(address.extended().is_empty());
    assert_eq!(address.street()[0].as_str(), "123 Main Street");
    assert_eq!(address.locality()[0].as_str(), "Any Town");
    assert_eq!(address.region()[0].as_str(), "CA");
    assert_eq!(address.code()[0].as_str(), "91921-1234");
    assert_eq!(address.country()[0].as_str(), "U.S.A.");
    assert_eq!(
        adr.params().geo().unwrap().to_string(),
        "\"geo:12.3457,78.910\""
    );
    assert_eq!(
        adr.params().label().unwrap().as_str(),
        "Mr. John Q. Public, Esq.\\nMail Drop: TNE QB\\n123 Main Street\\nAny Town, CA  91921-1234\\nU.S.A."
    );
    assert_eq!(
        adr.to_string(),
        "ADR;GEO=\"geo:12.3457,78.910\";LABEL=\"Mr. John Q. Public, Esq.\\nMail Drop: TNE QB\\n123 Main Street\\nAny Town, CA  91921-1234\\nU.S.A.\":;;123 Main Street;Any Town;CA;91921-1234;U.S.A."
    );
}

#[test]
fn adr_with_eight_components_errors() {
    assert!(matches!(error("ADR:a;b;c;d;e;f;g;h"), ParseError::Value(_)));
}

// ---- §6.4 communications ----

#[test]
fn tel_as_a_uri_has_the_rfc_examples() {
    round_trips(&[
        "TEL;VALUE=uri;PREF=1;TYPE=voice,home:tel:+1-555-555-5555;ext=5555",
        "TEL;VALUE=uri;TYPE=home:tel:+33-01-23-45-67",
    ]);
    // `TYPE="voice,home"` is written without the quotes: the same parameter.
    let tel = typed!(
        "TEL;VALUE=uri;PREF=1;TYPE=\"voice,home\":tel:+1-555-555-5555;ext=5555",
        Telephone
    );
    let TextOrUri::Uri(uri) = tel.value() else {
        panic!("VALUE=uri makes it a URI");
    };
    assert_eq!(uri.scheme(), "tel");
    assert_eq!(uri.path(), "+1-555-555-5555;ext=5555");
    assert_eq!(
        tel.params().types(),
        [TypeValue::Tel(TelType::Voice), TypeValue::Home]
    );
    assert_eq!(
        tel.to_string(),
        "TEL;VALUE=uri;PREF=1;TYPE=voice,home:tel:+1-555-555-5555;ext=5555"
    );
}

#[test]
fn tel_is_text_by_default_however_it_is_written() {
    // §6.4.1: "By default, it is a single free-form text value (for
    // backward compatibility with vCard 3)". Real numbers aren't URIs.
    for number in [
        "+1 (555) 123-4567",
        "555.1234",
        "tel:+1-555",
        "(555) 1234 ext. 5",
    ] {
        let line = format!("TEL;TYPE=cell:{number}");
        let tel = typed!(&line, Telephone);
        let TextOrUri::Text(text) = tel.value() else {
            panic!("{line} is text");
        };
        assert_eq!(text.as_str(), number);
        assert_eq!(tel.to_string(), line);
    }
}

#[test]
fn tel_with_value_text_is_text() {
    let tel = typed!("TEL;VALUE=text:+1 555", Telephone);
    assert!(matches!(tel.value(), TextOrUri::Text(_)));
    assert_eq!(tel.to_string(), "TEL;VALUE=text:+1 555");
}

#[test]
fn tel_with_value_uri_must_be_a_uri() {
    assert!(matches!(
        error("TEL;VALUE=uri:+1 555"),
        ParseError::Value(_)
    ));
}

#[test]
fn tel_takes_the_types_of_section_6_4_1() {
    let tel = typed!(
        "TEL;TYPE=text,voice,fax,cell,video,pager,textphone:1",
        Telephone
    );
    assert_eq!(tel.params().types().len(), 7);
    // Repeated works the same.
    let tel = typed!("TEL;TYPE=text;TYPE=voice:1", Telephone);
    assert_eq!(tel.params().types().len(), 2);
}

#[test]
fn tel_cannot_have_a_value_type_it_does_not_list() {
    for value in ["date", "utc-offset", "language-tag", "timestamp", "x-foo"] {
        assert!(
            matches!(
                error(&format!("TEL;VALUE={value}:1")),
                ParseError::ValueType {
                    property: "TEL",
                    ..
                }
            ),
            "{value}"
        );
    }
}

#[test]
fn a_value_type_is_case_insensitive() {
    assert!(matches!(
        typed!("TEL;VALUE=URI:tel:+1", Telephone).value(),
        TextOrUri::Uri(_)
    ));
}

#[test]
fn email_is_text() {
    round_trips(&[
        "EMAIL;TYPE=work:jqpublic@xyz.example.com",
        "EMAIL;PREF=1:jane_doe@example.com",
    ]);
    assert_eq!(
        typed!("EMAIL:jane_doe@example.com", Email).value().as_str(),
        "jane_doe@example.com"
    );
}

#[test]
fn email_takes_the_types_of_real_data() {
    let email = typed!("EMAIL;TYPE=INTERNET;TYPE=HOME:a@example.com", Email);
    assert_eq!(email.params().types().len(), 2);
}

#[test]
fn impp_is_a_uri() {
    round_trips(&["IMPP;PREF=1:xmpp:alice@example.com"]);
    assert_eq!(
        typed!("IMPP;PREF=1:xmpp:alice@example.com", Impp)
            .value()
            .scheme(),
        "xmpp"
    );
}

#[test]
fn lang_is_a_language_tag() {
    let lang = typed!("LANG;TYPE=work;PREF=1:en", Lang);
    assert_eq!(lang.value().as_str(), "en");
    assert_eq!(lang.to_string(), "LANG;PREF=1;TYPE=work:en");
    round_trips(&["LANG;TYPE=home:fr", "LANG;PREF=2;TYPE=work:fr-CA"]);
}

#[test]
fn lang_with_a_malformed_tag_errors() {
    assert!(matches!(error("LANG:en_US"), ParseError::Value(_)));
}

// ---- §6.5 geographical ----

#[test]
fn tz_is_text_by_default() {
    round_trips(&["TZ:Raleigh/North America"]);
    let tz = typed!("TZ:Raleigh/North America", Tz);
    let Timezone::Text(text) = tz.value() else {
        panic!("text");
    };
    assert_eq!(text.as_str(), "Raleigh/North America");
}

#[test]
fn tz_can_be_a_utc_offset() {
    round_trips(&["TZ;VALUE=utc-offset:-0500"]);
    let tz = typed!("TZ;VALUE=utc-offset:-0500", Tz);
    let Timezone::UtcOffset(offset) = tz.value() else {
        panic!("a UTC offset");
    };
    assert_eq!(*offset, UtcOffset::new(true, 5, Some(0)).unwrap());
}

#[test]
fn tz_can_be_a_uri() {
    round_trips(&["TZ;VALUE=uri:http://example.com/tz/raleigh"]);
    assert!(matches!(
        typed!("TZ;VALUE=uri:http://example.com/tz/raleigh", Tz).value(),
        Timezone::Uri(_)
    ));
}

#[test]
fn tz_offset_without_a_value_type_is_the_text_the_rfc_says() {
    // "The default is a single text value."
    assert!(matches!(typed!("TZ:-0500", Tz).value(), Timezone::Text(_)));
}

#[test]
fn tz_with_a_bad_offset_errors() {
    assert!(matches!(
        error("TZ;VALUE=utc-offset:Raleigh"),
        ParseError::Value(_)
    ));
    assert!(matches!(
        error("TZ;VALUE=date:19960415"),
        ParseError::ValueType { property: "TZ", .. }
    ));
}

#[test]
fn geo_is_a_uri() {
    round_trips(&["GEO:geo:37.386013,-122.082932"]);
    assert_eq!(
        typed!("GEO:geo:37.386013,-122.082932", Geo).value().path(),
        "37.386013,-122.082932"
    );
}

// ---- §6.6 organizational ----

#[test]
fn title_and_role_are_text() {
    round_trips(&["TITLE:Research Scientist", "ROLE:Project Leader"]);
    assert_eq!(
        typed!("TITLE:Research Scientist", Title).value().as_str(),
        "Research Scientist"
    );
    assert_eq!(
        typed!("ROLE:Project Leader", Role).value().as_str(),
        "Project Leader"
    );
}

#[test]
fn logo_is_a_uri() {
    round_trips(&["LOGO:http://www.example.com/pub/logos/abccorp.jpg"]);
    assert_eq!(
        typed!("LOGO:data:image/jpeg;base64,MIICajCC", Logo)
            .value()
            .scheme(),
        "data"
    );
}

#[test]
fn org_is_a_name_and_its_units() {
    let org = typed!(
        "ORG:ABC\\, Inc.;North American Division;Marketing",
        Organization
    );
    assert_eq!(org.value().name().as_str(), "ABC, Inc.");
    assert_eq!(org.value().units().len(), 2);
    assert_eq!(org.value().units()[1].as_str(), "Marketing");
    round_trips(&["ORG:ABC\\, Inc.;North American Division;Marketing"]);
}

#[test]
fn member_is_a_uri() {
    round_trips(&[
        "MEMBER:urn:uuid:03a0e51f-d1aa-4385-8a53-e29025acd8af",
        "MEMBER:mailto:subscriber1@example.com",
        "MEMBER:xmpp:subscriber2@example.com",
        "MEMBER:sip:subscriber3@example.com",
        "MEMBER:tel:+1-418-555-5555",
    ]);
}

#[test]
fn related_is_a_uri_by_default_and_text_when_reset() {
    round_trips(&[
        "RELATED;TYPE=friend:urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
        "RELATED;TYPE=contact:http://example.com/directory/jdoe.vcf",
    ]);
    assert!(matches!(
        typed!(
            "RELATED;TYPE=contact:http://example.com/directory/jdoe.vcf",
            Related
        )
        .value(),
        TextOrUri::Uri(_)
    ));
    let related = typed!(
        "RELATED;TYPE=co-worker;VALUE=text:Please contact my assistant Jane\r\n  Doe for any inquiries.",
        Related
    );
    let TextOrUri::Text(text) = related.value() else {
        panic!("VALUE=text");
    };
    assert_eq!(
        text.as_str(),
        "Please contact my assistant Jane Doe for any inquiries."
    );
    assert_eq!(
        related.to_string(),
        "RELATED;VALUE=text;TYPE=co-worker:Please contact my assistant Jane Doe for any inquiries."
    );
}

#[test]
fn related_without_value_text_must_be_a_uri() {
    assert!(matches!(
        error("RELATED;TYPE=co-worker:Please contact my assistant"),
        ParseError::Value(_)
    ));
}

// ---- §6.7 explanatory ----

#[test]
fn categories_is_a_list() {
    round_trips(&[
        "CATEGORIES:TRAVEL AGENT",
        "CATEGORIES:INTERNET,IETF,INDUSTRY,INFORMATION TECHNOLOGY",
    ]);
    let categories = typed!("CATEGORIES:a\\,b,c", Categories);
    let items: Vec<_> = categories
        .value()
        .items()
        .iter()
        .map(|t| t.as_str())
        .collect();
    assert_eq!(items, ["a,b", "c"]);
}

#[test]
fn note_is_text_with_its_escapes() {
    let note = typed!(
        "NOTE:This fax number is operational 0800 to 1715\r\n  EST\\, Mon-Fri.",
        Note
    );
    assert_eq!(
        note.value().as_str(),
        "This fax number is operational 0800 to 1715 EST, Mon-Fri."
    );
    assert_eq!(
        note.to_string(),
        "NOTE:This fax number is operational 0800 to 1715 EST\\, Mon-Fri."
    );
    assert_eq!(typed!("NOTE:a\\nb\\;c", Note).value().as_str(), "a\nb;c");
}

#[test]
fn prodid_is_text() {
    round_trips(&["PRODID:-//ONLINE DIRECTORY//NONSGML Version 1//EN"]);
}

#[test]
fn rev_is_a_timestamp() {
    round_trips(&["REV:19951031T222710Z"]);
    let rev = typed!("REV:19951031T222710Z", Revision);
    assert_eq!(rev.value().date().year(), Some(1995));
}

#[test]
fn rev_that_is_not_a_complete_timestamp_errors() {
    assert!(matches!(error("REV:19951031"), ParseError::Value(_)));
    assert!(matches!(error("REV:1995-10-31"), ParseError::Value(_)));
}

#[test]
fn sound_is_a_uri() {
    // A URI's scheme is case-insensitive, and is written in lower case.
    let sound = typed!(
        "SOUND:CID:JOHNQPUBLIC.part8.19960229T080000.xyzMail@example.com",
        Sound
    );
    assert_eq!(sound.value().scheme(), "cid");
    assert_eq!(
        sound.value().path(),
        "JOHNQPUBLIC.part8.19960229T080000.xyzMail@example.com"
    );
    round_trips(&[
        "SOUND:data:audio/basic;base64,MIICajCCAdOgAwIBAgICBEUwDQYJKoZIh",
    ]);
}

// ---- UID (§6.7.6) and its real-world leniency ----

#[test]
fn uid_is_a_uri_when_it_is_one() {
    round_trips(&["UID:urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6"]);
    assert!(matches!(
        typed!("UID:urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6", Uid)
            .value(),
        Uid::Uri(_)
    ));
}

#[test]
fn uid_that_is_not_a_uri_is_kept_as_text_not_rejected() {
    // Real data: `UID:1234-ABCD`. CardDAV keys everything on the UID
    // (RFC 6352 §6.3.2.1), so refusing the card would lose the contact.
    for plain in ["1234-ABCD", "0001", "a b c", "F81D4FAE-7DEC-11D0-A765"] {
        let line = format!("UID:{plain}");
        let uid = typed!(&line, Uid);
        let Uid::Text(text) = uid.value() else {
            panic!("{plain} is text");
        };
        assert_eq!(text.as_str(), plain);
        assert_eq!(uid.to_string(), line, "and is written back as it came");
    }
}

#[test]
fn uid_with_value_text_is_text_even_when_it_would_parse_as_a_uri() {
    let uid = typed!("UID;VALUE=text:urn:uuid:1", Uid);
    assert!(matches!(uid.value(), Uid::Text(_)));
    assert_eq!(uid.to_string(), "UID;VALUE=text:urn:uuid:1");
}

#[test]
fn uid_with_value_uri_insists_on_a_uri() {
    assert!(matches!(
        typed!("UID;VALUE=uri:urn:uuid:1", Uid).value(),
        Uid::Uri(_)
    ));
    assert!(matches!(
        error("UID;VALUE=uri:1234-ABCD"),
        ParseError::Value(_)
    ));
}

#[test]
fn uid_with_another_value_type_errors() {
    assert!(matches!(
        error("UID;VALUE=date:19960415"),
        ParseError::ValueType {
            property: "UID",
            ..
        }
    ));
}

// ---- CLIENTPIDMAP (§6.7.7) ----

#[test]
fn clientpidmap_is_a_number_and_a_uri() {
    round_trips(&[
        "CLIENTPIDMAP:1;urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b",
        "CLIENTPIDMAP:2;urn:uuid:d89c9c7a-2e1b-4832-82de-7e992d95faa5",
    ]);
    let map = typed!(
        "CLIENTPIDMAP:2;urn:uuid:d89c9c7a-2e1b-4832-82de-7e992d95faa5",
        ClientPidMap
    );
    assert_eq!(map.value().pid(), 2);
    assert_eq!(map.value().uri().scheme(), "urn");
}

#[test]
fn clientpidmap_must_not_have_a_pid_parameter() {
    // §6.7.7: "As a special exception, the PID parameter MUST NOT be
    // applied to this property."
    assert!(matches!(
        error("CLIENTPIDMAP;PID=1.1:1;urn:uuid:1"),
        ParseError::ParamNotAllowed {
            property: "CLIENTPIDMAP",
            param: "PID"
        }
    ));
}

#[test]
fn clientpidmap_source_identifiers_are_strictly_positive() {
    assert!(matches!(
        error("CLIENTPIDMAP:0;urn:uuid:1"),
        ParseError::Value(_)
    ));
    assert!(matches!(
        error("CLIENTPIDMAP:x;urn:uuid:1"),
        ParseError::Value(_)
    ));
    assert!(matches!(error("CLIENTPIDMAP:1"), ParseError::Value(_)));
}

#[test]
fn the_rfc_pid_example_parses() {
    // §6.7.7's own example of how PID and CLIENTPIDMAP go together.
    let card = parse(
        "TEL;PID=3.1,4.2;VALUE=uri:tel:+1-555-555-5555\r\nEMAIL;PID=4.1,5.2:jdoe@example.com\r\nCLIENTPIDMAP:1;urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b\r\nCLIENTPIDMAP:2;urn:uuid:d89c9c7a-2e1b-4832-82de-7e992d95faa5",
    )
    .unwrap();
    // The three lines, and the FN the helper adds.
    assert_eq!(card.properties().len(), 5);
    assert_eq!(card.properties()[0].params().pid().len(), 2);
}

#[test]
fn url_is_a_uri() {
    round_trips(&["URL:http://example.org/restaurant.french/~chezchic.html"]);
}

// ---- §6.7.9 VERSION stays a card-level property ----

#[test]
fn version_is_not_among_the_properties_and_still_typed() {
    let card = parse("FN:A").unwrap();
    assert!(
        card.properties()
            .iter()
            .all(|p| !matches!(p, Property::Version(_)))
    );
    assert_eq!(card.version().to_string(), "VERSION:4.0");
}

// ---- §6.8 security ----

#[test]
fn key_is_a_uri_by_default_and_text_when_reset() {
    round_trips(&[
        "KEY:http://www.example.com/keys/jdoe.cer",
        "KEY;MEDIATYPE=application/pgp-keys:ftp://example.com/keys/jdoe",
    ]);
    let key = typed!(
        "KEY;MEDIATYPE=application/pgp-keys:ftp://example.com/keys/jdoe",
        Key
    );
    assert!(matches!(key.value(), TextOrUri::Uri(_)));
    assert!(key.params().mediatype().is_some());

    let key = typed!("KEY;VALUE=text:a public key\\, in text", Key);
    let TextOrUri::Text(text) = key.value() else {
        panic!("VALUE=text");
    };
    assert_eq!(text.as_str(), "a public key, in text");
}

#[test]
fn key_as_a_data_uri() {
    let key = typed!(
        "KEY:data:application/pgp-keys;base64,MIICajCCAdOgAwIBAgICBE\r\n UwDQYJKoZIhvcNAQEEBQAwdzELMAkGA1UEBhMCVVMxLDAqBgNVBAoTI05l",
        Key
    );
    let TextOrUri::Uri(uri) = key.value() else {
        panic!("a URI");
    };
    assert_eq!(uri.scheme(), "data");
}

// ---- §6.9 calendar ----

#[test]
fn the_calendar_properties_are_uris() {
    round_trips(&[
        "FBURL;PREF=1:http://www.example.com/busy/janedoe",
        "FBURL;MEDIATYPE=text/calendar:ftp://example.com/busy/project-a.ifb",
        "CALADRURI;PREF=1:mailto:janedoe@example.com",
        "CALADRURI:http://example.com/calendar/jdoe",
        "CALURI;PREF=1:http://cal.example.com/calA",
        "CALURI;MEDIATYPE=text/calendar:ftp://ftp.example.com/calA.ics",
    ]);
    let fburl = typed!(
        "FBURL;PREF=1:http://www.example.com/busy/janedoe",
        FreeBusyUrl
    );
    assert_eq!(fburl.params().pref().unwrap().value(), 1);
}

// ---- §6.10 extended properties ----

#[test]
fn x_properties_keep_group_params_and_value() {
    let x = typed!("item1.X-ABLabel;X-P=1:Anniversary", Xprop);
    assert_eq!(x.group().unwrap().as_str(), "item1");
    assert_eq!(x.name(), "X-ABLABEL");
    assert_eq!(x.value().as_str(), "Anniversary");
    assert_eq!(x.params().extensions().len(), 1);
    assert_eq!(x.to_string(), "item1.X-ABLABEL;X-P=1:Anniversary");
}

#[test]
fn an_iana_property_is_kept_raw() {
    let iana = typed!("EXAMPLE-IANA-PROPERTY:a;b,c\\n", Iana);
    assert_eq!(iana.value().as_str(), "a;b,c\\n");
}

// ---- what each property refuses ----

#[test]
fn a_property_that_is_text_only_refuses_another_value_type() {
    for line in [
        "FN;VALUE=uri:a",
        "N;VALUE=uri:a",
        "NICKNAME;VALUE=uri:a",
        "ADR;VALUE=uri:a",
        "EMAIL;VALUE=uri:a",
        "TITLE;VALUE=uri:a",
        "ROLE;VALUE=uri:a",
        "ORG;VALUE=uri:a",
        "CATEGORIES;VALUE=uri:a",
        "NOTE;VALUE=uri:a",
        "PRODID;VALUE=uri:a",
        "KIND;VALUE=uri:group",
        "GENDER;VALUE=uri:M",
        "XML;VALUE=uri:a",
    ] {
        assert!(
            matches!(error(line), ParseError::ValueType { .. }),
            "{line}"
        );
    }
}

#[test]
fn a_property_that_is_uri_only_refuses_another_value_type() {
    for line in [
        "SOURCE;VALUE=text:http://a.example/",
        "PHOTO;VALUE=text:http://a.example/",
        "IMPP;VALUE=text:xmpp:a@example.com",
        "GEO;VALUE=text:geo:1,2",
        "LOGO;VALUE=text:http://a.example/",
        "MEMBER;VALUE=text:urn:uuid:1",
        "SOUND;VALUE=text:http://a.example/",
        "URL;VALUE=text:http://a.example/",
        "FBURL;VALUE=text:http://a.example/",
        "CALADRURI;VALUE=text:mailto:a@example.com",
        "CALURI;VALUE=text:http://a.example/",
        "LANG;VALUE=text:en",
        "REV;VALUE=date:19951031",
    ] {
        assert!(
            matches!(error(line), ParseError::ValueType { .. }),
            "{line}"
        );
    }
}

#[test]
fn a_property_may_name_its_own_value_type() {
    round_trips(&[
        "FN;VALUE=text:A",
        "SOURCE;VALUE=uri:http://a.example/",
        "LANG;VALUE=language-tag:en",
        "REV;VALUE=timestamp:19951031T222710Z",
    ]);
}

#[test]
fn type_values_of_tel_are_refused_elsewhere() {
    // §5.6: "type-param-tel MUST NOT be used with a property other than
    // TEL."
    for value in [
        "text",
        "voice",
        "fax",
        "cell",
        "video",
        "pager",
        "textphone",
    ] {
        assert!(
            matches!(
                error(&format!("EMAIL;TYPE={value}:a@example.com")),
                ParseError::TypeValue {
                    property: "EMAIL",
                    only: "TEL",
                    ..
                }
            ),
            "{value}"
        );
    }
    assert!(matches!(
        error("ADR;TYPE=home,cell:;;street"),
        ParseError::TypeValue {
            property: "ADR",
            only: "TEL",
            ..
        }
    ));
}

#[test]
fn type_values_of_related_are_refused_elsewhere() {
    // §5.6: "type-param-related MUST NOT be used with a property other than
    // RELATED."
    assert!(matches!(
        error("TEL;TYPE=friend:+1"),
        ParseError::TypeValue {
            property: "TEL",
            only: "RELATED",
            ..
        }
    ));
    assert!(matches!(
        error("URL;TYPE=colleague:http://a.example/"),
        ParseError::TypeValue {
            property: "URL",
            only: "RELATED",
            ..
        }
    ));
}

#[test]
fn work_and_home_and_unknown_types_are_fine_anywhere() {
    for line in [
        "EMAIL;TYPE=work:a@example.com",
        "ADR;TYPE=home:;;street",
        "URL;TYPE=pref:http://a.example/",
        "IMPP;TYPE=personal:xmpp:a@example.com",
    ] {
        assert!(parse(line).is_ok(), "{line}");
    }
    assert!(parse("RELATED;TYPE=friend,agent,emergency:urn:uuid:1").is_ok());
}

#[test]
fn a_bad_value_fails_the_whole_card() {
    // Ingest is strict about what is local to one property.
    assert!(matches!(error("SOURCE:not a uri"), ParseError::Value(_)));
    assert!(matches!(error("ADR;PREF=0:;;a"), ParseError::Param(_)));
}

// ---- a whole card ----

// §8 of the RFC.
#[test]
fn the_rfc_author_card_is_fully_typed() {
    let src = "BEGIN:VCARD\r\nVERSION:4.0\r\nKIND:individual\r\nFN:Simon Perreault\r\nN:Perreault;Simon;;;ing. jr,M.Sc.\r\nBDAY:--0203\r\nANNIVERSARY:20090808T1430-0500\r\nGENDER:M\r\nLANG;PREF=1:fr\r\nLANG;PREF=2:en\r\nORG;TYPE=work:Viagenie\r\nADR;TYPE=work:;Suite D2-630;2875 Laurier;\r\n Quebec;QC;G1V 2M2;Canada\r\nTEL;VALUE=uri;TYPE=\"work,voice\";PREF=1:tel:+1-418-656-9254;ext=102\r\nTEL;VALUE=uri;TYPE=\"work,cell,voice,video,text\":tel:+1-418-262-6501\r\nEMAIL;TYPE=work:simon.perreault@viagenie.ca\r\nGEO;TYPE=work:geo:46.772673,-71.282945\r\nKEY;TYPE=work;VALUE=uri:\r\n http://www.viagenie.ca/simon.perreault/simon.asc\r\nTZ:-0500\r\nURL;TYPE=home:http://nomis80.org\r\nEND:VCARD\r\n";
    let card = VCard::parse(src.as_bytes()).unwrap();
    assert!(
        card.properties()
            .iter()
            .all(|p| { !matches!(p, Property::Xprop(_) | Property::Iana(_)) })
    );
    assert_eq!(card.properties().len(), 17);
}
