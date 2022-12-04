mod linalg;
mod mvcalc;
mod nxn;

pub use crate::linalg::*;
pub use crate::mvcalc::*;
pub use crate::nxn::*;

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
}