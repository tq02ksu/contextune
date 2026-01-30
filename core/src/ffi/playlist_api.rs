//! Playlist FFI API
//!
//! C-compatible functions for playlist management

use crate::ffi::types::{validate_not_null, validate_not_null_mut, FFIResult};
use crate::playlist::{PlaylistManager, Track};
use parking_lot::Mutex;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;

/// Opaque handle to a playlist manager
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PlaylistManagerHandle {
    inner: *mut std::ffi::c_void,
}

impl PlaylistManagerHandle {
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

/// Convert PlaylistManager to opaque handle
fn manager_to_handle(manager: Arc<Mutex<PlaylistManager>>) -> PlaylistManagerHandle {
    let ptr = Arc::into_raw(manager) as *mut std::ffi::c_void;
    PlaylistManagerHandle { inner: ptr }
}

/// Convert opaque handle back to PlaylistManager
///
/// # Safety
/// The handle must be a valid handle created by `manager_to_handle`
unsafe fn handle_to_manager(handle: PlaylistManagerHandle) -> Option<Arc<Mutex<PlaylistManager>>> {
    if handle.is_null() {
        return None;
    }
    let ptr = handle.inner as *const Mutex<PlaylistManager>;
    Some(Arc::from_raw(ptr))
}

/// Borrow PlaylistManager from handle without consuming it
///
/// # Safety
/// The handle must be a valid handle and remain valid for the duration of the borrow
unsafe fn borrow_manager(handle: PlaylistManagerHandle) -> Option<&'static Mutex<PlaylistManager>> {
    if handle.is_null() {
        return None;
    }
    let ptr = handle.inner as *const Mutex<PlaylistManager>;
    Some(&*ptr)
}

/// Create a new playlist manager instance
///
/// # Safety
/// The caller must call `playlist_manager_destroy` to free the returned handle.
#[no_mangle]
pub unsafe extern "C" fn playlist_manager_create() -> PlaylistManagerHandle {
    let manager = PlaylistManager::new();
    manager_to_handle(Arc::new(Mutex::new(manager)))
}

/// Destroy a playlist manager instance
///
/// # Safety
/// The handle must be a valid handle returned by `playlist_manager_create`.
/// After calling this function, the handle becomes invalid.
#[no_mangle]
pub unsafe extern "C" fn playlist_manager_destroy(handle: PlaylistManagerHandle) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    // Convert handle back to Arc, which will drop and clean up
    let _manager = handle_to_manager(handle);
    FFIResult::Success
}

/// Create a new playlist
///
/// # Safety
/// - `handle` must be a valid playlist manager handle
/// - `name` must be a valid null-terminated C string
/// - `playlist_id_out` must be a valid pointer to write the playlist ID
/// - The caller must free the returned playlist ID string using `playlist_free_string`
#[no_mangle]
pub unsafe extern "C" fn playlist_create(
    handle: PlaylistManagerHandle,
    name: *const c_char,
    playlist_id_out: *mut *mut c_char,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null(name).into() {
        return result;
    }

    if let Err(result) = validate_not_null_mut(playlist_id_out).into() {
        return result;
    }

    let manager_mutex = match borrow_manager(handle) {
        Some(m) => m,
        None => return FFIResult::NullPointer,
    };

    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return FFIResult::InvalidArgument,
    };

    let manager = manager_mutex.lock();
    match manager.create_playlist(name_str) {
        Ok(id) => match CString::new(id) {
            Ok(c_id) => {
                *playlist_id_out = c_id.into_raw();
                FFIResult::Success
            }
            Err(_) => FFIResult::InternalError,
        },
        Err(_) => FFIResult::InternalError,
    }
}

/// Delete a playlist
///
/// # Safety
/// - `handle` must be a valid playlist manager handle
/// - `playlist_id` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn playlist_delete(
    handle: PlaylistManagerHandle,
    playlist_id: *const c_char,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null(playlist_id).into() {
        return result;
    }

    let manager_mutex = match borrow_manager(handle) {
        Some(m) => m,
        None => return FFIResult::NullPointer,
    };

    let id_str = match CStr::from_ptr(playlist_id).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::InvalidArgument,
    };

    let manager = manager_mutex.lock();
    match manager.delete_playlist(id_str) {
        Ok(_) => FFIResult::Success,
        Err(_) => FFIResult::NotFound,
    }
}

/// Get playlist count
///
/// # Safety
/// - `handle` must be a valid playlist manager handle
/// - `count_out` must be a valid pointer to write the count
#[no_mangle]
pub unsafe extern "C" fn playlist_get_count(
    handle: PlaylistManagerHandle,
    count_out: *mut usize,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null_mut(count_out).into() {
        return result;
    }

    let manager_mutex = match borrow_manager(handle) {
        Some(m) => m,
        None => return FFIResult::NullPointer,
    };

    let manager = manager_mutex.lock();
    match manager.playlist_count() {
        Ok(count) => {
            *count_out = count;
            FFIResult::Success
        }
        Err(_) => FFIResult::InternalError,
    }
}

/// Get playlist name
///
/// # Safety
/// - `handle` must be a valid playlist manager handle
/// - `playlist_id` must be a valid null-terminated C string
/// - `name_out` must be a valid pointer to write the name
/// - The caller must free the returned name string using `playlist_free_string`
#[no_mangle]
pub unsafe extern "C" fn playlist_get_name(
    handle: PlaylistManagerHandle,
    playlist_id: *const c_char,
    name_out: *mut *mut c_char,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null(playlist_id).into() {
        return result;
    }

    if let Err(result) = validate_not_null_mut(name_out).into() {
        return result;
    }

    let manager_mutex = match borrow_manager(handle) {
        Some(m) => m,
        None => return FFIResult::NullPointer,
    };

    let id_str = match CStr::from_ptr(playlist_id).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::InvalidArgument,
    };

    let manager = manager_mutex.lock();
    match manager.get_playlist(id_str) {
        Ok(playlist) => match CString::new(playlist.name) {
            Ok(c_name) => {
                *name_out = c_name.into_raw();
                FFIResult::Success
            }
            Err(_) => FFIResult::InternalError,
        },
        Err(_) => FFIResult::NotFound,
    }
}

/// Add a track to a playlist
///
/// # Safety
/// - `handle` must be a valid playlist manager handle
/// - `playlist_id` must be a valid null-terminated C string
/// - `file_path` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn playlist_add_track(
    handle: PlaylistManagerHandle,
    playlist_id: *const c_char,
    file_path: *const c_char,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null(playlist_id).into() {
        return result;
    }

    if let Err(result) = validate_not_null(file_path).into() {
        return result;
    }

    let manager_mutex = match borrow_manager(handle) {
        Some(m) => m,
        None => return FFIResult::NullPointer,
    };

    let id_str = match CStr::from_ptr(playlist_id).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::InvalidArgument,
    };

    let path_str = match CStr::from_ptr(file_path).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return FFIResult::InvalidArgument,
    };

    let track = Track::new(path_str);
    let manager = manager_mutex.lock();
    match manager.add_track_to_playlist(id_str, track) {
        Ok(_) => FFIResult::Success,
        Err(_) => FFIResult::NotFound,
    }
}

/// Remove a track from a playlist
///
/// # Safety
/// - `handle` must be a valid playlist manager handle
/// - `playlist_id` must be a valid null-terminated C string
/// - `track_index` must be a valid index within the playlist
#[no_mangle]
pub unsafe extern "C" fn playlist_remove_track(
    handle: PlaylistManagerHandle,
    playlist_id: *const c_char,
    track_index: usize,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null(playlist_id).into() {
        return result;
    }

    let manager_mutex = match borrow_manager(handle) {
        Some(m) => m,
        None => return FFIResult::NullPointer,
    };

    let id_str = match CStr::from_ptr(playlist_id).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::InvalidArgument,
    };

    let manager = manager_mutex.lock();
    match manager.remove_track_from_playlist(id_str, track_index) {
        Ok(_) => FFIResult::Success,
        Err(_) => FFIResult::InvalidArgument,
    }
}

/// Move a track within a playlist
///
/// # Safety
/// - `handle` must be a valid playlist manager handle
/// - `playlist_id` must be a valid null-terminated C string
/// - `from_index` and `to_index` must be valid indices within the playlist
#[no_mangle]
pub unsafe extern "C" fn playlist_move_track(
    handle: PlaylistManagerHandle,
    playlist_id: *const c_char,
    from_index: usize,
    to_index: usize,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null(playlist_id).into() {
        return result;
    }

    let manager_mutex = match borrow_manager(handle) {
        Some(m) => m,
        None => return FFIResult::NullPointer,
    };

    let id_str = match CStr::from_ptr(playlist_id).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::InvalidArgument,
    };

    let manager = manager_mutex.lock();
    match manager.move_track_in_playlist(id_str, from_index, to_index) {
        Ok(_) => FFIResult::Success,
        Err(_) => FFIResult::InvalidArgument,
    }
}

/// Get track count in a playlist
///
/// # Safety
/// - `handle` must be a valid playlist manager handle
/// - `playlist_id` must be a valid null-terminated C string
/// - `count_out` must be a valid pointer to write the count
#[no_mangle]
pub unsafe extern "C" fn playlist_get_track_count(
    handle: PlaylistManagerHandle,
    playlist_id: *const c_char,
    count_out: *mut usize,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null(playlist_id).into() {
        return result;
    }

    if let Err(result) = validate_not_null_mut(count_out).into() {
        return result;
    }

    let manager_mutex = match borrow_manager(handle) {
        Some(m) => m,
        None => return FFIResult::NullPointer,
    };

    let id_str = match CStr::from_ptr(playlist_id).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::InvalidArgument,
    };

    let manager = manager_mutex.lock();
    match manager.get_playlist(id_str) {
        Ok(playlist) => {
            *count_out = playlist.track_count();
            FFIResult::Success
        }
        Err(_) => FFIResult::NotFound,
    }
}

/// Shuffle a playlist
///
/// # Safety
/// - `handle` must be a valid playlist manager handle
/// - `playlist_id` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn playlist_shuffle(
    handle: PlaylistManagerHandle,
    playlist_id: *const c_char,
) -> FFIResult {
    if handle.is_null() {
        return FFIResult::NullPointer;
    }

    if let Err(result) = validate_not_null(playlist_id).into() {
        return result;
    }

    let manager_mutex = match borrow_manager(handle) {
        Some(m) => m,
        None => return FFIResult::NullPointer,
    };

    let id_str = match CStr::from_ptr(playlist_id).to_str() {
        Ok(s) => s,
        Err(_) => return FFIResult::InvalidArgument,
    };

    let manager = manager_mutex.lock();
    match manager.shuffle_playlist(id_str) {
        Ok(_) => FFIResult::Success,
        Err(_) => FFIResult::NotFound,
    }
}

/// Free a string returned by playlist functions
///
/// # Safety
/// - `string` must be a valid pointer returned by a playlist function
/// - After calling this function, the pointer becomes invalid
#[no_mangle]
pub unsafe extern "C" fn playlist_free_string(string: *mut c_char) {
    if !string.is_null() {
        let _ = CString::from_raw(string);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_manager_create_destroy() {
        unsafe {
            let handle = playlist_manager_create();
            assert!(!handle.is_null());

            let result = playlist_manager_destroy(handle);
            assert_eq!(result, FFIResult::Success);
        }
    }

    #[test]
    fn test_create_playlist() {
        unsafe {
            let handle = playlist_manager_create();
            let name = CString::new("Test Playlist").unwrap();
            let mut playlist_id: *mut c_char = std::ptr::null_mut();

            let result = playlist_create(handle, name.as_ptr(), &mut playlist_id);
            assert_eq!(result, FFIResult::Success);
            assert!(!playlist_id.is_null());

            playlist_free_string(playlist_id);
            playlist_manager_destroy(handle);
        }
    }

    #[test]
    fn test_delete_playlist() {
        unsafe {
            let handle = playlist_manager_create();
            let name = CString::new("Test").unwrap();
            let mut playlist_id: *mut c_char = std::ptr::null_mut();

            playlist_create(handle, name.as_ptr(), &mut playlist_id);
            let id_copy = CStr::from_ptr(playlist_id).to_owned();

            let result = playlist_delete(handle, id_copy.as_ptr());
            assert_eq!(result, FFIResult::Success);

            playlist_free_string(playlist_id);
            playlist_manager_destroy(handle);
        }
    }

    #[test]
    fn test_get_count() {
        unsafe {
            let handle = playlist_manager_create();
            let mut count: usize = 0;

            let result = playlist_get_count(handle, &mut count);
            assert_eq!(result, FFIResult::Success);
            assert_eq!(count, 0);

            // Create a playlist
            let name = CString::new("Test").unwrap();
            let mut playlist_id: *mut c_char = std::ptr::null_mut();
            playlist_create(handle, name.as_ptr(), &mut playlist_id);

            let result = playlist_get_count(handle, &mut count);
            assert_eq!(result, FFIResult::Success);
            assert_eq!(count, 1);

            playlist_free_string(playlist_id);
            playlist_manager_destroy(handle);
        }
    }

    #[test]
    fn test_add_track() {
        unsafe {
            let handle = playlist_manager_create();
            let name = CString::new("Test").unwrap();
            let mut playlist_id: *mut c_char = std::ptr::null_mut();

            playlist_create(handle, name.as_ptr(), &mut playlist_id);

            let file_path = CString::new("/path/to/song.mp3").unwrap();
            let result = playlist_add_track(handle, playlist_id, file_path.as_ptr());
            assert_eq!(result, FFIResult::Success);

            let mut track_count: usize = 0;
            playlist_get_track_count(handle, playlist_id, &mut track_count);
            assert_eq!(track_count, 1);

            playlist_free_string(playlist_id);
            playlist_manager_destroy(handle);
        }
    }

    #[test]
    fn test_remove_track() {
        unsafe {
            let handle = playlist_manager_create();
            let name = CString::new("Test").unwrap();
            let mut playlist_id: *mut c_char = std::ptr::null_mut();

            playlist_create(handle, name.as_ptr(), &mut playlist_id);

            // Add two tracks
            let file1 = CString::new("/song1.mp3").unwrap();
            let file2 = CString::new("/song2.mp3").unwrap();
            playlist_add_track(handle, playlist_id, file1.as_ptr());
            playlist_add_track(handle, playlist_id, file2.as_ptr());

            // Remove first track
            let result = playlist_remove_track(handle, playlist_id, 0);
            assert_eq!(result, FFIResult::Success);

            let mut track_count: usize = 0;
            playlist_get_track_count(handle, playlist_id, &mut track_count);
            assert_eq!(track_count, 1);

            playlist_free_string(playlist_id);
            playlist_manager_destroy(handle);
        }
    }

    #[test]
    fn test_move_track() {
        unsafe {
            let handle = playlist_manager_create();
            let name = CString::new("Test").unwrap();
            let mut playlist_id: *mut c_char = std::ptr::null_mut();

            playlist_create(handle, name.as_ptr(), &mut playlist_id);

            // Add three tracks
            for i in 0..3 {
                let file = CString::new(format!("/song{}.mp3", i)).unwrap();
                playlist_add_track(handle, playlist_id, file.as_ptr());
            }

            // Move track from index 0 to index 2
            let result = playlist_move_track(handle, playlist_id, 0, 2);
            assert_eq!(result, FFIResult::Success);

            playlist_free_string(playlist_id);
            playlist_manager_destroy(handle);
        }
    }

    #[test]
    fn test_shuffle() {
        unsafe {
            let handle = playlist_manager_create();
            let name = CString::new("Test").unwrap();
            let mut playlist_id: *mut c_char = std::ptr::null_mut();

            playlist_create(handle, name.as_ptr(), &mut playlist_id);

            // Add multiple tracks
            for i in 0..10 {
                let file = CString::new(format!("/song{}.mp3", i)).unwrap();
                playlist_add_track(handle, playlist_id, file.as_ptr());
            }

            let result = playlist_shuffle(handle, playlist_id);
            assert_eq!(result, FFIResult::Success);

            playlist_free_string(playlist_id);
            playlist_manager_destroy(handle);
        }
    }
}
