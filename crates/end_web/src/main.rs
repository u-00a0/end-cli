use std::ffi::c_char;

use end_web::{end_web_bootstrap, end_web_free_c_string, end_web_solve_from_aic_toml};

fn main() {
    let _ = (
        end_web_bootstrap as unsafe extern "C" fn(*const c_char) -> *mut c_char,
        end_web_solve_from_aic_toml
            as unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_char,
        end_web_free_c_string as unsafe extern "C" fn(*mut c_char),
    );
}
