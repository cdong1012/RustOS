#![feature(decl_macro)]
extern crate core;
extern crate std;
extern crate alloc;

#[cfg(not(target_endian = "little"))]
compile_error!("only little endian platforms supported");

pub fn ya() {
    println!("YEEEET");
}