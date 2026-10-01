use passacaglia_common::{rational, Rational};

use crate::structure::container::Container;
use crate::structure::elements::DurationalElement;

/// A container of [`DurationalElement`]s laid out back-to-back.
pub struct SequentialContainer<T: DurationalElement> {
    elements: Vec<T>,
    starts: Vec<Rational>,
}

impl<T: DurationalElement> SequentialContainer<T> {
    #[must_use]
    pub fn new(elements: Vec<T>) -> Self {
        let mut starts = Vec::with_capacity(elements.len());
        let mut acc = rational(0);
        for e in &elements {
            starts.push(acc);
            acc += e.duration();
        }
        SequentialContainer { elements, starts }
    }

    #[must_use]
    pub fn elements(&self) -> &[T] {
        &self.elements
    }
}

impl<T: DurationalElement> Container for SequentialContainer<T> {
    type Item = T;

    fn len(&self) -> usize {
        self.elements.len()
    }

    fn start(&self, i: usize) -> Rational {
        self.starts[i]
    }

    fn span(&self, i: usize) -> Rational {
        self.elements[i].duration()
    }

    fn item(&self, i: usize) -> &T {
        &self.elements[i]
    }
}
