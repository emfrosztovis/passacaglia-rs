use passacaglia_common::Rational;
use passacaglia_core::structure::{
    Container, DurationalElement, EventContainer, InstantaneousElement, Located,
    SequentialContainer, TemporalElement,
};

struct Elem {
    duration: Rational,
    name: &'static str,
}

impl TemporalElement for Elem {}
impl DurationalElement for Elem {
    fn duration(&self) -> Rational {
        self.duration
    }
}

fn elements() -> Vec<Elem> {
    vec![
        Elem {
            duration: Rational::new(1, 4),
            name: "A",
        },
        Elem {
            duration: Rational::new(1, 4),
            name: "B",
        },
        Elem {
            duration: Rational::new(1, 2),
            name: "C",
        },
    ]
}

#[test]
fn sequential_iteration() {
    let container = SequentialContainer::new(elements());

    let mut cursor = container.first().unwrap();
    assert_eq!(cursor.name, "A");
    assert_eq!(cursor.time(), Rational::new(0, 1));
    assert_eq!(cursor.end_time(), Rational::new(1, 4));

    cursor = cursor.next().unwrap();
    assert_eq!(cursor.name, "B");
    assert_eq!(cursor.time(), Rational::new(1, 4));
    assert_eq!(cursor.end_time(), Rational::new(1, 2));

    cursor = cursor.next().unwrap();
    assert_eq!(cursor.name, "C");
    assert_eq!(cursor.time(), Rational::new(1, 2));
    assert_eq!(cursor.end_time(), Rational::new(1, 1));

    assert!(cursor.next().is_none());
}

#[test]
fn sequential_reverse_iteration() {
    let container = SequentialContainer::new(elements());

    let mut cursor = container.last().unwrap();
    assert_eq!(cursor.name, "C");
    assert_eq!(cursor.time(), Rational::new(1, 2));
    assert_eq!(cursor.end_time(), Rational::new(1, 1));

    cursor = cursor.prev().unwrap();
    assert_eq!(cursor.name, "B");
    assert_eq!(cursor.time(), Rational::new(1, 4));
    assert_eq!(cursor.end_time(), Rational::new(1, 2));

    cursor = cursor.prev().unwrap();
    assert_eq!(cursor.name, "A");
    assert_eq!(cursor.time(), Rational::new(0, 1));
    assert_eq!(cursor.end_time(), Rational::new(1, 4));

    assert!(cursor.prev().is_none());
}

#[test]
fn sequential_at() {
    let container = SequentialContainer::new(elements());

    let c = container.cursor(1).unwrap();
    assert_eq!(c.name, "B");
    assert_eq!(c.time(), Rational::new(1, 4));
    assert!(container.cursor(3).is_none());
}

#[test]
fn sequential_cursor_at_time() {
    let container = SequentialContainer::new(elements());

    assert_eq!(container.cursor_at_time(Rational::new(0, 1)).unwrap().name, "A");
    assert_eq!(
        container.cursor_at_time(Rational::new(1, 10)).unwrap().name,
        "A"
    );
    assert_eq!(
        container.cursor_at_time(Rational::new(1, 4)).unwrap().name,
        "B"
    );
    assert_eq!(
        container.cursor_at_time(Rational::new(1, 2)).unwrap().name,
        "C"
    );
    assert_eq!(
        container.cursor_at_time(Rational::new(99, 100)).unwrap().name,
        "C"
    );
    assert!(container.cursor_at_time(Rational::new(1, 1)).is_none());
}

#[test]
fn sequential_cursor_before_time() {
    let container = SequentialContainer::new(elements());

    assert!(container.cursor_before_time(Rational::new(0, 1)).is_none());
    assert_eq!(
        container.cursor_before_time(Rational::new(1, 10)).unwrap().name,
        "A"
    );
    assert_eq!(
        container.cursor_before_time(Rational::new(1, 4)).unwrap().name,
        "A"
    );
    assert_eq!(
        container.cursor_before_time(Rational::new(13, 50)).unwrap().name,
        "B"
    );
    assert_eq!(
        container.cursor_before_time(Rational::new(3, 2)).unwrap().name,
        "C"
    );
}

struct Event {
    name: &'static str,
}

impl TemporalElement for Event {}
impl InstantaneousElement for Event {}

fn events() -> Vec<Located<Event>> {
    vec![
        Located {
            offset: Rational::new(0, 1),
            element: Event { name: "E1" },
        },
        Located {
            offset: Rational::new(1, 2),
            element: Event { name: "E2" },
        },
        Located {
            offset: Rational::new(1, 2),
            element: Event { name: "E3" },
        },
    ]
}

#[test]
fn event_iteration() {
    let container = EventContainer::new(events());

    let mut c = container.first().unwrap();
    assert_eq!(c.name, "E1");
    assert_eq!(c.time(), Rational::new(0, 1));

    c = c.next().unwrap();
    assert_eq!(c.name, "E2");
    assert_eq!(c.time(), Rational::new(1, 2));

    c = c.next().unwrap();
    assert_eq!(c.name, "E3");
    assert_eq!(c.time(), Rational::new(1, 2));

    assert!(c.next().is_none());
}

#[test]
fn event_backward_iteration() {
    let container = EventContainer::new(events());

    let mut c = container.last().unwrap();
    assert_eq!(c.name, "E3");

    c = c.prev().unwrap();
    assert_eq!(c.name, "E2");

    c = c.prev().unwrap();
    assert_eq!(c.name, "E1");

    assert!(c.prev().is_none());
}
