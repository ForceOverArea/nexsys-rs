pub use nexsys::mvcalc::round;

#[macro_export]
macro_rules! assert_thou {
    ($a:expr, $b:expr) => {
        assert!(($a - $b).abs() < 0.001)
    };
}

#[macro_export]
macro_rules! invertible_2x2 {
    () => {
        (
            NxN::from_cols(vec![
                vec![-1.0, 1.0],
                vec![ 1.5,-1.0]
            ], None).unwrap(),
            vec![
                vec![2.0, 2.0],
                vec![3.0, 2.0]
            ]
        )
    }
} 

#[macro_export]
macro_rules! invertible_3x3 {
    () => {
        (
            NxN::from_cols(vec![
                vec![ 1, 2,-1],
                vec![ 2, 1, 2],
                vec![-1, 2, 1]
            ], None).unwrap(),
            vec![
                vec![  3.0/16.0 as f64, 1.0/4.0 as f64, -5.0/16.0 as f64],
                vec![  1.0/4.0  as f64,     0.0 as f64,  1.0/4.0  as f64],
                vec![ -5.0/16.0 as f64, 1.0/4.0 as f64,  3.0/16.0 as f64]
            ]
        )
    };
}

#[macro_export]
macro_rules! invertible_4x4 {
    () => {
        (
            NxN::from_cols(vec![
                vec![ 4, 0, 0, 1],
                vec![ 0, 0, 1, 0],
                vec![ 0, 2, 2, 0],
                vec![ 0, 0, 0, 1]
            ], None).unwrap(),
            vec![
                vec![0.25, 0.0, 0.0, -0.25],
                vec![0.0, -1.0, 0.5,  0.0 ],
                vec![0.0,  1.0, 0.0,  0.0 ],
                vec![0.0,  0.0, 0.0,  1.0 ]
            ]
        )
    };
}

#[macro_export]
macro_rules! invertible_5x5 {
    () => {
        (
            NxN::from_cols(vec![
                vec![3, 11, 2, 17, 22],
                vec![4, 10, 12, 18, 23],
                vec![8, 9, 14, 19, 24],
                vec![5, 7, 15, 20, 25],
                vec![6, 13, 16, 21, 26]
            ], None).unwrap(),
            vec![
                vec![ 0.0181,-0.1818, 0.2909,-0.109, -0.0181],
                vec![-0.0090, 0.0909,-0.0204,-0.1954, 0.134 ],
                vec![-0.1181, 0.1818,-0.0159,-0.0409,-0.0068],
                vec![ 0.4236,-2.2363,-0.5468, 0.9081, 1.2513],
                vec![-0.269,  1.6909, 0.3945,-0.5854,-1.0309]
            ]
        )
    };
}