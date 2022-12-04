use std::collections::HashMap;
use super::Nexsys;
use pyo3::{
    Python,
    types::PyModule,
    PyResult,
    pymodule,
    pyclass,
    pymethods
};

/// The Python-accessible Nexsys solver object.
#[pyclass(name = "Nexsys")]
pub struct PyNexsys {
    system: Option<Nexsys>
}
#[pymethods]
impl PyNexsys {

    #[new]
    fn new<'a>(text: &'a str, tol: f64, limit: usize) -> PyResult<PyNexsys> {
        Ok(PyNexsys {
            system: Some(Nexsys::new(
                text, 
                Some((tol, limit))
            ))
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
fn nexsys_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyNexsys>()?;
    Ok(())
}