#[cfg(feature = "c_ffi")] 
use std::ffi::{CStr, CString, c_char, c_int, c_float};
use crate::solve;

/// Nexsys solver function exposed to C/C++
/// For ease of use, solution 
#[no_mangle]
pub extern "C" fn c_solve(
    system: *const c_char, 
    tolerance: c_float, 
    max_iterations: c_int, 
    allow_nonconvergence: bool
) {

    let sys = unsafe {
        String::from_utf8_lossy(
            CStr::from_ptr(system).to_bytes())
            .to_string()
    };

    match solve(
        &sys, 
        Some(tolerance as f64), 
        Some(max_iterations as usize), 
        allow_nonconvergence
    ) {
        Err(e) => {
            CString::new(format!("{e}"))
                .expect("rust error: failed to format error as CString")
                .as_ptr();
        },
        Ok(o) => {
            let (soln, _) = o;
            let mut ans = String::new();

            for (k, v) in soln {
                ans += &format!("{k}={}\n", v.as_f64());
            }

            CString::new(ans).expect("rust error: failed to format solution as CString")
        }
    }    
}