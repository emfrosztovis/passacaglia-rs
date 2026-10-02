#![warn(clippy::pedantic)]
#![allow(clippy::arc_with_non_send_sync)]

use std::rc::Rc;

use passacaglia_common::rational;
use passacaglia_core::std_hept::{scales, Interval, Pitch, PITCH_CLASSES};
use passacaglia_musicxml::{note, ToMxl};
use passacaglia_species_counterpoint::basic::CounterpointScoreBuilder;
use passacaglia_species_counterpoint::chord::Chord;
use passacaglia_species_counterpoint::clef::Clef;
use passacaglia_species_counterpoint::context::CounterpointContext;
use passacaglia_species_counterpoint::score::Parameters;
use passacaglia_species_counterpoint::voice::{parse_notes, NonHarmonicType, Note};
use quick_xml::Writer;

fn render_note(n: &Note, tie_start: bool) -> String {
    let mut writer = Writer::new(Vec::new());
    note(&mut writer, n, tie_start).unwrap();
    String::from_utf8(writer.into_inner()).unwrap()
}

#[test]
fn pitched_note() {
    let n = Note::new(rational(4), Pitch::parse("c4"), None);
    assert_eq!(
        render_note(&n, false),
        "<note><pitch><step>C</step><alter>0</alter><octave>4</octave></pitch><duration>8</duration></note>"
    );
}

#[test]
fn rest_note() {
    let n = Note::new(rational(2), None, None);
    assert_eq!(render_note(&n, false), "<note><rest/><duration>4</duration></note>");
}

#[test]
fn suspension_note() {
    let n = Note::new(rational(2), Pitch::parse("f4"), Some(NonHarmonicType::Suspension));
    assert_eq!(
        render_note(&n, false),
        "<note><pitch><step>F</step><alter>0</alter><octave>4</octave></pitch><duration>4</duration><notations><tied type=\"stop\"/></notations><lyric><text>S</text></lyric></note>"
    );
}

#[test]
fn tie_start_note() {
    let n = Note::new(rational(2), Pitch::parse("f4"), None);
    assert_eq!(
        render_note(&n, true),
        "<note><pitch><step>F</step><alter>0</alter><octave>4</octave></pitch><duration>4</duration><notations><tied type=\"start\"/></notations></note>"
    );
}

#[test]
fn score_document() {
    let ctx = std::rc::Rc::new(CounterpointContext::new(
        1,
        Parameters {
            measure_length: rational(4),
        },
    ));
    let mut builder = CounterpointScoreBuilder::new(ctx);
    builder.cantus(Clef::BASS, &[parse_notes(&[(Some("c3"), rational(4))])]);
    let score = builder.build(&passacaglia_core::std_hept::scales::c::MAJOR, None);

    let mxl = score.to_mxl();
    assert!(mxl.starts_with("<?xml version=\"1.0\"?>"));
    assert!(mxl.contains("<score-partwise version=\"4.0\">"));
    assert!(mxl.contains("<part-name>Cantus</part-name>"));
    assert!(mxl.contains("<sign>F</sign>"));
    assert!(mxl.contains("<step>C</step>"));
    assert!(mxl.contains("<duration>8</duration>"));
}

#[test]
fn harmony_direction_below_last_voice() {
    let ctx = Rc::new(CounterpointContext::new(
        1,
        Parameters {
            measure_length: rational(4),
        },
    ));
    let mut builder = CounterpointScoreBuilder::new(ctx);
    builder.cantus(
        Clef::TREBLE,
        &[parse_notes(&[(Some("c4"), rational(4))])],
    );
    builder.cantus(
        Clef::BASS,
        &[parse_notes(&[(Some("c3"), rational(4))])],
    );
    let chord = Chord::from_intervals_stacking(
        &[
            Interval::parse("M3").unwrap(),
            Interval::parse("m3").unwrap(),
        ],
        0,
        PITCH_CLASSES.c,
    );
    let score = builder.build(&scales::c::MAJOR, Some(&[chord]));

    let mxl = score.to_mxl();
    assert!(mxl.contains("<direction placement=\"below\">"));
    assert!(mxl.contains("<words>c|e|g</words>"));
}
