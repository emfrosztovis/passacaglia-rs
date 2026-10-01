use passacaglia_common::{rational, Rational};

use crate::structure::container::Container;
use crate::structure::elements::InstantaneousElement;

/// An event with its offset within an [`EventContainer`].
pub struct Located<T> {
    pub offset: Rational,
    pub element: T,
}

/// A container of zero-length [`InstantaneousElement`]s at arbitrary offsets.
pub struct EventContainer<T: InstantaneousElement> {
    elements: Vec<Located<T>>,
}

impl<T: InstantaneousElement> EventContainer<T> {
    #[must_use]
    pub fn new(elements: Vec<Located<T>>) -> Self {
        EventContainer { elements }
    }

    #[must_use]
    pub fn elements(&self) -> &[Located<T>] {
        &self.elements
    }
}

impl<T: InstantaneousElement> Container for EventContainer<T> {
    type Item = T;

    fn len(&self) -> usize {
        self.elements.len()
    }

    fn start(&self, i: usize) -> Rational {
        self.elements[i].offset
    }

    fn span(&self, _i: usize) -> Rational {
        rational(0)
    }

    fn item(&self, i: usize) -> &T {
        &self.elements[i].element
    }
}
