use nexsys_math::NxN;

pub fn round(num: f64, places: usize) -> f64 {
    let res = num.to_string();
    let idx = res.find('.').unwrap();

    let lead = res[0..idx].to_string();
    let tail = res[idx+1..res.len()][0..places].to_string();

    (lead + "." + &tail).parse::<f64>().unwrap()
}

pub fn invertible_matrix_2() -> NxN {
    let res = vec![ 
        vec![-1.0, 1.0], 
        vec![1.5, -1.0] 
    ];
    NxN::from_cols(res, None).unwrap()
}

pub fn invertible_matrix_3() -> NxN {
    let res = vec![
        vec![ 1.0, 2.0, -1.0], 
        vec![ 2.0, 1.0,  2.0],
        vec![-1.0, 2.0,  1.0] 
    ];
    NxN::from_cols(res, None).unwrap()
}