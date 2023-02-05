/// Provides implementations of single-variable and multivariate versions of Newton's method.
pub mod algos;
/// Provides code for math operations that are useful in multivariate calculus.
pub mod mvcalc;
/// Provides access to the Nexsys equation solver engine. Useful for solving equations in other code.
pub mod solver;
/// Provides data sets of common units and functions for converting between them.
pub mod units;
/// Provides tools for parsing text prior to passing to the equation solving engine.
pub mod parsing;
/// Different errors specific to Nexsys implementations of algorithms.
pub mod errors;
/// Not useful in Rust, but provides Python access to the Nexsys equation solving engine.
#[cfg(feature = "python_ffi")]
mod python_ffi;

/// Not useful in Rust, but provides C/C++ access to the Nexsy equation solving engine.
#[cfg(feature = "c_ffi")]
mod c_ffi;

use std::{collections::HashMap, error::Error};
use algos::Variable;
use solver::Nexsys;
use parsing::{compile, domains, guess_values};

/// Shorthand for the contents of a Nexsys Solution: a
/// `HashMap<String, Variable>` of variable values in the 
/// solution as well as a `Vec<String>` of the steps taken
/// to obtain the solution.
type SolverOutput = (HashMap<String, Variable>, Vec<String>);

/// Evaluates a string of nexsys-legal code and returns the 
/// solution to the system as well as the steps taken to obtain it.
pub fn solve(
    system: &str, 
    mut tolerance: Option<f64>, 
    mut max_iterations: Option<usize>, 
    allow_nonconvergence: bool
) -> Result<SolverOutput, Box<dyn Error>> {

    if tolerance        .is_none() { tolerance = Some(1E-10); }
    if max_iterations   .is_none() { max_iterations = Some(300); }

    let mut sys = Nexsys::new(
        compile(system)?.as_str(), 
        tolerance.unwrap(), 
        max_iterations.unwrap(),
        allow_nonconvergence
    );
    
    sys.mass_add_domains(domains(system));
    sys.mass_add_guess(guess_values(system));

    sys.solve()
}