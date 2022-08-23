extern crate num;
#[macro_use]
extern crate num_derive;

mod iw;
mod reglang;
mod regllvm;
mod regmem;

fn main() {
    regllvm::main();
}
