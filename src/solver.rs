use std::{collections::HashMap, error::Error};
use crate::{algos::*, cleanup, parsing::legal_variable, errors::SolverConvergenceError, SolverOutput};

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
enum Progress {
    Solved,
    NoneSolved
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
/// # Nexsys
/// Access point to the Nexsys equation solver engine
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
    /// The `Nexsys` struct is used for processing equations given in
    /// string format. This enables solving systems from text files
    /// or string data passed from other programs.
    pub fn new(text: &str, tolerance: f64, max_iterations: usize, allow_nonconvergence: bool) -> Nexsys {

        let equations = text.split('\n')
        .filter(|i| i.contains('='))
        .map(Equation::new)
        .collect();

        let guesses = HashMap::new();
        let domains = HashMap::new();
        let solution = HashMap::new();
        let log = vec![];

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

    /// Does the same thing as `Nexsys.edit()` but adds a `HashMap` of variables all at the same time.
    pub fn mass_add_edits(&mut self, values: HashMap<String, Variable>) {
        self.solution.extend(values.into_iter());
    }

    /// Specifies an initial guess value for the given variable
    pub fn guess(&mut self, var: &str, value: f64) {
        self.guesses.insert(var.to_string(), value);
    }

    /// Does the same thing as `Nexsys.guess()` but adds a `HashMap` of guess values all at the same time.
    pub fn mass_add_guess(&mut self, guesses: HashMap<String, f64>) {
        self.guesses.extend(guesses.into_iter());
    }

    /// Adds a domain specification for the given variable.
    pub fn domain(&mut self, var: &str, value: [f64; 2]) {
        self.domains.insert(var.to_string(), value);
    }

    /// Does the same thing as `Nexsys.domain()` but adds a `HashMap` of domains all at the same time.
    pub fn mass_add_domains(&mut self, domains: HashMap<String, [f64; 2]>) {
        self.domains.extend(domains.into_iter());
    }

    /// Sub in known variables to simplify an expression
    fn substitute(&self, mut expr: String) -> String {
        
        let vars = legal_variable(&expr);
        let knowns = self.solution.iter().filter(|&i| vars.contains(i.0));

        for v in knowns {
            expr = expr.replace(
                v.0, 
                &format!("({})", v.1.as_f64())
            );
        }
        expr
    }

    /// Solve any 1-unknown equations in the system.
    fn light_work(&mut self) -> Result<Progress, Box<dyn Error>> {

        let mut solved = 0;

        for eqn in &self.equations {
            if eqn.n_unknowns(&self.solution) != 1 { 
                continue // only operate on 1-unknown problems
            }

            solved += 1; // indicate that a solvable equation was found

            let target = &eqn.unknowns(&self.solution)[0];

            // obtain guess domain and value
            let v = Variable::new( 
                match self.guesses.get(target) {
                    Some(&val) => val,
                    None => 1.0
                },
                self.domains
                    .get(target)
                    .copied()
            );

            let expr = self.substitute(eqn.as_expr());
            let ans = match newton_raphson(
                &expr, 
                (target, v.clone()), 
                self.tolerance, 
                self.max_iterations
            )? {
                Solution::Converged(c) => Solution::Converged(c),
                Solution::NonConverged(_) => {
                    self.log.push("N.R. method did not converge. Attempted G.S.S. method instead.".to_string());
                    golden_search( // Try the golden search algorithm if newton's fails
                        &expr, 
                        (target, v), 
                        self.tolerance
                    )? // if golden search fails, throw an error
                }
            };

            match ans {
                Solution::Converged(o) => {
                    self.solution.insert(o.0.to_string(), o.1);
                    self.log.push(format!("solved {} for variable {}", cleanup!(eqn.as_text(), "\r"), target));
                },
                Solution::NonConverged(e) => {
                    if self.allow_nonconvergence {
                        self.solution.insert(e.0.to_string(), e.1);
                        self.log.push(format!("timeout while solving {} for variable {}", cleanup!(eqn.as_text(), "\r"), target));
                    } else {
                        return Err(Box::new(SolverConvergenceError))
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
    fn heavy_work(&mut self) -> Result<Progress, Box<dyn Error>> {
        
        let mut blks = BlockMgr::new(&self.solution);

        for eqn in &self.equations {
            blks.add_item(eqn);
        }

        let blocks = blks.constrained();
        
        if blocks.is_none() {
            return Ok(Progress::NoneSolved)
        }

        for block in blocks.unwrap() {

            // Lord forgive me for what I am about to do...
            let preprocess: Vec<String> = block.1.iter().map(

                |i| self.substitute(i.to_string())                  // Put this in memory for later reference

            ).collect(); 
            let system: Vec<&str> = preprocess.iter().map(
                
                |i| i.as_str()                                      // Create &str's that reference preprocess
                
            ).collect(); 
            
            // Build guess vector
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

            // Solve system and report status of solution
            let err_msg = format!("timeout solving system {:#?} for variables {:?}", block.1, guess.keys());
            let msg = format!("solved system {:#?} for variables {:?}", block.1, guess.keys());
            let ans = mv_newton_raphson(system, guess, self.tolerance, self.max_iterations)?;
            
            match ans {
                Solution::Converged(s) => {
                    self.log.push(cleanup!(msg, "[", "]", r#"""#, "\r")); // Add solver report to log
                    
                    self.solution.extend(s.into_iter().map(
                        |i| (i.0.to_string(), i.1)
                    ));
                },
                Solution::NonConverged(s) => {
                    if self.allow_nonconvergence {
                        self.log.push(cleanup!(err_msg, "[", "]", r#"""#, "\r")); // Add solver report to log
                        
                        self.solution.extend(s.into_iter().map(
                            |i| (i.0.to_string(), i.1)
                        ));
                    } else {
                        return Err(Box::new(SolverConvergenceError))
                    }
                }      
            }  
        }

        Ok(Progress::Solved)
    }

    /// Solves the equations passed to the Nexsys solver, consuming the `self` value and 
    /// returning the solution to the system as a `HashMap`.
    pub fn solve(mut self) -> Result<SolverOutput, Box<dyn Error>> {
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
                            return Ok((self.solution, self.log))
                        }
                    }
                }
            }
        }
    }
}