//! Audio pipeline performance benchmarks
//!
//! Benchmarks for audio decoding, processing, and output

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
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
        
        group.bench_with_input(
            BenchmarkId::new("simple", size),
            &samples,
            |b, samples| {
                b.iter(|| {
                    black_box(contexture_core::audio::checksum::calculate_simple_checksum(samples))
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("crc32", size),
            &samples,
            |b, samples| {
                b.iter(|| {
                    black_box(contexture_core::audio::checksum::calculate_crc32_checksum(samples))
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("md5", size),
            &samples,
            |b, samples| {
                b.iter(|| {
                    black_box(contexture_core::audio::checksum::calculate_md5_checksum(samples))
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("sha256", size),
            &samples,
            |b, samples| {
                b.iter(|| {
                    black_box(contexture_core::audio::checksum::calculate_sha256_checksum(samples))
                })
            },
        );
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
                    black_box(contexture_core::audio::checksum::calculate_audio_stats(samples))
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("calculate_rms", size),
            &samples,
            |b, samples| {
                b.iter(|| {
                    black_box(contexture_core::audio::checksum::calculate_rms(samples))
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("calculate_peak", size),
            &samples,
            |b, samples| {
                b.iter(|| {
                    black_box(contexture_core::audio::checksum::calculate_peak(samples))
                })
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
        
        group.bench_with_input(
            BenchmarkId::new("vec_push", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut vec = Vec::new();
                    for i in 0..size {
                        vec.push((i % 32767) as i16);
                    }
                    black_box(vec)
                })
            },
        );
        
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

criterion_group!(
    benches,
    benchmark_wav_decoding,
    benchmark_checksum_calculation,
    benchmark_audio_stats,
    benchmark_sample_conversion,
    benchmark_memory_allocation
);
criterion_main!(benches);
