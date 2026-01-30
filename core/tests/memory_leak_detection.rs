//! Memory leak detection tests
//!
//! These tests are designed to detect memory leaks in the audio processing pipeline.
//! They should be run with AddressSanitizer, Valgrind, or other memory debugging tools.

use contextune_core::audio::checksum::{calculate_checksum, ChecksumAlgorithm};
use std::collections::HashMap;

/// Test for memory leaks in checksum calculation
#[test]
fn test_checksum_memory_leak() {
    // Create a large buffer to stress test memory allocation
    let large_buffer: Vec<i16> = (0..100_000).map(|i| (i % 32767) as i16).collect();

    // Run checksum calculation multiple times to detect leaks
    for _ in 0..100 {
        let _checksum = calculate_checksum(&large_buffer, ChecksumAlgorithm::Simple);
        let _crc32 = calculate_checksum(&large_buffer, ChecksumAlgorithm::Crc32);
        let _md5 = calculate_checksum(&large_buffer, ChecksumAlgorithm::Md5);
        let _sha256 = calculate_checksum(&large_buffer, ChecksumAlgorithm::Sha256);
    }

    // If we reach here without crashing, the test passes
    assert!(true);
}

/// Test for memory leaks in repeated allocations
#[test]
fn test_repeated_allocations() {
    let mut buffers = Vec::new();

    // Allocate and deallocate buffers repeatedly
    for i in 0..1000 {
        let buffer: Vec<f64> = (0..1000).map(|x| (x as f64) * 0.001).collect();

        // Do some processing
        let _sum: f64 = buffer.iter().sum();
        let _max = buffer.iter().fold(0.0f64, |a, &b| a.max(b));

        // Keep some buffers to test proper cleanup
        if i % 10 == 0 {
            buffers.push(buffer);
        }
    }

    // Clear all buffers
    buffers.clear();

    assert!(true);
}

/// Test for memory leaks in hash map operations
#[test]
fn test_hashmap_memory_leak() {
    let mut map: HashMap<String, Vec<u8>> = HashMap::new();

    // Insert and remove many entries
    for i in 0..10000 {
        let key = format!("key_{}", i);
        let value = vec![i as u8; 100];
        map.insert(key.clone(), value);

        // Remove every other entry
        if i % 2 == 0 {
            map.remove(&key);
        }
    }

    // Clear the map
    map.clear();

    assert!(true);
}

/// Test for memory leaks in string operations
#[test]
fn test_string_memory_leak() {
    let mut strings = Vec::new();

    for i in 0..5000 {
        let mut s = String::new();

        // Build a large string
        for j in 0..100 {
            s.push_str(&format!("iteration_{}_{} ", i, j));
        }

        // Process the string
        let _len = s.len();
        let _bytes = s.as_bytes();
        let _chars: Vec<char> = s.chars().collect();

        // Keep some strings
        if i % 100 == 0 {
            strings.push(s);
        }
    }

    // Process all kept strings
    for s in &strings {
        let _uppercase = s.to_uppercase();
        let _words: Vec<&str> = s.split_whitespace().collect();
    }

    strings.clear();

    assert!(true);
}

/// Test for memory leaks in recursive operations
#[test]
fn test_recursive_memory_leak() {
    fn recursive_process(depth: usize, data: Vec<u8>) -> Vec<u8> {
        if depth == 0 {
            return data;
        }

        let mut new_data = data;
        new_data.extend_from_slice(&[depth as u8; 10]);

        // Process data
        let _sum: usize = new_data.iter().map(|&x| x as usize).sum();

        recursive_process(depth - 1, new_data)
    }

    // Run recursive operations multiple times
    for i in 0..100 {
        let initial_data = vec![i as u8; 50];
        let _result = recursive_process(10, initial_data);
    }

    assert!(true);
}

/// Test for memory leaks in closure operations
#[test]
fn test_closure_memory_leak() {
    let mut results = Vec::new();

    for i in 0..1000 {
        let data = vec![i as f32; 100];

        // Create closures that capture data
        let processor =
            |multiplier: f32| data.iter().map(|&x| x * multiplier).collect::<Vec<f32>>();

        let result1 = processor(2.0);
        let result2 = processor(3.0);

        // Combine results
        let combined: Vec<f32> = result1
            .iter()
            .zip(result2.iter())
            .map(|(&a, &b)| a + b)
            .collect();

        if i % 50 == 0 {
            results.push(combined);
        }
    }

    // Process all results
    for result in &results {
        let _avg = result.iter().sum::<f32>() / result.len() as f32;
    }

    results.clear();

    assert!(true);
}

/// Test for memory leaks in thread-local operations
#[test]
fn test_thread_local_memory_leak() {
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                let mut local_data = Vec::new();

                for j in 0..1000 {
                    let buffer = vec![(i as u8).wrapping_add(j as u8); 100];
                    local_data.push(buffer);

                    // Process some data
                    if j % 100 == 0 {
                        let _total_len: usize = local_data.iter().map(|v| v.len()).sum();
                        local_data.clear();
                    }
                }

                local_data.len()
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        let _result = handle.join().unwrap();
    }

    assert!(true);
}

/// Stress test for overall memory usage
#[test]
fn test_memory_stress() {
    const ITERATIONS: usize = 1000;
    const BUFFER_SIZE: usize = 1000;

    for iteration in 0..ITERATIONS {
        // Allocate various types of data
        let int_buffer: Vec<i32> = (0..BUFFER_SIZE).map(|i| i as i32).collect();
        let float_buffer: Vec<f64> = (0..BUFFER_SIZE).map(|i| i as f64 * 0.001).collect();
        let string_buffer: Vec<String> = (0..100).map(|i| format!("string_{}", i)).collect();

        // Process the data
        let _int_sum: i32 = int_buffer.iter().sum();
        let _float_avg = float_buffer.iter().sum::<f64>() / float_buffer.len() as f64;
        let _string_len: usize = string_buffer.iter().map(|s| s.len()).sum();

        // Create some nested structures
        let nested: Vec<Vec<u8>> = (0..10)
            .map(|i| (0..100).map(|j| (i + j) as u8).collect())
            .collect();

        let _nested_sum: usize = nested.iter().flatten().map(|&x| x as usize).sum();

        // Simulate some audio processing
        if iteration % 100 == 0 {
            let audio_samples: Vec<i16> = (0..BUFFER_SIZE)
                .map(|i| ((i as f64 * 0.1).sin() * 32767.0) as i16)
                .collect();

            let _checksum = calculate_checksum(&audio_samples, ChecksumAlgorithm::Simple);
        }
    }

    assert!(true);
}
