mod structs;

use meval::{Context, eval_str_with_context};
use std::{collections::HashMap};
use crate::mvcalc::{jacobian, functionify, mat_vec_mul, d_dx};

pub use structs::*;

/// Performs one iteration of Newton's method for a system of equations, returning the next guess vector. 
pub fn next_guess<'a>(system: &Vec<&'a str>, mut guess: HashMap<&'a str, Variable>) -> Result<HashMap<&'a str, Variable>, &'static str> {

    let mut j = jacobian(&system, &guess)?;
    j.invert()?;

    let fx = Vec::from_iter(
        system.iter().map(
            |&i| functionify(i)(&guess)
        )
    );
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
/// let root = newton_raphson(my_eqn, my_guess, 0.001, 500).unwrap();
/// 
/// assert_eq!(root.1.as_f64().round(), -1.0)
/// ```
/// 
/// This function returns a result that will always contain a 
/// `(&'a str, Variable)` even if it is erroneous. An `Err` result 
/// simply indicates that the algorithm didn't converge on a solution
/// before exceeding the iteration limit.
/// # Example
/// ```
/// use nexsys::algos::Variable;
/// use nexsys::algos::newton_raphson;
/// 
/// let my_eqn = "x^2 - 1";
/// let my_guess = ("x", Variable::new(-5.0, Some([-10.0, 0.0])));
/// 
/// // Notice how our iteration limit is 0.
/// // The correct answer won't be achieved within this limit.
/// let root = newton_raphson(my_eqn, my_guess, 0.001, 0);
/// 
/// match root {
///     Err(ans) => assert_ne!(ans.1.as_f64().round(), -1.0),
///     _ => panic!("This panic won't happen")
/// }
/// ```
pub fn newton_raphson<'a>(equation: &'a str, guess: (&'a str, Variable), tolerance: f64, max_iterations: usize) -> Result<(&'a str, Variable), (&'a str, Variable)> {

    let mut xi = guess.1;
    let mut ctx = Context::new();
    let err_msg = format!("failed to evaluate equation: {}", equation);

    // Lord, forgive me for what I am about to do...
    let mut f = |x:f64| eval_str_with_context(equation, ctx.var(guess.0, x))
        .expect(&err_msg).abs();

    let mut count: usize = 0;
    while &f(xi.as_f64()) > &tolerance.into() {

        let mut roc = d_dx(&mut f, xi.as_f64());

        if roc == 0.0 { roc = f64::MIN_POSITIVE } // Avoid crash
        
        xi.step( -&f(xi.as_f64()) / roc );
        
        count += 1;
        if max_iterations < count {
            return Err((guess.0, xi))
        }
    }
    Ok((guess.0, xi))
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
/// let ans = mv_newton_raphson(my_sys, guess, 0.001, 500).unwrap();
/// 
/// println!("{:#?}", ans);
///
/// assert_eq!(ans["x"].as_f64().round(), 0.0)
/// ```
pub fn mv_newton_raphson<'a>( system: Vec<&'a str>, mut guess: HashMap<&'a str, Variable>, tolerance: f64, max_iterations: usize ) -> Result<HashMap<&'a str, Variable>, &'static str> {
    let error = |guess: &HashMap<&str, Variable>| -> f64 {
        system.iter().map(
            |&i| {
                let mut ctx = Context::new();
                
                for j in guess {
                    ctx.var(*j.0, j.1.as_f64()); 
                }
                
                let exp = i.replace("=", "-");
                
                eval_str_with_context(&exp, ctx)
                .expect(
                    &format!(
                        "error function failed to evaluate the system string: {}", &exp
                    )
                ).abs()
            }
        ).sum()
    };
    
    let mut count: usize = 0;

    loop {
        let res = next_guess(&system, guess)?;
        
        let e = error(&res);
        guess = res;

        if e < tolerance { // Solution is valid and acceptable
            return Ok(guess)
        } else if count > max_iterations { // Solution is valid, but timed out. Add a warning
            guess.insert("__error__".into(), Variable::new(e, None));
            return Ok(guess)
        } 
        count += 1;
    }
}

/// Solves a single equation for a single unknown value.
/// This function is a more robust substitute for `newton_raphson()`,
/// although it can take significantly longer to return a result. (O(n) w.r.t. tolerance)
/// 
/// # Example
/// ```
/// use nexsys::algos::Variable;
/// use nexsys::algos::golden_search;
/// 
/// let my_eqn = "x^2 - 1";
/// let my_guess = ("x", Variable::new(-1.0, Some([-10.0, 0.0])));
/// 
/// let root = golden_search(my_eqn, my_guess, 0.001).unwrap();
/// 
/// assert_eq!(root.1.as_f64().round(), -1.0)
/// ```
/// 
/// This function returns a result that will always contain a 
/// `(&'a str, Variable)` even if it is erroneous. An `Err` result 
/// simply indicates that the algorithm didn't find a solution within
/// the tolerance passed to the function.
/// # Example
/// ```
/// use nexsys::algos::Variable;
/// use nexsys::algos::golden_search;
/// 
/// let my_eqn = "x^5000 - 1";
/// let my_guess = ("x", Variable::new(-2.5, Some([-3.0, -2.0])));
/// 
/// // Notice how our iteration limit is 0.
/// // The correct answer won't be achieved within this limit.
/// let root = golden_search(my_eqn, my_guess, 0.001);
/// 
/// match root {
///     Err(ans) => assert_ne!(ans.1.as_f64().round(), -1.0),
///     _ => panic!("This panic won't happen")
/// }
/// ```
pub fn golden_search<'a>(equation: &'a str, guess: (&'a str, Variable), tolerance: f64) -> Result<(&'a str, Variable), (&'a str, Variable)> {

    let gr = (5_f64.sqrt() + 1.0) / 2.0;
    let mut xi = guess.1;
    let mut ctx = Context::new();
    let err_msg = format!("failed to evaluate equation: {}", equation);

    let (mut a, mut d) = match xi.get_domain() {
        Some(d) => (d[0], d[1]),
        None => (-1E20, 1E20)
    };

    // Lord, forgive me for what I am about to do...
    let mut f = |x:f64| eval_str_with_context(equation, ctx.var(guess.0, x))
        .expect(&err_msg).abs();

    let mut b = d - (d - a) / gr;
    let mut c = a + (d - a) / gr;
    while (d - a).abs() > tolerance {

        if f(b) < f(c) {
            d = c;
        } else {
            a = b;
        }

        b = d - (d - a) / gr;
        c = a + (d - a) / gr;
    }

    xi.change((d + a)  / 2.0);
    if f(xi.as_f64()).abs() > tolerance {
        Err((guess.0, xi))
    } else {
        Ok((guess.0, xi))
    }
}