/// Provides implementations of single-variable and multivariate versions of Newton's method.
pub mod algos;
/// Provides code for math operations that are useful in multivariate calculus.
pub mod mvcalc;
/// Provides access to the Nexsys equation solver engine. Useful for solving equations in other code.
pub mod solver;
/// Provides tools for parsing text prior to passing to the equation solving engine.
pub mod parsing;
/// Not useful in Rust, but provides Python access to the Nexsys equation solving engine.
#[cfg(feature = "python_ffi")]
mod python_ffi;

use std::collections::HashMap;
use algos::Variable;
use solver::Nexsys;
use parsing::*;

pub fn solve(system: &str, mut tolerance: Option<f64>, mut max_iterations: Option<usize>, mut allow_nonconvergence: Option<bool>) -> Result<(HashMap<String, Variable>, Vec<String>), &'static str> {

    if let None = tolerance { tolerance = Some(1E-5); }
    if let None = max_iterations { max_iterations = Some(300); }
    if let None = allow_nonconvergence { allow_nonconvergence = Some(false); }

    let mut sys = Nexsys::new(
        system, 
        tolerance.unwrap(), 
        max_iterations.unwrap(),
        allow_nonconvergence.unwrap()
    );

    sys.mass_add_domains(domains(system));
    sys.mass_add_guess(guess_values(system));
    sys.mass_add_edits(imports(system, tolerance, max_iterations, allow_nonconvergence));

    sys.solve()
}