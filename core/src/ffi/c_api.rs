//! C-compatible API
//!
//! Exports C-compatible functions for FFI

use crate::ffi::types::{validate_not_null, validate_not_null_mut, AudioEngineHandle, FFIResult};
use std::os::raw::{c_char, c_double};

/// Create a new audio engine instance
///
/// # Safety
/// The caller must call `audio_engine_destroy` to free the returned handle.
#[no_mangle]
pub unsafe extern "C" fn audio_engine_create() -> AudioEngineHandle {
    // For now, return a dummy handle
    // This will be properly implemented in Phase 1
    AudioEngineHandle {
        inner: 0x1234 as *mut std::ffi::c_void,
    }
}

/// Destroy an audio engine instance
///
/// # Safety
/// The handle must be a valid handle returned by `audio_engine_create`.
/// After calling this function, the handle becomes invalid.
#[no_mangle]
pub unsafe extern "C" fn audio_engine_destroy(handle: AudioEngineHandle) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    // For now, just validate the handle
    // Actual cleanup will be implemented in Phase 1
    FFIResult::Success
}

/// Load an audio file
///
/// # Safety
/// - `handle` must be a valid audio engine handle
/// - `file_path` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn audio_engine_load_file(
    handle: AudioEngineHandle,
    file_path: *const c_char,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null(file_path).into() {
        return result;
    }

    // For now, just validate parameters
    // Actual implementation will be in Phase 1
    FFIResult::Success
}

/// Set volume
///
/// # Safety
/// `handle` must be a valid audio engine handle
#[no_mangle]
pub unsafe extern "C" fn audio_engine_set_volume(
    handle: AudioEngineHandle,
    volume: c_double,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if !(0.0..=1.0).contains(&volume) || volume.is_nan() || volume.is_infinite() {
        return FFIResult::InvalidArgument;
    }

    // For now, just validate parameters
    // Actual implementation will be in Phase 1
    FFIResult::Success
}

/// Get current playback position
///
/// # Safety
/// - `handle` must be a valid audio engine handle
/// - `position` must be a valid pointer to write the result
#[no_mangle]
pub unsafe extern "C" fn audio_engine_get_position(
    handle: AudioEngineHandle,
    position: *mut c_double,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null_mut(position).into() {
        return result;
    }

    // For now, return a dummy position
    *position = 0.0;
    FFIResult::Success
}

/// Play audio
///
/// # Safety
/// `handle` must be a valid audio engine handle
#[no_mangle]
pub unsafe extern "C" fn audio_engine_play(handle: AudioEngineHandle) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    // For now, just validate the handle
    FFIResult::Success
}

/// Pause audio
///
/// # Safety
/// `handle` must be a valid audio engine handle
#[no_mangle]
pub unsafe extern "C" fn audio_engine_pause(handle: AudioEngineHandle) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    // For now, just validate the handle
    FFIResult::Success
}

/// Stop audio
///
/// # Safety
/// `handle` must be a valid audio engine handle
#[no_mangle]
pub unsafe extern "C" fn audio_engine_stop(handle: AudioEngineHandle) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    // For now, just validate the handle
    FFIResult::Success
}

impl From<FFIResult> for Result<(), FFIResult> {
    fn from(result: FFIResult) -> Self {
        match result {
            FFIResult::Success => Ok(()),
            error => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_engine_create_destroy() {
        unsafe {
            let handle = audio_engine_create();
            assert!(!handle.is_null());

            let result = audio_engine_destroy(handle);
            assert_eq!(result, FFIResult::Success);
        }
    }

    #[test]
    fn test_destroy_null_handle() {
        unsafe {
            let null_handle = AudioEngineHandle::null();
            let result = audio_engine_destroy(null_handle);
            assert_eq!(result, FFIResult::NullPointer);
        }
    }

    #[test]
    fn test_volume_validation() {
        unsafe {
            let handle = audio_engine_create();

            // Valid volume
            assert_eq!(audio_engine_set_volume(handle, 0.5), FFIResult::Success);
            assert_eq!(audio_engine_set_volume(handle, 0.0), FFIResult::Success);
            assert_eq!(audio_engine_set_volume(handle, 1.0), FFIResult::Success);

            // Invalid volume
            assert_eq!(
                audio_engine_set_volume(handle, -0.1),
                FFIResult::InvalidArgument
            );
            assert_eq!(
                audio_engine_set_volume(handle, 1.1),
                FFIResult::InvalidArgument
            );

            // Null handle
            let null_handle = AudioEngineHandle::null();
            assert_eq!(
                audio_engine_set_volume(null_handle, 0.5),
                FFIResult::NullPointer
            );
        }
    }

    #[test]
    fn test_get_position() {
        unsafe {
            let handle = audio_engine_create();
            let mut position = 0.0;

            let result = audio_engine_get_position(handle, &mut position);
            assert_eq!(result, FFIResult::Success);
            assert_eq!(position, 0.0);

            // Test null pointer
            let result = audio_engine_get_position(handle, std::ptr::null_mut());
            assert_eq!(result, FFIResult::NullPointer);

            // Test null handle
            let null_handle = AudioEngineHandle::null();
            let result = audio_engine_get_position(null_handle, &mut position);
            assert_eq!(result, FFIResult::NullPointer);
        }
    }
}
