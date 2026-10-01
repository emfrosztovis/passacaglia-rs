use std::ops::Deref;

use passacaglia_common::Rational;

/// An ordered sequence of children, each with a start time and a span.
pub trait Container {
    type Item;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Local start time of child `i`.
    fn start(&self, i: usize) -> Rational;
    /// Duration (or extent) of child `i`.
    fn span(&self, i: usize) -> Rational;
    /// The element at index `i` (precondition: `i < len()`).
    fn item(&self, i: usize) -> &Self::Item;

    fn get(&self, i: usize) -> Option<&Self::Item> {
        (i < self.len()).then(|| self.item(i))
    }

    fn cursor(&self, i: usize) -> Option<Cursor<'_, Self, ()>> {
        (i < self.len()).then(|| Cursor {
            container: self,
            index: i,
            local_time: self.start(i),
            global_time: self.start(i),
            parent: (),
        })
    }

    fn first(&self) -> Option<Cursor<'_, Self, ()>> {
        self.cursor(0)
    }

    fn last(&self) -> Option<Cursor<'_, Self, ()>> {
        if self.is_empty() {
            return None;
        }
        self.cursor(self.len() - 1)
    }

    /// The cursor of the element containing `time` (start inclusive, end exclusive).
    fn cursor_at_time(&self, time: Rational) -> Option<Cursor<'_, Self, ()>> {
        for i in 0..self.len() {
            let s = self.start(i);
            if s <= time && s + self.span(i) > time {
                return self.cursor(i);
            }
        }
        None
    }

    /// The cursor of the last element strictly before `time`.
    fn cursor_before_time(&self, time: Rational) -> Option<Cursor<'_, Self, ()>> {
        let mut last = None;
        for i in 0..self.len() {
            if self.start(i) >= time {
                return last;
            }
            last = self.cursor(i);
        }
        last
    }

    fn iter_cursors(&self) -> Cursors<'_, Self> {
        Cursors {
            container: self,
            index: 0,
        }
    }
}

/// A cursor into a [`Container`] with a statically-typed parent cursor `P`.
///
/// `C` is the immediate container being indexed; `P` is the typed parent cursor
/// (`()` at the root). Cursors are `Copy` and allocation-free.
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
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn time(&self) -> Rational {
        self.local_time
    }

    pub fn global_time(&self) -> Rational {
        self.global_time
    }

    pub fn parent(&self) -> P {
        self.parent
    }

    pub fn container(&self) -> &'a C {
        self.container
    }

    pub fn span(&self) -> Rational {
        self.container.span(self.index)
    }

    pub fn end_time(&self) -> Rational {
        self.local_time + self.container.span(self.index)
    }

    pub fn global_end_time(&self) -> Rational {
        self.global_time + self.container.span(self.index)
    }

    pub fn prev(&self) -> Option<Self> {
        if self.index == 0 {
            return None;
        }
        let new_index = self.index - 1;
        let new_local = self.container.start(new_index);
        let parent_global = self.global_time - self.local_time;
        Some(Cursor {
            container: self.container,
            index: new_index,
            local_time: new_local,
            global_time: parent_global + new_local,
            parent: self.parent,
        })
    }

    pub fn next(&self) -> Option<Self> {
        if self.index + 1 >= self.container.len() {
            return None;
        }
        let new_index = self.index + 1;
        let new_local = self.container.start(new_index);
        let parent_global = self.global_time - self.local_time;
        Some(Cursor {
            container: self.container,
            index: new_index,
            local_time: new_local,
            global_time: parent_global + new_local,
            parent: self.parent,
        })
    }

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

    pub fn first_child(&self) -> Option<Cursor<'a, C::Item, Self>>
    where
        C::Item: Container,
    {
        if self.container.item(self.index).len() == 0 {
            return None;
        }
        self.child(0)
    }

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

/// A step target for cross-boundary navigation (`next_global`/`prev_global`).
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
    /// The next leaf in timeline order (crossing container boundaries).
    pub fn next_global(&self) -> Option<Self> {
        if let Some(n) = self.next() {
            return Some(n);
        }
        self.parent.next()?.first()
    }

    /// The previous leaf in timeline order (crossing container boundaries).
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
