use mir::ir::{BasicBlock, MirFunction, MirProgram, Statement, Terminator, Operand};
use std::collections::HashSet;

pub fn eliminate_dead_code_program(mut program: MirProgram) -> MirProgram {
    for func in program.functions.values_mut() {
        eliminate_dead_code_function(func);
    }
    program
}

pub fn eliminate_dead_code_function(func: &mut MirFunction) {
    // 1. Mark reachable blocks (simple BFS/DFS from block 0)
    let mut reachable = HashSet::new();
    let mut worklist = vec![0];
    
    while let Some(block_id) = worklist.pop() {
        if reachable.insert(block_id) {
            // Traverse successors
            if let Some(block) = func.basic_blocks.iter().find(|b| b.id == block_id) {
                match &block.terminator {
                    Terminator::Goto { target } => worklist.push(*target),
                    Terminator::If { then_target, else_target, .. } 
                    | Terminator::IfOk { then_target, else_target, .. } => {
                        worklist.push(*then_target);
                        worklist.push(*else_target);
                    }
                    Terminator::Return { .. } | Terminator::Unreachable => {}
                }
            }
        }
    }
    
    // 2. Remove unreachable blocks
    func.basic_blocks.retain(|b| reachable.contains(&b.id));
    
    // 3. (Future) Liveness analysis to remove unused local assignments
    // For now, removing unreachable blocks is a great start for DCE.
}
