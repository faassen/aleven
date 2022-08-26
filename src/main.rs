extern crate num;
#[macro_use]
extern crate num_derive;

mod assemble;
mod lang;
mod regllvm;

fn main() {
    regllvm::main();
}
