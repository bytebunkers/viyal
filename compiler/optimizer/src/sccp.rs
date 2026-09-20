use mir::ir::{BasicBlock, Local, MirFunction, Operand, Rvalue, Statement, Terminator};
use ast::{BinaryOp, Literal};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
enum Lattice {
    Bottom, // Uninitialized
    Constant(Literal),
    Top,    // Varying (cannot be a constant)
}

pub fn run(mut func: MirFunction) -> MirFunction {
    if func.basic_blocks.is_empty() {
        return func;
    }

    let mut values = vec![Lattice::Bottom; func.locals.len()];
    let mut executable_edges = HashSet::new(); // (from_block, to_block)
    let mut executable_blocks = HashSet::new(); // Blocks that are reachable
    
    let mut cfg_worklist = vec![(0, 0)]; // Fake edge to start block
    let mut ssa_worklist = Vec::new(); // Locals whose lattice value changed

    // Build def-use chains
    let mut uses_of_local: HashMap<usize, Vec<usize>> = HashMap::new(); // local -> blocks where it's used
    for block in &func.basic_blocks {
        for phi in &block.phis {
            for (op, _) in &phi.operands {
                if let Operand::Copy(Local(l)) = op {
                    uses_of_local.entry(*l).or_default().push(block.id);
                }
            }
        }
        for stmt in &block.statements {
            match stmt {
                Statement::Assign(_, rvalue) => {
                    extract_uses(rvalue, &mut uses_of_local, block.id);
                }
            }
        }
        match &block.terminator {
            Terminator::If { cond, .. } => {
                if let Operand::Copy(Local(l)) = cond {
                    uses_of_local.entry(*l).or_default().push(block.id);
                }
            }
            Terminator::IfOk { val, .. } => {
                if let Operand::Copy(Local(l)) = val {
                    uses_of_local.entry(*l).or_default().push(block.id);
                }
            }
            Terminator::Return { value: Some(val) } => {
                if let Operand::Copy(Local(l)) = val {
                    uses_of_local.entry(*l).or_default().push(block.id);
                }
            }
            _ => {}
        }
    }

    // Since we don't have arguments explicit in MirFunction easily available with their indices here without AST context,
    // we assume parameters and anything we don't understand is Top.
    // For now, any local that is never defined in the MIR (like parameters) is Top.
    let mut defined = vec![false; func.locals.len()];
    for block in &func.basic_blocks {
        for phi in &block.phis {
            defined[phi.dest.0] = true;
        }
        for stmt in &block.statements {
            if let Statement::Assign(Local(l), _) = stmt {
                defined[*l] = true;
            }
        }
    }
    for i in 0..func.locals.len() {
        if !defined[i] {
            values[i] = Lattice::Top;
        }
    }

    while let Some((pred, curr)) = cfg_worklist.pop() {
        if !executable_edges.contains(&(pred, curr)) {
            executable_edges.insert((pred, curr));
            
            let first_visit = !executable_blocks.contains(&curr);
            executable_blocks.insert(curr);

            let block = &func.basic_blocks[curr];

            // Evaluate Phis
            for phi in &block.phis {
                evaluate_phi(phi, &executable_edges, curr, &mut values, &mut ssa_worklist);
            }

            if first_visit {
                // Evaluate statements
                for stmt in &block.statements {
                    if let Statement::Assign(Local(dest), rvalue) = stmt {
                        evaluate_rvalue(rvalue, *dest, &mut values, &mut ssa_worklist);
                    }
                }

                // Evaluate terminator
                match &block.terminator {
                    Terminator::Goto { target } => {
                        cfg_worklist.push((curr, *target));
                    }
                    Terminator::If { cond, then_target, else_target } => {
                        match evaluate_operand(cond, &values) {
                            Lattice::Constant(Literal::Boolean(true)) => cfg_worklist.push((curr, *then_target)),
                            Lattice::Constant(Literal::Boolean(false)) => cfg_worklist.push((curr, *else_target)),
                            _ => {
                                cfg_worklist.push((curr, *then_target));
                                cfg_worklist.push((curr, *else_target));
                            }
                        }
                    }
                    Terminator::IfOk { then_target, else_target, .. } => {
                        // Can't easily statically know if Ok/Err, mark both reachable
                        cfg_worklist.push((curr, *then_target));
                        cfg_worklist.push((curr, *else_target));
                    }
                    _ => {}
                }
            }
        }

        // Process SSA worklist
        while let Some(local_idx) = ssa_worklist.pop() {
            if let Some(blocks) = uses_of_local.get(&local_idx) {
                for &b in blocks {
                    if executable_blocks.contains(&b) {
                        let block = &func.basic_blocks[b];
                        // Re-evaluate phis and statements that use this local
                        for phi in &block.phis {
                            if phi.operands.iter().any(|(op, _)| matches!(op, Operand::Copy(Local(l)) if *l == local_idx)) {
                                evaluate_phi(phi, &executable_edges, b, &mut values, &mut ssa_worklist);
                            }
                        }
                        for stmt in &block.statements {
                            if let Statement::Assign(Local(dest), rvalue) = stmt {
                                if uses_local(rvalue, local_idx) {
                                    evaluate_rvalue(rvalue, *dest, &mut values, &mut ssa_worklist);
                                }
                            }
                        }
                        
                        // Re-evaluate terminator if it's an IF and we are here
                        if let Terminator::If { cond, then_target, else_target } = &block.terminator {
                            if matches!(cond, Operand::Copy(Local(l)) if *l == local_idx) {
                                match evaluate_operand(cond, &values) {
                                    Lattice::Constant(Literal::Boolean(true)) => cfg_worklist.push((b, *then_target)),
                                    Lattice::Constant(Literal::Boolean(false)) => cfg_worklist.push((b, *else_target)),
                                    _ => {
                                        cfg_worklist.push((b, *then_target));
                                        cfg_worklist.push((b, *else_target));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Now rewrite the MIR based on the lattice values and reachability
    for block in &mut func.basic_blocks {
        if !executable_blocks.contains(&block.id) {
            // Block is unreachable
            block.statements.clear();
            block.phis.clear();
            block.terminator = Terminator::Unreachable;
            continue;
        }

        // Replace uses with constants
        for stmt in &mut block.statements {
            if let Statement::Assign(_, rvalue) = stmt {
                replace_constants(rvalue, &values);
            }
        }

        // Fix terminators
        let mut new_terminator = None;
        if let Terminator::If { cond, then_target, else_target } = &mut block.terminator {
            if let Operand::Copy(Local(l)) = cond {
                if let Lattice::Constant(Literal::Boolean(val)) = values[*l] {
                    if val {
                        new_terminator = Some(Terminator::Goto { target: *then_target });
                    } else {
                        new_terminator = Some(Terminator::Goto { target: *else_target });
                    }
                }
            }
        }
        if let Some(term) = new_terminator {
            block.terminator = term;
        }
    }

    func
}

fn evaluate_phi(phi: &mir::ir::Phi, exec_edges: &HashSet<(usize, usize)>, curr_block: usize, values: &mut Vec<Lattice>, ssa_worklist: &mut Vec<usize>) {
    let mut new_val = Lattice::Bottom;
    for (op, pred) in &phi.operands {
        if exec_edges.contains(&(*pred, curr_block)) {
            let op_val = evaluate_operand(op, values);
            new_val = meet(new_val, op_val);
        }
    }
    
    if values[phi.dest.0] != new_val {
        values[phi.dest.0] = new_val;
        ssa_worklist.push(phi.dest.0);
    }
}

fn meet(a: Lattice, b: Lattice) -> Lattice {
    match (a, b) {
        (Lattice::Bottom, x) | (x, Lattice::Bottom) => x,
        (Lattice::Constant(c1), Lattice::Constant(c2)) if c1 == c2 => Lattice::Constant(c1),
        _ => Lattice::Top,
    }
}

fn evaluate_operand(op: &Operand, values: &[Lattice]) -> Lattice {
    match op {
        Operand::Constant(lit) => Lattice::Constant(lit.clone()),
        Operand::Copy(Local(l)) => values[*l].clone(),
    }
}

fn evaluate_rvalue(rvalue: &Rvalue, dest: usize, values: &mut Vec<Lattice>, ssa_worklist: &mut Vec<usize>) {
    let new_val = match rvalue {
        Rvalue::Use(op) => evaluate_operand(op, values),
        Rvalue::BinaryOp(op_type, lhs, rhs) => {
            let l_val = evaluate_operand(lhs, values);
            let r_val = evaluate_operand(rhs, values);
            match (l_val, r_val) {
                (Lattice::Constant(Literal::Integer(l)), Lattice::Constant(Literal::Integer(r))) => {
                    match op_type {
                        BinaryOp::Add => Lattice::Constant(Literal::Integer(l.wrapping_add(r))),
                        BinaryOp::Sub => Lattice::Constant(Literal::Integer(l.wrapping_sub(r))),
                        BinaryOp::Mul => Lattice::Constant(Literal::Integer(l.wrapping_mul(r))),
                        BinaryOp::Div if r != 0 => Lattice::Constant(Literal::Integer(l / r)),
                        BinaryOp::Eq => Lattice::Constant(Literal::Boolean(l == r)),
                        BinaryOp::NotEq => Lattice::Constant(Literal::Boolean(l != r)),
                        BinaryOp::Less => Lattice::Constant(Literal::Boolean(l < r)),
                        BinaryOp::LessEq => Lattice::Constant(Literal::Boolean(l <= r)),
                        BinaryOp::Greater => Lattice::Constant(Literal::Boolean(l > r)),
                        BinaryOp::GreaterEq => Lattice::Constant(Literal::Boolean(l >= r)),
                        _ => Lattice::Top,
                    }
                }
                (Lattice::Bottom, _) | (_, Lattice::Bottom) => Lattice::Bottom,
                _ => Lattice::Top,
            }
        }
        _ => Lattice::Top,
    };

    if values[dest] != new_val {
        values[dest] = new_val;
        ssa_worklist.push(dest);
    }
}

fn extract_uses(rvalue: &Rvalue, uses: &mut HashMap<usize, Vec<usize>>, block_id: usize) {
    let mut add_use = |op: &Operand| {
        if let Operand::Copy(Local(l)) = op {
            uses.entry(*l).or_default().push(block_id);
        }
    };
    match rvalue {
        Rvalue::Use(op) | Rvalue::Length(op) | Rvalue::Try(op) | Rvalue::PropertyAccess(op, _) => add_use(op),
        Rvalue::BinaryOp(_, op1, op2) | Rvalue::Index(op1, op2) => {
            add_use(op1);
            add_use(op2);
        }
        Rvalue::Call { func, args } => {
            add_use(func);
            for arg in args { add_use(arg); }
        }
        Rvalue::MethodCall(obj, _, args) => {
            add_use(obj);
            for arg in args { add_use(arg); }
        }
        Rvalue::Array(ops) => {
            for op in ops { add_use(op); }
        }
        Rvalue::Map(pairs) => {
            for (k, v) in pairs {
                add_use(k);
                add_use(v);
            }
        }
        Rvalue::New(_, args) => {
            for arg in args { add_use(arg); }
        }
    }
}

fn uses_local(rvalue: &Rvalue, local: usize) -> bool {
    let mut uses = false;
    let mut check_use = |op: &Operand| {
        if let Operand::Copy(Local(l)) = op {
            if *l == local { uses = true; }
        }
    };
    match rvalue {
        Rvalue::Use(op) | Rvalue::Length(op) | Rvalue::Try(op) | Rvalue::PropertyAccess(op, _) => check_use(op),
        Rvalue::BinaryOp(_, op1, op2) | Rvalue::Index(op1, op2) => {
            check_use(op1);
            check_use(op2);
        }
        Rvalue::Call { func, args } => {
            check_use(func);
            for arg in args { check_use(arg); }
        }
        Rvalue::MethodCall(obj, _, args) => {
            check_use(obj);
            for arg in args { check_use(arg); }
        }
        Rvalue::Array(ops) => {
            for op in ops { check_use(op); }
        }
        Rvalue::Map(pairs) => {
            for (k, v) in pairs {
                check_use(k);
                check_use(v);
            }
        }
        Rvalue::New(_, args) => {
            for arg in args { check_use(arg); }
        }
    }
    uses
}

fn replace_constants(rvalue: &mut Rvalue, values: &[Lattice]) {
    let mut replace = |op: &mut Operand| {
        if let Operand::Copy(Local(l)) = op {
            if let Lattice::Constant(lit) = &values[*l] {
                *op = Operand::Constant(lit.clone());
            }
        }
    };
    match rvalue {
        Rvalue::Use(op) | Rvalue::Length(op) | Rvalue::Try(op) | Rvalue::PropertyAccess(op, _) => replace(op),
        Rvalue::BinaryOp(_, op1, op2) | Rvalue::Index(op1, op2) => {
            replace(op1);
            replace(op2);
        }
        Rvalue::Call { func, args } => {
            replace(func);
            for arg in args { replace(arg); }
        }
        Rvalue::MethodCall(obj, _, args) => {
            replace(obj);
            for arg in args { replace(arg); }
        }
        Rvalue::Array(ops) => {
            for op in ops { replace(op); }
        }
        Rvalue::Map(pairs) => {
            for (k, v) in pairs {
                replace(k);
                replace(v);
            }
        }
        Rvalue::New(_, args) => {
            for arg in args { replace(arg); }
        }
    }
}
