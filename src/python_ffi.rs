use std::collections::HashMap;
use pyo3::{
    Python,
    types::PyModule,
    PyResult,
    pymodule,
    pyclass,
    pymethods,
    pyfunction,
    wrap_pyfunction
};
use crate::{solver::Nexsys, algos::Variable, solve};

/// The Python-accessible Nexsys solver object.
#[pyclass(name = "Nexsys")]
pub struct PyNexsys {
    system: Option<Nexsys>
}
#[pymethods]
impl PyNexsys {
    /// Instantiates a new Nexsys object in Python (a.k.a. `__init__`)
    #[new]
    #[pyo3(signature = (text, tol = 1E-10, limit = 300, nonconvergence = false))]
    fn new(text: &str, tol: f64, limit: usize, nonconvergence: bool) -> PyResult<PyNexsys> {
        Ok(PyNexsys {
            system: Some(Nexsys::new(text, tol, limit, nonconvergence))
        })
    }

    /// Manually inserts a value into the system solution. This can be 
    /// used to parametrize Nexsys code in a way that is more 
    /// accessible to another program.
    pub fn edit(&mut self, var: &str, value: f64) {
        if let Some(n) = &mut self.system {
            n.edit(var, value);
        }
    } 

    /// Does the same thing as `Nexsys.edit()` but adds a `HashMap` of variables all at the same time.
    pub fn mass_add_edits(&mut self, values: HashMap<String, f64>) {
        let vals = values.into_iter()
        .map(|i| (i.0, Variable::new(i.1, None)))
        .collect();
        if let Some(n) = &mut self.system {
            n.mass_add_edits(vals);
        }
    }

    /// Specifies an initial guess value for the given variable
    pub fn guess(&mut self, var: &str, value: f64) {
        if let Some(n) = &mut self.system {
            n.guess(var, value);
        }
    }

    /// Does the same thing as `Nexsys.guess()` but adds a `HashMap` of guess values all at the same time.
    pub fn mass_add_guess(&mut self, guesses: HashMap<String, f64>) {
        if let Some(n) = &mut self.system {
            n.mass_add_guess(guesses);
        }
    }

    /// Adds a domain specification for the given variable.
    pub fn domain(&mut self, var: &str, value: Vec<f64>) {
        if let Some(n) = &mut self.system {
            n.domain(var, [value[0], value[1]]);
        }
    }

    /// Does the same thing as `Nexsys.domain()` but adds a `HashMap` of domains all at the same time.
    pub fn mass_add_domains(&mut self, domains: HashMap<String, [f64; 2]>) {
        if let Some(n) = &mut self.system {
            n.mass_add_domains(domains);
        }
    }

    /// Solves the equations passed to the Nexsys solver, consuming the `self` value and 
    /// returning the solution to the system as a `dict`. This method can only be called once.
    /// If called more than once on the same instance of the object in Python, it will crash.
    pub fn solve(&mut self) -> PyResult<(HashMap<String, f64>, Vec<String>)> {
        
        let opn = self.system.take(); 
        let n = opn.unwrap();
        let mut res = match n.solve() {
            Ok(o) => o,
            Err(e) => panic!("{}", e)
        };

        let soln = res.0.drain().map(
            |i| (i.0, i.1.as_f64())
        ).collect::<HashMap<String, f64>>();
        let log = res.1;

        Ok((soln, log))
    }
}

/// The Python-accessible Nexsys interpreter function
#[pyfunction]
#[pyo3(signature = (system, tolerance = 1E-10, max_iterations = 300, allow_nonconvergence = false))]
pub fn py_solve(system: &str, tolerance: f64, max_iterations: usize, allow_nonconvergence: bool) -> PyResult<(HashMap<String, f64>, Vec<String>)> {
    match solve(system, Some(tolerance), Some(max_iterations), allow_nonconvergence) {
        Ok(o) => {
            let (soln, log) = o;

            let pythonic = soln.into_iter().map(|i| (i.0, i.1.as_f64())).collect();

            Ok((pythonic, log))
        },
        Err(e) => panic!("{}", e)
    }
}

/// The nexsys Python module
#[pymodule]
fn nexsys(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyNexsys>()?;
    m.add_function(wrap_pyfunction!(py_solve, m)?)?;
    Ok(())
}