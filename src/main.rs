extern crate num;
#[macro_use]
extern crate num_derive;

mod assemble;
mod reglang;
mod regllvm;

fn main() {
    regllvm::main();
}
