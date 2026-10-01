use passacaglia_common::Rational;

/// Base marker for elements in musical structures.
pub trait TemporalElement {}

/// An element with a positive duration, such as a note or measure.
pub trait DurationalElement: TemporalElement {
    fn duration(&self) -> Rational;
}

/// A zero-length element (an "event").
pub trait InstantaneousElement: TemporalElement {}
