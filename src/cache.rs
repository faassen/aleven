use crate::lang::Instruction;
use crate::llvm::CodeGen;
use crate::program::Program;
use inkwell::values::FunctionValue;
use rustc_hash::FxHashMap;

type CallId = u16;
type FunctionValueId = usize;

type CacheKey<'ctx> = (u8, &'ctx [Instruction], Vec<FunctionValueId>);
type CacheValue<'ctx> = (FunctionValueId, FunctionValue<'ctx>);

pub struct FunctionValueCache<'ctx> {
    cache: FxHashMap<CacheKey<'ctx>, CacheValue<'ctx>>,
    current_function_value_id: FunctionValueId,
}

impl<'ctx> FunctionValueCache<'ctx> {
    pub fn new() -> FunctionValueCache<'ctx> {
        FunctionValueCache {
            cache: FxHashMap::default(),
            current_function_value_id: 0,
        }
    }

    pub fn compile(
        &mut self,
        call_id: CallId,
        program: &'ctx Program,
        codegen: &'ctx CodeGen,
        memory_size: u16,
    ) -> FxHashMap<CallId, FunctionValue<'ctx>> {
        FunctionValueCache::convert_dependencies(&self.compile_internal(
            call_id,
            program,
            codegen,
            memory_size,
        ))
    }

    fn compile_internal(
        &mut self,
        call_id: CallId,
        program: &'ctx Program,
        codegen: &'ctx CodeGen,
        memory_size: u16,
    ) -> FxHashMap<CallId, (FunctionValueId, FunctionValue<'ctx>)> {
        // given everything this function calls, compile dependencies
        let function = &program.get_function(call_id);
        let call_ids = function.get_call_id_set();

        let mut result = FxHashMap::default();
        for dependency_call_id in &call_ids {
            let dependency_map =
                self.compile_internal(*dependency_call_id, program, codegen, memory_size);
            result.extend(dependency_map);
        }

        // get function value ids for dependencies
        let function_value_ids: Vec<FunctionValueId> =
            call_ids.iter().map(|call_id| result[call_id].0).collect();

        let cache_key = (
            function.get_repeat(),
            function.get_instructions(),
            function_value_ids,
        );

        let entry = self.cache.get(&cache_key);
        let to_insert = if let Some(entry) = entry {
            // nice, a cache hit
            *entry
        } else {
            // now we have the information required to compile this function
            let function_value = function.compile(
                self.current_function_value_id,
                codegen,
                memory_size,
                &FunctionValueCache::convert_dependencies(&result),
            );
            let entry = (self.current_function_value_id, function_value);
            self.cache.insert(cache_key, entry);
            entry
        };
        result.insert(call_id, to_insert);
        self.current_function_value_id += 1;
        result
    }

    fn convert_dependencies(
        m: &FxHashMap<CallId, (FunctionValueId, FunctionValue<'ctx>)>,
    ) -> FxHashMap<CallId, FunctionValue<'ctx>> {
        m.iter().map(|(k, v)| (*k, v.1)).collect()
    }
}

impl<'ctx> Default for FunctionValueCache<'ctx> {
    fn default() -> Self {
        Self::new()
    }
}
