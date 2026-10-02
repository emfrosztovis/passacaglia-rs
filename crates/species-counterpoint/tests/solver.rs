#![warn(clippy::pedantic)]
#![allow(clippy::arc_with_non_send_sync)]

use std::rc::Rc;

use passacaglia_common::rational;
use passacaglia_core::std_hept::scales;
use passacaglia_species_counterpoint::basic::CounterpointScoreBuilder;
use passacaglia_species_counterpoint::context::CounterpointContext;
use passacaglia_species_counterpoint::score::Parameters;
use passacaglia_species_counterpoint::solver::{CounterpointSolver, CounterpointSolverRewardStrategy};

#[test]
fn empty_score_is_immediately_goal() {
    let ctx = Rc::new(CounterpointContext::new(
        2,
        Parameters {
            measure_length: rational(4),
        },
    ));
    let builder = CounterpointScoreBuilder::new(ctx.clone());
    let score = builder.build(&scales::c::MAJOR, None);

    let mut solver = CounterpointSolver::new(ctx);
    let result = solver.a_star(&score, CounterpointSolverRewardStrategy::Constant { value: 1.0 });
    assert!(result.is_some());
}
