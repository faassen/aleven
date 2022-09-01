extern crate num;
#[macro_use]
extern crate num_derive;

mod assemble;
mod cache;
mod function;
mod lang;
mod llvm;
mod llvmasm;
mod parser;
mod program;
pub mod run;

pub use assemble::Serializer;
pub use cache::FunctionValueCache;
pub use function::Function;
pub use lang::{Branch, BranchTarget, CallId, Immediate, Instruction, Load, Register, Store};
pub use llvm::CodeGen;
pub use program::Program;
