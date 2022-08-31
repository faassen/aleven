extern crate num;
#[macro_use]
extern crate num_derive;

pub mod assemble;
pub mod cache;
pub mod function;
pub mod lang;
pub mod llvm;
pub mod llvmasm;
pub mod parser;
pub mod program;
pub mod run;

fn main() {
    llvm::main();
}
