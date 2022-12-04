use std::collections::HashMap;
use nexsys_math::Variable;

use crate::Equation;

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
/// A block manager object for identifying constrained systems of equations.
pub struct BlockMgr<'a> {
    /// blocks i, j
    /// 
    /// i: the number of unknowns in the equations
    /// 
    /// j: the Vec<String> of unknowns common to those equations
    blocks: Vec<HashMap<Vec<String>, Vec<String>>>,
    ctx: &'a HashMap<String, Variable>
}
impl <'a> BlockMgr<'a> {
    /// Initializes a new BlockMgr object.
    pub fn new(ctx: &'a HashMap<String, Variable>) -> BlockMgr<'a> {
        BlockMgr { blocks: vec![], ctx }
    }

    /// Adds an equation to the BlockMgr, classifying it by number of unknowns and common unknowns.
    pub fn add_item(&mut self, expr: &Equation) {
        let n = expr.n_unknowns(&self.ctx);
        let uks = expr.unknowns(&self.ctx);

        if n < 1 {
            return; // do nothing if there are fewer than 2 unknowns in the equation
        }

        // Add slots to accommodate 
        if self.blocks.len() == 0 {
            self.blocks = Vec::with_capacity(n);
        }

        while self.blocks.len() < n {
            self.blocks.push(HashMap::new())
        }
        
        // Find the slot that the eqn belongs in
        if let Some(v) = self.blocks[n-1].get_mut(&uks) {
            v.push(expr.as_expr());
        } else {
            self.blocks[n-1].insert(uks, vec![expr.as_expr()]);
        }

    }

    /// Returns properly constrained systems of equations or returns `None` if none exist in the system.
    pub fn constrained(mut self) -> Option<Vec<(Vec<String>, Vec<String>)>> {

        let mut eqns = vec![];

        // Identify constrained blocks of equations
        for i in 0..self.blocks.len() {
            for j in self.blocks[i].drain() {
                if j.1.len() == i + 1 {
                    eqns.push((j.0, j.1));
                }
            }
        }
        if eqns.len() == 0 {
            None
        } else {
            Some(eqns)
        }
    }
}