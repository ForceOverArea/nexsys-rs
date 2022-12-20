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

use std::{collections::HashMap, error::Error};
use algos::Variable;
use solver::Nexsys;
use parsing::*;

pub fn solve(
    system: &str, 
    mut tolerance: Option<f64>, 
    mut max_iterations: Option<usize>, 
    allow_nonconvergence: bool
) -> Result<(HashMap<String, Variable>, Vec<String>), Box<dyn Error>> {

    if let None = tolerance { tolerance = Some(1E-5); }
    if let None = max_iterations { max_iterations = Some(300); }

    // let includes = includes(system);     // NOTE: Includes are pending implementation on account of "I think they're shitty and also bad".
    let mut preprocess  = conversions(system)?;
    preprocess = consts(&preprocess)?;
    preprocess = conditionals(&preprocess)?;

    let mut sys = Nexsys::new(
        preprocess.as_str(), 
        tolerance.unwrap(), 
        max_iterations.unwrap(),
        allow_nonconvergence
    );
    
    sys.mass_add_domains(domains(system));
    sys.mass_add_guess(guess_values(system));
    sys.mass_add_edits(imports(system, tolerance, max_iterations, allow_nonconvergence));

    sys.solve()
}