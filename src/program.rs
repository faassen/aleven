use inkwell::execution_engine::JitFunction;
use inkwell::values::FunctionValue;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::function::Function;
use crate::lang::{Instruction, Processor};
use crate::llvm::CodeGen;
use crate::llvm::ProgramFunc;
// use crate::CodeGen;

pub struct Program {
    functions: Vec<Function>,
}

impl Program {
    pub fn new(functions: &[&[Instruction]]) -> Program {
        Program {
            functions: functions
                .iter()
                .map(|instructions| Function::new(instructions))
                .collect(),
        }
    }

    pub fn from_instructions(instructions: &[Instruction]) -> Program {
        Program::new(&[instructions])
    }

    pub fn interpret(&self, memory: &mut [u8]) {
        let mut processor = Processor::new();
        self.interpret_with_processor(memory, &mut processor);
    }

    pub fn interpret_with_processor(&self, memory: &mut [u8], processor: &mut Processor) {
        self.call(memory, processor, 0);
    }

    pub fn call(&self, memory: &mut [u8], processor: &mut Processor, id: usize) {
        self.functions[id].interpret(memory, processor, &self.functions);
    }

    // pub fn compile_function(id: u16, function: &Function, codegen: &CodeGen) {
    //     function.compile(codegen, memory_len)
    // }

    pub fn compile<'ctx>(
        &'ctx self,
        codegen: &'ctx CodeGen,
        memory_size: u16,
    ) -> JitFunction<ProgramFunc> {
        let mut known = KnownFunctionValues::new();
        let dependency_map = known.compile(0, &self, codegen, memory_size);
        codegen.compile_program(&dependency_map).unwrap()
    }
}

type CallId = u16;
type FunctionValueId = usize;

struct KnownFunctionValues<'ctx> {
    function_values: FxHashMap<CallId, (FunctionValueId, FunctionValue<'ctx>)>,
    current_function_value_id: FunctionValueId,
}

impl<'ctx> KnownFunctionValues<'ctx> {
    pub fn new() -> KnownFunctionValues<'ctx> {
        KnownFunctionValues {
            function_values: FxHashMap::default(),
            current_function_value_id: 0,
        }
    }

    fn compile(
        &mut self,
        call_id: CallId,
        program: &Program,
        codegen: &'ctx CodeGen,
        memory_size: u16,
    ) -> FxHashMap<CallId, FunctionValue<'ctx>> {
        // given everything this function calls, compile dependencies
        let function = &program.functions[call_id as usize];
        let call_ids = function.get_call_ids();

        let mut result = FxHashMap::default();
        for dependency_call_id in call_ids {
            let dependency_map = self.compile(dependency_call_id, program, codegen, memory_size);
            result.extend(dependency_map);
        }
        // now we have the information required to compile this function
        let function_value = function.compile(
            self.current_function_value_id,
            codegen,
            memory_size,
            &result,
        );
        self.current_function_value_id += 1;
        result.insert(call_id, function_value);
        result
    }

    // fn get_function_value_id(&self, id: CallId) -> Option<FunctionValueId> {
    //     self.function_values.get(&id).map(|&(id, _)| id)
    // }

    // fn get(
    //     &self,
    //     id: CallId,
    //     function: &Function,
    //     codegen: &'ctx CodeGen,
    //     memory_len: u16,
    // ) -> Option<&(FunctionValueId, FunctionValue<'ctx>)> {
    //     self.function_values.get(&id)
    // }

    // pub fn get_function_value_ids(&self, call_ids: &[CallId]) -> FxHashSet<FunctionValueId> {
    //     call_ids
    //         .iter()
    //         .map(|id| self.get_function_value_id(*id).unwrap())
    //         .collect()
    // }
}

// fn break_cycles(functions: &[&[Instruction]], ancestors: &[u16]) -> FxHashMap<u16, Function> {
//     let mut result = FxHashMap::default();
//     for (id, instructions) in functions.iter().enumerate() {
//         let function = Function::new(instructions);
//         function.get_call_ids().iter().for_each(|&call_id| {
//             if ancestors.contains(&call_id) {
//                 result.insert(call_id, Function::new(instructions));
//             }
//         });
//         let mut ancestors = ancestors.to_vec();
//         ancestors.push(id as u16);
//         let new_functions = break_cycles(functions, &ancestors);
//         result.extend(new_functions.into_iter());
//         result.insert(id as u16, function);
//     }
//     result
// }
