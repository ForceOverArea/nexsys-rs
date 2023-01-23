use std::{error::Error};
use crate::{
    mvcalc::*, 
    errors::{NxNInversionError, NxNCreationError}
};

/// An n x n matrix with a `Vec` containing the variables in each column if they are given.
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct NxN {
    pub size: usize,
    pub vars: Option<Vec<String>>,  // Optional header column for annotating which variables are 
    mat: Vec<Vec<f64>>
}
impl NxN {

    /// Initializes an NxN identity matrix of the specified size
    /// # Example
    /// ```
    /// use nexsys::mvcalc::NxN;
    /// 
    /// let my_matrix = NxN::identity(3);
    /// let check = vec![ 
    ///     vec![1.0, 0.0, 0.0], 
    ///     vec![0.0, 1.0, 0.0], 
    ///     vec![0.0, 0.0, 1.0] 
    /// ];
    /// 
    /// assert_eq!(my_matrix.to_vec(), check);
    /// ```
    pub fn identity(size: usize) -> NxN {
        let mut mat = vec![];
        for i in 0..size {
            let mut col = vec![];
            for j in 0..size {
                if i == j {
                    col.push(1_f64);
                } else {
                    col.push(0_f64);
                }
            }
            mat.push(col);
        }
        NxN { size, mat, vars: None }
    }

    /// Initializes an NxN matrix of given values from a `Vec<Vec<f64>>`
    /// # Example
    /// ```
    /// use nexsys::mvcalc::NxN;
    /// 
    /// let my_vars = vec!["x", "y", "z"];
    /// let my_cols = vec![
    ///     vec![1.0, 2.0, 3.0],
    ///     vec![4.0, 5.0, 6.0],
    ///     vec![7.0, 8.0, 9.0]
    /// ];
    ///  
    /// let my_matrix = NxN::from_cols(
    ///     my_cols.clone(), 
    ///     Some(my_vars)
    /// ).unwrap();
    /// 
    /// assert_eq!(my_matrix.to_vec(), my_cols);
    /// ```
    pub fn from_cols<T>(cols: Vec<Vec<T>>, col_vars: Option<Vec<&str>>) -> Result<NxN, Box<dyn Error>>
    where
        T: Into<f64> + Copy
    {
        let mut vars = None;

        if let Some(v) = col_vars {
            vars = Some(v.iter().map(|&i| i.to_string()).collect());
        }

        if cols.len() != cols[0].len() {
            Err(Box::new(NxNCreationError))
        } else {
            let size = cols.len();
            let mat = cols.iter().map(
                |i| {
                    i.iter()
                    .map(|&j| j.into())
                    .collect()
                }
            ).collect();
            Ok(NxN { size, vars, mat })
        }
    }

    /// Mutates a row, scaling it by the given value
    /// # Example
    /// ```
    /// use nexsys::mvcalc::NxN;
    /// 
    /// let mut my_matrix = NxN::identity(3);
    /// 
    /// let check = vec![ 
    ///     vec![1.0, 0.0, 0.0], 
    ///     vec![0.0, 2.0, 0.0], 
    ///     vec![0.0, 0.0, 1.0] 
    /// ];
    /// 
    /// my_matrix.scale_row(1, 2);
    /// 
    /// assert_eq!(my_matrix.to_vec(), check);
    /// ```
    pub fn scale_row<T>(&mut self, row: usize, scalar: T)
    where
        T: Into<f64> + Copy
    { 
        let n = self.size;
        for i in 0..n {
            self.mat[i][row] *= scalar.into();
        }
    }

    /// Adds a given row vector to a row in the matrix
    /// # Example
    /// ```
    /// use nexsys::mvcalc::NxN;
    /// 
    /// let mut my_matrix = NxN::identity(3);
    /// let check = vec![ 
    ///     vec![1.0, 2.0, 0.0], 
    ///     vec![0.0, 3.0, 0.0], 
    ///     vec![0.0, 2.0, 1.0] 
    /// ];
    /// my_matrix.add_to_row(1, &vec![2, 2, 2]);
    /// assert_eq!(my_matrix.to_vec(), check);
    /// ```
    pub fn add_to_row<T>(&mut self, row: usize, vec: &[T]) 
    where 
        T: Into<f64> + Copy,
        f64: From<T>
    {
        let n = self.size;
        for (i, v) in vec.iter().enumerate().take(n) {
            self.mat[i][row] += f64::from(*v);
        }
    }

    /// Returns a row from the matrix
    /// # Example
    /// ```
    /// use nexsys::mvcalc::NxN;
    /// 
    /// let mut my_matrix = NxN::identity(3);
    /// 
    /// let check = vec![0.0, 0.0, 1.0];
    /// 
    /// assert_eq!(my_matrix.get_row(2), check);
    /// ```
    pub fn get_row(&self, row: usize) -> Vec<f64> {
        let n = self.size;
        let mut res = vec![];
        for i in 0..n {
            res.push(self.mat[i][row]);
        }
        res
    }

    /// Inversion method for 2x2 matrices
    fn invert_2x2(&mut self) -> Result<(), Box<dyn Error>> {
        
        let m = &self.mat;
        
        let m11 = m[0][0];
        let m12 = m[1][0];
        let m21 = m[0][1];
        let m22 = m[1][1];

        let det = m11*m22 - m12*m21;

        if det == 0_f64 {
            return Err(Box::new(NxNInversionError))
        }
    
        self.mat = vec![
            vec![ // column 1
                m22/det, 
                -m21/det
            ],
            vec![ // column 2
                -m12/det,  
                m11/det
            ]
        ];

        Ok(())    
    }

    /// Inversion method for 3x3 matrices
    fn invert_3x3(&mut self) -> Result<(), Box<dyn Error>> {

        let m = &self.mat;
        let m11 = m[0][0];
        let m12 = m[1][0];
        let m13 = m[2][0];
        let m21 = m[0][1];
        let m22 = m[1][1];
        let m23 = m[2][1];
        let m31 = m[0][2];
        let m32 = m[1][2];
        let m33 = m[2][2];

        let det:f64 = m11*m22*m33 + m21*m32*m13 + m31*m12*m23 - m11*m32*m23 - m31*m22*m13 - m21*m12*m33;

        if det == 0_f64 {
            return Err(Box::new(NxNInversionError))
        }

        self.mat = vec![
            vec![ // column 1
                (m22*m33 - m23*m32)/det, 
                (m23*m31 - m21*m33)/det, 
                (m21*m32 - m22*m31)/det
            ],
            vec![ // column 2
                (m13*m32 - m12*m33)/det,
                (m11*m33 - m13*m31)/det,
                (m12*m31 - m11*m32)/det
            ],
            vec![ // column 3
                (m12*m23 - m13*m22)/det,
                (m13*m21 - m11*m23)/det,
                (m11*m22 - m12*m21)/det 
            ],
        ];

        Ok(())
    }

    /// Inversion method for 4x4 matrices
    fn invert_4x4(&mut self) -> Result<(), Box<dyn Error>> {
        let m = &self.mat;
        
        let a11 = m[0][0];
        let a12 = m[1][0];
        let a13 = m[2][0];
        let a14 = m[3][0];
        let a21 = m[0][1];
        let a22 = m[1][1];
        let a23 = m[2][1];
        let a24 = m[3][1];
        let a31 = m[0][2];
        let a32 = m[1][2];
        let a33 = m[2][2];
        let a34 = m[3][2];
        let a41 = m[0][3];
        let a42 = m[1][3];
        let a43 = m[2][3];
        let a44 = m[3][3];

        let det: f64 =  a11*a22*a33*a44 + a11*a23*a34*a42 + a11*a24*a32*a43 +
                        a12*a21*a34*a43 + a12*a23*a31*a44 + a12*a24*a33*a41 + 
                        a13*a21*a32*a44 + a13*a22*a34*a41 + a13*a24*a31*a42 + 
                        a14*a21*a33*a42 + a14*a22*a34*a43 + a14*a23*a32*a41 -
                        a11*a22*a34*a43 - a11*a23*a32*a44 - a11*a24*a33*a42 -
                        a12*a21*a33*a44 - a12*a23*a34*a41 - a12*a24*a31*a43 -
                        a13*a21*a34*a42 - a13*a22*a31*a44 - a13*a24*a32*a41 -
                        a14*a21*a32*a43 - a14*a22*a33*a41 - a14*a23*a31*a42;
                        
        if det == 0_f64 {
            return Err(Box::new(NxNInversionError))
        }

        let b11 = (a22*a33*a44 + a23*a34*a42 + a24*a32*a43 - a22*a34*a43 - a23*a32*a44 - a24*a33*a42) / det;
        let b12 = (a12*a34*a43 + a13*a32*a44 + a14*a33*a42 - a12*a33*a44 - a13*a34*a42 - a14*a32*a43) / det;
        let b13 = (a12*a23*a44 + a13*a24*a42 + a14*a22*a43 - a12*a24*a43 - a13*a22*a44 - a14*a23*a42) / det;
        let b14 = (a12*a24*a33 + a13*a22*a34 + a14*a23*a32 - a12*a23*a34 - a13*a24*a32 - a14*a22*a33) / det;
        let b21 = (a21*a34*a43 + a23*a31*a44 + a24*a33*a41 - a21*a33*a44 - a23*a34*a41 - a24*a31*a43) / det;
        let b22 = (a11*a33*a44 + a13*a34*a41 + a14*a31*a43 - a11*a34*a43 - a13*a31*a44 - a14*a33*a41) / det;
        let b23 = (a11*a24*a43 + a13*a21*a44 + a14*a23*a41 - a11*a23*a44 - a13*a24*a41 - a14*a21*a43) / det;
        let b24 = (a11*a23*a34 + a13*a24*a31 + a14*a21*a33 - a11*a24*a33 - a13*a21*a34 - a14*a23*a31) / det;
        let b31 = (a21*a32*a44 + a22*a34*a41 + a24*a31*a42 - a21*a34*a42 - a22*a31*a44 - a24*a32*a41) / det;
        let b32 = (a11*a34*a42 + a12*a31*a44 + a14*a32*a41 - a11*a32*a44 - a12*a34*a41 - a14*a31*a42) / det;
        let b33 = (a11*a22*a44 + a12*a24*a41 + a14*a21*a42 - a11*a24*a42 - a12*a21*a44 - a14*a22*a41) / det;
        let b34 = (a11*a24*a32 + a12*a21*a34 + a14*a22*a31 - a11*a22*a34 - a12*a24*a31 - a14*a21*a32) / det;
        let b41 = (a21*a33*a42 + a22*a31*a43 + a23*a32*a41 - a21*a32*a43 - a22*a33*a41 - a23*a31*a42) / det;
        let b42 = (a11*a32*a43 + a12*a33*a41 + a13*a31*a42 - a11*a33*a42 - a12*a31*a43 - a13*a32*a41) / det;
        let b43 = (a11*a23*a42 + a12*a21*a43 + a13*a22*a41 - a11*a22*a43 - a12*a23*a41 - a13*a21*a42) / det;
        let b44 = (a11*a22*a33 + a12*a23*a31 + a13*a21*a32 - a11*a23*a32 - a12*a21*a33 - a13*a22*a31) / det;

        self.mat = vec![
            vec![b11, b21, b31, b41],
            vec![b12, b22, b32, b42],
            vec![b13, b23, b33, b43],
            vec![b14, b24, b34, b44],     
        ];
        
        Ok(())
    }

    /// Inversion method for nxn matrices where n > 4
    fn invert_nxn(&mut self) -> Result<(), Box<dyn Error>> {
        let n = self.size;
        let mut inv = NxN::identity(n);

        for c in 0..n {
            for r in 0..n {
                if c == r {
                    continue; // guard clause against modifying the diagonal
                } else {
                    if self.mat[c][c] == 0_f64 { 
                        return Err(Box::new(NxNInversionError))
                    }
                    // get the scalar that needs to be applied to the row vector
                    let scalar = - self.mat[c][r] / self.mat[c][c];

                    // create the row vector to add to self & row vector to add to inv
                    let v = scale_vec(self.get_row(c), scalar);
                    let vi = scale_vec(inv.get_row(c), scalar);

                    self.add_to_row(r, &v); // add the vector to self
                    inv.add_to_row(r, &vi); // perform the same operation on the identity matrix
                }
            }
        }

        for i in 0..n {
            let scalar = 1.0 / self.mat[i][i];
            self.scale_row(i, scalar);
            inv.scale_row(i, scalar);
        }

        // println!("{:?}", self.mat);

        // Assign the identity matrix's values to self.mat
        self.mat = inv.to_vec();
        Ok(())
    }

    /// inverts the matrix, if possible. This method returns a result that
    /// indicates whether the inversion was successful or not.
    /// # Example
    /// ```
    /// use nexsys::mvcalc::NxN;
    /// 
    /// let mut my_matrix = NxN::from_cols(vec![ 
    ///    vec![-1.0, 1.0], 
    ///    vec![ 1.5,-1.0] 
    /// ], None).unwrap();
    /// 
    /// my_matrix.invert().unwrap();
    /// 
    /// let inverse = vec![ 
    ///     vec![2.0, 2.0], 
    ///     vec![3.0, 2.0] 
    /// ];
    /// 
    /// assert_eq!(my_matrix.to_vec(), inverse);
    /// ```
    pub fn invert(&mut self) -> Result<(), Box<dyn Error>> {

        // Different inversion methods are chosen to mitigate 
        // computational expense.
        if self.size == 2 {

            Ok(self.invert_2x2()?)
        
        } else if self.size == 3 {

            Ok(self.invert_3x3()?)

        } else if self.size == 4 {

            Ok(self.invert_4x4()?)

        } else {
        
            Ok(self.invert_nxn()?)
        
        }

    }

    /// Returns the matrix as `Vec<Vec<f64>>`, consuming the `self` value in the process
    pub fn to_vec(self) -> Vec<Vec<f64>> {
        self.mat
    }
}