mod tools;

use std::collections::HashMap;
use nexsys::mvcalc::*;
use nexsys::algos::{Variable, stitch_hm};
use tools::round;

#[test]
fn _2x2_matrix_inversion() {
    let (mut _2, inv) = invertible_2x2!();
    _2.invert().unwrap();
    assert_eq!(_2.to_vec(), inv);
}

#[test]
fn _3x3_matrix_inversion() {
    let (mut _3, inv) = invertible_3x3!();
    _3.invert().unwrap();
    assert_eq!(_3.to_vec(), inv)
}

#[test]
fn _4x4_matrix_inversion() {
    let (mut _4, inv) = invertible_4x4!();
    _4.invert().unwrap();
    assert_eq!(_4.to_vec(), inv)
}

#[test]
fn _5x5_matrix_inversion() {

    let (mut _5, inv) = invertible_5x5!();
    _5.invert().unwrap();

    // Truncate resulting matrix values
    let res = _5
    .to_vec()
    .iter()
    .map(
        |i| {
            i.iter().map(|&j| {
                round(j, 4).unwrap()
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
    let (mut my_matrix, _) = invertible_2x2!();
    my_matrix.add_to_row(1, &vec![1.0, 2.0]);
    let check = vec![
        vec![-1.0, 2.0],
        vec![1.5, 1.0]
    ];

    assert_eq!(my_matrix.to_vec(), check);

    let (mut my_matrix, _) = invertible_3x3!();
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
    let (mut my_matrix, _) = invertible_2x2!();
    my_matrix.scale_row(1, 0.0);
    let check = vec![
        vec![-1.0, 0.0],
        vec![1.5, 0.0]
    ];
    assert_eq!(my_matrix.to_vec(), check);
}

#[test]
fn test_nxn_row_get() {
    let (my_matrix, _) = invertible_2x2!();

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