extern crate num;
#[macro_use]
extern crate num_derive;

mod iw;
mod reglang;
mod regmem;

fn main() {
    println!("Hello, world!");
    iw::main();
}
