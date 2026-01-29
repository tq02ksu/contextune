# Performance Testing and Regression Detection

This document describes how to run performance benchmarks and detect performance regressions in the Contexture audio core.

## Overview

The project uses [Criterion.rs](https://github.com/bheisler/criterion.rs) for benchmarking and includes custom scripts for regression detection.

## Quick Start

### 1. Run Benchmarks

Run all benchmarks:

```bash
cargo bench
```

Run specific benchmark group:

```bash
cargo bench --bench audio_pipeline -- cpu_usage
```

### 2. Establish Baseline

Before making changes, establish a performance baseline:

```bash
./scripts/benchmark-regression.sh baseline
```

This will:
- Run all benchmarks
- Save results to `target/criterion-baseline/`
- Create a baseline for future comparisons

### 3. Make Changes

Make your code changes, optimizations, or add new features.

### 4. Compare Performance

After making changes, compare against the baseline:

```bash
./scripts/benchmark-regression.sh compare
```

This will:
- Run benchmarks with your changes
- Compare against the baseline
- Report any regressions or improvements
- Exit with error code if regressions detected (>5% slower)

### 5. Update Baseline (Optional)

If your changes improve performance or you want to accept the new baseline:

```bash
./scripts/benchmark-regression.sh update
```

## Benchmark Categories

### Audio Decoding
- WAV file decoding at different sample rates (44.1/48/96 kHz)
- Measures decoding latency

### Checksum Calculation
- Simple hash, CRC32, MD5, SHA256
- Tests with different data sizes (1K to 1M samples)

### Audio Statistics
- RMS calculation
- Peak detection
- Comprehensive audio stats

### Sample Format Conversion
- i16 ↔ f32 ↔ f64 conversions
- Measures conversion overhead

### Memory Operations
- Buffer allocation patterns
- Ring buffer operations
- Memory pooling strategies

### CPU-Intensive Operations
- Multi-pass processing (volume, DC offset, normalization)
- Spectral analysis
- Convolution
- Resampling
- Dynamic range compression

### Sustained CPU Load
- Real-time processing chains
- Batch processing workloads

### Cache Performance
- Sequential vs strided vs random access patterns
- Cache-friendly optimizations

### SIMD Operations
- Scalar vs chunked processing
- SIMD-friendly algorithms

## Regression Detection

### Threshold Configuration

The default regression threshold is **5%**. You can customize it:

**Bash script:**
```bash
# Edit scripts/benchmark-regression.sh
REGRESSION_THRESHOLD=3  # 3% threshold
```

**Python analyzer:**
```bash
python3 scripts/analyze-benchmarks.py --threshold 3.0
```

### Understanding Results

The analyzer categorizes changes as:

- **Regression** ❌: Performance degraded by more than threshold
- **Improvement** ✅: Performance improved by more than threshold
- **Stable** ➖: Performance change within threshold
- **New** 🆕: New benchmark not in baseline
- **Missing** ⚠️: Benchmark removed since baseline

### Example Output

```
================================================================================
BENCHMARK REGRESSION ANALYSIS
================================================================================

Summary:
  Total benchmarks:  45
  Regressions:       2 ❌
  Improvements:      5 ✅
  Stable:            38 ➖
  New:               0 🆕
  Missing:           0 ⚠️
  Threshold:         ±5.0%

--------------------------------------------------------------------------------
REGRESSIONS (slower performance):
--------------------------------------------------------------------------------
  ❌ cpu_usage/convolution/100000
     Baseline: 15.23 ms
     Current:  16.89 ms
     Change:   +10.89% (slower)

  ❌ checksum/md5/1000000
     Baseline: 2.45 ms
     Current:  2.58 ms
     Change:   +5.31% (slower)

--------------------------------------------------------------------------------
IMPROVEMENTS (faster performance):
--------------------------------------------------------------------------------
  ✅ cpu_usage/resampling/100000
     Baseline: 25.67 ms
     Current:  22.14 ms
     Change:   -13.75% (faster)

  ✅ buffer_operations/buffer_write
     Baseline: 3.45 µs
     Current:  2.98 µs
     Change:   -13.62% (faster)
```

## CI Integration

### GitHub Actions

The performance workflow (`.github/workflows/performance.yml`) automatically:

1. Runs benchmarks on every push to main
2. Compares against the previous commit
3. Comments on PRs with performance changes
4. Fails if regressions exceed threshold

### Local CI

Run the same checks locally:

```bash
./scripts/ci-local.sh
```

This includes:
- Code formatting
- Linting (clippy)
- Tests
- Benchmarks (quick mode)

## Best Practices

### 1. Establish Baseline Before Changes

Always create a baseline before starting optimization work:

```bash
git checkout main
./scripts/benchmark-regression.sh baseline
git checkout your-feature-branch
```

### 2. Run Benchmarks Multiple Times

For accurate results, run benchmarks multiple times:

```bash
cargo bench -- --sample-size 100
```

### 3. Minimize System Noise

For consistent results:
- Close unnecessary applications
- Disable CPU frequency scaling (if possible)
- Run on a quiet system
- Use the same hardware for comparisons

### 4. Document Performance Changes

When making performance-critical changes:
- Document expected performance impact
- Include benchmark results in PR description
- Explain any regressions (if acceptable)

### 5. Profile Before Optimizing

Use profiling tools to identify bottlenecks:

```bash
# CPU profiling with perf (Linux)
cargo build --release
perf record --call-graph dwarf ./target/release/examples/audio_checksum
perf report

# Memory profiling with valgrind
cargo build --release
valgrind --tool=massif ./target/release/examples/audio_checksum
```

## Troubleshooting

### Benchmarks Take Too Long

Reduce sample size for quick checks:

```bash
cargo bench -- --sample-size 10
```

### Inconsistent Results

Benchmarks can be affected by:
- System load
- CPU frequency scaling
- Thermal throttling
- Background processes

Try:
1. Closing other applications
2. Running multiple times
3. Using a dedicated benchmark machine

### Python Analyzer Not Found

The bash script falls back to simple comparison if Python is not available.

Install Python 3:

```bash
# macOS
brew install python3

# Ubuntu/Debian
sudo apt-get install python3

# Windows
# Download from python.org
```

### Baseline Not Found

If you see "No baseline found":

```bash
./scripts/benchmark-regression.sh baseline
```

## Advanced Usage

### Custom Benchmark Groups

Add new benchmark groups in `core/benches/audio_pipeline.rs`:

```rust
fn benchmark_my_feature(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_feature");
    
    group.bench_function("my_test", |b| {
        b.iter(|| {
            // Your code here
            black_box(result)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    // ... existing benchmarks
    benchmark_my_feature
);
```

### Benchmark Specific Functions

```bash
cargo bench --bench audio_pipeline -- "cpu_usage/convolution"
```

### Generate HTML Reports

Criterion automatically generates HTML reports in `target/criterion/`:

```bash
# Open in browser (macOS)
open target/criterion/report/index.html

# Linux
xdg-open target/criterion/report/index.html
```

### Export Results

Export benchmark results for external analysis:

```bash
cargo bench -- --save-baseline my-baseline
```

## Performance Targets

### Latency Targets

- WAV decoding (44.1kHz): < 2ms per second of audio
- Checksum calculation: < 1ms per 100K samples
- Sample conversion: < 20µs per 100K samples

### Throughput Targets

- Real-time processing: Must process faster than real-time
- Batch processing: > 10x real-time speed

### Memory Targets

- Buffer allocation: < 100µs for 8KB buffer
- Memory pooling: < 1µs for buffer reuse

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Benchmarking Best Practices](https://easyperf.net/blog/2018/08/26/Basics-of-profiling)
