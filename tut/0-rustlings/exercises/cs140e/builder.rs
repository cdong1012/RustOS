// FIXME: Make me pass! Diff budget: 30 lines.

<<<<<<< HEAD
#[derive(Default)]
=======
// I AM NOT DONE

>>>>>>> skeleton/lab2
struct Builder {
    string: Option<String>,
    number: Option<usize>,
}
<<<<<<< HEAD
#[warn(non_camel_case_types)]
trait string<T> {
    fn string(mut self, string: T)->Self;
}
impl Builder {
    fn number(mut self, num : usize) -> Self {
        self.number = Some(num);
        self
    }
}

impl string<String> for Builder {
    fn string(mut self, string: String) -> Self{
        self.string = Some(string);
        self
    }
}
impl string<&str> for Builder {
    fn string(mut self, string: &str) -> Self {
        self.string = Some(string.to_string());
        self
    }
}
impl ToString for Builder {
    // Implement the trait
    fn to_string(&self) -> String{
        let mut result : String = "".to_owned();
        let both : bool;
        match (&self.string, &self.number) {
            (Some(string), Some(number)) => {
                result.push_str(&string[..]); result.push_str(" "); result.push_str(&number.to_string()[..]);},
            (None, Some(number))         => result.push_str(&number.to_string()[..]),
            (Some(string), None)         => result.push_str(&string[..]),
            (None, None)                 => result.push_str("")
        }
        result
    }
}
=======
>>>>>>> skeleton/lab2

// Do not modify this function.
#[test]
fn builder() {
    let empty = Builder::default().to_string();
    assert_eq!(empty, "");

    let just_str = Builder::default().string("hi").to_string();
    assert_eq!(just_str, "hi");

    let just_num = Builder::default().number(254).to_string();
    assert_eq!(just_num, "254");

    let a = Builder::default()
        .string("hello, world!")
        .number(200)
        .to_string();

    assert_eq!(a, "hello, world! 200");

    let b = Builder::default()
        .string("hello, world!")
        .number(200)
        .string("bye now!")
        .to_string();

    assert_eq!(b, "bye now! 200");

    let c = Builder::default()
        .string("heap!".to_owned())
        .to_string();

    assert_eq!(c, "heap!");
}
