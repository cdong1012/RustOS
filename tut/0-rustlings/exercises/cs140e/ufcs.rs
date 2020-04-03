// FIXME: Make me pass! Diff budget: 2 lines.

struct Dummy;

pub trait Foo {
    fn foo(&self) -> usize { 1 }
}

pub trait FooToo {
    fn foo(&self) -> usize { 2 }
}

<<<<<<< HEAD
impl Foo for Dummy {
}
=======
impl Foo for Dummy { }
>>>>>>> skeleton/lab2

impl FooToo for Dummy { }

#[test]
fn ufcs() {
    let dummy = Dummy;

    let x = <Dummy as Foo>::foo(&dummy);
    let y = <Dummy as FooToo>::foo(&dummy);

    // Values for `x` and `y` must come from calling `foo()` methods.
    assert_eq!(x, 1);
    assert_eq!(y, 2);
}
