use passacaglia_common::{rational, rational_to_string, rotate_array, Rational};

#[test]
fn rational_formatting() {
    assert_eq!(rational_to_string(Rational::new(1, 2), false, false), "1/2");
    assert_eq!(rational_to_string(Rational::new(1, 2), false, true), "1/2");
    assert_eq!(rational_to_string(Rational::new(3, 1), true, false), "+3");
    assert_eq!(rational_to_string(Rational::new(3, 1), false, true), "3");
    assert_eq!(
        rational_to_string(Rational::new(7, 3), true, true),
        "+2 1/3"
    );
    assert_eq!(
        rational_to_string(Rational::new(-3, 2), false, false),
        "-3/2"
    );
}

#[test]
fn rotate_array_values() {
    let arr = [1, 2, 3, 4, 5];
    assert_eq!(rotate_array(&arr, 0), vec![1, 2, 3, 4, 5]);
    assert_eq!(rotate_array(&arr, 1), vec![2, 3, 4, 5, 1]);
    assert_eq!(rotate_array(&arr, 5), vec![1, 2, 3, 4, 5]);
    assert_eq!(rotate_array(&arr, 6), vec![2, 3, 4, 5, 1]);
    assert_eq!(rotate_array(&arr, -1), vec![5, 1, 2, 3, 4]);
    assert_eq!(rotate_array(&arr, -6), vec![5, 1, 2, 3, 4]);
    let empty: [i32; 0] = [];
    assert_eq!(rotate_array(&empty, 1), Vec::<i32>::new());
}

#[test]
fn rational_identity() {
    assert_eq!(rational(0), Rational::new(0, 1));
    assert_eq!(rational(12), Rational::new(12, 1));
}
