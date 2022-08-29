extern crate num;
#[macro_use]
extern crate num_derive;

pub mod assemble;
pub mod function;
mod lang;
pub mod llvm;
mod llvmasm;

fn main() {
    llvm::main();
}
