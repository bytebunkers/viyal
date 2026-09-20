use mir::ir::{BasicBlock, MirFunction, Terminator};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Cfg {
    pub num_blocks: usize,
    pub preds: Vec<Vec<usize>>,
    pub succs: Vec<Vec<usize>>,
    /// Immediate dominator for each block. None if unreachable or entry block.
    pub idom: Vec<Option<usize>>,
    /// Children in the dominator tree
    pub dom_children: Vec<Vec<usize>>,
    /// Dominance frontier for each block
    pub dom_frontiers: Vec<HashSet<usize>>,
}

impl Cfg {
    pub fn new(func: &MirFunction) -> Self {
        let num_blocks = func.basic_blocks.len();
        let mut cfg = Self {
            num_blocks,
            preds: vec![Vec::new(); num_blocks],
            succs: vec![Vec::new(); num_blocks],
            idom: vec![None; num_blocks],
            dom_children: vec![Vec::new(); num_blocks],
            dom_frontiers: vec![HashSet::new(); num_blocks],
        };

        cfg.build_edges(func);
        
        if num_blocks > 0 {
            cfg.compute_dominators();
            cfg.compute_dominance_frontiers();
        }

        cfg
    }

    fn build_edges(&mut self, func: &MirFunction) {
        for block in &func.basic_blocks {
            let u = block.id;
            let mut add_edge = |v: usize| {
                if !self.succs[u].contains(&v) {
                    self.succs[u].push(v);
                }
                if !self.preds[v].contains(&u) {
                    self.preds[v].push(u);
                }
            };

            match &block.terminator {
                Terminator::Goto { target } => {
                    add_edge(*target);
                }
                Terminator::If { then_target, else_target, .. } => {
                    add_edge(*then_target);
                    add_edge(*else_target);
                }
                Terminator::IfOk { then_target, else_target, .. } => {
                    add_edge(*then_target);
                    add_edge(*else_target);
                }
                Terminator::Return { .. } | Terminator::Unreachable => {}
            }
        }
    }

    /// Computes the immediate dominator for each block using the
    /// Cooper, Harvey, Kennedy algorithm ("A Simple, Fast Dominance Algorithm").
    fn compute_dominators(&mut self) {
        let entry = 0;
        
        // Post-order traversal for fast convergence
        let mut post_order = Vec::new();
        let mut visited = vec![false; self.num_blocks];
        self.dfs_post_order(entry, &mut visited, &mut post_order);
        
        // Map block id to post-order index
        let mut post_order_idx = vec![0; self.num_blocks];
        for (idx, &node) in post_order.iter().enumerate() {
            post_order_idx[node] = idx;
        }

        self.idom[entry] = Some(entry);
        let mut changed = true;

        while changed {
            changed = false;
            
            // Iterate in reverse post-order (except entry)
            for &b in post_order.iter().rev() {
                if b == entry {
                    continue;
                }

                // Find a processed predecessor
                let mut new_idom = None;
                for &p in &self.preds[b] {
                    if self.idom[p].is_some() {
                        new_idom = Some(p);
                        break;
                    }
                }

                if let Some(mut new_idom_val) = new_idom {
                    for &p in &self.preds[b] {
                        if p != new_idom_val && self.idom[p].is_some() {
                            new_idom_val = self.intersect(p, new_idom_val, &post_order_idx);
                        }
                    }

                    if self.idom[b] != Some(new_idom_val) {
                        self.idom[b] = Some(new_idom_val);
                        changed = true;
                    }
                }
            }
        }

        // Build the dominator tree children
        for b in 1..self.num_blocks {
            if let Some(idom_b) = self.idom[b] {
                if idom_b != b {
                    self.dom_children[idom_b].push(b);
                }
            }
        }
    }

    fn intersect(&self, mut b1: usize, mut b2: usize, post_order_idx: &[usize]) -> usize {
        while b1 != b2 {
            while post_order_idx[b1] < post_order_idx[b2] {
                b1 = self.idom[b1].unwrap();
            }
            while post_order_idx[b2] < post_order_idx[b1] {
                b2 = self.idom[b2].unwrap();
            }
        }
        b1
    }

    fn dfs_post_order(&self, node: usize, visited: &mut [bool], post_order: &mut Vec<usize>) {
        visited[node] = true;
        for &succ in &self.succs[node] {
            if !visited[succ] {
                self.dfs_post_order(succ, visited, post_order);
            }
        }
        post_order.push(node);
    }

    /// Computes the Dominance Frontier for each block.
    fn compute_dominance_frontiers(&mut self) {
        for b in 0..self.num_blocks {
            if self.preds[b].len() >= 2 {
                for &p in &self.preds[b] {
                    let mut runner = p;
                    while runner != self.idom[b].unwrap() {
                        self.dom_frontiers[runner].insert(b);
                        runner = self.idom[runner].unwrap_or(runner);
                        if runner == self.idom[runner].unwrap_or(runner) {
                            // Hit root or disconnected
                            break; 
                        }
                    }
                }
            }
        }
    }
}
