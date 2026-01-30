//! C-compatible API
//!
//! Exports C-compatible functions for FFI

use crate::audio::engine::{AudioEngine, AudioEngineInterface, AudioEvent, PlaybackState};
use crate::ffi::types::{
    validate_not_null, validate_not_null_mut, AudioEngineHandle, FFIAudioCallback, FFIAudioEvent,
    FFIAudioEventType, FFIPlaybackState, FFIResult,
};
use parking_lot::Mutex;
use std::ffi::CString;
use std::os::raw::{c_char, c_double, c_void};
use std::sync::Arc;

/// Convert AudioEngine to opaque handle
fn engine_to_handle(engine: Arc<Mutex<AudioEngine>>) -> AudioEngineHandle {
    let ptr = Arc::into_raw(engine) as *mut std::ffi::c_void;
    AudioEngineHandle { inner: ptr }
}

/// Convert opaque handle back to AudioEngine
///
/// # Safety
/// The handle must be a valid handle created by `engine_to_handle`
unsafe fn handle_to_engine(handle: AudioEngineHandle) -> Option<Arc<Mutex<AudioEngine>>> {
    if handle.is_null() {
        return None;
    }
    let ptr = handle.inner as *const Mutex<AudioEngine>;
    Some(Arc::from_raw(ptr))
}

/// Borrow AudioEngine from handle without consuming it
///
/// # Safety
/// The handle must be a valid handle and remain valid for the duration of the borrow
unsafe fn borrow_engine(handle: AudioEngineHandle) -> Option<&'static Mutex<AudioEngine>> {
    if handle.is_null() {
        return None;
    }
    let ptr = handle.inner as *const Mutex<AudioEngine>;
    Some(&*ptr)
}

/// Create a new audio engine instance
///
/// # Safety
/// The caller must call `audio_engine_destroy` to free the returned handle.
#[no_mangle]
pub unsafe extern "C" fn audio_engine_create() -> AudioEngineHandle {
    match AudioEngine::new() {
        Ok(engine) => engine_to_handle(Arc::new(Mutex::new(engine))),
        Err(_) => AudioEngineHandle::null(),
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

    // Convert handle back to Arc, which will drop and clean up
    let _engine = handle_to_engine(handle);
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

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let path_str = match crate::ffi::types::c_string_to_rust_str(file_path) {
        Ok(s) => s,
        Err(_) => return FFIResult::InvalidArgument,
    };

    let mut engine = engine_mutex.lock();
    match engine.load_file(path_str) {
        Ok(_) => FFIResult::Success,
        Err(_) => FFIResult::InternalError,
    }
}

/// Set volume (0.0 to 1.0)
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

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let mut engine = engine_mutex.lock();
    match engine.set_volume(volume as f32) {
        Ok(_) => FFIResult::Success,
        Err(_) => FFIResult::InternalError,
    }
}

/// Set volume with ramping (0.0 to 1.0)
///
/// # Safety
/// `handle` must be a valid audio engine handle
#[no_mangle]
pub unsafe extern "C" fn audio_engine_set_volume_ramped(
    handle: AudioEngineHandle,
    volume: c_double,
    ramp_duration_ms: u32,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if !(0.0..=1.0).contains(&volume) || volume.is_nan() || volume.is_infinite() {
        return FFIResult::InvalidArgument;
    }

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let mut engine = engine_mutex.lock();
    match engine.set_volume_ramped(volume as f32, ramp_duration_ms) {
        Ok(_) => FFIResult::Success,
        Err(_) => FFIResult::InternalError,
    }
}

/// Mute audio (preserves volume setting)
///
/// # Safety
/// `handle` must be a valid audio engine handle
#[no_mangle]
pub unsafe extern "C" fn audio_engine_mute(handle: AudioEngineHandle) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let mut engine = engine_mutex.lock();
    match engine.mute() {
        Ok(_) => FFIResult::Success,
        Err(_) => FFIResult::InternalError,
    }
}

/// Unmute audio (restores previous volume)
///
/// # Safety
/// `handle` must be a valid audio engine handle
#[no_mangle]
pub unsafe extern "C" fn audio_engine_unmute(handle: AudioEngineHandle) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let mut engine = engine_mutex.lock();
    match engine.unmute() {
        Ok(_) => FFIResult::Success,
        Err(_) => FFIResult::InternalError,
    }
}

/// Check if audio is muted
///
/// # Safety
/// - `handle` must be a valid audio engine handle
/// - `is_muted` must be a valid pointer to write the result (0 = false, 1 = true)
#[no_mangle]
pub unsafe extern "C" fn audio_engine_is_muted(
    handle: AudioEngineHandle,
    is_muted: *mut u8,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null_mut(is_muted).into() {
        return result;
    }

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let engine = engine_mutex.lock();
    *is_muted = if engine.is_muted() { 1 } else { 0 };
    FFIResult::Success
}

/// Get current volume (0.0 to 1.0)
///
/// # Safety
/// - `handle` must be a valid audio engine handle
/// - `volume` must be a valid pointer to write the result
#[no_mangle]
pub unsafe extern "C" fn audio_engine_get_volume(
    handle: AudioEngineHandle,
    volume: *mut c_double,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null_mut(volume).into() {
        return result;
    }

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let engine = engine_mutex.lock();
    *volume = engine.volume() as c_double;
    FFIResult::Success
}

/// Get current playback position in seconds
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

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let engine = engine_mutex.lock();
    let pos_samples = engine.position();
    let sample_rate = engine.format().map(|f| f.sample_rate).unwrap_or(44100);
    *position = pos_samples as c_double / sample_rate as c_double;
    FFIResult::Success
}

/// Get total duration in seconds
///
/// # Safety
/// - `handle` must be a valid audio engine handle
/// - `duration` must be a valid pointer to write the result
#[no_mangle]
pub unsafe extern "C" fn audio_engine_get_duration(
    handle: AudioEngineHandle,
    duration: *mut c_double,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null_mut(duration).into() {
        return result;
    }

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let engine = engine_mutex.lock();
    let dur_samples = engine.duration().unwrap_or(0);
    let sample_rate = engine.format().map(|f| f.sample_rate).unwrap_or(44100);
    *duration = dur_samples as c_double / sample_rate as c_double;
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

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let mut engine = engine_mutex.lock();
    match engine.play() {
        Ok(_) => FFIResult::Success,
        Err(_) => FFIResult::InternalError,
    }
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

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let mut engine = engine_mutex.lock();
    match engine.pause() {
        Ok(_) => FFIResult::Success,
        Err(_) => FFIResult::InternalError,
    }
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

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let mut engine = engine_mutex.lock();
    match engine.stop() {
        Ok(_) => FFIResult::Success,
        Err(_) => FFIResult::InternalError,
    }
}

/// Seek to a specific position in seconds
///
/// # Safety
/// `handle` must be a valid audio engine handle
#[no_mangle]
pub unsafe extern "C" fn audio_engine_seek(
    handle: AudioEngineHandle,
    position: c_double,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if position < 0.0 || position.is_nan() || position.is_infinite() {
        return FFIResult::InvalidArgument;
    }

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let mut engine = engine_mutex.lock();
    let sample_rate = engine.format().map(|f| f.sample_rate).unwrap_or(44100);
    let position_samples = (position * sample_rate as c_double) as u64;
    match engine.seek(position_samples) {
        Ok(_) => FFIResult::Success,
        Err(_) => FFIResult::InternalError,
    }
}

/// Check if audio is currently playing
///
/// # Safety
/// - `handle` must be a valid audio engine handle
/// - `is_playing` must be a valid pointer to write the result (0 = false, 1 = true)
#[no_mangle]
pub unsafe extern "C" fn audio_engine_is_playing(
    handle: AudioEngineHandle,
    is_playing: *mut u8,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null_mut(is_playing).into() {
        return result;
    }

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let engine = engine_mutex.lock();
    use crate::audio::engine::PlaybackState;
    *is_playing = if engine.state() == PlaybackState::Playing {
        1
    } else {
        0
    };
    FFIResult::Success
}

/// Register a callback for audio events
///
/// # Safety
/// - `handle` must be a valid audio engine handle
/// - `callback` must be a valid function pointer that remains valid for the lifetime of the registration
/// - `user_data` can be any pointer (including null) and will be passed back to the callback
/// - The callback must be thread-safe and must not call back into the audio engine
#[no_mangle]
pub unsafe extern "C" fn audio_engine_set_callback(
    handle: AudioEngineHandle,
    callback: FFIAudioCallback,
    user_data: *mut c_void,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let callback_data = Arc::new(Mutex::new(CallbackData {
        callback,
        user_data,
    }));

    let callback_data_clone = callback_data.clone();
    let rust_callback = Box::new(move |event: AudioEvent| {
        let (ffi_event, _error_cstring) = audio_event_to_ffi(&event);
        let data = callback_data_clone.lock();
        // SAFETY: The callback function pointer is provided by the caller
        // and they guarantee it's valid and thread-safe
        unsafe {
            (data.callback)(ffi_event, data.user_data);
        }
        // _error_cstring is dropped here, keeping the string alive during the callback
    });

    let mut engine = engine_mutex.lock();
    engine.set_callback(rust_callback);

    FFIResult::Success
}

/// Clear the registered callback
///
/// # Safety
/// `handle` must be a valid audio engine handle
#[no_mangle]
pub unsafe extern "C" fn audio_engine_clear_callback(handle: AudioEngineHandle) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    let engine_mutex = match borrow_engine(handle) {
        Some(e) => e,
        None => return FFIResult::NullPointer,
    };

    let mut engine = engine_mutex.lock();
    engine.clear_callback();

    FFIResult::Success
}

/// Convert Rust PlaybackState to FFI PlaybackState
fn playback_state_to_ffi(state: PlaybackState) -> FFIPlaybackState {
    match state {
        PlaybackState::Stopped => FFIPlaybackState::Stopped,
        PlaybackState::Playing => FFIPlaybackState::Playing,
        PlaybackState::Paused => FFIPlaybackState::Paused,
        PlaybackState::Buffering => FFIPlaybackState::Buffering,
        PlaybackState::Error => FFIPlaybackState::Error,
    }
}

/// Convert Rust AudioEvent to FFI AudioEvent
fn audio_event_to_ffi(event: &AudioEvent) -> (FFIAudioEvent, Option<CString>) {
    let (event_type, state, position, error_cstring) = match event {
        AudioEvent::StateChanged(s) => (
            FFIAudioEventType::StateChanged,
            playback_state_to_ffi(*s),
            0,
            None,
        ),
        AudioEvent::PositionChanged(pos) => (
            FFIAudioEventType::PositionChanged,
            FFIPlaybackState::Stopped,
            *pos,
            None,
        ),
        AudioEvent::TrackEnded => (
            FFIAudioEventType::TrackEnded,
            FFIPlaybackState::Stopped,
            0,
            None,
        ),
        AudioEvent::Error(msg) => {
            let cstring = CString::new(msg.as_str()).unwrap_or_else(|_| CString::new("").unwrap());
            (
                FFIAudioEventType::Error,
                FFIPlaybackState::Error,
                0,
                Some(cstring),
            )
        }
        AudioEvent::BufferUnderrun => (
            FFIAudioEventType::BufferUnderrun,
            FFIPlaybackState::Stopped,
            0,
            None,
        ),
    };

    let error_ptr = error_cstring
        .as_ref()
        .map(|s| s.as_ptr())
        .unwrap_or(std::ptr::null());

    let ffi_event = FFIAudioEvent {
        event_type,
        state,
        position,
        error_message: error_ptr,
    };

    (ffi_event, error_cstring)
}

/// Structure to hold callback and user data
struct CallbackData {
    callback: FFIAudioCallback,
    user_data: *mut c_void,
}

// SAFETY: We ensure thread safety by only accessing this through Arc<Mutex<>>
unsafe impl Send for CallbackData {}
unsafe impl Sync for CallbackData {}

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

            audio_engine_destroy(handle);
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

            audio_engine_destroy(handle);
        }
    }

    #[test]
    fn test_get_volume() {
        unsafe {
            let handle = audio_engine_create();
            let mut volume = 0.0;

            // Get initial volume
            let result = audio_engine_get_volume(handle, &mut volume);
            assert_eq!(result, FFIResult::Success);
            assert!(volume >= 0.0 && volume <= 1.0);

            // Set and get volume
            audio_engine_set_volume(handle, 0.75);
            audio_engine_get_volume(handle, &mut volume);
            assert!((volume - 0.75).abs() < 0.01);

            audio_engine_destroy(handle);
        }
    }

    #[test]
    fn test_is_playing() {
        unsafe {
            let handle = audio_engine_create();
            let mut is_playing = 0;

            let result = audio_engine_is_playing(handle, &mut is_playing);
            assert_eq!(result, FFIResult::Success);
            assert_eq!(is_playing, 0); // Should not be playing initially

            audio_engine_destroy(handle);
        }
    }

    #[test]
    fn test_seek_validation() {
        unsafe {
            let handle = audio_engine_create();

            // Valid seek positions
            assert_eq!(audio_engine_seek(handle, 0.0), FFIResult::Success);
            assert_eq!(audio_engine_seek(handle, 10.5), FFIResult::Success);

            // Invalid seek positions
            assert_eq!(audio_engine_seek(handle, -1.0), FFIResult::InvalidArgument);

            audio_engine_destroy(handle);
        }
    }

    #[test]
    fn test_callback_registration() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        unsafe extern "C" fn test_callback(event: FFIAudioEvent, user_data: *mut c_void) {
            if !user_data.is_null() {
                let counter = &*(user_data as *const AtomicU32);
                counter.fetch_add(1, Ordering::SeqCst);

                // Verify event type is valid
                match event.event_type {
                    FFIAudioEventType::StateChanged => {
                        // State should be valid
                        assert!(matches!(
                            event.state,
                            FFIPlaybackState::Stopped
                                | FFIPlaybackState::Playing
                                | FFIPlaybackState::Paused
                                | FFIPlaybackState::Buffering
                                | FFIPlaybackState::Error
                        ));
                    }
                    FFIAudioEventType::PositionChanged => {
                        // Position should be set
                    }
                    FFIAudioEventType::TrackEnded => {}
                    FFIAudioEventType::Error => {
                        // Error message should be valid (can be null)
                    }
                    FFIAudioEventType::BufferUnderrun => {}
                }
            }
        }

        unsafe {
            let handle = audio_engine_create();
            let counter = Arc::new(AtomicU32::new(0));
            let counter_ptr = Arc::as_ptr(&counter) as *mut c_void;

            // Register callback
            let result = audio_engine_set_callback(handle, test_callback, counter_ptr);
            assert_eq!(result, FFIResult::Success);

            // Trigger some events by seeking (which emits PositionChanged)
            audio_engine_seek(handle, 1.0);

            // Give some time for callback to be called
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Clear callback
            let result = audio_engine_clear_callback(handle);
            assert_eq!(result, FFIResult::Success);

            // Seek again - callback should not be called
            let count_before = counter.load(Ordering::SeqCst);
            audio_engine_seek(handle, 2.0);
            std::thread::sleep(std::time::Duration::from_millis(10));
            let count_after = counter.load(Ordering::SeqCst);

            // Count should not have increased after clearing callback
            assert_eq!(count_before, count_after);

            audio_engine_destroy(handle);
        }
    }

    #[test]
    fn test_callback_with_null_handle() {
        unsafe extern "C" fn dummy_callback(_event: FFIAudioEvent, _user_data: *mut c_void) {}

        unsafe {
            let null_handle = AudioEngineHandle::null();
            let result =
                audio_engine_set_callback(null_handle, dummy_callback, std::ptr::null_mut());
            assert_eq!(result, FFIResult::NullPointer);

            let result = audio_engine_clear_callback(null_handle);
            assert_eq!(result, FFIResult::NullPointer);
        }
    }

    #[test]
    fn test_callback_with_null_user_data() {
        use std::sync::atomic::{AtomicBool, Ordering};

        static CALLBACK_CALLED: AtomicBool = AtomicBool::new(false);

        unsafe extern "C" fn test_callback(_event: FFIAudioEvent, user_data: *mut c_void) {
            // Verify user_data is null as expected
            assert!(user_data.is_null());
            CALLBACK_CALLED.store(true, Ordering::SeqCst);
        }

        unsafe {
            let handle = audio_engine_create();

            // Register callback with null user_data
            let result = audio_engine_set_callback(handle, test_callback, std::ptr::null_mut());
            assert_eq!(result, FFIResult::Success);

            // Trigger an event
            audio_engine_seek(handle, 1.0);
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Verify callback was called
            assert!(CALLBACK_CALLED.load(Ordering::SeqCst));

            audio_engine_destroy(handle);
        }
    }
}
