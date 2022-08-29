extern crate num;
#[macro_use]
extern crate num_derive;

mod assemble;
mod function;
mod lang;
mod llvm;
mod llvmasm;

pub use assemble::Assembler;
pub use function::Function;
pub use lang::{Branch, BranchTarget, Immediate, Instruction, Load, Register, Store};
pub use llvm::CodeGen;
