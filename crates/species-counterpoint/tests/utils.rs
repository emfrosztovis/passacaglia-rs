#![warn(clippy::pedantic)]

use passacaglia_core::std_hept::Interval;
use passacaglia_species_counterpoint::rules::utils::is_perfect_consonance;

#[test]
fn is_perfect_consonance_values() {
    assert!(is_perfect_consonance(&Interval::parse("P1").unwrap()));
    assert!(is_perfect_consonance(&Interval::parse("P5").unwrap()));
    assert!(is_perfect_consonance(&Interval::parse("P8").unwrap()));
    assert!(is_perfect_consonance(&Interval::parse("d2").unwrap()));

    assert!(!is_perfect_consonance(&Interval::parse("P4").unwrap()));
    assert!(!is_perfect_consonance(&Interval::parse("M3").unwrap()));

    assert!(is_perfect_consonance(&Interval::parse("P12").unwrap()));
    assert!(is_perfect_consonance(&Interval::parse("-P19").unwrap()));
}
