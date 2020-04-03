// FIXME: Make me compile! Diff budget: 3 lines.

// Do not modify the inner type &'a T.
struct RefWrapper<'a, T>(&'a T);

// Do not modify the inner type &'b RefWrapper<'a, T>.
struct RefWrapperWrapper<'a, 'b, T>(&'b RefWrapper<'a, T>);

impl<'a, 'b, T> RefWrapperWrapper <'a, 'b, T>{
    fn inner(&'b self) -> &'a T {
        (self.0).0
    }
}

pub fn main() { }
