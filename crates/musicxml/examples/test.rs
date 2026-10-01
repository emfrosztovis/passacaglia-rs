use std::{sync::Arc};

use passacaglia_common::rational;
use passacaglia_core::std_hept;
use passacaglia_musicxml::ToMxl;

use passacaglia_species_counterpoint::NonHarmonicType;
#[allow(unused_imports)]
use passacaglia_species_counterpoint::{CounterpointContext, CounterpointScoreBuilder, CounterpointSolver, CounterpointSolverRewardStrategy, Parameters, rules, species1, species2, species5};

#[allow(clippy::arc_with_non_send_sync)]
fn main() {
    let mut ctx = CounterpointContext::new(
        4, 
        Parameters { measure_length: rational(4) }
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

    ctx.harmonic_tone_rules = vec![
        Arc::new(&rules::enforce_chord_tone),
    ];

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

    let arc = Arc::new(ctx);

    let score = 
        CounterpointScoreBuilder::new(arc.clone())
        .soprano(&species5())
        // .alto(&species1())
        // .tenor(&species1())
        // .bass(&species1())
        .build(&std_hept::scales::c::MAJOR, None)
    ;

    let mut solver = CounterpointSolver::new(arc.clone());
    solver.on_progress = Some(Box::new(|p| {
        println!("{} {} {}", p.iteration, p.furthest, p.measure_index);
    }));

    solver.report_interval = 2000;
    solver.batch = 50;
    solver.remove_old = 5;
    
    let result = solver.a_star(&score, 
        CounterpointSolverRewardStrategy::Constant { value: 25.0 }
    );

    if let Some(x) = result {
        println!("ok");
        print!("{}", x.to_mxl());
    }
}