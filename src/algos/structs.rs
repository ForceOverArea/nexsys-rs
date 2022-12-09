use std::collections::HashMap;
use crate::parsing::legal_variable;

/// Effectively an `f64`, but with an optional domain that the value must be on.
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct Variable {
    value: f64,
    domain: Option<[f64; 2]>
}
impl Variable {
    /// Instantiates a new `Variable` struct with a specified value and domain.
    pub fn new(value: f64, domain:Option<[f64; 2]>) -> Variable {
        Variable {
            value, 
            domain,
        }
    }

    /// Allows the ability to mutate `self.value` if the new value is on `self.domain`.
    pub fn change(&mut self, qty: f64) {
        match self.domain {
            Some(bounds) => {
                if bounds[0] < qty && qty < bounds[1] {
                    self.value = qty;
                } else if bounds[0] > qty { // if qty is o.o.b., then move self.value to the bound 
                    self.value = bounds[0];
                } else {
                    self.value = bounds[1];
                }
            }
            None => {
                // This comment is here exclusively to commemorate the STUPIDEST bug I have ever written:
                // self.value += qty; <- note how the variable's value is increased instead of changed
                //            ~~         if no domain is specified. 
                self.value = qty;
            }
        }
    }

    /// Mutates the domain of a variable. 
    pub fn change_domain(&mut self, dmn: Option<[f64; 2]>) {
        self.domain = dmn;
    }

    /// Allows the ability to mutate `self.value` by adding `qty` to it if the sum of `self.value` and `qty` is on `self.domain`.
    pub fn step(&mut self, qty: f64) {
        match self.domain {
            Some(bounds) => {
                if bounds[0] < self.value + qty && self.value + qty < bounds[1] {
                    self.value += qty;
                } else if bounds[0] > self.value + qty { // if qty is o.o.b., then move self.value to the bound 
                    self.value = bounds[0];
                } else {
                    self.value = bounds[1];
                }
            }
            None => {
                self.value += qty; // IT'S. THIS. LINE. EVERY. GODDAMN. TIME.
            }
        }
    }

    /// Returns `self.value` as `f64`.
    pub fn as_f64(&self) -> f64 {
        self.value
    }

    pub fn get_domain(&self) -> Option<[f64; 2]> {
        self.domain
    }
}

/// Represents an equation and gives info about its known and unknown variables
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct Equation {
    text: String,
    vars: Vec<String>,
    n: usize
}
impl Equation {
    /// Initializes a new `Equation` struct
    pub fn new(text: &str) -> Equation {
        let mut vars = legal_variable(text);

        vars.sort();

        let n = vars.len();

        Equation { text: text.to_string(), vars, n }
    }

    /// Returns the equation as an expression that evaluates to 0 when the system is solved.
    pub fn as_expr(&self) -> String {
        let terms = self.text.split("=").collect::<Vec<&str>>();
        format!("{} - ({})", terms[0], terms[1])
    }

    /// Returns the equation as a `&str`.
    pub fn as_text(&self) -> String {
        self.text.clone()
    }

    /// Returns a list of variables used in the equation
    pub fn vars(&self) -> Vec<String> {
        self.vars.clone()
    }

    /// Returns the number of unknown variables in the equation.
    pub fn n_unknowns(&self, ctx: &HashMap<String, Variable>) -> usize {
        self.n - self.vars.iter().filter(
            |&i| ctx.contains_key(i)
        ).collect::<Vec<&String>>().len()
    }

    /// Returns a `Vec` containing the variables that are unknowns in the equation.
    pub fn unknowns(&self, ctx: &HashMap<String, Variable>) -> Vec<String> {
        self.vars.iter().filter(
            |&i| !ctx.contains_key(i)
        ).map(
            |i| i.clone()
        ).collect::<Vec<String>>()
    }

}

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