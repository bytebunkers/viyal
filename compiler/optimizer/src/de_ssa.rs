use mir::ir::{BasicBlock, Local, LocalDecl, MirFunction, Operand, Rvalue, Statement, Terminator};
use ast::Type;
use std::collections::HashMap;

/// Eliminates Phi nodes by inserting Move operations at the end of predecessor blocks.
/// Uses a temporary variable approach to correctly handle parallel copy semantics
/// (solving the Swap and Lost Copy problems).
pub fn eliminate_phis(mut func: MirFunction) -> MirFunction {
    // Collect all phi operations to insert per predecessor block
    // pred_block_id -> Vec<(dest_local, src_operand)>
    let mut copies_to_insert: HashMap<usize, Vec<(Local, Operand)>> = HashMap::new();

    for block in &mut func.basic_blocks {
        let phis = std::mem::take(&mut block.phis);
        for phi in phis {
            for (operand, pred_id) in phi.operands {
                copies_to_insert.entry(pred_id).or_default().push((phi.dest, operand));
            }
        }
    }

    // Now insert the copies into the predecessor blocks
    for (pred_id, copies) in copies_to_insert {
        let mut temp_assigns = Vec::new();
        let mut final_assigns = Vec::new();

        for (dest, src) in copies {
            // Create a temporary variable for the source
            let temp_local_idx = func.locals.len();
            func.locals.push(LocalDecl {
                ty: Type::Named("Any".to_string(), Vec::new()), // We don't have exact type readily available, but it doesn't matter for MIR/Bytecode
                name: Some(format!("_phi_tmp_{}", temp_local_idx)),
                is_mut: false,
            });
            let temp_local = Local(temp_local_idx);

            // temp = src
            temp_assigns.push(Statement::Assign(temp_local, Rvalue::Use(src)));
            // dest = temp
            final_assigns.push(Statement::Assign(dest, Rvalue::Use(Operand::Copy(temp_local))));
        }

        // We insert these statements just before the terminator.
        let block = &mut func.basic_blocks[pred_id];
        
        // Append all temp assigns, then all final assigns to emulate parallel execution
        block.statements.extend(temp_assigns);
        block.statements.extend(final_assigns);
    }

    func
}
