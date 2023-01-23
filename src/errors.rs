use std::{error::Error, fmt::{self, Display}};

/// More concise syntax for implementing `Error` and `Display` for both structs and enums
macro_rules! impl_err {
    ($s:ty, $e:expr) => {
        impl Error for $s {}
        impl Display for $s {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, $e)
            }
        }
    };
    ($s:ty, $($p:path, $e:expr),*) => {
        impl Error for $s {}
        impl Display for $s {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $($p => write!(f, $e),)*
                }
            }
        }
    };
}

#[derive(Debug)]
pub struct NxNInversionError;
impl_err!(
    NxNInversionError, 
    "matrix could not be inverted"
);

#[derive(Debug)]
pub struct NxNCreationError;
impl_err!(
    NxNCreationError,
    "columns did not form an nxn matrix"
);

#[derive(Debug)]
pub struct NxNMultiplicationError;
impl_err!(
    NxNMultiplicationError,
    "failed to multiply matrix by vector."
);

#[derive(Debug)]
pub struct VecMultiplicationError;
impl_err!(
    VecMultiplicationError,
    "tried to dot vectors of different sizes"
);

#[derive(Debug)]
pub enum SolverDivisionByZeroError {
    NewtonRaphsonDivByZeroError,
    MVNewtonRaphsonDivByZeroError,
    GoldSectionSearchDivByZeroError
}
impl_err!(
    SolverDivisionByZeroError,
    SolverDivisionByZeroError::NewtonRaphsonDivByZeroError,     "newton-raphson solver tried to divide by zero",
    SolverDivisionByZeroError::MVNewtonRaphsonDivByZeroError,   "multivariate newton-raphson solver tried to divide by zero",
    SolverDivisionByZeroError::GoldSectionSearchDivByZeroError, "golden section search solver tried to divide by zero"
);

#[derive(Debug)]
pub struct RoundingError;
impl_err!(
    RoundingError,
    "number not valid for rounding"
);

/// Error type for issues with the conditional expression formatter in `nexsys::parsing`
#[derive(Debug)]
pub enum ConditionFormatError {
    ConditionalSyntax,
    Comparator
}
impl_err!(
    ConditionFormatError,
    ConditionFormatError::ConditionalSyntax,    "conditional statement failed to compile",
    ConditionFormatError::Comparator,           "invalid comparison operator. valid operators are: <, >, <=, >=, ==, !="
);

#[derive(Debug)]
pub struct ConversionFormatError;
impl_err!(
    ConversionFormatError,
    "conversion factor failed to compile"
);

#[derive(Debug)]
pub struct ConstFormatError;
impl_err!(
    ConstFormatError,
    "constant failed to compile"
);

#[derive(Debug)]
pub struct UnitConversionError;
impl_err!(
    UnitConversionError,
    "failed to identify conversion factors"
);

#[derive(Debug)]
pub struct SolverConvergenceError;
impl_err!(
    SolverConvergenceError,
    "solver algorithm did not converge. consider allowing non-convergent solutions, or try to remove discontinuities from your system"
);