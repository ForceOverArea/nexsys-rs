mod structs;

use meval::{Context, eval_str_with_context};
use std::collections::HashMap;
use std::error::Error;
use crate::errors::SolverDivisionByZeroError;
use crate::mvcalc::{jacobian, functionify, mat_vec_mul, d_dx};
use crate::parsing::conditional;

pub use structs::*;

/// Enum used to indicate whether the solution produced converged or not.
pub enum Solution<T> {
    Converged(T),
    NonConverged(T)
}
impl <T> Solution<T> {
    /// Returns the contained value, consuming the `self` value. 
    /// Unlike other implementations of `unwrap`, this method never panics.
    pub fn unwrap(self) -> T {
        match self {
            Solution::Converged(t) => t,
            Solution::NonConverged(t) => t
        }
    }
}

/// Function for returning non-empty context with Nexsys-custom functions. E.g. `if()`.
pub fn new_context() -> Context<'static> {
    let mut ctx = Context::new();
    ctx.funcn("if", conditional, 5_usize);
    ctx
}

/// Performs one iteration of Newton's method for a system of equations, returning the next guess vector. 
fn next_guess<'a>(system: &Vec<&'a str>, mut guess: HashMap<&'a str, Variable>) -> Result<HashMap<&'a str, Variable>, Box<dyn Error>> {

    let mut j = jacobian(system, &guess)?;
    j.invert()?;

    let mut err = None;

    let fx = Vec::from_iter(
        system.iter().map(
            |&i| match functionify(i)(&guess) {
                Ok(o) => o,
                Err(e) => {
                    err = Some(e);  // same song and dance as in mvcalc...
                    f64::NAN        // ...this fn returns f64, the overall function returns Result
                }
            }
        )
    );

    if let Some(e) = err {
        return Err(e)
    }

    let x_n = stitch_hm(
        j.vars.clone().unwrap(), // don't want to clone here, but Vec<String> shouldn't be super costly
        mat_vec_mul(j, fx)?
    );

    for v in &mut guess {
        v.1.step(-x_n[&v.0.to_string()])
    }

    Ok(guess)
}

/// Reverses the operation performed by `split_hm`.
pub fn stitch_hm<K, V>(mut keys: Vec<K>, mut vals: Vec<V>) -> HashMap<K, V> 
where
    K: std::hash::Hash + std::cmp::Eq
{
    let mut res = HashMap::new();
    for _ in 0..keys.len() {
        res.insert(
            keys.pop().unwrap(), 
            vals.pop().unwrap()
        );
    }
    res
}

/// Solves a single equation for a single unknown value. 
/// `mv_newton_raphson` can also be used for this scenario, but this 
/// function is a more lightweight and reasonable choice.
/// 
/// # Example
/// ```
/// use nexsys::algos::Variable;
/// use nexsys::algos::newton_raphson;
/// 
/// let my_eqn = "x^2 - 1";
/// let my_guess = ("x", Variable::new(-5.0, Some([-10.0, 0.0])));
/// 
/// let root = newton_raphson(my_eqn, my_guess, 0.001, 500).unwrap().unwrap();
/// 
/// assert_eq!(root.1.as_f64().round(), -1.0)
/// ```
pub fn newton_raphson<'a>(equation: &'a str, guess: (&'a str, Variable), tolerance: f64, max_iterations: usize) 
-> Result<Solution<(&'a str, Variable)>, Box<dyn Error>> {

    let mut xi = guess.1;
    let mut ctx = new_context();
    ctx.funcn("if", conditional, 5_usize);
    
    // Lord, forgive me for what I am about to do...
    let mut f = |x:f64| -> Result<f64, Box<dyn Error>> {
        match eval_str_with_context(equation, ctx.var(guess.0, x)) {
            Ok(o) => Ok(o.abs()),
            Err(e) => Err(Box::new(e))
        }
    };

    let mut count: usize = 0;
    while f(xi.as_f64())? > tolerance {

        let roc = d_dx(&mut f, xi.as_f64())?;

        if roc == 0.0 { return Err(Box::new(SolverDivisionByZeroError::NewtonRaphsonDivByZeroError)) } // Avoid crash
        
        xi.step( -&f(xi.as_f64())? / roc );
        
        count += 1;
        if count > max_iterations {
            return Ok(Solution::NonConverged((guess.0, xi)))
        }
    }
    Ok(Solution::Converged((guess.0, xi)))
}

/// Attempts to solve the equations passed to `system` via the Newton-Raphson method.
/// # Example
/// ```
/// use std::collections::HashMap;
/// use nexsys::algos::Variable;
/// use nexsys::algos::mv_newton_raphson;
/// 
/// let my_sys = vec!["x^2 + y", "y - x"];
/// let guess = HashMap::from([
///     ("x", Variable::new(1.0, None)),
///     ("y", Variable::new(1.0, None))
/// ]);
/// let ans = mv_newton_raphson(my_sys, guess, 0.001, 500).unwrap().unwrap();
/// 
/// println!("{:#?}", ans);
///
/// assert_eq!(ans["x"].as_f64().round(), 0.0)
/// ```
pub fn mv_newton_raphson<'a>( system: Vec<&'a str>, mut guess: HashMap<&'a str, Variable>, tolerance: f64, max_iterations: usize ) 
-> Result<Solution<HashMap<&'a str, Variable>>, Box<dyn Error>> {

    let error = |guess: &HashMap<&str, Variable>| -> Result<f64, Box<dyn Error>> {
        let mut err = None;
        let residual = system.iter().map(
            |&i| {
                let mut ctx = new_context();
                
                for j in guess {
                    ctx.var(*j.0, j.1.as_f64()); 
                }
                
                let exp = i.replace('=', "-");
                
                match eval_str_with_context(exp, ctx) {
                    Ok(o) => o.abs(),
                    Err(e) => {
                        err = Some(e);
                        f64::NAN
                    }
                }
            }
        ).sum();

        if let Some(e) = err {
            return Err(Box::new(e))
        }
        Ok(residual)
    };
    
    let mut count: usize = 0;

    loop {
        let res = next_guess(&system, guess)?;
        
        let e = error(&res)?;
        guess = res;

        if e < tolerance { // Solution is valid and acceptable
            return Ok(Solution::Converged(guess))
        } else if count > max_iterations { // Solution is valid, but timed out. Add a warning
            println!("count: {count}\nerror{e}");
            return Ok(Solution::NonConverged(guess))
        } 
        count += 1;
    }
}

/// Solves a single equation for a single unknown value.
/// This function is a more robust substitute for `newton_raphson()`,
/// although it can take significantly longer to return a result. (Time increases w.r.t. a decrease in tolerance)
/// 
/// # Example
/// ```
/// use nexsys::algos::Variable;
/// use nexsys::algos::golden_search;
/// 
/// let my_eqn = "x^2 - 1";
/// let my_guess = ("x", Variable::new(-1.0, Some([-10.0, 0.0])));
/// 
/// let root = golden_search(my_eqn, my_guess, 0.001).unwrap().unwrap();
/// 
/// assert_eq!(root.1.as_f64().round(), -1.0)
/// ```
pub fn golden_search<'a>(equation: &'a str, guess: (&'a str, Variable), tolerance: f64) 
-> Result<Solution<(&'a str, Variable)>, Box<dyn Error>> {

    let gr = (5_f64.sqrt() + 1.0) / 2.0;
    let mut xi = guess.1;
    let mut ctx = new_context();

    let (mut a, mut d) = match xi.get_domain() {
        Some(d) => (d[0], d[1]),
        None => (-1E20, 1E20)
    };

    // Lord, forgive me for what I am about to do...
    let mut f = |x:f64| -> Result<f64, Box<dyn Error>> {
        match eval_str_with_context(equation, ctx.var(guess.0, x)) {
            Ok(o) => Ok(o.abs()),
            Err(e) => Err(Box::new(e)) 
        }
    };

    let mut b = d - (d - a) / gr;
    let mut c = a + (d - a) / gr;
    while (d - a).abs() > tolerance {

        if f(b)? < f(c)? {
            d = c;
        } else {
            a = b;
        }

        b = d - (d - a) / gr;
        c = a + (d - a) / gr;
    }

    xi.change((d + a)  / 2.0);

    Ok(Solution::Converged((guess.0, xi))) // this is to maintain interchangeability with `newton_raphson()`
}
