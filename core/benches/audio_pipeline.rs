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

/// Benchmark CPU-intensive operations with different workload sizes
fn benchmark_cpu_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_usage");

    // Test different workload sizes
    let workload_sizes = [1000, 10000, 100000];

    for size in workload_sizes {
        let samples: Vec<i16> = (0..size).map(|i| (i % 32767) as i16).collect();

        // CPU-intensive: Multiple passes of processing
        group.bench_with_input(
            BenchmarkId::new("multi_pass_processing", size),
            &samples,
            |b, samples| {
                b.iter(|| {
                    let mut output = samples.clone();

                    // Pass 1: Volume adjustment
                    for sample in &mut output {
                        *sample = (*sample as f32 * 0.8) as i16;
                    }

                    // Pass 2: DC offset removal
                    let avg: i32 =
                        output.iter().map(|&s| s as i32).sum::<i32>() / output.len() as i32;
                    for sample in &mut output {
                        *sample = (*sample as i32 - avg) as i16;
                    }

                    // Pass 3: Normalization
                    let peak = output.iter().map(|&s| s.abs()).max().unwrap_or(1);
                    let scale = i16::MAX as f32 / peak as f32;
                    for sample in &mut output {
                        *sample = (*sample as f32 * scale) as i16;
                    }

                    black_box(output)
                })
            },
        );

        // CPU-intensive: FFT-like computation (simplified)
        group.bench_with_input(
            BenchmarkId::new("spectral_analysis", size),
            &samples,
            |b, samples| {
                b.iter(|| {
                    let mut spectrum = vec![0.0f64; samples.len()];

                    // Simplified spectral computation (not real FFT)
                    for (i, &sample) in samples.iter().enumerate() {
                        let normalized = sample as f64 / i16::MAX as f64;
                        spectrum[i] = normalized * normalized;
                    }

                    // Compute magnitude
                    let magnitude: f64 = spectrum.iter().sum();

                    black_box(magnitude)
                })
            },
        );

        // CPU-intensive: Convolution (simplified)
        group.bench_with_input(
            BenchmarkId::new("convolution", size),
            &samples,
            |b, samples| {
                let kernel = [0.2f32, 0.2, 0.2, 0.2, 0.2]; // Simple averaging kernel
                let kernel_size = kernel.len();

                b.iter(|| {
                    let mut output = Vec::with_capacity(samples.len());

                    for i in 0..samples.len() {
                        let mut sum = 0.0f32;
                        let mut count = 0;

                        for (j, &k) in kernel.iter().enumerate() {
                            let idx = i as i32 + j as i32 - (kernel_size / 2) as i32;
                            if idx >= 0 && idx < samples.len() as i32 {
                                sum += samples[idx as usize] as f32 * k;
                                count += 1;
                            }
                        }

                        output.push((sum / count as f32) as i16);
                    }

                    black_box(output)
                })
            },
        );

        // CPU-intensive: Resampling simulation
        group.bench_with_input(
            BenchmarkId::new("resampling", size),
            &samples,
            |b, samples| {
                b.iter(|| {
                    let ratio = 1.5; // Upsample by 1.5x
                    let new_size = (samples.len() as f64 * ratio) as usize;
                    let mut output = Vec::with_capacity(new_size);

                    for i in 0..new_size {
                        let src_pos = i as f64 / ratio;
                        let src_idx = src_pos as usize;
                        let frac = src_pos - src_idx as f64;

                        let sample = if src_idx + 1 < samples.len() {
                            // Linear interpolation
                            let s1 = samples[src_idx] as f64;
                            let s2 = samples[src_idx + 1] as f64;
                            (s1 + (s2 - s1) * frac) as i16
                        } else {
                            samples[src_idx]
                        };

                        output.push(sample);
                    }

                    black_box(output)
                })
            },
        );

        // CPU-intensive: Dynamic range compression
        group.bench_with_input(
            BenchmarkId::new("compression", size),
            &samples,
            |b, samples| {
                b.iter(|| {
                    let threshold = 16384i16; // -6dB
                    let ratio = 4.0f32; // 4:1 compression
                    let mut output = Vec::with_capacity(samples.len());

                    for &sample in samples {
                        let abs_sample = sample.abs();
                        let compressed = if abs_sample > threshold {
                            let over = abs_sample - threshold;
                            let compressed_over = (over as f32 / ratio) as i16;
                            let result = threshold + compressed_over;
                            if sample < 0 {
                                -result
                            } else {
                                result
                            }
                        } else {
                            sample
                        };
                        output.push(compressed);
                    }

                    black_box(output)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark sustained CPU load scenarios
fn benchmark_sustained_cpu_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("sustained_cpu_load");
    group.sample_size(10); // Fewer samples for long-running tests

    let sample_count = 441000; // 10 seconds at 44.1kHz
    let samples: Vec<i16> = (0..sample_count).map(|i| (i % 32767) as i16).collect();

    // Simulate real-time processing workload
    group.bench_function("realtime_processing_chain", |b| {
        b.iter(|| {
            let mut output = samples.clone();

            // Chain of processing operations
            // 1. Volume adjustment
            for sample in &mut output {
                *sample = (*sample as f32 * 0.9) as i16;
            }

            // 2. EQ simulation (3-band)
            let window_size = 100;
            for i in window_size..output.len() - window_size {
                let low: i32 = output[i - window_size..i].iter().map(|&s| s as i32).sum();
                let mid = output[i] as i32;
                let high: i32 = output[i..i + window_size].iter().map(|&s| s as i32).sum();

                let eq_sample = (low / window_size as i32 + mid + high / window_size as i32) / 3;
                output[i] = eq_sample.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            }

            // 3. Limiter
            let limit = 28000i16;
            for sample in &mut output {
                if sample.abs() > limit {
                    *sample = if *sample > 0 { limit } else { -limit };
                }
            }

            black_box(output)
        })
    });

    // Simulate batch processing workload
    group.bench_function("batch_processing", |b| {
        b.iter(|| {
            let mut results = Vec::new();

            // Process in chunks
            for chunk in samples.chunks(4410) {
                // 100ms chunks
                // Calculate RMS
                let rms = contexture_core::audio::checksum::calculate_rms(chunk);

                // Calculate peak
                let peak = contexture_core::audio::checksum::calculate_peak(chunk);

                // Calculate checksum
                let checksum = contexture_core::audio::checksum::calculate_simple_checksum(chunk);

                results.push((rms, peak, checksum));
            }

            black_box(results)
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
    benchmark_memory_pooling,
    benchmark_cpu_intensive_processing,
    benchmark_parallel_processing,
    benchmark_simd_operations,
    benchmark_cpu_usage,
    benchmark_sustained_cpu_load
);
criterion_main!(benches);

/// Benchmark CPU-intensive audio processing operations
fn benchmark_cpu_intensive_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_intensive");

    let sample_count = 44100; // 1 second at 44.1kHz
    let samples: Vec<i16> = (0..sample_count).map(|i| (i % 32767) as i16).collect();

    // Volume adjustment (simple multiplication)
    group.bench_function("volume_adjustment", |b| {
        b.iter(|| {
            let volume = 0.75;
            let output: Vec<i16> = samples
                .iter()
                .map(|&s| (s as f32 * volume) as i16)
                .collect();
            black_box(output)
        })
    });

    // Mixing two audio streams
    group.bench_function("audio_mixing", |b| {
        let samples2: Vec<i16> = (0..sample_count)
            .map(|i| ((i + 1000) % 32767) as i16)
            .collect();

        b.iter(|| {
            let output: Vec<i16> = samples
                .iter()
                .zip(samples2.iter())
                .map(|(&s1, &s2)| {
                    let mixed = (s1 as i32 + s2 as i32) / 2;
                    mixed.clamp(i16::MIN as i32, i16::MAX as i32) as i16
                })
                .collect();
            black_box(output)
        })
    });

    // Simple low-pass filter (moving average)
    group.bench_function("lowpass_filter", |b| {
        b.iter(|| {
            let window_size = 5;
            let mut output = Vec::with_capacity(sample_count);

            for i in 0..sample_count {
                let start = i.saturating_sub(window_size / 2);
                let end = (i + window_size / 2 + 1).min(sample_count);

                let sum: i32 = samples[start..end].iter().map(|&s| s as i32).sum();
                let avg = sum / (end - start) as i32;
                output.push(avg as i16);
            }

            black_box(output)
        })
    });

    // Normalization (find peak and scale)
    group.bench_function("normalization", |b| {
        b.iter(|| {
            let peak = samples.iter().map(|&s| s.abs()).max().unwrap_or(1);
            let scale = i16::MAX as f32 / peak as f32;

            let output: Vec<i16> = samples.iter().map(|&s| (s as f32 * scale) as i16).collect();
            black_box(output)
        })
    });

    // Fade in/out
    group.bench_function("fade_in_out", |b| {
        b.iter(|| {
            let fade_samples = 4410; // 0.1 second fade
            let mut output = samples.clone();

            // Fade in
            for (i, sample) in output
                .iter_mut()
                .enumerate()
                .take(fade_samples.min(sample_count))
            {
                let factor = i as f32 / fade_samples as f32;
                *sample = (*sample as f32 * factor) as i16;
            }

            // Fade out
            let fade_start = sample_count.saturating_sub(fade_samples);
            for (i, sample) in output
                .iter_mut()
                .enumerate()
                .skip(fade_start)
                .take(sample_count - fade_start)
            {
                let factor = (sample_count - i) as f32 / fade_samples as f32;
                *sample = (*sample as f32 * factor) as i16;
            }

            black_box(output)
        })
    });

    group.finish();
}

/// Benchmark parallel processing capabilities
fn benchmark_parallel_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_processing");

    let sample_count = 441000; // 10 seconds at 44.1kHz
    let samples: Vec<i16> = (0..sample_count).map(|i| (i % 32767) as i16).collect();

    // Sequential processing
    group.bench_function("sequential_volume", |b| {
        b.iter(|| {
            let volume = 0.75;
            let output: Vec<i16> = samples
                .iter()
                .map(|&s| (s as f32 * volume) as i16)
                .collect();
            black_box(output)
        })
    });

    // Sequential checksum
    group.bench_function("sequential_checksum", |b| {
        b.iter(|| {
            let checksum = contexture_core::audio::checksum::calculate_simple_checksum(&samples);
            black_box(checksum)
        })
    });

    group.finish();
}

/// Benchmark SIMD-friendly operations
fn benchmark_simd_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_operations");

    let size = 100000;
    let samples: Vec<i16> = (0..size).map(|i| (i % 32767) as i16).collect();

    // Scalar addition
    group.bench_function("scalar_add", |b| {
        let samples2: Vec<i16> = (0..size).map(|i| ((i + 1000) % 32767) as i16).collect();

        b.iter(|| {
            let mut output = vec![0i16; size];
            for i in 0..size {
                output[i] = samples[i].saturating_add(samples2[i]);
            }
            black_box(output)
        })
    });

    // Scalar multiplication
    group.bench_function("scalar_multiply", |b| {
        b.iter(|| {
            let factor = 0.5f32;
            let mut output = vec![0i16; size];
            for i in 0..size {
                output[i] = (samples[i] as f32 * factor) as i16;
            }
            black_box(output)
        })
    });

    // Chunked processing (SIMD-friendly)
    group.bench_function("chunked_multiply", |b| {
        b.iter(|| {
            let factor = 0.5f32;
            let output: Vec<i16> = samples
                .chunks_exact(4)
                .flat_map(|chunk| chunk.iter().map(|&s| (s as f32 * factor) as i16))
                .collect();
            black_box(output)
        })
    });

    group.finish();
}
