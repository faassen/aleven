use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::OptimizationLevel;

pub fn save_asm(module: &Module) {
    Target::initialize_native(&InitializationConfig::default())
        .expect("Failed to initialize native target");

    let triple = TargetMachine::get_default_triple();
    let cpu = TargetMachine::get_host_cpu_name().to_string();
    let features = TargetMachine::get_host_cpu_features().to_string();

    // let pass_manager_builder = PassManagerBuilder::create();
    // pass_manager_builder.set_optimization_level(OptimizationLevel::Aggressive);

    // let pass_manager = PassManager::create(module);

    // pass_manager_builder.populate_function_pass_manager(&pass_manager);
    // pass_manager.add_demote_memory_to_register_pass();

    let target = Target::from_triple(&triple).unwrap();
    let machine = target
        .create_target_machine(
            &triple,
            &cpu,
            &features,
            OptimizationLevel::Aggressive,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();
    // machine.add_analysis_passes(&pass_manager);

    machine
        .write_to_file(module, FileType::Assembly, "out.asm".as_ref())
        .unwrap();
}
