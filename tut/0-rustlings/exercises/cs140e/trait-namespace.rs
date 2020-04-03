// FIXME: Make me compile! Diff budget: 2 lines.

// Do not change this module.
mod a {
    pub trait MyTrait {
        fn foo(&self) {  }
    }

    pub struct MyType;

<<<<<<< HEAD
    impl MyTrait for MyType {}
    
=======
    impl MyTrait for MyType {  }
>>>>>>> skeleton/lab2
}

// Do not modify this function.
fn main() {
    let x = a::MyType;
    use a::MyTrait;
    x.foo();
}
