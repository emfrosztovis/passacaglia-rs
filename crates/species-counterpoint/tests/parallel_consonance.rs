#![warn(clippy::pedantic)]
#![allow(
    clippy::arc_with_non_send_sync,
    clippy::float_cmp,
    clippy::too_many_lines
)]

use std::sync::Arc;

use passacaglia_common::rational;
use passacaglia_core::std_hept::scales;
use passacaglia_species_counterpoint::basic::CounterpointScoreBuilder;
use passacaglia_species_counterpoint::clef::Clef;
use passacaglia_species_counterpoint::context::CounterpointContext;
use passacaglia_species_counterpoint::rules::forbid_perfects_by_similar_motion;
use passacaglia_species_counterpoint::score::Parameters;
use passacaglia_species_counterpoint::voice::parse_notes;

fn context() -> Arc<CounterpointContext> {
    Arc::new(CounterpointContext::new(
        2,
        Parameters {
            measure_length: rational(4),
        },
    ))
}

#[test]
fn consecutive_perfects() {
    let ctx = context();

    let mut builder = CounterpointScoreBuilder::new(ctx.clone());
    builder.cantus(
        Clef::TREBLE,
        &[
            parse_notes(&[
                (Some("c5"), rational(1)),
                (Some("b4"), rational(1)),
                (Some("a4"), rational(1)),
                (Some("g4"), rational(1)),
            ]),
            parse_notes(&[(Some("f4"), rational(4))]),
        ],
    );
    builder.cantus(
        Clef::BASS,
        &[
            parse_notes(&[(Some("e3"), rational(4))]),
            parse_notes(&[(Some("f3"), rational(4))]),
        ],
    );
    let score = builder.build(&scales::c::MAJOR, None);
    let note = score.voices[1].note_at(rational(4)).unwrap();
    assert_eq!(forbid_perfects_by_similar_motion(&ctx, &score, note), 0.0);

    let mut builder = CounterpointScoreBuilder::new(ctx.clone());
    builder.cantus(
        Clef::TREBLE,
        &[
            parse_notes(&[
                (Some("c5"), rational(1)),
                (Some("b4"), rational(1)),
                (Some("a4"), rational(1)),
                (Some("g4"), rational(1)),
            ]),
            parse_notes(&[(Some("f4"), rational(4))]),
        ],
    );
    builder.cantus(
        Clef::BASS,
        &[
            parse_notes(&[(Some("g3"), rational(4))]),
            parse_notes(&[(Some("f3"), rational(4))]),
        ],
    );
    let score = builder.build(&scales::c::MAJOR, None);
    let note = score.voices[1].note_at(rational(4)).unwrap();
    assert_eq!(
        forbid_perfects_by_similar_motion(&ctx, &score, note),
        f64::INFINITY
    );

    let mut builder = CounterpointScoreBuilder::new(ctx.clone());
    builder.cantus(
        Clef::TREBLE,
        &[
            parse_notes(&[
                (Some("c5"), rational(1)),
                (Some("b4"), rational(1)),
                (Some("a4"), rational(1)),
                (Some("g4"), rational(1)),
            ]),
            parse_notes(&[(Some("f4"), rational(4))]),
        ],
    );
    builder.cantus(
        Clef::BASS,
        &[
            parse_notes(&[(Some("c3"), rational(4))]),
            parse_notes(&[(Some("f3"), rational(4))]),
        ],
    );
    let score = builder.build(&scales::c::MAJOR, None);
    let note = score.voices[1].note_at(rational(4)).unwrap();
    assert_eq!(
        forbid_perfects_by_similar_motion(&ctx, &score, note),
        f64::INFINITY
    );

    let mut builder = CounterpointScoreBuilder::new(ctx.clone());
    builder.cantus(
        Clef::TREBLE,
        &[
            parse_notes(&[
                (Some("e5"), rational(2)),
                (Some("f5"), rational(1)),
                (Some("g5"), rational(1)),
            ]),
            parse_notes(&[(Some("a5"), rational(4))]),
        ],
    );
    builder.cantus(
        Clef::ALTO,
        &[
            parse_notes(&[
                (Some("e4"), rational(2)),
                (Some("d4"), rational(1)),
                (Some("c4"), rational(1)),
            ]),
            parse_notes(&[(Some("a4"), rational(4))]),
        ],
    );
    builder.cantus(
        Clef::BASS,
        &[
            parse_notes(&[
                (Some("c3"), rational(2)),
                (Some("d3"), rational(1)),
                (Some("e3"), rational(1)),
            ]),
            parse_notes(&[(None, rational(4))]),
        ],
    );
    let score = builder.build(&scales::c::MAJOR, None);
    let note = score.voices[1].note_at(rational(4)).unwrap();
    assert_eq!(
        forbid_perfects_by_similar_motion(&ctx, &score, note),
        f64::INFINITY
    );
}

#[test]
fn perfect_by_similar_motion() {
    let ctx = context();

    let mut builder = CounterpointScoreBuilder::new(ctx.clone());
    builder.cantus(
        Clef::TREBLE,
        &[
            parse_notes(&[(Some("g4"), rational(4))]),
            parse_notes(&[(Some("f4"), rational(4))]),
        ],
    );
    builder.cantus(
        Clef::BASS,
        &[
            parse_notes(&[(Some("a3"), rational(4))]),
            parse_notes(&[(Some("f3"), rational(4))]),
        ],
    );
    let score = builder.build(&scales::c::MAJOR, None);
    let note = score.voices[1].note_at(rational(4)).unwrap();
    assert_eq!(
        forbid_perfects_by_similar_motion(&ctx, &score, note),
        f64::INFINITY
    );
}
