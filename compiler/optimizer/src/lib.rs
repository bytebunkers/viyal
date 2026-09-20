pub mod cfg;
pub mod sccp;
pub mod gvn;
pub mod ssa;
pub mod de_ssa;
pub mod constant_folding;
pub mod dce;

use mir::ir::MirProgram;

pub fn optimize(mut program: MirProgram) -> MirProgram {
    let mut optimized_funcs = std::collections::HashMap::new();

    for (name, func) in program.functions {
        // 1. Construct SSA Form
        let mut opt_func = ssa::construct_ssa(func);

        // 2. Perform deep optimizations on SSA form
        opt_func = sccp::run(opt_func);
        opt_func = gvn::run(opt_func);
        
        // 3. Eliminate Phi nodes (De-SSA)
        opt_func = de_ssa::eliminate_phis(opt_func);

        optimized_funcs.insert(name, opt_func);
    }
    
    program.functions = optimized_funcs;
    
    // 4. Dead code elimination (Runs on MirProgram to clean up unreachable blocks post-SCCP)
    program = dce::eliminate_dead_code_program(program);
    
    program
}
