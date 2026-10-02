#![warn(clippy::pedantic)]
#![allow(clippy::arc_with_non_send_sync)]

use std::rc::Rc;

use passacaglia_common::rational;
use passacaglia_core::std_hept::scales;
use passacaglia_species_counterpoint::{
    import_rules, rules, species1, species2, species3, species4, species5, CounterpointContext,
    CounterpointScoreBuilder, CounterpointSolver, CounterpointSolverRewardStrategy,
    NonHarmonicType, Parameters, Score,
};

/// A fully-configured context mirroring the reference setup used by the debug UI.
fn context() -> Rc<CounterpointContext> {
    let mut ctx = CounterpointContext::new(
        4,
        Parameters {
            measure_length: rational(4),
        },
    );

    ctx.harmony_rules = import_rules![
        rules::enforce_functional_progression_major,
        rules::enforce_valid_chords,
    ];

    ctx.local_rules = import_rules![
        rules::limit_consecutive_leaps,
        rules::forbid_perfects_by_similar_motion,
        rules::forbid_nearby_perfects,
        rules::prioritize_voice_motion,
        rules::enforce_vertical_consonance_with_moving_local,
    ];

    ctx.candidate_rules_before = import_rules![
        rules::enforce_scale_tones,
        rules::enforce_stepwise_around_short_notes,
        rules::enforce_passing_tones,
        rules::enforce_neighbor_tones,
        rules::enforce_suspension,
        rules::forbid_voice_overlapping2,
        rules::avoid_repeat2,
    ];

    ctx.candidate_rules_after = import_rules![
        rules::enforce_melody_intervals,
        rules::enforce_leap_preparation,
        rules::enforce_leap_resolution,
    ];

    ctx.harmonic_tone_rules = import_rules![rules::enforce_chord_tone];

    ctx.non_harmonic_tone_rules.insert(NonHarmonicType::Neighbor, import_rules![
        rules::make_neighbor_tone,
    ]);

    ctx.non_harmonic_tone_rules.insert(NonHarmonicType::PassingTone, import_rules![
        rules::make_passing_tone,
    ]);

    ctx.non_harmonic_tone_rules.insert(NonHarmonicType::Suspension, import_rules![
        rules::make_suspension,
    ]);

    ctx.allow_unison = true;

    Rc::new(ctx)
}

/// Solve `score` with the same parameters as the reference debug UI.
fn solve(ctx: Rc<CounterpointContext>, score: &Score) -> bool {
    let mut solver = CounterpointSolver::new(ctx);
    solver.batch = 50;
    solver.remove_old = 5;
    solver.report_interval = 100_000;
    solver
        .a_star(score, CounterpointSolverRewardStrategy::Constant { value: 25.0 })
        .is_some()
}

#[test]
fn monophonic_species1_solves() {
    let ctx = context();
    let score = CounterpointScoreBuilder::new(ctx.clone())
        .soprano(&species1())
        .build(&scales::c::MAJOR, None);
    assert!(solve(ctx, &score));
}

#[test]
fn monophonic_species2_solves() {
    let ctx = context();
    let score = CounterpointScoreBuilder::new(ctx.clone())
        .soprano(&species2())
        .build(&scales::c::MAJOR, None);
    assert!(solve(ctx, &score));
}

#[test]
fn monophonic_species3_solves() {
    let ctx = context();
    let score = CounterpointScoreBuilder::new(ctx.clone())
        .soprano(&species3())
        .build(&scales::c::MAJOR, None);
    assert!(solve(ctx, &score));
}

#[test]
fn monophonic_species4_solves() {
    let ctx = context();
    let score = CounterpointScoreBuilder::new(ctx.clone())
        .soprano(&species4())
        .build(&scales::c::MAJOR, None);
    assert!(solve(ctx, &score));
}

#[test]
fn monophonic_species5_solves() {
    let ctx = context();
    let score = CounterpointScoreBuilder::new(ctx.clone())
        .soprano(&species5())
        .build(&scales::c::MAJOR, None);
    assert!(solve(ctx, &score));
}

#[test]
fn four_voice_species1_solves() {
    let ctx = context();
    let score = CounterpointScoreBuilder::new(ctx.clone())
        .soprano(&species1())
        .alto(&species1())
        .tenor(&species1())
        .bass(&species1())
        .build(&scales::c::MAJOR, None);
    assert!(solve(ctx, &score));
}
