// move_semantics2.rs
// Make me compile without changing line 13!
// Execute `rustlings hint move_semantics2` for hints :)

fn main() { // second in hint
    let vec0 = Vec::new();

    let mut vec1 = fill_vec(&vec0);

    // Do not change the following line!
    println!("{} has length {} content `{:?}`", "vec0", vec0.len(), vec0);

    vec1.push(88);

    println!("{} has length {} content `{:?}`", "vec1", vec1.len(), vec1);
}

fn fill_vec(vec: &Vec<i32>) -> Vec<i32> {
    let mut vec2 = Vec::new();
    for x in vec {
        vec2.push(*x);
    }

    vec2.push(22);
    vec2.push(44);
    vec2.push(66);

    vec2
}
/*
fn main() { // third in hint, mutably borrow to push directly
    let mut vec0 = Vec::new();
    let vec = &mut vec0;
    fill_vec(vec);

    // Do not change the following line!
    println!("{} has length {} content `{:?}`", "vec0", vec0.len(), vec0);

    //vec1.push(88);

    //println!("{} has length {} content `{:?}`", "vec1", vec1.len(), vec1);
}

fn fill_vec(vec: &mut Vec<i32>) {
    vec.push(22);
    vec.push(44);
    vec.push(66);
} */