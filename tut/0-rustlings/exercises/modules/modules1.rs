// modules1.rs
// Make me compile! Execute `rustlings hint modules1` for hints :)


mod sausage_factory {
    pub(super) fn make_sausage() {
        println!("sausage!");
    }
}

fn main() {
    sausage_factory::make_sausage();
}
