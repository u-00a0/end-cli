use std::ffi::c_void;

use end_web::{end_web_bootstrap, end_web_free_slice, end_web_solve_from_aic_toml};

fn main() {
    let _ = (
        end_web_bootstrap as unsafe extern "C" fn(*const c_void) -> *mut c_void,
        end_web_solve_from_aic_toml
            as unsafe extern "C" fn(*const c_void, *const c_void) -> *mut c_void,
        end_web_free_slice as unsafe extern "C" fn(*mut c_void),
    );
}
