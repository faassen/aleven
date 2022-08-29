extern crate num;
#[macro_use]
extern crate num_derive;

pub mod assemble;
mod lang;
pub mod llvm;
mod llvmasm;
pub mod program;

fn main() {
    llvm::main();
}
