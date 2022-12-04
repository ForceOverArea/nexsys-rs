use nexsys_math::Variable;
use std::collections::HashMap;
/// Solving and optimization algorithms
mod algos;
/// Commonly used regex patterns for the solver  
mod re;
/// Equation struct for parsing equations
mod equation;
/// Block manager for identifying constrained systems
mod block_mgr;
/// A module containing a Python-accessible version of the Nexsys solver.
#[cfg(feature = "python_ffi")]
mod python_ffi;

// Import algos mod for public use and import re mod for private use by solver
pub use crate::equation::*;
pub use crate::block_mgr::*;
pub use crate::algos::*;
pub (in crate) use crate::re::*;

#[derive(PartialEq)]
enum Progress {
    Solved,
    NoneSolved
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
/// # Nexsys
/// Struct responsible for solving equations 
pub struct Nexsys {
    equations : Vec<Equation>,
    guesses: HashMap<String, f64>,
    domains: HashMap<String, [f64; 2]>,
    solution: HashMap<String, Variable>,
    log: Vec<String>,
    tolerance: f64,
    max_iterations: usize,
    allow_nonconvergence: bool
}
impl Nexsys {
    /// Initializes a new Nexsys solver from a string.
    /// 
    /// The `Nexsys` struct is used for lexing Nexsys-valid code in
    /// string format. This enables solving systems from text files
    /// or string data passed from other programs.
    pub fn new(text: &str, params: Option<(f64, usize)>) -> Nexsys {
        
        let mut tolerance = 0.001;
        let mut max_iterations = 300_usize;

        if let Some(p) = params {
            tolerance = p.0;
            max_iterations = p.1;
        }

        let equations = text.split("\n").filter(|i| i.contains("=")).map(
            |i| Equation::new(i)
        ).collect();

        let guesses = guess_values(text.into());
        let domains = domains(text.into());
        let solution = HashMap::new();
        let log = vec![];
        let allow_nonconvergence = false;

        Nexsys { equations, guesses, domains, solution, log, tolerance, max_iterations, allow_nonconvergence }
    }

    /// Manually inserts a value into the system solution. This can be 
    /// used to parametrize Nexsys code in a way that is more 
    /// accessible to another program.
    pub fn edit(&mut self, var: &str, value: f64) {
        self.solution.insert(
            var.to_string(), 
            Variable::new(
                value, 
                None
            )
        );
    }

    /// Sub in known variables to simplify an expression
    fn substitute(&self, mut expr: String) -> String {
        for v in &self.solution {
            expr = expr.replace(
                v.0, 
                &format!("{}", v.1.as_f64())
            );
        }
        expr
    }

    /// Solve any 1-unknown equations in the system.
    fn light_work(&mut self) -> Result<Progress, &'static str> {

        let mut solved = 0;

        for eqn in &self.equations {
            if eqn.n_unknowns(&self.solution) != 1 { 
                continue // only operate on 1-unknown problems
            }

            solved += 1; // indicate that a solvable equation was found

            let target = &eqn.unknowns(&self.solution)[0];

            let v = Variable::new( 
                match self.guesses.get(target) {
                    Some(&val) => val,
                    None => 1.0
                },
                match self.domains.get(target) {
                    Some(&dom) => Some(dom),
                    None => None
                }
            );

            let expr = self.substitute(eqn.as_expr());

            let ans = newton_raphson(
                &expr, 
                (target, v), 
                self.tolerance, 
                self.max_iterations
            );

            match ans {
                Ok(o) => {
                    self.solution.insert(o.0.to_string(), o.1);
                    self.log.push(format!("solved 1-unknown problem `{}` for the variable `{}`", eqn.as_text(), target));
                },
                Err(e) => {
                    if self.allow_nonconvergence {
                        self.solution.insert(e.0.to_string(), e.1);
                        self.log.push(format!("timeout on solving 1-unknown problem `{}` for the variable `{}`", eqn.as_text(), target));
                    } else {
                        return Err("failed to evaluate 1-unknown equation")
                    }
                }
            }
        }

        if solved == 0 {
            return Ok(Progress::NoneSolved) // let the caller know that no light work exists in the system
        }

        Ok(Progress::Solved)
    }

    /// Identify, group, and solve properly constrained systems of equations in the system.
    fn heavy_work(&mut self) -> Result<Progress, &'static str> {
        
        let mut blks = BlockMgr::new(&self.solution);

        for eqn in &self.equations {
            blks.add_item(eqn);
        }

        let blocks = blks.constrained();
        
        if let None = blocks {
            return Ok(Progress::NoneSolved)
        }

        for block in blocks.unwrap() {

            // Lord forgive me for what I am about to do...
            let preprocess: Vec<String> = block.1.iter().map( |i| self.substitute(i.to_string()) ).collect();
            let system: Vec<&str> = preprocess.iter().map(|i| i.as_str()).collect();

            let mut guess = HashMap::new();

            for v in &block.0 {

                let k = v.as_str();
                let mut value = 1.0;
                let mut domain = None;

                // get guess val if present
                if let Some(val) = self.guesses.get(k) {
                    value = *val;
                }

                // get domain value if present
                if let Some(dom) = self.domains.get(k) {
                    domain = Some(*dom);
                }

                guess.insert(k, Variable::new(value, domain));
            }

            let msg = format!(
                "solved a system of equations {:#?} for the variables {:#?}", 
                system, 
                guess.keys()
            );
            let err_msg = format!(
                "timeout on solving system of equations {:#?} for the variables {:#?}", 
                system, 
                guess.keys()
            );

            let ans = mv_newton_raphson(system, guess, self.tolerance, self.max_iterations)?;
            
            // Add solver report to log
            if ans.contains_key("__error__") {
                self.log.push(err_msg);
            } else {
                self.log.push(msg);
            }

            let findings = ans.iter().map(
                |i| (i.0.to_string(), i.1.clone())
            );

            self.solution.extend(findings);
        }

        Ok(Progress::Solved)
    }

    /// Solves the equations passed to the Nexsys solver, consuming the `self` value and 
    /// returning the solution to the system as a `HashMap`.
    pub fn solve(mut self) -> Result<(HashMap<String, Variable>, Vec<String>), &'static str> {
        loop {
            match self.light_work()? {
                Progress::Solved => {
                    continue;
                },
                Progress::NoneSolved => {
                    match self.heavy_work()? {
                        Progress::Solved => {
                            continue;
                        },
                        Progress::NoneSolved => {
                            return Ok(
                                (self.solution, self.log)
                            )
                        }
                    }
                }
            }
        }
    }
}