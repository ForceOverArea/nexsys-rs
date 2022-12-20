mod nxn;

use meval::eval_str_with_context;
use std::{
    collections::HashMap, 
    ops::{Add, Sub, Mul, Div}, 
    fmt::Display, 
    hash::Hash, 
    iter::Sum, 
    error::Error
};
use crate::{
    algos::{Variable, new_context}, 
    errors::{NxNMultiplicationError, RoundingError, VecMultiplicationError}
};

pub use nxn::NxN;

/// Rounds a value to the specified number of decimal places
pub fn round(num: f64, places: usize) -> Result<f64, Box<dyn Error>>{
    let res = num.to_string();
    let idx = match res.find('.') {
        Some(i) => i,
        None => return Err(Box::new(RoundingError))
    };
    
    let lead = res[0..idx].to_string();
    let tail = res[idx+1..res.len()][0..places].to_string();

    let _decider = res[places..places+1].to_string().parse::<f64>()?;

    Ok((lead + "." + &tail).parse::<f64>()?)
}

/// Takes a mathematical expression given as a string and returns a function.
pub fn functionify<S>(text: S) -> impl Fn(&HashMap<S, Variable>) -> Result<f64, Box<dyn Error>>
where
    S: Copy + AsRef<str> + Display + Into<String>
{
    let func = move |v:&HashMap<S, Variable>| -> Result<f64, Box<dyn Error>> {
        
        let mut ctx = new_context();
        
        for k in v {
            ctx.var(*k.0, k.1.as_f64());
        }
        
        match eval_str_with_context(text, ctx) {
            Ok(o) => Ok(o),
            Err(e) => Err(Box::new(e))
        }
    };

    func
}

/// Returns the derivative of a function at a point.
pub fn d_dx<T>(mut func: impl FnMut(T) -> Result<T, Box<dyn Error>>, x: T) -> Result<T, Box<dyn Error>>
where
    T: Copy + Add<T, Output = T> + Add<f64, Output = T> + Sub<T, Output = T> + Div<f64, Output = T>
{
    let dx = 1e-7;
    let res = ( func(x + dx)? - func(x)? ) / dx;

    Ok(res)
}

/// Returns the partial derivative of a function w.r.t. the `target` variable.
/// # Example
/// ```
/// use nexsys::mvcalc::partial_d_dx;
/// use nexsys::algos::Variable;
/// use std::collections::HashMap;
/// let expr = "x^2 + y - z";
/// 
/// let X = HashMap::from([
///     ("x", Variable::new(1_f64, None)),
///     ("y", Variable::new(1_f64, None)),
///     ("z", Variable::new(1_f64, None))
/// ]);
/// 
/// let dFdx = partial_d_dx(expr, &X, "x").unwrap();
/// assert_eq!(dFdx.round(), 2_f64);
/// ```
pub fn partial_d_dx<S>(expr: S, guess: &HashMap<S, Variable>, target: S) -> Result<f64, Box<dyn Error>>
where 
    S: Copy + AsRef<str> + Display + Into<String> + Eq + Hash
{
    // copy the guess vector
    let mut temp = guess.clone();

    // create an actual function from the given expression
    let func = functionify(expr);

    // create a partial function of the target variable
    let partial = move |x:f64| -> Result<f64, Box<dyn Error>> {
        if let Some(v) = temp.get_mut(&target) {
            v.change(x);
        }
        func(&temp)
    };

    // take the derivative of the partial function
    Ok( d_dx(partial, guess[&target].as_f64())? )
}

/// Returns the dot product of two given vectors.
pub fn vec_vec_dot<T, U>(lhs: &Vec<T>, rhs: &Vec<U>) -> Result<T, Box<dyn Error>> 
where   
    T: Copy + Mul<U> + Sum::<<T as Mul<U>>::Output>,
    U: Copy
{
    if lhs.len() != rhs.len() {
        return Err(Box::new(VecMultiplicationError))
    }
    let mut count = 0;
    let dot_prod = lhs.iter().map(
        |&i| {
            let res = i * rhs[count];
            count += 1;
            res
        }
    ).sum();

    Ok(dot_prod)
}

/// Multiplies a matrix and a column vector.
pub fn mat_vec_mul<T>(lhs: NxN, rhs: Vec<T>) -> Result<Vec<T>, Box<dyn Error>> 
where
    T: Copy + Mul<f64> + Sum::<<T as Mul<f64>>::Output>
{
    if lhs.size != rhs.len() {
        return Err(Box::new(NxNMultiplicationError))
    }

    let mat = lhs.to_vec();
    let mut res = vec![];

    for i in 0..rhs.len() {

        let mut row = vec![];

        for j in 0..rhs.len() {
            row.push(mat[j][i]);
        }

        res.push(vec_vec_dot(&rhs, &row)?)
    }
    Ok(res)
}

/// Scales a vector by the given value.
pub fn scale_vec<T, U>(vec: Vec<T>, scalar: U) -> Vec<T> 
where 
    T: Copy + Mul<U>, 
    Vec<T>: FromIterator<<T as Mul<U>>::Output>,
    U: Copy
{
    vec.iter().map( |&i| i * scalar ).collect()
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
/// use nexsys::mvcalc::jacobian;
/// use nexsys::algos::Variable;
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
pub fn jacobian(system: &Vec<&str>, guess: &HashMap<&str, Variable>) -> Result<NxN, Box<dyn Error>> {
    if system.len() != guess.keys().len() { 
        panic!("ERR: System is not properly constrained!") // guard clause against invalid problems
    } 

    let size = system.len();
    let mut mat = Vec::new();
    let vec = split_hm(guess.clone());

    let mut err = None; // storage for any meval errors returned during matrix creation

    for c in 0..size {
        let col = Vec::from_iter(
            system.iter().map(
                |&i| match partial_d_dx(i, guess, vec.0[c]) {
                    Ok(o) => o,
                    Err(e) => {
                        err = Some(e);  // now we can report errors from this closure
                        f64::NAN        // ...and still meet the `-> f64` requirement!
                    }
                }
            )
        );

        if let Some(e) = err {
            return Err(e);
        }

        mat.push(col);
    };

    Ok( NxN::from_cols( mat, Some(vec.0) )? )
}