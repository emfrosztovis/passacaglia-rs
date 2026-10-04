use std::ops::Deref;

use passacaglia_common::Rational;

/// An ordered sequence where each element has a start time and a span.
pub trait Container {
    type Item;

    /// Number of elements in the container.
    fn len(&self) -> usize;

    /// Whether the container has no children.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether children are laid out back-to-back (`start(i + 1) == start(i) +
    /// span(i)`). Sequential containers can navigate cursors in O(1) by adding or
    /// subtracting a span instead of recomputing the prefix sum.
    fn is_sequential(&self) -> bool {
        false
    }
    /// Local start time of child `i`.
    fn start(&self, i: usize) -> Rational;
    /// Duration (or extent) of child `i`.
    fn span(&self, i: usize) -> Rational;
    /// The element at index `i` (precondition: `i < len()`).
    fn item(&self, i: usize) -> &Self::Item;

    /// Retrieves the element at index `i`.
    fn get(&self, i: usize) -> Option<&Self::Item> {
        (i < self.len()).then(|| self.item(i))
    }

    /// Returns a cursor pointing to the element at index `i`.
    fn cursor(&self, i: usize) -> Option<Cursor<'_, Self, ()>> {
        (i < self.len()).then(|| Cursor {
            container: self,
            index: i,
            local_time: self.start(i),
            global_time: self.start(i),
            parent: (),
        })
    }

    /// Returns a cursor pointing to the first element, if any.
    fn first(&self) -> Option<Cursor<'_, Self, ()>> {
        self.cursor(0)
    }

    /// Returns a cursor pointing to the last element, if any.
    fn last(&self) -> Option<Cursor<'_, Self, ()>> {
        if self.is_empty() {
            return None;
        }
        self.cursor(self.len() - 1)
    }

    /// Returns a cursor pointint to the element that contains the given time point (start 
    /// inclusive, end exclusive), if any.
    fn cursor_at_time(&self, time: Rational) -> Option<Cursor<'_, Self, ()>> {
        if self.is_sequential() {
            let mut s = self.start(0);
            for i in 0..self.len() {
                let e = s + self.span(i);
                if s <= time && e > time {
                    return Some(Cursor {
                        container: self,
                        index: i,
                        local_time: s,
                        global_time: s,
                        parent: (),
                    });
                }
                s = e;
            }
            return None;
        }
        for i in 0..self.len() {
            let s = self.start(i);
            if s <= time && s + self.span(i) > time {
                return self.cursor(i);
            }
        }
        None
    }

    /// Returns a cursor pointing to the last element strictly before `time`.
    fn cursor_before_time(&self, time: Rational) -> Option<Cursor<'_, Self, ()>> {
        if self.is_sequential() {
            let mut last = None;
            let mut s = self.start(0);
            for i in 0..self.len() {
                if s >= time {
                    return last;
                }
                last = Some(Cursor {
                    container: self,
                    index: i,
                    local_time: s,
                    global_time: s,
                    parent: (),
                });
                s += self.span(i);
            }
            return last;
        }
        let mut last = None;
        for i in 0..self.len() {
            if self.start(i) >= time {
                return last;
            }
            last = self.cursor(i);
        }
        last
    }

    /// Returns an iterator over the cursors pointing to the elements.
    fn iter_cursors(&self) -> Cursors<'_, Self> {
        Cursors {
            container: self,
            index: 0,
        }
    }
}

/// An allocation-free cursor into a [`Container`].
///
/// `C` is the immediate container being indexed. `P` is the typed parent cursor, i.e. a cursor 
/// pointing to this cursor's container. If no parent is available, `P` is `()`.
pub struct Cursor<'a, C: Container + ?Sized, P> {
    container: &'a C,
    index: usize,
    local_time: Rational,
    global_time: Rational,
    parent: P,
}

impl<C: Container + ?Sized, P: Copy> Clone for Cursor<'_, C, P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C: Container + ?Sized, P: Copy> Copy for Cursor<'_, C, P> {}

impl<C: Container + ?Sized, P> Deref for Cursor<'_, C, P> {
    type Target = C::Item;

    fn deref(&self) -> &C::Item {
        self.container.item(self.index)
    }
}

impl<'a, C: Container + ?Sized, P: Copy> Cursor<'a, C, P> {
    /// Returns the index of the cursor in the container.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the local start time of the element under the cursor, i.e. time relative to the 
    /// start of the container.
    pub fn time(&self) -> Rational {
        self.local_time
    }

    /// Returns the global time of the element under the cursor, i.e. time relative to the topmost 
    /// container that the cursor knows of.
    pub fn global_time(&self) -> Rational {
        self.global_time
    }

    /// Returns the parent cursor, or `()` if there is none.
    pub fn parent(&self) -> P {
        self.parent
    }

    // Returns a reference to the container of this cursor.
    pub fn container(&self) -> &'a C {
        self.container
    }

    /// Returns the duration of the element under the cursor.
    pub fn span(&self) -> Rational {
        self.container.span(self.index)
    }

    /// Returns the local end time of the element under the cursor, i.e. time relative to the start 
    /// of the container.
    pub fn end_time(&self) -> Rational {
        self.local_time + self.container.span(self.index)
    }

    /// Returns the global end time of the element under the cursor, i.e. time relative to the 
    /// topmost container that the cursor knows of.
    pub fn global_end_time(&self) -> Rational {
        self.global_time + self.container.span(self.index)
    }

    /// Returns a cursor to the element before the one this cursor points to, if any.
    pub fn prev(&self) -> Option<Self> {
        if self.index == 0 {
            return None;
        }
        let new_index = self.index - 1;
        let new_local = if self.container.is_sequential() {
            self.local_time - self.container.span(new_index)
        } else {
            self.container.start(new_index)
        };
        let parent_global = self.global_time - self.local_time;
        Some(Cursor {
            container: self.container,
            index: new_index,
            local_time: new_local,
            global_time: parent_global + new_local,
            parent: self.parent,
        })
    }

    /// Returns a cursor to the element after the one this cursor points to, if any.
    pub fn next(&self) -> Option<Self> {
        if self.index + 1 >= self.container.len() {
            return None;
        }
        let new_index = self.index + 1;
        let new_local = if self.container.is_sequential() {
            self.local_time + self.container.span(self.index)
        } else {
            self.container.start(new_index)
        };
        let parent_global = self.global_time - self.local_time;
        Some(Cursor {
            container: self.container,
            index: new_index,
            local_time: new_local,
            global_time: parent_global + new_local,
            parent: self.parent,
        })
    }

    /// Returns a cursor to a children of the element that this cursor points to.
    pub fn child(&self, i: usize) -> Option<Cursor<'a, C::Item, Self>>
    where
        C::Item: Container,
    {
        let child = self.container.item(self.index);
        if i >= child.len() {
            return None;
        }
        Some(Cursor {
            container: child,
            index: i,
            local_time: child.start(i),
            global_time: self.global_time + child.start(i),
            parent: *self,
        })
    }

    /// Returns a cursor to the first child of the element that this cursor points to.
    pub fn first_child(&self) -> Option<Cursor<'a, C::Item, Self>>
    where
        C::Item: Container,
    {
        if self.container.item(self.index).len() == 0 {
            return None;
        }
        self.child(0)
    }

    /// Returns a cursor to the last child of the element that this cursor points to.
    pub fn last_child(&self) -> Option<Cursor<'a, C::Item, Self>>
    where
        C::Item: Container,
    {
        let child = self.container.item(self.index);
        if child.len() == 0 {
            return None;
        }
        self.child(child.len() - 1)
    }
}

/// A step target for cross-boundary navigation, used by [`Cursor::next_global`] and 
/// [`Cursor::prev_global`].
///
/// Implemented by `()` (the root, which does not step) and by every cursor whose
/// element type is itself a [`Container`].
pub trait Stepper<'a, C: Container + ?Sized>: Copy {
    fn next(self) -> Option<Self>;
    fn prev(self) -> Option<Self>;
    fn first(self) -> Option<Cursor<'a, C, Self>>;
    fn last(self) -> Option<Cursor<'a, C, Self>>;
}

impl<'a, C: Container + ?Sized> Stepper<'a, C> for () {
    fn next(self) -> Option<Self> {
        None
    }
    fn prev(self) -> Option<Self> {
        None
    }
    fn first(self) -> Option<Cursor<'a, C, Self>> {
        None
    }
    fn last(self) -> Option<Cursor<'a, C, Self>> {
        None
    }
}

impl<'a, C, D, P> Stepper<'a, C> for Cursor<'a, D, P>
where
    C: Container + ?Sized,
    D: Container<Item = C> + ?Sized,
    P: Copy,
{
    fn next(self) -> Option<Self> {
        Cursor::next(&self)
    }
    fn prev(self) -> Option<Self> {
        Cursor::prev(&self)
    }
    fn first(self) -> Option<Cursor<'a, C, Self>> {
        Cursor::first_child(&self)
    }
    fn last(self) -> Option<Cursor<'a, C, Self>> {
        Cursor::last_child(&self)
    }
}

impl<'a, C: Container + ?Sized, P: Stepper<'a, C>> Cursor<'a, C, P> {
    /// Returns the next same-level element, crossing container boundaries.
    pub fn next_global(&self) -> Option<Self> {
        if let Some(n) = self.next() {
            return Some(n);
        }
        self.parent.next()?.first()
    }

    /// Returns the previous same-level element, crossing container boundaries.
    pub fn prev_global(&self) -> Option<Self> {
        if let Some(p) = self.prev() {
            return Some(p);
        }
        self.parent.prev()?.last()
    }
}

/// An iterator over the cursors of a container (top-level, parent `()`).
pub struct Cursors<'a, C: Container + ?Sized> {
    container: &'a C,
    index: usize,
}

impl<'a, C: Container + ?Sized> Iterator for Cursors<'a, C> {
    type Item = Cursor<'a, C, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.container.len() {
            return None;
        }
        let c = self.container.cursor(self.index);
        self.index += 1;
        c
    }
}
