use std::{collections::HashMap, fs::{self, File}, rc::Rc};

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
        rules::enforce_functional_progression_major(),
        rules::enforce_valid_chords(),
    ];

    ctx.local_rules = vec![
        rules::limit_consecutive_leaps(),
        rules::forbid_perfects_by_similar_motion(),
        rules::forbid_nearby_perfects(),
        rules::prioritize_voice_motion(),
        rules::enforce_vertical_consonance_with_moving_local(),
    ];

    ctx.candidate_rules_before = vec![
        rules::enforce_scale_tones(),
        rules::enforce_stepwise_around_short_notes(),
        rules::enforce_passing_tones(),
        rules::enforce_neighbor_tones(),
        rules::enforce_suspension(),
        rules::forbid_voice_overlapping2(),
        rules::avoid_repeat2(),
    ];

    ctx.candidate_rules_after = vec![
        rules::enforce_melody_intervals(),
        rules::enforce_leap_preparation(),
        rules::enforce_leap_resolution(),
    ];

    ctx.harmonic_tone_rules = vec![
        rules::enforce_chord_tone(),
    ];

    ctx.non_harmonic_tone_rules = HashMap::from([
        (NonHarmonicType::Neighbor, vec![rules::make_neighbor_tone()]),
        (NonHarmonicType::PassingTone, vec![rules::make_passing_tone()]),
        (NonHarmonicType::Suspension, vec![rules::make_suspension()]),
    ]);

    ctx.allow_unison = true;

    let ctx = Rc::new(ctx);

    let score = 
        CounterpointScoreBuilder::new(ctx.clone())
        .soprano(&species5())
        .alto(&species5())
        .tenor(&species5())
        .bass(&species5())
        .build(&std_hept::scales::c::MAJOR, None)
    ;

    let mut solver = CounterpointSolver::new(ctx.clone());

    solver.set_reporter(|p| {
        println!("{} {} {}", p.iteration, p.furthest, p.measure_index);
    });

    solver.report_interval = 2000;
    solver.batch = 50;
    solver.remove_old = 5;

    let result = {
        let guard = pprof::ProfilerGuardBuilder::default()
            .frequency(1000)
            .blocklist(&["libc", "libgcc", "pthread", "vdso"])
            .build()
            .unwrap();

        let r = solver.a_star(&score, 
            CounterpointSolverRewardStrategy::Constant { value: 25.0 }
        );

        if let Ok(report) = guard.report().build() {
            fs::write("report.txt", format!("{report:?}").as_bytes()).unwrap();
            let file = File::create("flamegraph.svg").unwrap();
            report.flamegraph(file).unwrap();
            println!("flamegraph generated");
        }
        r
    };

    if let Some(_x) = result {
        println!("ok");
        // print!("{}", x.to_mxl());
    }
}