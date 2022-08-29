extern crate num;
#[macro_use]
extern crate num_derive;

pub mod assemble;
pub mod function;
pub mod lang;
pub mod llvm;
pub mod llvmasm;
pub mod program;

fn main() {
    llvm::main();
}
