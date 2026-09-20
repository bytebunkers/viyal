use mir::ir::{BasicBlock, Local, MirFunction, Operand, Rvalue, Statement};
use std::collections::HashMap;

/// Performs Global Value Numbering (GVN) / Common Subexpression Elimination (CSE).
/// Assumes the MIR is in SSA form.
pub fn run(mut func: MirFunction) -> MirFunction {
    if func.basic_blocks.is_empty() {
        return func;
    }

    // Mapping from a canonical string representation of an Rvalue to the local
    // that first computed this value.
    let mut value_table: HashMap<String, Local> = HashMap::new();

    // Mapping from a local to its value number (which is just the local index of the first computation)
    // Actually, because we just replace Rvalues with Rvalue::Use(Operand::Copy(first_local)),
    // we just need value_table.

    for block in &mut func.basic_blocks {
        for stmt in &mut block.statements {
            if let Statement::Assign(dest, rvalue) = stmt {
                // We only perform CSE on side-effect-free, deterministic Rvalues.
                if is_pure(rvalue) {
                    let key = canonicalize(rvalue);
                    if let Some(existing_local) = value_table.get(&key) {
                        // We found a redundant computation!
                        // Replace this Rvalue with a simple use of the existing local.
                        *rvalue = Rvalue::Use(Operand::Copy(*existing_local));
                    } else {
                        // First time seeing this computation
                        value_table.insert(key, *dest);
                    }
                }
            }
        }
    }

    func
}

/// Returns true if the Rvalue is deterministic and side-effect free.
fn is_pure(rvalue: &Rvalue) -> bool {
    match rvalue {
        Rvalue::Use(_) | Rvalue::Length(_) | Rvalue::BinaryOp(_, _, _) => true,
        Rvalue::Index(_, _) | Rvalue::PropertyAccess(_, _) => true, // Assuming no getters with side-effects for now
        // Function calls, method calls, array/map allocations, and try operations might have side effects
        // or return different objects, so they are not pure for GVN purposes.
        Rvalue::Call { .. } | Rvalue::MethodCall(..) | Rvalue::Array(_) | Rvalue::Map(_) | Rvalue::New(_, _) | Rvalue::Try(_) => false,
    }
}

/// Generates a canonical string representation of an Rvalue.
fn canonicalize(rvalue: &Rvalue) -> String {
    match rvalue {
        Rvalue::Use(op) => format!("Use({})", canonicalize_operand(op)),
        Rvalue::Length(op) => format!("Len({})", canonicalize_operand(op)),
        Rvalue::BinaryOp(bin_op, lhs, rhs) => {
            // For commutative operations, we could sort the operands to find more matches,
            // but for a simple MVP we just format them as-is.
            format!("{:?}({}, {})", bin_op, canonicalize_operand(lhs), canonicalize_operand(rhs))
        }
        Rvalue::Index(arr, idx) => format!("Index({}, {})", canonicalize_operand(arr), canonicalize_operand(idx)),
        Rvalue::PropertyAccess(obj, prop) => format!("Prop({}, {})", canonicalize_operand(obj), prop),
        _ => "".to_string(), // Unreachable because of is_pure check
    }
}

fn canonicalize_operand(op: &Operand) -> String {
    match op {
        Operand::Constant(lit) => format!("Const({:?})", lit),
        Operand::Copy(Local(l)) => format!("Loc({})", l),
    }
}
