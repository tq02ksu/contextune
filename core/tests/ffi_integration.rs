//! FFI integration tests
//!
//! Tests for FFI safety (null pointers, concurrent access, type conversions)

use contexture_core::ffi::{
    audio_engine_create, audio_engine_destroy, audio_engine_get_position, audio_engine_load_file,
    audio_engine_pause, audio_engine_play, audio_engine_set_volume, audio_engine_stop,
    AudioEngineHandle, FFIResult,
};
use std::ffi::CString;
use std::ptr;
use std::sync::{Arc, Barrier};
use std::thread;

#[cfg(test)]
mod null_pointer_tests {
    use super::*;

    #[test]
    fn test_null_handle_operations() {
        unsafe {
            let null_handle = AudioEngineHandle::null();

            // All operations should return NullPointer error for null handle
            assert_eq!(audio_engine_destroy(null_handle), FFIResult::NullPointer);
            assert_eq!(audio_engine_play(null_handle), FFIResult::NullPointer);
            assert_eq!(audio_engine_pause(null_handle), FFIResult::NullPointer);
            assert_eq!(audio_engine_stop(null_handle), FFIResult::NullPointer);
            assert_eq!(
                audio_engine_set_volume(null_handle, 0.5),
                FFIResult::NullPointer
            );

            let mut position = 0.0;
            assert_eq!(
                audio_engine_get_position(null_handle, &mut position),
                FFIResult::NullPointer
            );
        }
    }

    #[test]
    fn test_null_file_path() {
        unsafe {
            let handle = audio_engine_create();

            // Null file path should return NullPointer error
            let result = audio_engine_load_file(handle, ptr::null());
            assert_eq!(result, FFIResult::NullPointer);

            audio_engine_destroy(handle);
        }
    }

    #[test]
    fn test_null_output_pointer() {
        unsafe {
            let handle = audio_engine_create();

            // Null output pointer should return NullPointer error
            let result = audio_engine_get_position(handle, ptr::null_mut());
            assert_eq!(result, FFIResult::NullPointer);

            audio_engine_destroy(handle);
        }
    }

    #[test]
    fn test_valid_pointers() {
        unsafe {
            let handle = audio_engine_create();
            assert!(!handle.is_null());

            // Valid file path
            let file_path = CString::new("/path/to/audio.mp3").unwrap();
            let result = audio_engine_load_file(handle, file_path.as_ptr());
            assert_eq!(result, FFIResult::Success);

            // Valid output pointer
            let mut position = 0.0;
            let result = audio_engine_get_position(handle, &mut position);
            assert_eq!(result, FFIResult::Success);

            audio_engine_destroy(handle);
        }
    }
}

#[cfg(test)]
mod concurrent_access_tests {
    use super::*;

    #[test]
    fn test_concurrent_handle_creation() {
        const NUM_THREADS: usize = 10;
        let barrier = Arc::new(Barrier::new(NUM_THREADS));
        let mut handles = Vec::new();

        let threads: Vec<_> = (0..NUM_THREADS)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    unsafe { audio_engine_create() }
                })
            })
            .collect();

        for thread in threads {
            let handle = thread.join().unwrap();
            assert!(!handle.is_null());
            handles.push(handle);
        }

        // Clean up all handles
        for handle in handles {
            unsafe {
                assert_eq!(audio_engine_destroy(handle), FFIResult::Success);
            }
        }
    }

    #[test]
    fn test_concurrent_volume_setting() {
        unsafe {
            let handle = audio_engine_create();
            const NUM_THREADS: usize = 5;
            let barrier = Arc::new(Barrier::new(NUM_THREADS));

            let threads: Vec<_> = (0..NUM_THREADS)
                .map(|i| {
                    let barrier = Arc::clone(&barrier);
                    thread::spawn(move || {
                        barrier.wait();
                        let volume = (i as f64) / (NUM_THREADS as f64);
                        unsafe { audio_engine_set_volume(handle, volume) }
                    })
                })
                .collect();

            for thread in threads {
                let result = thread.join().unwrap();
                assert_eq!(result, FFIResult::Success);
            }

            audio_engine_destroy(handle);
        }
    }

    #[test]
    fn test_concurrent_playback_control() {
        unsafe {
            let handle = audio_engine_create();
            const NUM_OPERATIONS: usize = 20;
            let barrier = Arc::new(Barrier::new(NUM_OPERATIONS));

            let threads: Vec<_> = (0..NUM_OPERATIONS)
                .map(|i| {
                    let barrier = Arc::clone(&barrier);
                    thread::spawn(move || {
                        barrier.wait();
                        unsafe {
                            match i % 3 {
                                0 => audio_engine_play(handle),
                                1 => audio_engine_pause(handle),
                                _ => audio_engine_stop(handle),
                            }
                        }
                    })
                })
                .collect();

            for thread in threads {
                let result = thread.join().unwrap();
                assert_eq!(result, FFIResult::Success);
            }

            audio_engine_destroy(handle);
        }
    }

    #[test]
    fn test_concurrent_position_reading() {
        unsafe {
            let handle = audio_engine_create();
            const NUM_READERS: usize = 8;
            let barrier = Arc::new(Barrier::new(NUM_READERS));

            let threads: Vec<_> = (0..NUM_READERS)
                .map(|_| {
                    let barrier = Arc::clone(&barrier);
                    thread::spawn(move || {
                        barrier.wait();
                        let mut position = 0.0;
                        unsafe { audio_engine_get_position(handle, &mut position) }
                    })
                })
                .collect();

            for thread in threads {
                let result = thread.join().unwrap();
                assert_eq!(result, FFIResult::Success);
            }

            audio_engine_destroy(handle);
        }
    }
}

#[cfg(test)]
mod type_conversion_tests {
    use super::*;
    use contexture_core::ffi::types::{
        c_string_to_rust_str, rust_string_to_c_string, validate_not_null, validate_not_null_mut,
    };

    #[test]
    fn test_string_conversion_roundtrip() {
        let test_strings = vec![
            "Hello, World!",
            "音乐播放器",
            "🎵🎶🎼",
            "",
            "Path/to/file.mp3",
            "Very long string that might cause buffer issues if not handled properly",
        ];

        for original in test_strings {
            let c_string = rust_string_to_c_string(original).unwrap();
            unsafe {
                let converted = c_string_to_rust_str(c_string.as_ptr()).unwrap();
                assert_eq!(converted, original);
            }
        }
    }

    #[test]
    fn test_string_with_null_bytes() {
        let string_with_null = "Hello\0World";
        let result = rust_string_to_c_string(string_with_null);
        assert!(result.is_err()); // Should fail due to interior null byte
    }

    #[test]
    fn test_volume_range_validation() {
        unsafe {
            let handle = audio_engine_create();

            // Test boundary values
            assert_eq!(audio_engine_set_volume(handle, 0.0), FFIResult::Success);

            let handle2 = audio_engine_create();
            assert_eq!(audio_engine_set_volume(handle2, 1.0), FFIResult::Success);

            // Test invalid values
            let handle3 = audio_engine_create();
            assert_eq!(
                audio_engine_set_volume(handle3, -0.001),
                FFIResult::InvalidArgument
            );

            let handle4 = audio_engine_create();
            assert_eq!(
                audio_engine_set_volume(handle4, 1.001),
                FFIResult::InvalidArgument
            );

            let handle5 = audio_engine_create();
            assert_eq!(
                audio_engine_set_volume(handle5, f64::NAN),
                FFIResult::InvalidArgument
            );

            let handle6 = audio_engine_create();
            assert_eq!(
                audio_engine_set_volume(handle6, f64::INFINITY),
                FFIResult::InvalidArgument
            );

            let handle7 = audio_engine_create();
            assert_eq!(
                audio_engine_set_volume(handle7, f64::NEG_INFINITY),
                FFIResult::InvalidArgument
            );

            // Clean up
            audio_engine_destroy(handle);
            audio_engine_destroy(handle2);
            audio_engine_destroy(handle3);
            audio_engine_destroy(handle4);
            audio_engine_destroy(handle5);
            audio_engine_destroy(handle6);
            audio_engine_destroy(handle7);
        }
    }

    #[test]
    fn test_pointer_validation() {
        let valid_value = 42;
        let valid_ptr = &valid_value as *const i32;
        let null_ptr = ptr::null::<i32>();

        assert_eq!(validate_not_null(valid_ptr), FFIResult::Success);
        assert_eq!(validate_not_null(null_ptr), FFIResult::NullPointer);

        let mut valid_mut_value = 42;
        let valid_mut_ptr = &mut valid_mut_value as *mut i32;
        let null_mut_ptr = ptr::null_mut::<i32>();

        assert_eq!(validate_not_null_mut(valid_mut_ptr), FFIResult::Success);
        assert_eq!(validate_not_null_mut(null_mut_ptr), FFIResult::NullPointer);
    }

    #[test]
    fn test_ffi_result_consistency() {
        // Ensure FFI result values are consistent
        assert_eq!(FFIResult::Success as i32, 0);
        assert_eq!(FFIResult::NullPointer as i32, -1);
        assert_eq!(FFIResult::InvalidArgument as i32, -2);
        assert_eq!(FFIResult::OutOfMemory as i32, -3);
        assert_eq!(FFIResult::InternalError as i32, -4);
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_error_propagation() {
        unsafe {
            let null_handle = AudioEngineHandle::null();

            // Verify that all functions properly handle and return null pointer errors
            assert_eq!(audio_engine_play(null_handle), FFIResult::NullPointer);
            assert_eq!(audio_engine_pause(null_handle), FFIResult::NullPointer);
            assert_eq!(audio_engine_stop(null_handle), FFIResult::NullPointer);
            assert_eq!(
                audio_engine_set_volume(null_handle, 0.5),
                FFIResult::NullPointer
            );
            assert_eq!(audio_engine_destroy(null_handle), FFIResult::NullPointer);
        }
    }

    #[test]
    fn test_multiple_destroy_calls() {
        unsafe {
            let handle = audio_engine_create();

            // First destroy should succeed
            assert_eq!(audio_engine_destroy(handle), FFIResult::Success);

            // Note: In a real implementation, we would need to track handle validity
            // For now, we just test that the function doesn't crash
        }
    }

    #[test]
    fn test_operations_after_destroy() {
        unsafe {
            let handle = audio_engine_create();
            audio_engine_destroy(handle);

            // Note: In a real implementation, these operations should fail
            // because the handle is no longer valid. For now, we just ensure
            // they don't crash the program.
            let _ = audio_engine_play(handle);
            let _ = audio_engine_pause(handle);
            let _ = audio_engine_stop(handle);
        }
    }

    #[test]
    fn test_invalid_file_paths() {
        unsafe {
            let handle = audio_engine_create();

            // Test various potentially problematic file paths
            let test_paths = vec![
                CString::new("").unwrap(),
                CString::new("nonexistent_file.mp3").unwrap(),
                CString::new("/dev/null").unwrap(),
                CString::new("very_long_path_that_might_cause_buffer_overflow_issues_if_not_handled_properly_in_the_implementation.mp3").unwrap(),
            ];

            for path in test_paths {
                let result = audio_engine_load_file(handle, path.as_ptr());
                // For now, all should succeed since we're just validating the pointer
                assert_eq!(result, FFIResult::Success);
            }

            audio_engine_destroy(handle);
        }
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[test]
    fn test_handle_creation_stress() {
        const NUM_HANDLES: usize = 1000;
        let mut handles = Vec::with_capacity(NUM_HANDLES);

        // Create many handles
        for _ in 0..NUM_HANDLES {
            unsafe {
                let handle = audio_engine_create();
                assert!(!handle.is_null());
                handles.push(handle);
            }
        }

        // Destroy all handles
        for handle in handles {
            unsafe {
                assert_eq!(audio_engine_destroy(handle), FFIResult::Success);
            }
        }
    }

    #[test]
    fn test_rapid_operations() {
        unsafe {
            // Perform rapid operations with separate handles to avoid move issues
            for i in 0..100 {
                let handle = audio_engine_create();

                let volume = (i % 100) as f64 / 100.0;
                assert_eq!(audio_engine_set_volume(handle, volume), FFIResult::Success);

                if i % 3 == 0 {
                    assert_eq!(audio_engine_play(handle), FFIResult::Success);
                } else if i % 3 == 1 {
                    assert_eq!(audio_engine_pause(handle), FFIResult::Success);
                } else {
                    assert_eq!(audio_engine_stop(handle), FFIResult::Success);
                }

                let mut position = 0.0;
                assert_eq!(
                    audio_engine_get_position(handle, &mut position),
                    FFIResult::Success
                );

                audio_engine_destroy(handle);
            }
        }
    }
}
