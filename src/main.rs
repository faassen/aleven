extern crate num;
#[macro_use]
extern crate num_derive;

pub mod assembler;
pub mod cache;
pub mod function;
pub mod lang;
pub mod llvm;
pub mod llvmasm;
pub mod opcodetype;
pub mod program;
pub mod run;
pub mod serializer;

fn main() {
    llvm::main();
}
