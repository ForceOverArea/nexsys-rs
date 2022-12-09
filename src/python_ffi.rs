use std::collections::HashMap;
use pyo3::{
    Python,
    types::PyModule,
    PyResult,
    pymodule,
    pyclass,
    pymethods
};
use crate::{solver::Nexsys, algos::Variable};

/// The Python-accessible Nexsys solver object.
#[pyclass(name = "Nexsys")]
pub struct PyNexsys {
    system: Option<Nexsys>
}
#[pymethods]
impl PyNexsys {
    #[new]
    fn new<'a>(text: &'a str, tol: f64, limit: usize, nonconvergence: bool) -> PyResult<PyNexsys> {
        Ok(PyNexsys {
            system: Some(Nexsys::new(text, tol, limit, nonconvergence))
        })
    }

    /// Manually inserts a value into the system solution. This can be 
    /// used to parametrize Nexsys code in a way that is more 
    /// accessible to another program.
    pub fn edit<'a>(&mut self, var: &'a str, value: f64) {
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
        let mut res = n.solve()
        .expect("failed to solve system of equations");

        let soln = res.0.drain().map(
            |i| (i.0, i.1.as_f64())
        ).collect::<HashMap<String, f64>>();
        let log = res.1;

        Ok((soln, log))
    }
}

#[pymodule]
fn nexsys(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyNexsys>()?;
    Ok(())
}