//! Audio pipeline performance benchmarks
//!
//! Benchmarks for audio decoding, processing, and output

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::Path;

/// Benchmark WAV file decoding latency
fn benchmark_wav_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("wav_decoding");

    // Test different sample rates
    let sample_rates = [44100, 48000, 96000];

    for sr in sample_rates {
        let file_path = format!("test_data/reference/{}Hz_16bit/sine_1kHz.wav", sr);
        let path = Path::new(&file_path);

        if !path.exists() {
            eprintln!("Warning: Test file not found: {}", file_path);
            continue;
        }

        group.bench_with_input(
            BenchmarkId::new("decode", sr),
            &file_path,
            |b, file_path| {
                b.iter(|| {
                    let mut reader = hound::WavReader::open(file_path).unwrap();
                    let samples: Vec<i16> = reader
                        .samples::<i16>()
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap();
                    black_box(samples)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark audio checksum calculation
fn benchmark_checksum_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("checksum");

    // Generate test data of different sizes
    let sizes = [1000, 10000, 100000, 1000000];

    for size in sizes {
        let samples: Vec<i16> = (0..size).map(|i| (i % 32767) as i16).collect();

        group.bench_with_input(BenchmarkId::new("simple", size), &samples, |b, samples| {
            b.iter(|| {
                black_box(contexture_core::audio::checksum::calculate_simple_checksum(
                    samples,
                ))
            })
        });

        group.bench_with_input(BenchmarkId::new("crc32", size), &samples, |b, samples| {
            b.iter(|| {
                black_box(contexture_core::audio::checksum::calculate_crc32_checksum(
                    samples,
                ))
            })
        });

        group.bench_with_input(BenchmarkId::new("md5", size), &samples, |b, samples| {
            b.iter(|| {
                black_box(contexture_core::audio::checksum::calculate_md5_checksum(
                    samples,
                ))
            })
        });

        group.bench_with_input(BenchmarkId::new("sha256", size), &samples, |b, samples| {
            b.iter(|| {
                black_box(contexture_core::audio::checksum::calculate_sha256_checksum(
                    samples,
                ))
            })
        });
    }

    group.finish();
}

/// Benchmark audio statistics calculation
fn benchmark_audio_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_stats");

    let sizes = [1000, 10000, 100000, 1000000];

    for size in sizes {
        let samples: Vec<i16> = (0..size).map(|i| (i % 32767) as i16).collect();

        group.bench_with_input(
            BenchmarkId::new("calculate_stats", size),
            &samples,
            |b, samples| {
                b.iter(|| {
                    black_box(contexture_core::audio::checksum::calculate_audio_stats(
                        samples,
                    ))
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("calculate_rms", size),
            &samples,
            |b, samples| {
                b.iter(|| black_box(contexture_core::audio::checksum::calculate_rms(samples)))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("calculate_peak", size),
            &samples,
            |b, samples| {
                b.iter(|| black_box(contexture_core::audio::checksum::calculate_peak(samples)))
            },
        );
    }

    group.finish();
}

/// Benchmark sample format conversion
fn benchmark_sample_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample_conversion");

    let size = 100000;
    let samples_i16: Vec<i16> = (0..size).map(|i| (i % 32767) as i16).collect();

    group.bench_function("i16_to_f32", |b| {
        b.iter(|| {
            let converted: Vec<f32> = samples_i16
                .iter()
                .map(|&s| s as f32 / i16::MAX as f32)
                .collect();
            black_box(converted)
        })
    });

    group.bench_function("i16_to_f64", |b| {
        b.iter(|| {
            let converted: Vec<f64> = samples_i16
                .iter()
                .map(|&s| s as f64 / i16::MAX as f64)
                .collect();
            black_box(converted)
        })
    });

    let samples_f32: Vec<f32> = samples_i16
        .iter()
        .map(|&s| s as f32 / i16::MAX as f32)
        .collect();

    group.bench_function("f32_to_i16", |b| {
        b.iter(|| {
            let converted: Vec<i16> = samples_f32
                .iter()
                .map(|&s| (s * i16::MAX as f32) as i16)
                .collect();
            black_box(converted)
        })
    });

    group.finish();
}

/// Benchmark memory allocation patterns
fn benchmark_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    let sizes = [1000, 10000, 100000];

    for size in sizes {
        group.bench_with_input(
            BenchmarkId::new("vec_with_capacity", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut vec = Vec::with_capacity(size);
                    for i in 0..size {
                        vec.push((i % 32767) as i16);
                    }
                    black_box(vec)
                })
            },
        );

        group.bench_with_input(BenchmarkId::new("vec_push", size), &size, |b, &size| {
            b.iter(|| {
                let mut vec = Vec::new();
                for i in 0..size {
                    vec.push((i % 32767) as i16);
                }
                black_box(vec)
            })
        });

        group.bench_with_input(
            BenchmarkId::new("vec_from_iter", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let vec: Vec<i16> = (0..size).map(|i| (i % 32767) as i16).collect();
                    black_box(vec)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark buffer operations (ring buffer simulation)
fn benchmark_buffer_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_operations");

    // Simulate ring buffer operations
    let buffer_size = 8192; // Typical audio buffer size
    let data: Vec<i16> = (0..1024).map(|i| (i % 32767) as i16).collect();

    group.bench_function("buffer_write", |b| {
        let mut buffer = vec![0i16; buffer_size];
        let mut write_pos = 0;
        b.iter(|| {
            for &sample in &data {
                buffer[write_pos] = sample;
                write_pos = (write_pos + 1) % buffer_size;
            }
            black_box(write_pos)
        })
    });

    group.bench_function("buffer_read", |b| {
        let buffer = vec![0i16; buffer_size];
        let mut read_pos = 0;
        b.iter(|| {
            let mut output = Vec::with_capacity(1024);
            for _ in 0..1024 {
                output.push(buffer[read_pos]);
                read_pos = (read_pos + 1) % buffer_size;
            }
            black_box(output)
        })
    });

    group.bench_function("buffer_copy", |b| {
        let buffer = vec![0i16; buffer_size];
        b.iter(|| {
            let mut dest = vec![0i16; 1024];
            dest.copy_from_slice(&buffer[0..1024]);
            black_box(dest)
        })
    });

    group.finish();
}

/// Benchmark audio processing memory patterns
fn benchmark_audio_processing_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_processing_memory");

    let sample_count = 44100; // 1 second at 44.1kHz
    let samples: Vec<i16> = (0..sample_count).map(|i| (i % 32767) as i16).collect();

    // In-place processing
    group.bench_function("in_place_volume", |b| {
        b.iter(|| {
            let mut output = samples.clone();
            let volume = 0.5;
            for sample in &mut output {
                *sample = (*sample as f32 * volume) as i16;
            }
            black_box(output)
        })
    });

    // Allocate new buffer
    group.bench_function("new_buffer_volume", |b| {
        b.iter(|| {
            let volume = 0.5;
            let output: Vec<i16> = samples
                .iter()
                .map(|&s| (s as f32 * volume) as i16)
                .collect();
            black_box(output)
        })
    });

    // Interleaved to planar conversion
    group.bench_function("interleaved_to_planar", |b| {
        let channels = 2;
        let interleaved: Vec<i16> = (0..sample_count * channels)
            .map(|i| (i % 32767) as i16)
            .collect();

        b.iter(|| {
            let mut left = Vec::with_capacity(sample_count);
            let mut right = Vec::with_capacity(sample_count);

            for chunk in interleaved.chunks_exact(2) {
                left.push(chunk[0]);
                right.push(chunk[1]);
            }

            black_box((left, right))
        })
    });

    // Planar to interleaved conversion
    group.bench_function("planar_to_interleaved", |b| {
        let left: Vec<i16> = (0..sample_count).map(|i| (i % 32767) as i16).collect();
        let right: Vec<i16> = (0..sample_count)
            .map(|i| ((i + 1000) % 32767) as i16)
            .collect();

        b.iter(|| {
            let mut interleaved = Vec::with_capacity(sample_count * 2);

            for i in 0..sample_count {
                interleaved.push(left[i]);
                interleaved.push(right[i]);
            }

            black_box(interleaved)
        })
    });

    group.finish();
}

/// Benchmark cache-friendly vs cache-unfriendly access patterns
fn benchmark_cache_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_patterns");

    let size = 100000;
    let data: Vec<i16> = (0..size).map(|i| (i % 32767) as i16).collect();

    // Sequential access (cache-friendly)
    group.bench_function("sequential_access", |b| {
        b.iter(|| {
            let mut sum = 0i64;
            for &sample in &data {
                sum += sample as i64;
            }
            black_box(sum)
        })
    });

    // Strided access (less cache-friendly)
    group.bench_function("strided_access", |b| {
        b.iter(|| {
            let mut sum = 0i64;
            let stride = 16;
            for i in (0..size).step_by(stride) {
                sum += data[i] as i64;
            }
            black_box(sum)
        })
    });

    // Random access (cache-unfriendly)
    group.bench_function("random_access", |b| {
        let indices: Vec<usize> = (0..1000)
            .map(|i| (i * 97) % size) // Pseudo-random pattern
            .collect();

        b.iter(|| {
            let mut sum = 0i64;
            for &idx in &indices {
                sum += data[idx] as i64;
            }
            black_box(sum)
        })
    });

    group.finish();
}

/// Benchmark memory pooling strategies
fn benchmark_memory_pooling(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pooling");

    let buffer_size = 4096;

    // No pooling - allocate every time
    group.bench_function("no_pooling", |b| {
        b.iter(|| {
            let buffer = vec![0i16; buffer_size];
            black_box(buffer)
        })
    });

    // Reuse buffer
    group.bench_function("buffer_reuse", |b| {
        let mut buffer = vec![0i16; buffer_size];
        b.iter(|| {
            buffer.fill(0);
            black_box(buffer.len())
        })
    });

    // Clear and reuse
    group.bench_function("clear_and_reuse", |b| {
        let mut buffer = vec![0i16; buffer_size];
        b.iter(|| {
            buffer.clear();
            buffer.resize(buffer_size, 0);
            black_box(buffer.len())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_wav_decoding,
    benchmark_checksum_calculation,
    benchmark_audio_stats,
    benchmark_sample_conversion,
    benchmark_memory_allocation,
    benchmark_buffer_operations,
    benchmark_audio_processing_memory,
    benchmark_cache_patterns,
    benchmark_memory_pooling
);
criterion_main!(benches);
