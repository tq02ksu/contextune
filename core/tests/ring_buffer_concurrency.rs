//! Concurrency tests for ring buffer
//!
//! Tests the lock-free ring buffer under concurrent access from multiple threads

use contextune_core::audio::format::{AudioFormat, SampleFormat};
use contextune_core::audio::ring_buffer::{AudioRingBuffer, RingBufferConfig};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_concurrent_single_producer_single_consumer() {
    let format = AudioFormat::new(44100, 2, SampleFormat::F64);
    let config = RingBufferConfig::standard(format);

    let (producer, consumer) = AudioRingBuffer::new(config).unwrap();

    let producer = Arc::new(producer);
    let consumer = Arc::new(consumer);

    let stop_flag = Arc::new(AtomicBool::new(false));
    let samples_written = Arc::new(AtomicUsize::new(0));
    let samples_read = Arc::new(AtomicUsize::new(0));

    // Producer thread
    let producer_handle = {
        let producer = producer.clone();
        let stop_flag = stop_flag.clone();
        let samples_written = samples_written.clone();

        thread::spawn(move || {
            let mut counter = 0.0;
            while !stop_flag.load(Ordering::Relaxed) {
                let samples: Vec<f64> = (0..100)
                    .map(|i| {
                        counter += 1.0;
                        (counter + i as f64) % 1000.0
                    })
                    .collect();

                let written = producer.write(&samples);
                samples_written.fetch_add(written, Ordering::Relaxed);

                if written < samples.len() {
                    // Buffer full, wait a bit
                    thread::sleep(Duration::from_micros(100));
                }
            }
        })
    };

    // Consumer thread
    let consumer_handle = {
        let consumer = consumer.clone();
        let stop_flag = stop_flag.clone();
        let samples_read = samples_read.clone();

        thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                let mut buffer = vec![0.0; 100];
                let read = consumer.read(&mut buffer);
                samples_read.fetch_add(read, Ordering::Relaxed);

                if read == 0 {
                    // Buffer empty, wait a bit
                    thread::sleep(Duration::from_micros(100));
                }
            }
        })
    };

    // Run for 100ms
    thread::sleep(Duration::from_millis(100));
    stop_flag.store(true, Ordering::Relaxed);

    producer_handle.join().unwrap();
    consumer_handle.join().unwrap();

    let written = samples_written.load(Ordering::Relaxed);
    let read = samples_read.load(Ordering::Relaxed);

    println!("Samples written: {}, read: {}", written, read);

    // Should have written and read a significant amount
    assert!(written > 1000, "Should have written at least 1000 samples");
    assert!(read > 1000, "Should have read at least 1000 samples");

    // Read should be close to written (within buffer capacity)
    let diff = if written > read {
        written - read
    } else {
        read - written
    };
    assert!(
        diff < consumer.capacity(),
        "Difference should be less than buffer capacity"
    );
}

#[test]
fn test_concurrent_multiple_readers() {
    let format = AudioFormat::new(44100, 2, SampleFormat::F64);
    let config = RingBufferConfig::standard(format);

    let (producer, consumer) = AudioRingBuffer::new(config).unwrap();

    let producer = Arc::new(producer);
    let consumer = Arc::new(consumer);

    let stop_flag = Arc::new(AtomicBool::new(false));
    let total_read = Arc::new(AtomicUsize::new(0));

    // Producer thread - write continuously
    let producer_handle = {
        let producer = producer.clone();
        let stop_flag = stop_flag.clone();

        thread::spawn(move || {
            let mut counter = 0.0;
            while !stop_flag.load(Ordering::Relaxed) {
                let samples: Vec<f64> = (0..1000)
                    .map(|i| {
                        counter += 1.0;
                        (counter + i as f64) % 1000.0
                    })
                    .collect();

                producer.write(&samples);
                thread::sleep(Duration::from_micros(100));
            }
        })
    };

    // Multiple consumer threads (reading from same consumer)
    let num_readers = 3;
    let mut reader_handles = vec![];

    for reader_id in 0..num_readers {
        let consumer = consumer.clone();
        let stop_flag = stop_flag.clone();
        let total_read = total_read.clone();

        let handle = thread::spawn(move || {
            let mut local_read = 0;
            while !stop_flag.load(Ordering::Relaxed) {
                let mut buffer = vec![0.0; 100];
                let read = consumer.read(&mut buffer);
                local_read += read;

                if read == 0 {
                    thread::sleep(Duration::from_micros(100));
                }
            }
            println!("Reader {} read {} samples", reader_id, local_read);
            total_read.fetch_add(local_read, Ordering::Relaxed);
        });

        reader_handles.push(handle);
    }

    // Run for 100ms
    thread::sleep(Duration::from_millis(100));
    stop_flag.store(true, Ordering::Relaxed);

    producer_handle.join().unwrap();
    for handle in reader_handles {
        handle.join().unwrap();
    }

    let total = total_read.load(Ordering::Relaxed);
    println!("Total samples read by all readers: {}", total);

    // Should have read a significant amount
    assert!(total > 1000, "Should have read at least 1000 samples total");
}

#[test]
fn test_concurrent_stress_test() {
    let format = AudioFormat::new(44100, 2, SampleFormat::F64);
    let config = RingBufferConfig::low_latency(format); // Use smaller buffer for more contention

    let (producer, consumer) = AudioRingBuffer::new(config).unwrap();

    let producer = Arc::new(producer);
    let consumer = Arc::new(consumer);

    let stop_flag = Arc::new(AtomicBool::new(false));
    let write_count = Arc::new(AtomicUsize::new(0));
    let read_count = Arc::new(AtomicUsize::new(0));
    let underrun_count = Arc::new(AtomicUsize::new(0));

    // Fast producer
    let producer_handle = {
        let producer = producer.clone();
        let stop_flag = stop_flag.clone();
        let write_count = write_count.clone();

        thread::spawn(move || {
            let mut value = 0.0;
            while !stop_flag.load(Ordering::Relaxed) {
                let samples: Vec<f64> = (0..10)
                    .map(|_| {
                        value += 1.0;
                        value
                    })
                    .collect();

                let written = producer.write(&samples);
                write_count.fetch_add(written, Ordering::Relaxed);

                // No sleep - write as fast as possible
            }
        })
    };

    // Fast consumer
    let consumer_handle = {
        let consumer = consumer.clone();
        let stop_flag = stop_flag.clone();
        let read_count = read_count.clone();
        let underrun_count = underrun_count.clone();

        thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                let mut buffer = vec![0.0; 10];
                let read = consumer.read(&mut buffer);
                read_count.fetch_add(read, Ordering::Relaxed);

                // Check for underrun
                if consumer.is_underrun(0.1) {
                    underrun_count.fetch_add(1, Ordering::Relaxed);
                }

                // No sleep - read as fast as possible
            }
        })
    };

    // Run for 50ms
    thread::sleep(Duration::from_millis(50));
    stop_flag.store(true, Ordering::Relaxed);

    producer_handle.join().unwrap();
    consumer_handle.join().unwrap();

    let written = write_count.load(Ordering::Relaxed);
    let read = read_count.load(Ordering::Relaxed);
    let underruns = underrun_count.load(Ordering::Relaxed);

    println!(
        "Stress test: written={}, read={}, underruns={}",
        written, read, underruns
    );

    // Should have processed many samples
    assert!(written > 100, "Should have written many samples");
    assert!(read > 100, "Should have read many samples");

    // Underruns are expected in stress test with small buffer
    println!("Underruns detected: {}", underruns);
}

#[test]
fn test_concurrent_wrap_around() {
    let format = AudioFormat::new(44100, 1, SampleFormat::F64);
    let config = RingBufferConfig {
        buffer_duration_seconds: 0.1, // Small buffer to force wrap-around
        format,
        allow_overwrite: false,
        underrun_threshold: 0.1,
    };

    let (producer, consumer) = AudioRingBuffer::new(config).unwrap();

    let producer = Arc::new(producer);
    let consumer = Arc::new(consumer);

    let stop_flag = Arc::new(AtomicBool::new(false));
    let wrap_count = Arc::new(AtomicUsize::new(0));

    // Get capacity from consumer
    let capacity = consumer.capacity();

    // Producer thread
    let producer_handle = {
        let producer = producer.clone();
        let stop_flag = stop_flag.clone();
        let wrap_count = wrap_count.clone();

        thread::spawn(move || {
            let mut total_written = 0;

            while !stop_flag.load(Ordering::Relaxed) {
                let samples = vec![1.0; 50];
                let written = producer.write(&samples);
                total_written += written;

                // Check if we've wrapped around
                if total_written > capacity {
                    wrap_count.fetch_add(1, Ordering::Relaxed);
                    total_written = 0;
                }

                thread::sleep(Duration::from_micros(10));
            }
        })
    };

    // Consumer thread
    let consumer_handle = {
        let consumer = consumer.clone();
        let stop_flag = stop_flag.clone();

        thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                let mut buffer = vec![0.0; 50];
                consumer.read(&mut buffer);
                thread::sleep(Duration::from_micros(10));
            }
        })
    };

    // Run for 100ms
    thread::sleep(Duration::from_millis(100));
    stop_flag.store(true, Ordering::Relaxed);

    producer_handle.join().unwrap();
    consumer_handle.join().unwrap();

    let wraps = wrap_count.load(Ordering::Relaxed);
    println!("Buffer wrapped around {} times", wraps);

    // Should have wrapped around multiple times with small buffer
    assert!(wraps > 0, "Buffer should have wrapped around at least once");
}

#[test]
fn test_concurrent_peek_and_read() {
    let format = AudioFormat::new(44100, 2, SampleFormat::F64);
    let config = RingBufferConfig::standard(format);

    let (producer, consumer) = AudioRingBuffer::new(config).unwrap();

    let producer = Arc::new(producer);
    let consumer = Arc::new(consumer);

    let stop_flag = Arc::new(AtomicBool::new(false));

    // Producer thread
    let producer_handle = {
        let producer = producer.clone();
        let stop_flag = stop_flag.clone();

        thread::spawn(move || {
            let mut value = 0.0;
            while !stop_flag.load(Ordering::Relaxed) {
                let samples: Vec<f64> = (0..100)
                    .map(|_| {
                        value += 1.0;
                        value
                    })
                    .collect();

                producer.write(&samples);
                thread::sleep(Duration::from_micros(100));
            }
        })
    };

    // Peek thread
    let peek_handle = {
        let consumer = consumer.clone();
        let stop_flag = stop_flag.clone();

        thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                let mut buffer = vec![0.0; 50];
                consumer.peek(&mut buffer);
                thread::sleep(Duration::from_micros(50));
            }
        })
    };

    // Read thread
    let read_handle = {
        let consumer = consumer.clone();
        let stop_flag = stop_flag.clone();

        thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                let mut buffer = vec![0.0; 50];
                consumer.read(&mut buffer);
                thread::sleep(Duration::from_micros(150));
            }
        })
    };

    // Run for 100ms
    thread::sleep(Duration::from_millis(100));
    stop_flag.store(true, Ordering::Relaxed);

    producer_handle.join().unwrap();
    peek_handle.join().unwrap();
    read_handle.join().unwrap();

    // Test passes if no panics or data corruption
    println!("Concurrent peek and read test completed successfully");
}

#[test]
fn test_concurrent_status_monitoring() {
    let format = AudioFormat::new(44100, 2, SampleFormat::F64);
    let config = RingBufferConfig::standard(format);

    let (producer, consumer) = AudioRingBuffer::new(config).unwrap();

    let producer = Arc::new(producer);
    let consumer = Arc::new(consumer);

    let stop_flag = Arc::new(AtomicBool::new(false));
    let max_utilization = Arc::new(AtomicUsize::new(0));

    // Producer thread
    let producer_handle = {
        let producer = producer.clone();
        let stop_flag = stop_flag.clone();

        thread::spawn(move || {
            let mut value = 0.0;
            while !stop_flag.load(Ordering::Relaxed) {
                let samples: Vec<f64> = (0..1000)
                    .map(|_| {
                        value += 1.0;
                        value
                    })
                    .collect();

                producer.write(&samples);
                thread::sleep(Duration::from_micros(500));
            }
        })
    };

    // Consumer thread
    let consumer_handle = {
        let consumer = consumer.clone();
        let stop_flag = stop_flag.clone();

        thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                let mut buffer = vec![0.0; 500];
                consumer.read(&mut buffer);
                thread::sleep(Duration::from_micros(1000));
            }
        })
    };

    // Monitor thread
    let monitor_handle = {
        let consumer = consumer.clone();
        let stop_flag = stop_flag.clone();
        let max_utilization = max_utilization.clone();

        thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                let status = consumer.status();
                let utilization_percent = (status.utilization * 100.0) as usize;

                let current_max = max_utilization.load(Ordering::Relaxed);
                if utilization_percent > current_max {
                    max_utilization.store(utilization_percent, Ordering::Relaxed);
                }

                thread::sleep(Duration::from_millis(1));
            }
        })
    };

    // Run for 100ms
    thread::sleep(Duration::from_millis(100));
    stop_flag.store(true, Ordering::Relaxed);

    producer_handle.join().unwrap();
    consumer_handle.join().unwrap();
    monitor_handle.join().unwrap();

    let max_util = max_utilization.load(Ordering::Relaxed);
    println!("Maximum buffer utilization: {}%", max_util);

    // Should have seen some buffer utilization
    assert!(max_util > 0, "Should have detected some buffer utilization");
}

#[test]
fn test_concurrent_underrun_detection() {
    let format = AudioFormat::new(44100, 1, SampleFormat::F64);
    let config = RingBufferConfig::low_latency(format);

    let (producer, consumer) = AudioRingBuffer::new(config).unwrap();

    let producer = Arc::new(producer);
    let consumer = Arc::new(consumer);

    let stop_flag = Arc::new(AtomicBool::new(false));

    // Slow producer (intentionally cause underruns)
    let producer_handle = {
        let producer = producer.clone();
        let stop_flag = stop_flag.clone();

        thread::spawn(move || {
            let mut value = 0.0;
            while !stop_flag.load(Ordering::Relaxed) {
                let samples: Vec<f64> = (0..10)
                    .map(|_| {
                        value += 1.0;
                        value
                    })
                    .collect();

                producer.write(&samples);
                thread::sleep(Duration::from_millis(5)); // Slow producer
            }
        })
    };

    // Fast consumer
    let consumer_handle = {
        let consumer = consumer.clone();
        let stop_flag = stop_flag.clone();

        thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                let mut buffer = vec![0.0; 100];
                consumer.read(&mut buffer);

                // Check health
                consumer.check_health(0.05);

                thread::sleep(Duration::from_micros(100)); // Fast consumer
            }
        })
    };

    // Run for 100ms
    thread::sleep(Duration::from_millis(100));
    stop_flag.store(true, Ordering::Relaxed);

    producer_handle.join().unwrap();
    consumer_handle.join().unwrap();

    let underruns = consumer.underrun_count();
    println!("Detected {} underruns", underruns);

    // Should have detected underruns with slow producer
    assert!(
        underruns > 0,
        "Should have detected underruns with slow producer"
    );
}
