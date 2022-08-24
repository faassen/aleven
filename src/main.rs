extern crate num;
#[macro_use]
extern crate num_derive;

mod reglang;
mod regllvm;
mod regmem;

fn main() {
    regllvm::main();
}
