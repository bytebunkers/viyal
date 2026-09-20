use crate::cfg::Cfg;
use mir::ir::{BasicBlock, Local, LocalDecl, MirFunction, Operand, Phi, Rvalue, Statement, Terminator};
use std::collections::{HashMap, HashSet};

pub fn construct_ssa(mut func: MirFunction) -> MirFunction {
    if func.basic_blocks.is_empty() {
        return func;
    }

    let cfg = Cfg::new(&func);
    
    // 1. Find all blocks where each local is assigned.
    let mut defs_per_local: HashMap<usize, HashSet<usize>> = HashMap::new();
    for block in &func.basic_blocks {
        for stmt in &block.statements {
            if let Statement::Assign(Local(loc_idx), _) = stmt {
                defs_per_local.entry(*loc_idx).or_default().insert(block.id);
            }
        }
    }

    // 2. Insert Phi nodes based on Iterated Dominance Frontiers
    let mut phi_placements: HashMap<usize, HashSet<usize>> = HashMap::new(); // local -> block ids
    
    for (loc_idx, def_blocks) in &defs_per_local {
        let mut worklist: Vec<usize> = def_blocks.iter().copied().collect();
        let mut in_worklist: HashSet<usize> = worklist.iter().copied().collect();
        let mut has_phi = HashSet::new();

        while let Some(b) = worklist.pop() {
            in_worklist.remove(&b);
            
            for &df_block in &cfg.dom_frontiers[b] {
                if !has_phi.contains(&df_block) {
                    has_phi.insert(df_block);
                    phi_placements.entry(*loc_idx).or_default().insert(df_block);
                    
                    if !in_worklist.contains(&df_block) {
                        in_worklist.insert(df_block);
                        worklist.push(df_block);
                    }
                }
            }
        }
    }

    // Actually insert the Phi nodes with dummy operands
    for (loc_idx, blocks) in phi_placements {
        for block_id in blocks {
            let phi = Phi {
                dest: Local(loc_idx), // Will be renamed
                orig_local: Local(loc_idx),
                operands: Vec::new(), // Will be filled during renaming
            };
            func.basic_blocks[block_id].phis.push(phi);
        }
    }

    // 3. Rename variables
    let mut renamer = SsaRenamer::new(&mut func);
    renamer.rename_block(0, &cfg);
    
    func
}

struct SsaRenamer<'a> {
    func: &'a mut MirFunction,
    /// Stack of active versions for each original local
    versions: HashMap<usize, Vec<usize>>,
    /// Number of versions generated for each original local
    counters: HashMap<usize, usize>,
}

impl<'a> SsaRenamer<'a> {
    fn new(func: &'a mut MirFunction) -> Self {
        let mut versions = HashMap::new();
        let mut counters = HashMap::new();
        
        // Push the initial versions (0) for all locals (especially parameters)
        for i in 0..func.locals.len() {
            versions.insert(i, vec![i]);
            counters.insert(i, 1); // 1 version exists (the original)
        }
        
        Self {
            func,
            versions,
            counters,
        }
    }

    fn new_version(&mut self, orig: usize) -> usize {
        let count = *self.counters.get(&orig).unwrap();
        self.counters.insert(orig, count + 1);
        
        // Create the new LocalDecl based on the original
        let orig_decl = self.func.locals[orig].clone();
        let new_idx = self.func.locals.len();
        self.func.locals.push(LocalDecl {
            ty: orig_decl.ty,
            name: orig_decl.name.map(|n| format!("{}_{}", n, count)),
            is_mut: false, // SSA variables are immutable!
        });
        
        self.versions.get_mut(&orig).unwrap().push(new_idx);
        new_idx
    }

    fn get_current_version(&self, orig: usize) -> usize {
        *self.versions.get(&orig).unwrap().last().unwrap_or(&orig)
    }

    fn rename_block(&mut self, block_id: usize, cfg: &Cfg) {
        let mut pushes = HashMap::new(); // orig_local -> number of versions pushed in this block

        let mut phis = std::mem::take(&mut self.func.basic_blocks[block_id].phis);
        let mut statements = std::mem::take(&mut self.func.basic_blocks[block_id].statements);
        let mut terminator = std::mem::replace(&mut self.func.basic_blocks[block_id].terminator, Terminator::Unreachable);

        // 1. Rename Phi destinations
        for phi in &mut phis {
            let orig = phi.dest.0;
            let new_version = self.new_version(orig);
            phi.dest = Local(new_version);
            *pushes.entry(orig).or_insert(0) += 1;
        }

        // 2. Rename regular statements
        for stmt in &mut statements {
            match stmt {
                Statement::Assign(dest, rvalue) => {
                    self.rename_rvalue(rvalue);
                    let orig = dest.0;
                    let new_version = self.new_version(orig);
                    *dest = Local(new_version);
                    *pushes.entry(orig).or_insert(0) += 1;
                }
            }
        }
        
        // Rename terminator
        self.rename_terminator(&mut terminator);

        // Put them back
        self.func.basic_blocks[block_id].phis = phis;
        self.func.basic_blocks[block_id].statements = statements;
        self.func.basic_blocks[block_id].terminator = terminator;

        // 3. Fill Phi operands in successors
        let succs = cfg.succs[block_id].clone();
        for succ_id in succs {
            let mut succ_phis = std::mem::take(&mut self.func.basic_blocks[succ_id].phis);
            for phi in &mut succ_phis {
                let orig = phi.orig_local.0;
                let current_version = self.get_current_version(orig);
                phi.operands.push((Operand::Copy(Local(current_version)), block_id));
            }
            self.func.basic_blocks[succ_id].phis = succ_phis;
        }

        // 4. Visit dominator tree children
        let children = cfg.dom_children[block_id].clone();
        for child in children {
            self.rename_block(child, cfg);
        }

        // 5. Pop versions pushed in this block
        for (orig, count) in pushes {
            let stack = self.versions.get_mut(&orig).unwrap();
            for _ in 0..count {
                stack.pop();
            }
        }
    }

    fn rename_rvalue(&self, rvalue: &mut Rvalue) {
        match rvalue {
            Rvalue::Use(op) | Rvalue::Length(op) | Rvalue::Try(op) | Rvalue::PropertyAccess(op, _) => self.rename_operand(op),
            Rvalue::BinaryOp(_, op1, op2) | Rvalue::Index(op1, op2) => {
                self.rename_operand(op1);
                self.rename_operand(op2);
            }
            Rvalue::Call { func, args } => {
                self.rename_operand(func);
                for arg in args {
                    self.rename_operand(arg);
                }
            }
            Rvalue::MethodCall(obj, _, args) => {
                self.rename_operand(obj);
                for arg in args {
                    self.rename_operand(arg);
                }
            }
            Rvalue::Array(ops) => {
                for op in ops {
                    self.rename_operand(op);
                }
            }
            Rvalue::Map(pairs) => {
                for (k, v) in pairs {
                    self.rename_operand(k);
                    self.rename_operand(v);
                }
            }
            Rvalue::New(_, args) => {
                for arg in args {
                    self.rename_operand(arg);
                }
            }
        }
    }

    fn rename_operand(&self, op: &mut Operand) {
        if let Operand::Copy(local) = op {
            local.0 = self.get_current_version(local.0);
        }
    }

    fn rename_terminator(&self, term: &mut Terminator) {
        match term {
            Terminator::If { cond, .. } => self.rename_operand(cond),
            Terminator::IfOk { val, .. } => self.rename_operand(val),
            Terminator::Return { value: Some(val) } => self.rename_operand(val),
            _ => {}
        }
    }
}
