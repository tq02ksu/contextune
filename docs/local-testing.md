# Local Testing Procedures

This document provides comprehensive guidance for running tests locally during development of the Contextune Music Player Plugin.

## Table of Contents

- [Quick Start](#quick-start)
- [Test Categories](#test-categories)
- [Running Tests](#running-tests)
- [Audio Testing](#audio-testing)
- [Memory Safety Testing](#memory-safety-testing)
- [Security Testing](#security-testing)
- [Performance Testing](#performance-testing)
- [Coverage Testing](#coverage-testing)
- [Troubleshooting](#troubleshooting)
- [CI Simulation](#ci-simulation)

## Quick Start

### Prerequisites

Ensure you have the following installed:

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Audio dependencies (Linux)
sudo apt-get install libasound2-dev pulseaudio

# Audio dependencies (macOS)
# Usually included with Xcode Command Line Tools

# Audio dependencies (Windows)
# Usually included with Windows SDK
```

### Basic Test Run

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Test Categories

### 1. Unit Tests
- **Location**: Inline with source code (`#[cfg(test)]` modules)
- **Purpose**: Test individual functions and modules
- **Command**: `cargo test --lib`

### 2. Integration Tests
- **Location**: `core/tests/` directory
- **Purpose**: Test component interactions
- **Command**: `cargo test --tests`

### 3. Documentation Tests
- **Location**: Embedded in documentation comments
- **Purpose**: Ensure code examples in docs work
- **Command**: `cargo test --doc`

### 4. Audio Quality Tests
- **Location**: `core/tests/audio_quality.rs`, etc.
- **Purpose**: Verify audio processing accuracy
- **Command**: `cargo test-audio`

### 5. Memory Safety Tests
- **Location**: Various test files with sanitizer configurations
- **Purpose**: Detect memory errors and undefined behavior
- **Command**: `cargo test-memory-safety`

### 6. FFI Tests
- **Location**: `core/tests/ffi_integration.rs`
- **Purpose**: Test Foreign Function Interface safety
- **Command**: `cargo test-ffi`

## Running Tests

### Standard Test Commands

```bash
# All tests (recommended for regular development)
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --tests

# Documentation tests only
cargo test --doc

# Test with release optimizations
cargo test --release

# Test specific module
cargo test audio::

# Test with pattern matching
cargo test buffer

# Verbose output
cargo test -- --nocapture

# Show test output even for passing tests
cargo test -- --show-output

# Run tests in single thread (useful for debugging)
cargo test -- --test-threads=1
```

### Audio-Specific Test Commands

```bash
# All audio quality tests
cargo test-audio

# Bit-perfect verification
cargo test --test bit_perfect_verification

# Frequency response tests
cargo test --test frequency_response

# THD+N measurement tests
cargo test --test thd_n_measurement

# CUE parser tests
cargo test-cue
```

### Memory Safety Test Commands

```bash
# All memory safety tests
cargo test-memory-safety

# AddressSanitizer tests (requires nightly)
cargo +nightly test-asan

# Miri tests (undefined behavior detection)
cargo test-miri

# Run with Valgrind (Linux only)
./scripts/run-valgrind-tests.sh
```

## Audio Testing

### Setting Up Audio Environment

#### Linux (PulseAudio)
```bash
# Install PulseAudio
sudo apt-get install pulseaudio pulseaudio-utils

# Set up null sink for testing
./scripts/setup_audio.sh

# Verify setup
pactl list sinks | grep null
```

#### macOS (CoreAudio)
```bash
# Install additional audio tools (optional)
brew install sox

# No special setup required - uses system audio
```

#### Windows (WASAPI)
```bash
# No special setup required - uses system audio
# Ensure Windows SDK is installed for development
```

### Running Audio Tests

```bash
# Generate test audio files first
cd core
cargo run --example generate_test_audio

# Run audio quality tests
cargo test --test audio_quality -- --nocapture

# Run specific audio test
cargo test test_sine_wave_generation -- --nocapture

# Run with specific audio device (if needed)
AUDIO_DEVICE="null" cargo test --test audio_quality
```

### Audio Test Troubleshooting

**No audio device found:**
```bash
# Linux: Check PulseAudio
pulseaudio --check -v
systemctl --user status pulseaudio

# macOS: Check system audio
system_profiler SPAudioDataType

# Windows: Check audio services
sc query audiosrv
```

**Permission denied:**
```bash
# Linux: Add user to audio group
sudo usermod -a -G audio $USER
# Log out and back in

# macOS: Grant microphone access in System Preferences
# Windows: Run as administrator if needed
```

## Memory Safety Testing

### AddressSanitizer (ASan)

```bash
# Install nightly Rust
rustup install nightly

# Run with AddressSanitizer
cargo +nightly test-asan

# Run specific test with ASan
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test test_name --target x86_64-unknown-linux-gnu
```

### MemorySanitizer (MSan)

```bash
# Linux only - requires special setup
./scripts/run-msan-tests.sh
```

### Miri (Undefined Behavior Detection)

```bash
# Install Miri
rustup +nightly component add miri

# Run Miri tests
cargo test-miri

# Run specific test with Miri
cargo +nightly miri test test_name
```

### Valgrind (Linux)

```bash
# Install Valgrind
sudo apt-get install valgrind

# Run tests with Valgrind
./scripts/run-valgrind-tests.sh

# Run specific test with Valgrind
valgrind --tool=memcheck --leak-check=full cargo test test_name
```

## Security Testing

### Vulnerability Scanning

```bash
# Install security tools
cargo install cargo-audit cargo-deny

# Run security audit
cargo audit

# Run dependency policy check
cargo deny-check

# Comprehensive security scan
cargo security-scan

# Run security scan with verbose output
./scripts/security-scan.sh --verbose
```

### Fuzzing

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# List available fuzz targets
cargo fuzz list

# Run fuzzing (example)
cargo fuzz run ffi_safety

# Run fuzzing for specific duration
timeout 60s cargo fuzz run cue_parser
```

## Performance Testing

### Benchmarks

```bash
# Install criterion (if not already installed)
cargo install cargo-criterion

# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench audio_pipeline

# Run benchmarks with baseline
cargo bench -- --save-baseline current

# Compare with previous baseline
cargo bench -- --baseline previous
```

### Profiling

```bash
# Install profiling tools
cargo install cargo-profdata

# Profile with perf (Linux)
perf record --call-graph=dwarf cargo test test_name
perf report

# Profile with Instruments (macOS)
# Use Xcode Instruments GUI

# Profile with Visual Studio (Windows)
# Use Visual Studio Diagnostic Tools
```

## Coverage Testing

### Basic Coverage

```bash
# Install Tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo coverage

# Generate HTML report and open
cargo coverage-html

# Generate coverage for CI
cargo coverage-ci
```

### Advanced Coverage Options

```bash
# Exclude slow tests
cargo coverage --exclude-slow

# Generate specific format
./scripts/coverage-report.sh --xml-only

# Set custom threshold
./scripts/coverage-report.sh --threshold 90

# Generate coverage without threshold check
cargo coverage-check
```

## Troubleshooting

### Common Issues

#### Test Compilation Errors

```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Check for conflicting features
cargo tree --duplicates
```

#### Audio Test Failures

```bash
# Check audio system
# Linux
pulseaudio --check -v

# macOS
sudo launchctl list | grep audio

# Windows
sc query audiosrv
```

#### Memory Test Failures

```bash
# Increase stack size for tests
RUST_MIN_STACK=8388608 cargo test

# Run with single thread
cargo test -- --test-threads=1

# Check system limits
ulimit -a
```

#### Permission Issues

```bash
# Linux: Fix audio permissions
sudo usermod -a -G audio $USER

# macOS: Reset permissions
tccutil reset Microphone

# Windows: Run as administrator
# Right-click terminal -> "Run as administrator"
```

### Debug Mode Testing

```bash
# Run tests in debug mode with full output
RUST_LOG=debug cargo test -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test

# Run with full backtrace
RUST_BACKTRACE=full cargo test
```

### Test-Specific Environment Variables

```bash
# Skip slow tests
SKIP_SLOW_TESTS=1 cargo test

# Use specific audio device
AUDIO_DEVICE=default cargo test

# Set test timeout
TEST_TIMEOUT=60 cargo test

# Enable test logging
RUST_LOG=contextune=debug cargo test
```

## CI Simulation

### Local CI Script

```bash
# Run the same checks as CI
./scripts/ci-local.sh

# Include benchmarks
./scripts/ci-local.sh --bench

# Include coverage
./scripts/ci-local.sh --coverage

# Simulate CI environment
CI=true ./scripts/ci-local.sh
```

### Platform-Specific Testing

```bash
# Test on specific target
cargo test --target x86_64-unknown-linux-gnu

# Cross-compilation testing (if cross is installed)
cross test --target x86_64-pc-windows-gnu

# Test with different feature flags
cargo test --no-default-features
cargo test --all-features
cargo test --features "feature1,feature2"
```

### Docker Testing (Optional)

```bash
# Build test container
docker build -f Dockerfile.test -t contextune-test .

# Run tests in container
docker run --rm contextune-test cargo test

# Interactive testing
docker run --rm -it contextune-test bash
```

## Test Organization Best Practices

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_functionality() {
        // Arrange
        let input = create_test_input();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
    
    #[test]
    #[should_panic(expected = "specific error message")]
    fn test_error_conditions() {
        function_that_should_panic();
    }
    
    #[test]
    #[ignore] // Use for slow tests
    fn test_slow_operation() {
        // Long-running test
    }
}
```

### Test Naming Conventions

- `test_` prefix for all test functions
- Descriptive names: `test_audio_buffer_overflow_protection`
- Group related tests in modules
- Use `#[ignore]` for slow tests
- Use `#[should_panic]` for error condition tests

### Test Data Management

```bash
# Test data location
core/test_data/
├── audio/          # Audio test files
├── cue/           # CUE sheet test files
├── reference/     # Reference data for comparison
└── fixtures/      # Test fixtures and mocks
```

## Continuous Testing

### File Watching

```bash
# Install cargo-watch
cargo install cargo-watch

# Run tests on file changes
cargo watch -x test

# Run specific tests on changes
cargo watch -x "test audio::"

# Run tests and clear screen
cargo watch -c -x test
```

### Pre-commit Testing

```bash
# Install pre-commit hooks (if available)
pre-commit install

# Run pre-commit checks manually
pre-commit run --all-files

# Test before committing
git add . && ./scripts/ci-local.sh && git commit
```

---

For questions about testing procedures, please refer to the [Contributing Guidelines](../CONTRIBUTING.md) or open an issue on GitHub.