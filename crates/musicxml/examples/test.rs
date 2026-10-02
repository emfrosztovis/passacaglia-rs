use std::{collections::HashMap, rc::Rc};

use passacaglia_common::rational;
use passacaglia_core::std_hept;
use passacaglia_musicxml::ToMxl;

use passacaglia_species_counterpoint::NonHarmonicType;
#[allow(unused_imports)]
use passacaglia_species_counterpoint::{CandidateRule, CounterpointContext, CounterpointScoreBuilder, CounterpointSolver, CounterpointSolverRewardStrategy, Parameters, import_rules, rules, species1, species2, species5};

#[allow(clippy::arc_with_non_send_sync)]
fn main() {
    let mut ctx = CounterpointContext::new(
        4, 
        Parameters { measure_length: rational(4) }
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

    ctx.harmonic_tone_rules = import_rules![
        rules::enforce_chord_tone,
    ];

    ctx.non_harmonic_tone_rules = HashMap::from([
        (NonHarmonicType::Neighbor, import_rules!(CandidateRule; rules::make_neighbor_tone)),
        (NonHarmonicType::PassingTone, import_rules!(CandidateRule; rules::make_passing_tone)),
        (NonHarmonicType::Suspension, import_rules!(CandidateRule; rules::make_suspension)),
    ]);

    ctx.allow_unison = true;

    let ctx = Rc::new(ctx);

    let score = 
        CounterpointScoreBuilder::new(ctx.clone())
        .soprano(&species5())
        // .alto(&species1())
        // .tenor(&species1())
        // .bass(&species1())
        .build(&std_hept::scales::c::MAJOR, None)
    ;

    let mut solver = CounterpointSolver::new(ctx.clone());

    solver.set_reporter(|p| {
        println!("{} {} {}", p.iteration, p.furthest, p.measure_index);
    });

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