//! FFI type conversions
//!
//! Type conversions between Rust and C/Java types

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

/// FFI-safe result type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FFIResult {
    /// Operation completed successfully
    Success = 0,
    /// A null pointer was passed where a valid pointer was expected
    NullPointer = -1,
    /// An invalid argument was provided
    InvalidArgument = -2,
    /// Out of memory error
    OutOfMemory = -3,
    /// Internal error occurred
    InternalError = -4,
}

/// FFI-safe audio engine handle
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AudioEngineHandle {
    pub(crate) inner: *mut c_void,
}

// SAFETY: AudioEngineHandle is just a pointer wrapper for FFI
// The actual thread safety is handled by the underlying implementation
unsafe impl Send for AudioEngineHandle {}
unsafe impl Sync for AudioEngineHandle {}

impl AudioEngineHandle {
    /// Create a null handle
    pub fn null() -> Self {
        Self {
            inner: std::ptr::null_mut(),
        }
    }

    /// Check if the handle is null
    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }
}

/// Convert Rust string to C string
pub fn rust_string_to_c_string(s: &str) -> Result<CString, std::ffi::NulError> {
    CString::new(s)
}

/// Convert C string to Rust string
/// 
/// # Safety
/// The caller must ensure that `ptr` is a valid null-terminated C string
/// and that it remains valid for the duration of the returned string slice.
pub unsafe fn c_string_to_rust_str<'a>(ptr: *const c_char) -> Result<&'a str, std::str::Utf8Error> {
    if ptr.is_null() {
        return Ok("");
    }
    
    let c_str = CStr::from_ptr(ptr);
    c_str.to_str()
}

/// Validate pointer is not null
pub fn validate_not_null<T>(ptr: *const T) -> FFIResult {
    if ptr.is_null() {
        FFIResult::NullPointer
    } else {
        FFIResult::Success
    }
}

/// Validate mutable pointer is not null
pub fn validate_not_null_mut<T>(ptr: *mut T) -> FFIResult {
    if ptr.is_null() {
        FFIResult::NullPointer
    } else {
        FFIResult::Success
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::raw::c_int;

    #[test]
    fn test_ffi_result_values() {
        assert_eq!(FFIResult::Success as c_int, 0);
        assert_eq!(FFIResult::NullPointer as c_int, -1);
        assert_eq!(FFIResult::InvalidArgument as c_int, -2);
        assert_eq!(FFIResult::OutOfMemory as c_int, -3);
        assert_eq!(FFIResult::InternalError as c_int, -4);
    }

    #[test]
    fn test_audio_engine_handle() {
        let null_handle = AudioEngineHandle::null();
        assert!(null_handle.is_null());

        let valid_handle = AudioEngineHandle {
            inner: 0x1234 as *mut c_void,
        };
        assert!(!valid_handle.is_null());
    }

    #[test]
    fn test_validate_not_null() {
        let valid_ptr = &42 as *const i32;
        let null_ptr = std::ptr::null::<i32>();

        assert_eq!(validate_not_null(valid_ptr), FFIResult::Success);
        assert_eq!(validate_not_null(null_ptr), FFIResult::NullPointer);
    }

    #[test]
    fn test_validate_not_null_mut() {
        let mut value = 42;
        let valid_ptr = &mut value as *mut i32;
        let null_ptr = std::ptr::null_mut::<i32>();

        assert_eq!(validate_not_null_mut(valid_ptr), FFIResult::Success);
        assert_eq!(validate_not_null_mut(null_ptr), FFIResult::NullPointer);
    }

    #[test]
    fn test_string_conversions() {
        let rust_str = "Hello, World!";
        let c_string = rust_string_to_c_string(rust_str).unwrap();
        
        unsafe {
            let converted_back = c_string_to_rust_str(c_string.as_ptr()).unwrap();
            assert_eq!(converted_back, rust_str);
        }
    }

    #[test]
    fn test_null_c_string_conversion() {
        unsafe {
            let result = c_string_to_rust_str(std::ptr::null());
            assert_eq!(result.unwrap(), "");
        }
    }
}
