use super::{NxN, Variable};
use meval::{Context, eval_str_with_context};
use std::{collections::HashMap, ops::{Add, Sub, Div}, fmt::Display, hash::Hash};

/// Takes a mathematical expression given as a string and returns a function.
pub fn functionify<S>(text: S) -> impl Fn(&HashMap<S, Variable>) -> f64
where
    S: Copy + AsRef<str> + Display + Into<String>
{
    let func = move |v:&HashMap<S, Variable>| -> f64 {
        let mut ctx = Context::new();
        
        for k in v {
            ctx.var(*k.0, k.1.as_f64());
        }

        eval_str_with_context(text, ctx).expect(&format!("failed to evaluate expression: {}", text))
    };
    func
}

/// Returns the derivative of a function at a point.
pub fn d_dx<T>(mut func: impl FnMut(T) -> T, x: T) -> T 
where
    T: Copy + Add<T, Output = T> + Add<f64, Output = T> + Sub<T, Output = T> + Div<f64, Output = T>
{
    let dx = 1e-7;
    ( func(x + dx) - func(x) ) / dx
}

/// Returns the partial derivative of a function w.r.t. the `target` variable.
/// # Example
/// ```
/// use nexsys_math::partial_d_dx;
/// use nexsys_math::Variable;
/// use std::collections::HashMap;
/// let expr = "x^2 + y - z";
/// 
/// let X = HashMap::from([
///     ("x", Variable::new(1_f64, None)),
///     ("y", Variable::new(1_f64, None)),
///     ("z", Variable::new(1_f64, None))
/// ]);
/// 
/// let dFdx = partial_d_dx(expr, &X, "x");
/// assert_eq!(dFdx.round(), 2_f64);
/// ```
pub fn partial_d_dx<S>(expr: S, guess: &HashMap<S, Variable>, target: S) -> f64 
where 
    S: Copy + AsRef<str> + Display + Into<String> + Eq + Hash
{
    // copy the guess vector
    let mut temp = guess.clone();

    // create an actual function from the given expression
    let func = functionify(expr);

    // create a partial function of the target variable
    let partial = move |x:f64| -> f64 {
        if let Some(v) = temp.get_mut(&target) {
            v.change(x);
        }
        func(&temp)
    };

    // take the derivative of the partial function
    d_dx(partial, guess[&target].as_f64())
}

/// Returns a tuple of `Vec`s that contain the keys and values of the original HashMap. 
/// The index of the key will be the same as its corresponding value's index.
/// 
/// This function only exists for use in `pub fn jacobian()`.
fn split_hm<K, V>(hm: HashMap<K, V>) -> (Vec<K>, Vec<V>) {
    let mut keys = Vec::new();
    let mut vals = Vec::new();

    for i in hm {
        keys.push(i.0);
        vals.push(i.1);
    }

    (keys, vals)
}

/// Returns the (numerical) `NxN` Jacobian matrix of a given system of equations at the vector given by `guess`.
/// 
/// Note that the resulting matrix's columns will be in a random order, so extra care is needed to identify which
/// variable occupies which column by checking the ordering of `self.vars`.
/// # Example
/// ```
/// use nexsys_math::jacobian;
/// use nexsys_math::Variable;
/// use std::collections::HashMap;
/// 
/// let my_sys = vec![
///     "x^2 + y",
///     "y   - x"
/// ];
/// let guess = HashMap::from([
///     ("x", Variable::new(1.0, None)),
///     ("y", Variable::new(1.0, None))
/// ]);
/// 
/// let j = jacobian(&my_sys, &guess);
/// 
/// // j.to_vec() will return roughly:
/// // vec![
/// //      vec![2.0, -1.0],
/// //      vec![1.0, 1.0]
/// // ];
/// ```
pub fn jacobian(system: &Vec<&str>, guess: &HashMap<&str, Variable>) -> Result<NxN, &'static str> {
    if system.len() != guess.keys().len() { 
        panic!("ERR: System is not properly constrained!") // guard clause against invalid problems
    } 

    let size = system.len();
    let mut mat = Vec::new();
    let vec = split_hm(guess.clone());

    for c in 0..size {
        let col = Vec::from_iter(
            system.iter().map(
                |&i| partial_d_dx(i, guess, vec.0[c])
            )
        );
        mat.push(col);
    };

    NxN::from_cols( mat, Some(vec.0) )
}