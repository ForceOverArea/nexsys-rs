use std::collections::HashMap;

use nexsys_math::*;
use nexsys_core::stitch_hm;
use super::tools::*;

#[test]
fn _2x2_matrix_inversion() {

    let mut _2 = NxN::from_cols(vec![
        vec![-1.0, 1.0],
        vec![ 1.5,-1.0]
    ], None).unwrap();

    _2.invert().unwrap();

    let inv = vec![
        vec![2.0, 2.0],
        vec![3.0, 2.0]
    ];

    assert_eq!(_2.to_vec(), inv);
}

#[test]
fn _3x3_matrix_inversion() {

    let mut _3 = NxN::from_cols(vec![
        vec![ 1, 2,-1],
        vec![ 2, 1, 2],
        vec![-1, 2, 1]
    ], None).unwrap();

    _3.invert().unwrap();

    let inv = vec![
        vec![  3.0/16.0 as f64, 1.0/4.0 as f64, -5.0/16.0 as f64],
        vec![  1.0/4.0  as f64,     0.0 as f64,  1.0/4.0  as f64],
        vec![ -5.0/16.0 as f64, 1.0/4.0 as f64,  3.0/16.0 as f64]
    ];

    assert_eq!(_3.to_vec(), inv)
}

#[test]
fn _4x4_matrix_inversion() {
    
    let mut _4 = NxN::from_cols(vec![
        vec![ 4, 0, 0, 1],
        vec![ 0, 0, 1, 0],
        vec![ 0, 2, 2, 0],
        vec![ 0, 0, 0, 1]
    ], None).unwrap();

    _4.invert().unwrap();

    let inv = vec![
        vec![0.25, 0.0, 0.0, -0.25],
        vec![0.0, -1.0, 0.5,  0.0 ],
        vec![0.0,  1.0, 0.0,  0.0 ],
        vec![0.0,  0.0, 0.0,  1.0 ]
    ];

    assert_eq!(_4.to_vec(), inv)
}

#[test]
fn _5x5_matrix_inversion() {

    let mut _5 = NxN::from_cols(vec![
        vec![3, 11, 2, 17, 22],
        vec![4, 10, 12, 18, 23],
        vec![8, 9, 14, 19, 24],
        vec![5, 7, 15, 20, 25],
        vec![6, 13, 16, 21, 26]
    ], None).unwrap();

    _5.invert().unwrap();

    let inv = vec![
        vec![ 0.0181,-0.1818, 0.2909,-0.109, -0.0181],
        vec![-0.0090, 0.0909,-0.0204,-0.1954, 0.134 ],
        vec![-0.1181, 0.1818,-0.0159,-0.0409,-0.0068],
        vec![ 0.4236,-2.2363,-0.5468, 0.9081, 1.2513],
        vec![-0.269,  1.6909, 0.3945,-0.5854,-1.0309]
    ];

    // Truncate resulting matrix values
    let res = _5
    .to_vec()
    .iter()
    .map(
        |i| {
            i.iter().map(|&j| {
                round(j, 4)
            }).collect::<Vec<f64>>()
        }).collect::<Vec<Vec<f64>>>();

    assert_eq!(res, inv)
}

#[test]
fn test_mat_vec_mul() {
    let my_matrix = NxN::identity(3);

    let my_vec = vec![ 2.0, 2.0, 2.0 ];

    assert_eq!(
        mat_vec_mul(my_matrix, my_vec.clone()).unwrap(),
        my_vec
    )
}

#[test]
fn test_nxn_row_add() {
    let mut my_matrix = invertible_matrix_2();
    my_matrix.add_to_row(1, &vec![1.0, 2.0]);
    let check = vec![
        vec![-1.0, 2.0],
        vec![1.5, 1.0]
    ];

    assert_eq!(my_matrix.to_vec(), check);

    let mut my_matrix = invertible_matrix_3();
    my_matrix.add_to_row(1, &vec![-2.0, -1.0, -2.0]);
    let check = vec![ 
        vec![ 1.0, 0.0, -1.0], 
        vec![ 2.0, 0.0,  2.0],
        vec![-1.0, 0.0,  1.0] 
    ];

    assert_eq!(my_matrix.to_vec(), check);
}

#[test]
fn test_nxn_row_scale() {
    let mut my_matrix = invertible_matrix_2();

    my_matrix.scale_row(1, 0.0);

    let check = vec![
        vec![-1.0, 0.0],
        vec![1.5, 0.0]
    ];

    assert_eq!(my_matrix.to_vec(), check);
}

#[test]
fn test_nxn_row_get() {
    let my_matrix = invertible_matrix_2();

    assert_eq!(my_matrix.get_row(0), vec![-1.0, 1.5]);
    assert_eq!(my_matrix.get_row(1), vec![1.0, -1.0])
}

#[test]
fn test_jacobian() {
    let my_sys = vec![
        "x^2 + y",
        "y   - x"
    ];

    let guess = HashMap::from([
        ("x", Variable::new(1.0, None)),
        ("y", Variable::new(1.0, None))
    ]);

    let my_j = jacobian(&my_sys, &guess).unwrap();

    let cols = stitch_hm(my_j.vars.clone().unwrap(), my_j.to_vec());

    assert_eq!(cols["x"][0].round(),  2.0);
    assert_eq!(cols["x"][1].round(), -1.0);
    assert_eq!(cols["y"][1].round(),  1.0);
    assert_eq!(cols["y"][0].round(),  1.0);
}