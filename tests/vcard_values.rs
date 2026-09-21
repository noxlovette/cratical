//! vCard value types (issue #46): every production of RFC 6350 §4, and the
//! structured values of §6, with the inputs the ABNF rejects.
#![cfg(feature = "rfc-6350")]
// `3.14` is the literal of the RFC 6350 §4.6 example, not an approximation of
// pi.
#![allow(clippy::approx_constant)]

use cratical::vcard::values::{
    Address, Boolean, ClientPidMap, Date, DateAndOrTime, DateAndOrTimeList,
    DateList, DateTime, DateTimeList, Float, FloatList, Gender, Integer,
    IntegerList, LanguageTag, Name, Organization, ParamValue, Sex, Text,
    TextList, Time, TimeList, Timestamp, TimestampList, Uri, UtcOffset, Zone,
};

/// Parses `input`, and checks that it is written back as it came.
macro_rules! round_trips {
    ($ty:ty, $($input:expr),+ $(,)?) => {$(
        let parsed = <$ty>::try_from($input.as_bytes())
            .unwrap_or_else(|e| panic!("{:?} should parse: {e}", $input));
        assert_eq!(parsed.to_string(), $input, "round trip of {:?}", $input);
    )+};
}

/// Checks that none of the inputs parse.
macro_rules! rejects {
    ($ty:ty, $($input:expr),+ $(,)?) => {$(
        assert!(
            <$ty>::try_from($input.as_bytes()).is_err(),
            "{:?} should be rejected as {}",
            $input,
            stringify!($ty)
        );
    )+};
}

// ---- §4.1 text, and the escaping of §3.4 ----

#[test]
fn text_decodes_the_escapes_of_section_3_4() {
    let text = Text::try_from(&b"a\\,b\\;c\\\\d\\ne\\Nf"[..]).unwrap();
    assert_eq!(text.as_str(), "a,b;c\\d\ne\nf");
}

#[test]
fn text_with_an_unescaped_comma_or_semicolon_is_accepted() {
    // Real producers write `NOTE:Hello, world`; the comma only has to be
    // escaped when the value is a list.
    let text = Text::try_from(&b"Hello, world; bye"[..]).unwrap();
    assert_eq!(text.as_str(), "Hello, world; bye");
}

#[test]
fn text_is_written_with_the_escapes_a_list_needs() {
    let text = Text::from("a,b;c\\d\ne");
    assert_eq!(text.to_string(), "a\\,b\\;c\\\\d\\ne");
}

#[test]
fn the_formatted_line_break_example_of_section_4_1_round_trips() {
    let wire =
        "Mythical Manager\\nHyjinx Software Division\\nBabsCo\\, Inc.\\n";
    let text = Text::try_from(wire.as_bytes()).unwrap();
    assert_eq!(
        text.as_str(),
        "Mythical Manager\nHyjinx Software Division\nBabsCo, Inc.\n"
    );
    assert_eq!(text.to_string(), wire);
}

// ---- text-list and the other lists ----

#[test]
fn text_list_splits_on_unescaped_commas_only() {
    let list =
        TextList::try_from(&b"this is one value,this is another"[..]).unwrap();
    assert_eq!(list.items().len(), 2);
    assert_eq!(list.items()[1].as_str(), "this is another");

    let list = TextList::try_from(
        &b"this is a single value\\, with a comma encoded"[..],
    )
    .unwrap();
    assert_eq!(list.items().len(), 1);
    assert_eq!(
        list.items()[0].as_str(),
        "this is a single value, with a comma encoded"
    );
}

#[test]
fn text_list_keeps_empty_elements() {
    assert_eq!(TextList::try_from(&b"a,,b"[..]).unwrap().items().len(), 3);
    // `text-list = text *("," text)` and `text = *TEXT-CHAR`: one empty text.
    assert_eq!(TextList::try_from(&b""[..]).unwrap().items().len(), 1);
    round_trips!(TextList, "a,,b", "a\\,b,c", "");
}

#[test]
fn integer_list_of_section_4_5() {
    let list = IntegerList::try_from(&b"+1234556790,432109876"[..]).unwrap();
    let values: Vec<i64> = list.items().iter().map(Integer::value).collect();
    assert_eq!(values, [1234556790, 432109876]);
}

#[test]
fn float_list_of_section_4_6() {
    let list = FloatList::try_from(&b"1.333,3.14"[..]).unwrap();
    let values: Vec<f64> = list.items().iter().map(Float::value).collect();
    assert_eq!(values, [1.333, 3.14]);
    round_trips!(FloatList, "1.333,3.14");
}

#[test]
fn the_other_lists_are_comma_separated_too() {
    round_trips!(DateList, "19850412,1985-04,--0412");
    round_trips!(TimeList, "102200,-2200,--00Z");
    round_trips!(DateTimeList, "19961022T140000,--1022T1400");
    round_trips!(DateAndOrTimeList, "19961022T140000,19850412,T102200Z");
    round_trips!(TimestampList, "19961022T140000Z,19961022T140000-0500");
}

#[test]
fn lists_need_every_element_to_be_valid() {
    rejects!(IntegerList, "1,x", "1,", ",1");
    rejects!(DateList, "19850412,", "19850412,198504");
    rejects!(TimestampList, "19961022T140000,19961022T1400");
}

// ---- §4.4 boolean ----

#[test]
fn boolean_is_case_insensitive() {
    for (wire, value) in [
        ("TRUE", true),
        ("false", false),
        ("True", true),
        ("fALSE", false),
    ] {
        assert_eq!(
            Boolean::try_from(wire.as_bytes()).unwrap().value(),
            value,
            "{wire}"
        );
    }
}

#[test]
fn boolean_is_written_upper_case() {
    assert_eq!(Boolean::new(true).to_string(), "TRUE");
    assert_eq!(Boolean::new(false).to_string(), "FALSE");
}

#[test]
fn boolean_rejects_everything_else() {
    rejects!(Boolean, "", "yes", "1", "0", "TRUEE", " TRUE", "T", "on");
}

// ---- §4.5 integer ----

#[test]
fn integer_examples_of_section_4_5() {
    for (wire, value) in [
        ("1234567890", 1234567890),
        ("-1234556790", -1234556790),
        ("+1234556790", 1234556790),
        ("0", 0),
    ] {
        assert_eq!(
            Integer::try_from(wire.as_bytes()).unwrap().value(),
            value,
            "{wire}"
        );
    }
}

#[test]
fn integer_covers_the_64_bit_range_of_the_rfc() {
    assert_eq!(
        Integer::try_from(&b"9223372036854775807"[..])
            .unwrap()
            .value(),
        i64::MAX
    );
    assert_eq!(
        Integer::try_from(&b"-9223372036854775808"[..])
            .unwrap()
            .value(),
        i64::MIN
    );
    // Wider than iCalendar's 32 bits.
    assert!(Integer::try_from(&b"4294967296"[..]).is_ok());
    rejects!(Integer, "9223372036854775808", "-9223372036854775809");
}

#[test]
fn integer_rejects_everything_that_is_not_sign_and_digits() {
    rejects!(
        Integer, "", "+", "-", "1.0", "1e3", " 1", "1 ", "0x10", "1_0", "--1",
        "+-1"
    );
}

#[test]
fn integer_is_written_without_a_plus() {
    assert_eq!(Integer::try_from(&b"+5"[..]).unwrap().to_string(), "5");
    round_trips!(Integer, "1234567890", "-1234556790");
}

// ---- §4.6 float ----

#[test]
fn float_examples_of_section_4_6() {
    for (wire, value) in [
        ("20.30", 20.30),
        ("1000000.0000001", 1000000.0000001),
        ("-3.14", -3.14),
        ("+1.5", 1.5),
        ("5", 5.0),
    ] {
        assert_eq!(
            Float::try_from(wire.as_bytes()).unwrap().value(),
            value,
            "{wire}"
        );
    }
}

#[test]
fn float_forbids_scientific_notation_and_the_rest_of_rusts_syntax() {
    // `f64::from_str` accepts all of these; the ABNF is
    // `[sign] 1*DIGIT ["." 1*DIGIT]`.
    rejects!(
        Float, "", "-", "+", "1e5", "1E5", "1e-5", ".5", "5.", "-.5", "inf",
        "-inf", "NaN", "1.2.3", "0x1", " 1", "1 ", "1,5"
    );
}

#[test]
fn float_is_never_written_in_scientific_notation() {
    for value in [1e21, 1e-7, 123456789012345680000.0, -2.5e-10] {
        let written = Float::new(value).unwrap().to_string();
        assert!(
            !written.contains(['e', 'E']),
            "{value} written as {written}"
        );
        let back = Float::try_from(written.as_bytes()).unwrap();
        assert_eq!(back.value(), value, "{written}");
    }
}

#[test]
fn float_refuses_what_the_grammar_cannot_write() {
    assert!(Float::new(f64::NAN).is_err());
    assert!(Float::new(f64::INFINITY).is_err());
    assert!(Float::new(f64::NEG_INFINITY).is_err());
}

// ---- §4.7 utc-offset ----

#[test]
fn utc_offset_is_sign_hour_and_optional_minute() {
    let offset = UtcOffset::try_from(&b"-0500"[..]).unwrap();
    assert!(offset.is_negative());
    assert_eq!((offset.hour(), offset.minute()), (5, Some(0)));
    assert_eq!(offset.seconds_east(), -5 * 3600);

    let offset = UtcOffset::try_from(&b"+0100"[..]).unwrap();
    assert_eq!(offset.seconds_east(), 3600);

    // `utc-offset = sign hour [minute]`
    let offset = UtcOffset::try_from(&b"+01"[..]).unwrap();
    assert_eq!((offset.hour(), offset.minute()), (1, None));
    assert_eq!(offset.seconds_east(), 3600);

    let offset = UtcOffset::try_from(&b"+0530"[..]).unwrap();
    assert_eq!(offset.seconds_east(), 5 * 3600 + 30 * 60);
}

#[test]
fn utc_offset_keeps_whether_the_minutes_were_written() {
    round_trips!(UtcOffset, "-08", "-0800", "+01", "+0000", "+2359");
}

#[test]
fn utc_offset_rejects_what_is_not_basic_format_within_range() {
    rejects!(
        UtcOffset,
        "",
        "0500",
        "+5",
        "+050",
        "+05000",
        "+0560",
        "+2400",
        "+01:00",
        "+01:00:00",
        "+010000",
        "Z",
        " +01",
        "+ 1",
        "+a0"
    );
}

#[test]
fn zone_is_an_upper_case_z_or_an_offset() {
    assert_eq!(Zone::try_from(&b"Z"[..]).unwrap(), Zone::Utc);
    assert!(matches!(
        Zone::try_from(&b"-0800"[..]).unwrap(),
        Zone::Offset(_)
    ));
    rejects!(Zone, "z", "UTC", "", "GMT");
    round_trips!(Zone, "Z", "-0800", "+05");
}

// ---- §4.3.1 date ----

#[test]
fn date_examples_of_section_4_3_1() {
    let date = Date::try_from(&b"19850412"[..]).unwrap();
    assert_eq!(
        (date.year(), date.month(), date.day()),
        (Some(1985), Some(4), Some(12))
    );

    let date = Date::try_from(&b"1985-04"[..]).unwrap();
    assert_eq!(
        (date.year(), date.month(), date.day()),
        (Some(1985), Some(4), None)
    );

    let date = Date::try_from(&b"1985"[..]).unwrap();
    assert_eq!(
        (date.year(), date.month(), date.day()),
        (Some(1985), None, None)
    );

    let date = Date::try_from(&b"--0412"[..]).unwrap();
    assert_eq!(
        (date.year(), date.month(), date.day()),
        (None, Some(4), Some(12))
    );

    let date = Date::try_from(&b"---12"[..]).unwrap();
    assert_eq!(
        (date.year(), date.month(), date.day()),
        (None, None, Some(12))
    );
}

#[test]
fn date_month_alone_is_the_reduced_truncated_form() {
    // `"--" month [day]`
    let date = Date::try_from(&b"--04"[..]).unwrap();
    assert_eq!(
        (date.year(), date.month(), date.day()),
        (None, Some(4), None)
    );
}

#[test]
fn date_round_trips_every_form() {
    round_trips!(
        Date, "19850412", "1985-04", "1985", "--0412", "--04", "---12", "0001"
    );
}

#[test]
fn date_rejects_yyyymm_and_the_extended_format() {
    // "YYYYMM is disallowed to prevent confusion with YYMMDD", and
    // "YYYY-MM-DD is disallowed since we are using the basic format".
    rejects!(Date, "198504", "1985-04-12", "1985-0412", "198504-12");
}

#[test]
fn date_rejects_malformed_shapes() {
    rejects!(
        Date,
        "",
        "85",
        "198",
        "19850",
        "1985041",
        "198504123",
        "--",
        "---",
        "----12",
        "--0",
        "--041",
        "--04123",
        "---1",
        "---123",
        "1985-",
        "1985-4",
        "1985-041",
        "-1985",
        "+1985",
        "1985 ",
        " 1985",
        "19x5",
        "--0a12",
        "1985T",
        "T1985"
    );
}

#[test]
fn date_rejects_out_of_range_parts() {
    rejects!(
        Date, "19850001", "19851301", "19850100", "19850132", "19850431",
        "19850631", "19850931", "19851131", "--0001", "--1301", "--0100",
        "--0132", "--0431", "---00", "---32", "1985-00", "1985-13"
    );
}

#[test]
fn date_february_29th_needs_a_leap_year_or_no_year() {
    assert!(Date::try_from(&b"19840229"[..]).is_ok());
    assert!(Date::try_from(&b"20000229"[..]).is_ok());
    // A date without a year has no year to say it isn't leap.
    assert!(Date::try_from(&b"--0229"[..]).is_ok());
    rejects!(Date, "19850229", "19000229", "21000229", "--0230");
}

#[test]
fn date_says_how_reduced_it_is() {
    let is = |wire: &str| {
        let d = Date::try_from(wire.as_bytes()).unwrap();
        (d.is_noreduc(), d.is_complete())
    };
    assert_eq!(is("19850412"), (true, true));
    assert_eq!(is("--0412"), (true, false));
    assert_eq!(is("---12"), (true, false));
    assert_eq!(is("1985-04"), (false, false));
    assert_eq!(is("1985"), (false, false));
    assert_eq!(is("--04"), (false, false));
}

#[test]
fn date_can_be_built_only_in_the_shapes_the_grammar_has() {
    assert!(Date::new(Some(1985), Some(4), Some(12)).is_ok());
    assert!(Date::new(None, Some(4), Some(12)).is_ok());
    assert!(Date::new(Some(1985), None, Some(12)).is_err());
    assert!(Date::new(None, None, None).is_err());
    assert!(Date::new(Some(10000), None, None).is_err());
}

// ---- §4.3.2 time ----

#[test]
fn time_examples_of_section_4_3_2() {
    let time = Time::try_from(&b"102200"[..]).unwrap();
    assert_eq!(
        (time.hour(), time.minute(), time.second(), time.zone()),
        (Some(10), Some(22), Some(0), None)
    );

    let time = Time::try_from(&b"1022"[..]).unwrap();
    assert_eq!(
        (time.hour(), time.minute(), time.second()),
        (Some(10), Some(22), None)
    );

    let time = Time::try_from(&b"10"[..]).unwrap();
    assert_eq!(
        (time.hour(), time.minute(), time.second()),
        (Some(10), None, None)
    );

    let time = Time::try_from(&b"-2200"[..]).unwrap();
    assert_eq!(
        (time.hour(), time.minute(), time.second()),
        (None, Some(22), Some(0))
    );

    let time = Time::try_from(&b"--00"[..]).unwrap();
    assert_eq!(
        (time.hour(), time.minute(), time.second()),
        (None, None, Some(0))
    );

    let time = Time::try_from(&b"102200Z"[..]).unwrap();
    assert_eq!(time.zone(), Some(Zone::Utc));

    let time = Time::try_from(&b"102200-0800"[..]).unwrap();
    match time.zone() {
        Some(Zone::Offset(offset)) => {
            assert_eq!(offset.seconds_east(), -8 * 3600);
        }
        other => panic!("expected an offset, got {other:?}"),
    }
}

#[test]
fn time_minute_alone_is_the_truncated_form() {
    // `"-" minute [second] [zone]`
    let time = Time::try_from(&b"-22"[..]).unwrap();
    assert_eq!(
        (time.hour(), time.minute(), time.second()),
        (None, Some(22), None)
    );
}

#[test]
fn time_round_trips_every_form() {
    round_trips!(
        Time,
        "102200",
        "1022",
        "10",
        "-2200",
        "-22",
        "--00",
        "102200Z",
        "102200-0800",
        "10Z",
        "10-08",
        "1022+0530",
        "-2200Z",
        "--00Z",
        "-22-0800",
        "235960"
    );
}

#[test]
fn time_zone_can_follow_the_hour_alone() {
    // `hour [minute [second]] [zone]`, where the zone's own sign follows a
    // bare hour without being taken for a truncation dash.
    let time = Time::try_from(&b"10-08"[..]).unwrap();
    assert_eq!((time.hour(), time.minute()), (Some(10), None));
    assert!(time.zone().is_some());
}

#[test]
fn time_rejects_malformed_shapes() {
    rejects!(
        Time,
        "",
        "1",
        "102",
        "10220",
        "1022000",
        "-",
        "--",
        "---00",
        "-2",
        "-220",
        "--0",
        "--000",
        "10:22:00",
        "10:22",
        "102200.5",
        "102200,5",
        "102200z",
        "102200 ",
        " 102200",
        "T102200",
        "102200+",
        "102200+8",
        "102200-08:00",
        "102200Zulu",
        "1a",
        "-2a",
        "102200Z-0800"
    );
}

#[test]
fn time_the_midnight_hour_is_00_never_24() {
    assert!(Time::try_from(&b"000000"[..]).is_ok());
    rejects!(Time, "24", "2400", "240000");
}

#[test]
fn time_rejects_out_of_range_parts() {
    rejects!(
        Time, "1060", "106000", "102261", "-60", "-2261", "--61", "99"
    );
}

#[test]
fn time_accepts_a_leap_second() {
    // "00-58/59/60 depending on leap second"
    assert!(Time::try_from(&b"235960"[..]).is_ok());
    assert!(Time::try_from(&b"--60"[..]).is_ok());
}

#[test]
fn time_says_how_truncated_it_is() {
    let is = |wire: &str| {
        let t = Time::try_from(wire.as_bytes()).unwrap();
        (t.is_notrunc(), t.is_complete())
    };
    assert_eq!(is("102200"), (true, true));
    assert_eq!(is("102200Z"), (true, true));
    assert_eq!(is("1022"), (true, false));
    assert_eq!(is("10"), (true, false));
    assert_eq!(is("-2200"), (false, false));
    assert_eq!(is("--00"), (false, false));
}

// ---- §4.3.3 date-time ----

#[test]
fn date_time_examples_of_section_4_3_3() {
    let dt = DateTime::try_from(&b"19961022T140000"[..]).unwrap();
    assert_eq!(dt.date().year(), Some(1996));
    assert_eq!(dt.time().hour(), Some(14));

    let dt = DateTime::try_from(&b"--1022T1400"[..]).unwrap();
    assert_eq!((dt.date().year(), dt.date().month()), (None, Some(10)));
    assert_eq!((dt.time().hour(), dt.time().minute()), (Some(14), Some(0)));

    let dt = DateTime::try_from(&b"---22T14"[..]).unwrap();
    assert_eq!(dt.date().day(), Some(22));
    assert_eq!(dt.time().hour(), Some(14));

    round_trips!(
        DateTime,
        "19961022T140000",
        "--1022T1400",
        "---22T14",
        "19961022T140000Z",
        "19961022T1400-0500"
    );
}

#[test]
fn date_time_needs_a_date_noreduc_and_a_time_notrunc() {
    // `date-time = date-noreduc time-designator time-notrunc`
    rejects!(
        DateTime,
        // reduced date
        "1996T14",
        "1996-10T14",
        "--10T14",
        // truncated time
        "19961022T-2200",
        "19961022T--00",
        // no designator, or a wrong one
        "19961022140000",
        "19961022t140000",
        "19961022 140000",
        // nothing on one side
        "19961022T",
        "T140000",
        "",
        // more than one designator
        "19961022T14T00"
    );
}

#[test]
fn date_time_can_only_be_built_from_the_right_parts() {
    let full = Date::try_from(&b"19961022"[..]).unwrap();
    let reduced = Date::try_from(&b"1996"[..]).unwrap();
    let whole_day = Time::try_from(&b"14"[..]).unwrap();
    let truncated = Time::try_from(&b"-2200"[..]).unwrap();
    assert!(DateTime::new(full, whole_day).is_ok());
    assert!(DateTime::new(reduced, whole_day).is_err());
    assert!(DateTime::new(full, truncated).is_err());
}

// ---- §4.3.4 date-and-or-time ----

#[test]
fn date_and_or_time_examples_of_section_4_3_4() {
    round_trips!(
        DateAndOrTime,
        "19961022T140000",
        "--1022T1400",
        "---22T14",
        "19850412",
        "1985-04",
        "1985",
        "--0412",
        "---12",
        "T102200",
        "T1022",
        "T10",
        "T-2200",
        "T--00",
        "T102200Z",
        "T102200-0800",
    );
}

#[test]
fn date_and_or_time_says_which_of_the_three_it_is() {
    let parse = |wire: &str| DateAndOrTime::try_from(wire.as_bytes()).unwrap();
    assert!(matches!(
        parse("19961022T140000"),
        DateAndOrTime::DateTime(_)
    ));
    assert!(matches!(parse("---22T14"), DateAndOrTime::DateTime(_)));
    assert!(matches!(parse("19850412"), DateAndOrTime::Date(_)));
    assert!(matches!(parse("1985-04"), DateAndOrTime::Date(_)));
    assert!(matches!(parse("--0412"), DateAndOrTime::Date(_)));
    assert!(matches!(parse("T102200"), DateAndOrTime::Time(_)));
    assert!(matches!(parse("T-2200"), DateAndOrTime::Time(_)));
}

#[test]
fn date_and_or_time_a_bare_time_needs_its_t() {
    // "a stand-alone TIME value is always preceded by a 'T'": without it,
    // these are dates or nothing.
    rejects!(DateAndOrTime, "102200", "-2200", "--00", "10220");
    // ...and `1022` is the year 1022, which is why.
    let value = DateAndOrTime::try_from(&b"1022"[..]).unwrap();
    assert!(matches!(value, DateAndOrTime::Date(d) if d.year() == Some(1022)));
}

#[test]
fn date_and_or_time_rejects_what_is_none_of_the_three() {
    rejects!(
        DateAndOrTime,
        "",
        "T",
        "19961022T",
        "19961022T-2200",
        "1996T14",
        "TT10",
        "19961022T140000T",
        "t102200",
        "1985-04-12",
        "198504",
        "T24",
        "T102200z"
    );
}

// ---- §4.3.5 timestamp ----

#[test]
fn timestamp_examples_of_section_4_3_5() {
    round_trips!(
        Timestamp,
        "19961022T140000",
        "19961022T140000Z",
        "19961022T140000-05",
        "19961022T140000-0500"
    );
    let ts = Timestamp::try_from(&b"19961022T140000Z"[..]).unwrap();
    assert_eq!(
        (ts.date().year(), ts.date().month(), ts.date().day()),
        (Some(1996), Some(10), Some(22))
    );
    assert_eq!(
        (ts.time().hour(), ts.time().minute(), ts.time().second()),
        (Some(14), Some(0), Some(0))
    );
    assert_eq!(ts.time().zone(), Some(Zone::Utc));
}

#[test]
fn timestamp_needs_a_complete_date_and_a_complete_time() {
    // `timestamp = date-complete time-designator time-complete`
    rejects!(
        Timestamp,
        "19961022T1400",
        "19961022T14",
        "19961022T-2200",
        "--1022T140000",
        "---22T140000",
        "1996T140000",
        "1996-10T140000",
        "19961022140000",
        "19961022T",
        "T140000",
        "",
        "19961022T140000Zulu",
        "19961301T140000",
        "19961022T250000"
    );
}

// ---- §4.8 language-tag ----

#[test]
fn language_tag_is_an_rfc_5646_tag() {
    for tag in [
        "en",
        "en-US",
        "tr",
        "zh-Hant-TW",
        "de-CH-1996",
        "es-419",
        "x-private",
    ] {
        assert_eq!(
            LanguageTag::try_from(tag.as_bytes())
                .unwrap_or_else(|e| panic!("{tag}: {e}"))
                .as_str(),
            tag
        );
    }
    rejects!(
        LanguageTag,
        "",
        "en_US",
        "-en",
        "en-",
        "e",
        "toolongsubtag",
        "en-toolongsubtag",
        "en US",
        "en--US"
    );
}

#[test]
fn language_tag_keeps_its_case() {
    // Tags are case-insensitive, but what was written is what's written back.
    round_trips!(LanguageTag, "EN-us", "zh-hant-tw");
}

// ---- URIs carrying data: and urn:uuid: ----

#[test]
fn uri_examples_of_section_4_2() {
    round_trips!(
        Uri,
        "http://www.example.com/my/picture.jpg",
        "ldap://ldap.example.com/cn=babs%20jensen"
    );
}

#[test]
fn uri_holds_an_inline_data_uri() {
    // vCard 4.0 inlines PHOTO, LOGO, SOUND and KEY as a data: URI.
    let data = "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAAMCAgMCAgMDAwMEAwMEBQgFBQQEBQoHBwYIDAoMDAsKCwsNDhIQDQ4RDgsLEBYQERMUFRUVDA8XGBYUGBIUFRT/";
    round_trips!(Uri, data);
    round_trips!(
        Uri,
        "data:text/plain;charset=utf-8;base64,SGVsbG8sIHdvcmxkIQ=="
    );
    round_trips!(Uri, "data:,Hello%2C%20World!");
}

#[test]
fn uri_holds_a_urn_uuid() {
    round_trips!(
        Uri,
        "urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
        "urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b"
    );
}

#[test]
fn uri_holds_the_other_schemes_vcard_uses() {
    round_trips!(
        Uri,
        "tel:+1-555-555-5555;ext=5555",
        "mailto:jane_doe@example.com",
        "geo:37.386013,-122.082932",
        "sip:alice@example.com",
        "xmpp:alice@example.com"
    );
}

#[test]
fn uri_rejects_a_relative_reference() {
    rejects!(Uri, "", "not a uri", "/just/a/path", "America/New_York");
}

// ---- param-value ----

#[test]
fn param_value_unquotes_and_decodes_the_caret_encoding() {
    let parse = |wire: &str| ParamValue::try_from(wire.as_bytes()).unwrap();
    assert_eq!(parse("plain").as_str(), "plain");
    assert_eq!(parse("\"a,b;c:d\"").as_str(), "a,b;c:d");
    assert_eq!(parse("line^nbreak").as_str(), "line\nbreak");
    assert_eq!(parse("say ^'hi^'").as_str(), "say \"hi\"");
    assert_eq!(parse("a^^b").as_str(), "a^b");
    // RFC 6868 §3.2: an undefined sequence is left as it is.
    assert_eq!(parse("a^zb").as_str(), "a^zb");
    assert_eq!(parse("").as_str(), "");
}

#[test]
fn param_value_is_written_quoted_only_when_it_has_to_be() {
    // "Property parameter value elements that contain the COLON, SEMICOLON,
    // or COMMA character separators MUST be specified as quoted-string".
    assert_eq!(ParamValue::new("plain text").to_string(), "plain text");
    assert_eq!(ParamValue::new("a,b").to_string(), "\"a,b\"");
    assert_eq!(ParamValue::new("a;b").to_string(), "\"a;b\"");
    assert_eq!(ParamValue::new("a:b").to_string(), "\"a:b\"");
}

#[test]
fn param_value_is_written_without_a_dquote_or_a_raw_newline() {
    // "Property parameter values MUST NOT contain the DQUOTE character."
    assert_eq!(ParamValue::new("say \"hi\"").to_string(), "say ^'hi^'");
    assert_eq!(ParamValue::new("a\nb").to_string(), "a^nb");
    assert_eq!(ParamValue::new("a^b").to_string(), "a^^b");
}

#[test]
fn param_value_survives_being_written_and_read() {
    for text in [
        "a,b",
        "say \"hi\"",
        "two\nlines, with: punctuation;",
        "^^",
        "^n",
        "",
    ] {
        let written = ParamValue::new(text).to_string();
        let back = ParamValue::try_from(written.as_bytes()).unwrap();
        assert_eq!(back.as_str(), text, "via {written:?}");
    }
}

#[test]
fn param_value_rejects_a_stray_dquote() {
    rejects!(ParamValue, "ab\"c", "\"abc", "abc\"", "\"a\"b\"");
}

// ---- N (§6.2.2) ----

fn text_of(items: &[Text]) -> Vec<&str> {
    items.iter().map(|t| t.as_str()).collect()
}

#[test]
fn name_examples_of_section_6_2_2() {
    let name = Name::try_from(&b"Public;John;Quinlan;Mr.;Esq."[..]).unwrap();
    assert_eq!(text_of(name.family()), ["Public"]);
    assert_eq!(text_of(name.given()), ["John"]);
    assert_eq!(text_of(name.additional()), ["Quinlan"]);
    assert_eq!(text_of(name.prefixes()), ["Mr."]);
    assert_eq!(text_of(name.suffixes()), ["Esq."]);

    let name =
        Name::try_from(&b"Stevenson;John;Philip,Paul;Dr.;Jr.,M.D.,A.C.P."[..])
            .unwrap();
    assert_eq!(text_of(name.additional()), ["Philip", "Paul"]);
    assert_eq!(text_of(name.suffixes()), ["Jr.", "M.D.", "A.C.P."]);

    round_trips!(
        Name,
        "Public;John;Quinlan;Mr.;Esq.",
        "Stevenson;John;Philip,Paul;Dr.;Jr.,M.D.,A.C.P."
    );
}

#[test]
fn name_keeps_empty_components_where_they_were() {
    let name = Name::try_from(&b"Doe;;;;"[..]).unwrap();
    assert_eq!(text_of(name.family()), ["Doe"]);
    assert!(name.given().is_empty());
    assert!(name.additional().is_empty());
    assert!(name.prefixes().is_empty());
    assert!(name.suffixes().is_empty());
    round_trips!(Name, "Doe;;;;", ";;;;", ";John;;;", ";;;;Jr.");
}

#[test]
fn name_written_short_is_written_back_short() {
    // `N:Doe` and `N:Doe;;;;` mean the same, and each keeps its text.
    let short = Name::try_from(&b"Doe"[..]).unwrap();
    let long = Name::try_from(&b"Doe;;;;"[..]).unwrap();
    assert_eq!(short.family(), long.family());
    assert!(short.given().is_empty());
    assert_eq!(short.to_string(), "Doe");
    assert_eq!(long.to_string(), "Doe;;;;");
    round_trips!(Name, "Doe;John", "");
}

#[test]
fn name_built_by_hand_has_all_five_components() {
    let name = Name::new(
        vec![Text::from("Doe")],
        vec![Text::from("John")],
        vec![],
        vec![],
        vec![],
    );
    assert_eq!(name.to_string(), "Doe;John;;;");
}

#[test]
fn name_escapes_separate_from_delimiters() {
    let name = Name::try_from(&b"Smith\\;Jones;A\\,B,C;;;"[..]).unwrap();
    assert_eq!(text_of(name.family()), ["Smith;Jones"]);
    // The escaped comma isn't a list separator; the plain one is.
    assert_eq!(text_of(name.given()), ["A,B", "C"]);
    assert_eq!(name.to_string(), "Smith\\;Jones;A\\,B,C;;;");
}

#[test]
fn name_has_no_more_than_five_components() {
    rejects!(Name, "a;b;c;d;e;f", ";;;;;");
}

// ---- ADR (§6.3.1) ----

#[test]
fn address_example_of_section_6_3_1() {
    let adr = Address::try_from(
        &b";;123 Main Street;Any Town;CA;91921-1234;U.S.A."[..],
    )
    .unwrap();
    assert!(adr.pobox().is_empty());
    assert!(adr.extended().is_empty());
    assert_eq!(text_of(adr.street()), ["123 Main Street"]);
    assert_eq!(text_of(adr.locality()), ["Any Town"]);
    assert_eq!(text_of(adr.region()), ["CA"]);
    assert_eq!(text_of(adr.code()), ["91921-1234"]);
    assert_eq!(text_of(adr.country()), ["U.S.A."]);
    round_trips!(Address, ";;123 Main Street;Any Town;CA;91921-1234;U.S.A.");
}

#[test]
fn address_components_can_be_lists_and_hold_escapes() {
    let adr = Address::try_from(
        &b"Mail Drop: TNE QB;;123 Main\\nStreet,Suite 4;Any Town\\, USA;CA;91921-1234;U.S.A."[..],
    )
    .unwrap();
    assert_eq!(text_of(adr.pobox()), ["Mail Drop: TNE QB"]);
    assert_eq!(text_of(adr.street()), ["123 Main\nStreet", "Suite 4"]);
    assert_eq!(text_of(adr.locality()), ["Any Town, USA"]);
    assert_eq!(
        adr.to_string(),
        "Mail Drop: TNE QB;;123 Main\\nStreet,Suite 4;Any Town\\, USA;CA;91921-1234;U.S.A."
    );
}

#[test]
fn address_keeps_the_separators_of_missing_components() {
    // "When a component value is missing, the associated component
    // separator MUST still be specified."
    round_trips!(Address, ";;;;;;", ";;;Any Town;;;", "PO Box 1");
}

#[test]
fn address_has_no_more_than_seven_components() {
    rejects!(Address, "a;b;c;d;e;f;g;h", ";;;;;;;");
}

// ---- ORG (§6.6.4) ----

#[test]
fn organization_example_of_section_6_6_4() {
    let org = Organization::try_from(
        &b"ABC\\, Inc.;North American Division;Marketing"[..],
    )
    .unwrap();
    assert_eq!(org.name().as_str(), "ABC, Inc.");
    assert_eq!(
        text_of(org.units()),
        ["North American Division", "Marketing"]
    );
    assert_eq!(
        org.to_string(),
        "ABC\\, Inc.;North American Division;Marketing"
    );
}

#[test]
fn organization_units_are_optional_and_keep_empty_ones() {
    let org = Organization::try_from(&b"ABC"[..]).unwrap();
    assert_eq!(org.name().as_str(), "ABC");
    assert!(org.units().is_empty());

    let org = Organization::try_from(&b"ABC;"[..]).unwrap();
    assert_eq!(text_of(org.units()), [""]);
    round_trips!(Organization, "ABC", "ABC;", "ABC;;Sales", "", ";Sales");
}

#[test]
fn organization_units_are_not_lists() {
    // Unlike N and ADR, an ORG component holds a `component`, not a
    // `list-component`: an escaped comma is a comma, and stays one.
    let org = Organization::try_from(&b"A\\,B;C\\,D"[..]).unwrap();
    assert_eq!(org.name().as_str(), "A,B");
    assert_eq!(text_of(org.units()), ["C,D"]);
}

// ---- GENDER (§6.2.7) ----

#[test]
fn gender_examples_of_section_6_2_7() {
    let g = |wire: &str| Gender::try_from(wire.as_bytes()).unwrap();

    assert_eq!(g("M").sex(), Some(Sex::Male));
    assert!(g("M").identity().is_none());
    assert_eq!(g("F").sex(), Some(Sex::Female));

    let gender = g("M;Fellow");
    assert_eq!(gender.sex(), Some(Sex::Male));
    assert_eq!(gender.identity().unwrap().as_str(), "Fellow");

    let gender = g("F;grrrl");
    assert_eq!(gender.sex(), Some(Sex::Female));
    assert_eq!(gender.identity().unwrap().as_str(), "grrrl");

    let gender = g("O;intersex");
    assert_eq!(gender.sex(), Some(Sex::Other));
    assert_eq!(gender.identity().unwrap().as_str(), "intersex");

    let gender = g(";it's complicated");
    assert_eq!(gender.sex(), None);
    assert_eq!(gender.identity().unwrap().as_str(), "it's complicated");

    round_trips!(
        Gender,
        "M",
        "F",
        "M;Fellow",
        "F;grrrl",
        "O;intersex",
        ";it's complicated"
    );
}

#[test]
fn gender_knows_all_five_letters() {
    for (letter, sex) in [
        ("M", Sex::Male),
        ("F", Sex::Female),
        ("O", Sex::Other),
        ("N", Sex::NotApplicable),
        ("U", Sex::Unknown),
    ] {
        assert_eq!(
            Gender::try_from(letter.as_bytes()).unwrap().sex(),
            Some(sex),
            "{letter}"
        );
    }
}

#[test]
fn gender_sex_may_be_empty_and_so_may_the_whole_value() {
    // `sex = "" / "M" / ...`, and `GENDER-value = sex [";" text]`.
    let gender = Gender::try_from(&b""[..]).unwrap();
    assert_eq!(gender.sex(), None);
    assert!(gender.identity().is_none());
    round_trips!(Gender, "");
}

#[test]
fn gender_keeps_whether_the_identity_was_written() {
    let bare = Gender::try_from(&b"M"[..]).unwrap();
    let empty = Gender::try_from(&b"M;"[..]).unwrap();
    assert!(bare.identity().is_none());
    assert_eq!(empty.identity().unwrap().as_str(), "");
    round_trips!(Gender, "M;", ";");
}

#[test]
fn gender_sex_letter_is_case_insensitive_and_written_upper_case() {
    // ABNF string literals are case-insensitive.
    let gender = Gender::try_from(&b"f;grrrl"[..]).unwrap();
    assert_eq!(gender.sex(), Some(Sex::Female));
    assert_eq!(gender.to_string(), "F;grrrl");
}

#[test]
fn gender_identity_is_text() {
    let gender = Gender::try_from(&b"O;a\\, b\\; c"[..]).unwrap();
    assert_eq!(gender.identity().unwrap().as_str(), "a, b; c");
    assert_eq!(gender.to_string(), "O;a\\, b\\; c");
}

#[test]
fn gender_rejects_what_is_not_a_sex_and_an_identity() {
    rejects!(
        Gender,
        "X",
        "MF",
        "Male",
        "1",
        " M",
        "M;a;b",
        "M;;",
        "M;a\\;b;c",
        "MM;x"
    );
}

// ---- CLIENTPIDMAP (§6.7.7) ----

#[test]
fn clientpidmap_examples_of_section_6_7_7() {
    let map = ClientPidMap::try_from(
        &b"1;urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b"[..],
    )
    .unwrap();
    assert_eq!(map.pid(), 1);
    assert_eq!(
        map.uri().to_string(),
        "urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b"
    );

    let map = ClientPidMap::try_from(
        &b"2;urn:uuid:d89c9c7a-2e1b-4832-82de-7e992d95faa5"[..],
    )
    .unwrap();
    assert_eq!(map.pid(), 2);

    round_trips!(
        ClientPidMap,
        "1;urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b",
        "2;urn:uuid:d89c9c7a-2e1b-4832-82de-7e992d95faa5",
        "10;http://example.com/source"
    );
}

#[test]
fn clientpidmap_uri_may_hold_semicolons() {
    let map =
        ClientPidMap::try_from(&b"1;data:text/plain;base64,SGk="[..]).unwrap();
    assert_eq!(map.pid(), 1);
    assert_eq!(map.uri().to_string(), "data:text/plain;base64,SGk=");
}

#[test]
fn clientpidmap_needs_digits_a_semicolon_and_a_uri() {
    rejects!(
        ClientPidMap,
        "",
        "1",
        "1;",
        ";urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b",
        "a;urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b",
        "-1;urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b",
        "+1;urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b",
        "1.1;urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b",
        "1;not a uri",
        "1:urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b",
        "99999999999;urn:uuid:3df403f4-5924-4bb7-b077-3c711d9eb34b"
    );
}

#[test]
fn clientpidmap_source_identifier_is_strictly_positive() {
    // §6.7.7: "PID source identifiers MUST be strictly positive. Zero is
    // not allowed."
    rejects!(ClientPidMap, "0;urn:uuid:1");
    assert!(ClientPidMap::new(0, Uri::parse("urn:uuid:1").unwrap()).is_err());
    assert_eq!(
        ClientPidMap::new(1, Uri::parse("urn:uuid:1").unwrap())
            .unwrap()
            .pid(),
        1
    );
}
