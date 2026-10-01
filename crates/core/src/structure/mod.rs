//! Musical structure containers and cursors.
//!
//! A [`Container`] is an ordered sequence of children with a start time and
//! span; [`SequentialContainer`] and [`EventContainer`] are the two time models.
//! A [`Cursor`] is a cheap, `Copy` zipper into a (possibly nested) container with
//! a statically-typed parent chain.

mod container;
mod elements;
mod event;
mod sequential;

pub use container::{Container, Cursor, Cursors, Stepper};
pub use elements::{DurationalElement, InstantaneousElement, TemporalElement};
pub use event::{EventContainer, Located};
pub use sequential::SequentialContainer;
