use std::{ ops::Mul, iter::Sum };

use crate::nxn::NxN;

/// Returns the dot product of two given vectors.
pub fn vec_vec_dot<T, U>(lhs: &Vec<T>, rhs: &Vec<U>) -> Result<T, &'static str> 
where   
    T: Copy + Mul<U> + Sum::<<T as Mul<U>>::Output>,
    U: Copy
{
    if lhs.len() != rhs.len() {
        return Err("vectors must be the same size!")
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
pub fn mat_vec_mul<T>(lhs: NxN, rhs: Vec<T>) -> Result<Vec<T>, &'static str> 
where
    T: Copy + Mul<f64> + Sum::<<T as Mul<f64>>::Output>
{
    if lhs.size != rhs.len() {
        return Err("vectors must be the same size!")
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