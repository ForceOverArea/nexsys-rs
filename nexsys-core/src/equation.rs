use nexsys_math::Variable;
use std::collections::HashMap;
use super::re::*;

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