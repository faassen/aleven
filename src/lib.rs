extern crate num;
#[macro_use]
extern crate num_derive;

mod assembler;
mod cache;
mod disassembler;
mod function;
mod lang;
mod llvm;
mod llvmasm;
mod program;
pub mod run;
mod serializer;

pub use assembler::{parse, parse_program};
pub use cache::FunctionValueCache;
pub use disassembler::disassemble;
pub use function::Function;
pub use llvm::CodeGen;
pub use program::Program;
pub use serializer::Serializer;
