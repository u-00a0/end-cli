use std::{ffi::c_void, mem::ManuallyDrop};

use crate::dto::envelope_json;
use crate::{BootstrapPayload, Error, Lang, Result, SolvePayload, bootstrap, solve_from_aic_toml};

/// FFI slice representation.
#[repr(C)]
struct Slice {
    ptr: *const u8,
    len: usize,
    cap: usize,
}

impl Slice {
    /// Create a Slice from a string.
    /// Reuses the string allocation and transfers ownership to the FFI caller.
    fn from_string(s: String) -> *mut c_void {
        let mut bytes = ManuallyDrop::new(s.into_bytes());
        let ptr = bytes.as_mut_ptr() as *const u8;
        let len = bytes.len();
        let cap = bytes.capacity();
        Box::into_raw(Box::new(Self { ptr, len, cap })).cast::<c_void>()
    }

    /// # Safety
    /// `ptr` must point to a valid `Slice` allocated by the caller.
    unsafe fn from_raw<'a>(ptr: *const c_void, name: &'static str) -> Result<&'a Self> {
        if ptr.is_null() {
            return Err(Error::NullPointer { name });
        }
        // SAFETY: caller promises `ptr` points to a valid `Slice`.
        Ok(unsafe { &*ptr.cast::<Self>() })
    }

    fn as_str(&self, name: &'static str) -> Result<&str> {
        if self.ptr.is_null() {
            return Err(Error::NullPointer { name });
        }
        // SAFETY: caller guarantees `(ptr, len)` is a valid byte slice.
        let bytes = unsafe { std::slice::from_raw_parts(self.ptr, self.len) };
        std::str::from_utf8(bytes).map_err(|_| Error::InvalidUtf8 { name })
    }
}

unsafe fn read_lang(lang: *const c_void) -> Result<Lang> {
    let lang = unsafe { Slice::from_raw(lang, "lang")? };
    Lang::parse(lang.as_str("lang")?)
}

unsafe fn read_aic_toml<'a>(aic_toml: *const c_void) -> Result<&'a str> {
    let aic_toml = unsafe { Slice::from_raw(aic_toml, "aic_toml")? };
    aic_toml.as_str("aic_toml")
}

#[unsafe(no_mangle)]
/// Free a Slice allocated by Rust.
///
/// # Safety
/// `slice` must be a pointer previously returned by `end_web_bootstrap` or
/// `end_web_solve_from_aic_toml`, and must be freed exactly once.
pub unsafe extern "C" fn end_web_free_slice(slice: *mut c_void) {
    if slice.is_null() {
        return;
    }
    // SAFETY: caller ensures `slice` came from `Slice::from_string` and is freed once.
    let slice = unsafe { Box::from_raw(slice.cast::<Slice>()) };
    if slice.cap == 0 || slice.ptr.is_null() {
        return;
    }
    // SAFETY: `from_string` built `(ptr, len, cap)` from a `Vec<u8>`.
    unsafe {
        _ = Vec::from_raw_parts(slice.ptr as *mut u8, slice.len, slice.cap);
    }
}

#[unsafe(no_mangle)]
/// Build bootstrap payload JSON string (`catalog` + default `aic.toml`).
///
/// # Safety
/// `lang` must be a valid pointer to a Slice.
pub unsafe extern "C" fn end_web_bootstrap(lang: *const c_void) -> *mut c_void {
    let result = unsafe { read_lang(lang) }.and_then(bootstrap);
    Slice::from_string(envelope_json::<BootstrapPayload>(result))
}

#[unsafe(no_mangle)]
/// Run optimization from `aic.toml` text and return JSON result.
///
/// # Safety
/// `lang` and `aic_toml` must be valid pointers to Slice.
pub unsafe extern "C" fn end_web_solve_from_aic_toml(
    lang: *const c_void,
    aic_toml: *const c_void,
) -> *mut c_void {
    let result = (|| {
        let lang = unsafe { read_lang(lang)? };
        let aic_toml = unsafe { read_aic_toml(aic_toml)? };
        solve_from_aic_toml(lang, aic_toml)
    })();
    Slice::from_string(envelope_json::<SolvePayload>(result))
}
