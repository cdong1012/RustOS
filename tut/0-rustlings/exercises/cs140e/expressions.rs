// FIXME: Make me pass! Diff budget: 10 lines.
// Do not `use` any items.

// Do not change the following two lines.
#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
struct IntWrapper(isize);

<<<<<<< HEAD
fn max<T : PartialOrd>(first : T, second : T) -> T {
    if first > second {
        first
    } else {
        second
    }
}
// Implement a generic function here
// fn max...

=======
>>>>>>> skeleton/lab2
#[test]
fn expressions() {
    assert_eq!(max(1usize, 3), 3);
    assert_eq!(max(1u8, 3), 3);
    assert_eq!(max(1u8, 3), 3);
    assert_eq!(max(IntWrapper(120), IntWrapper(248)), IntWrapper(248));
}
