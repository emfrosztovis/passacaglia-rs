#![warn(clippy::pedantic)]
#![allow(clippy::arc_with_non_send_sync)]

use std::sync::Arc;

use passacaglia_common::rational;
use passacaglia_core::std_hept::scales;
use passacaglia_species_counterpoint::{
    rules, species1, species2, species3, species4, species5, CounterpointContext,
    CounterpointScoreBuilder, CounterpointSolver, CounterpointSolverRewardStrategy,
    NonHarmonicType, Parameters, Score,
};

/// A fully-configured context mirroring the reference setup used by the debug UI.
fn context() -> Arc<CounterpointContext> {
    let mut ctx = CounterpointContext::new(
        4,
        Parameters {
            measure_length: rational(4),
        },
    );

    ctx.harmony_rules = vec![
        Arc::new(&rules::enforce_functional_progression_major),
        Arc::new(&rules::enforce_valid_chords),
    ];

    ctx.local_rules = vec![
        Arc::new(&rules::limit_consecutive_leaps),
        Arc::new(&rules::forbid_perfects_by_similar_motion),
        Arc::new(&rules::forbid_nearby_perfects),
        Arc::new(&rules::prioritize_voice_motion),
        Arc::new(&rules::enforce_vertical_consonance_with_moving_local),
    ];

    ctx.candidate_rules_before = vec![
        Arc::new(&rules::enforce_scale_tones),
        Arc::new(&rules::enforce_stepwise_around_short_notes),
        Arc::new(&rules::enforce_passing_tones),
        Arc::new(&rules::enforce_neighbor_tones),
        Arc::new(&rules::enforce_suspension),
        Arc::new(&rules::forbid_voice_overlapping2),
        Arc::new(&rules::avoid_repeat2),
    ];

    ctx.candidate_rules_after = vec![
        Arc::new(&rules::enforce_melody_intervals),
        Arc::new(&rules::enforce_leap_preparation),
        Arc::new(&rules::enforce_leap_resolution),
    ];

    ctx.harmonic_tone_rules = vec![Arc::new(&rules::enforce_chord_tone)];

    ctx.non_harmonic_tone_rules.insert(NonHarmonicType::Neighbor, vec![
        Arc::new(&rules::make_neighbor_tone),
    ]);

    ctx.non_harmonic_tone_rules.insert(NonHarmonicType::PassingTone, vec![
        Arc::new(&rules::make_passing_tone),
    ]);

    ctx.non_harmonic_tone_rules.insert(NonHarmonicType::Suspension, vec![
        Arc::new(&rules::make_suspension),
    ]);

    ctx.allow_unison = true;

    Arc::new(ctx)
}

/// Solve `score` with the same parameters as the reference debug UI.
fn solve(ctx: Arc<CounterpointContext>, score: &Score) -> bool {
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
