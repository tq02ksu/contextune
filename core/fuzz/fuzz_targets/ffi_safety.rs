#![no_main]

use libfuzzer_sys::fuzz_target;
use contextune_core::ffi::{
    audio_engine_create, audio_engine_destroy, audio_engine_load_file,
    audio_engine_set_volume, audio_engine_get_position, audio_engine_play,
    audio_engine_pause, audio_engine_stop, AudioEngineHandle, FFIResult
};
use std::ffi::CString;

#[derive(Debug)]
enum FFIOperation {
    Create,
    Destroy(AudioEngineHandle),
    LoadFile(AudioEngineHandle, Vec<u8>),
    SetVolume(AudioEngineHandle, f64),
    GetPosition(AudioEngineHandle),
    Play(AudioEngineHandle),
    Pause(AudioEngineHandle),
    Stop(AudioEngineHandle),
}

fn parse_operations(data: &[u8]) -> Vec<FFIOperation> {
    let mut operations = Vec::new();
    let mut i = 0;
    let mut handles = Vec::new();
    
    while i < data.len() {
        if i + 1 >= data.len() {
            break;
        }
        
        let op_type = data[i] % 8;
        i += 1;
        
        match op_type {
            0 => {
                // Create operation
                operations.push(FFIOperation::Create);
            }
            1 => {
                // Destroy operation
                if !handles.is_empty() {
                    let handle_idx = data[i] as usize % handles.len();
                    operations.push(FFIOperation::Destroy(handles[handle_idx]));
                }
                i += 1;
            }
            2 => {
                // LoadFile operation
                if !handles.is_empty() && i + 8 < data.len() {
                    let handle_idx = data[i] as usize % handles.len();
                    i += 1;
                    let path_len = (data[i] % 100) as usize;
                    i += 1;
                    let path_data = if i + path_len <= data.len() {
                        data[i..i + path_len].to_vec()
                    } else {
                        data[i..].to_vec()
                    };
                    operations.push(FFIOperation::LoadFile(handles[handle_idx], path_data));
                    i += path_len;
                }
            }
            3 => {
                // SetVolume operation
                if !handles.is_empty() && i + 8 < data.len() {
                    let handle_idx = data[i] as usize % handles.len();
                    i += 1;
                    // Create volume from bytes
                    let volume_bytes = [
                        data[i], data[i+1], data[i+2], data[i+3],
                        data[i+4], data[i+5], data[i+6], data[i+7]
                    ];
                    let volume = f64::from_le_bytes(volume_bytes);
                    operations.push(FFIOperation::SetVolume(handles[handle_idx], volume));
                    i += 8;
                }
            }
            4 => {
                // GetPosition operation
                if !handles.is_empty() {
                    let handle_idx = data[i] as usize % handles.len();
                    operations.push(FFIOperation::GetPosition(handles[handle_idx]));
                }
                i += 1;
            }
            5 => {
                // Play operation
                if !handles.is_empty() {
                    let handle_idx = data[i] as usize % handles.len();
                    operations.push(FFIOperation::Play(handles[handle_idx]));
                }
                i += 1;
            }
            6 => {
                // Pause operation
                if !handles.is_empty() {
                    let handle_idx = data[i] as usize % handles.len();
                    operations.push(FFIOperation::Pause(handles[handle_idx]));
                }
                i += 1;
            }
            7 => {
                // Stop operation
                if !handles.is_empty() {
                    let handle_idx = data[i] as usize % handles.len();
                    operations.push(FFIOperation::Stop(handles[handle_idx]));
                }
                i += 1;
            }
            _ => unreachable!(),
        }
        
        // Limit number of handles to prevent excessive memory usage
        if handles.len() < 10 && op_type == 0 {
            handles.push(AudioEngineHandle::null()); // Placeholder, will be updated
        }
    }
    
    operations
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    
    let operations = parse_operations(data);
    let mut handles = Vec::new();
    
    for operation in operations {
        unsafe {
            match operation {
                FFIOperation::Create => {
                    let handle = audio_engine_create();
                    handles.push(handle);
                }
                FFIOperation::Destroy(handle) => {
                    let _ = audio_engine_destroy(handle);
                    // In a real implementation, we would remove the handle from our list
                }
                FFIOperation::LoadFile(handle, path_data) => {
                    // Try to create a valid C string, ignore if it contains null bytes
                    if let Ok(c_string) = CString::new(path_data) {
                        let _ = audio_engine_load_file(handle, c_string.as_ptr());
                    }
                }
                FFIOperation::SetVolume(handle, volume) => {
                    let _ = audio_engine_set_volume(handle, volume);
                }
                FFIOperation::GetPosition(handle) => {
                    let mut position = 0.0;
                    let _ = audio_engine_get_position(handle, &mut position);
                }
                FFIOperation::Play(handle) => {
                    let _ = audio_engine_play(handle);
                }
                FFIOperation::Pause(handle) => {
                    let _ = audio_engine_pause(handle);
                }
                FFIOperation::Stop(handle) => {
                    let _ = audio_engine_stop(handle);
                }
            }
        }
    }
    
    // Clean up all handles
    for handle in handles {
        unsafe {
            let _ = audio_engine_destroy(handle);
        }
    }
});
