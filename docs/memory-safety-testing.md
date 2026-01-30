# Memory Safety Testing Guide

This document describes how to run memory safety tests for the Contextune music player using Rust's built-in tools and cargo commands.

## Overview

Memory safety testing is crucial for audio applications to prevent crashes, memory leaks, and undefined behavior that could cause audio glitches or system instability.

## Available Tools

### 1. AddressSanitizer (ASan)
Detects memory errors like buffer overflows, use-after-free, and memory leaks.

**Requirements:**
- Rust nightly toolchain
- Linux target (x86_64-unknown-linux-gnu)

**Usage:**
```bash
# Install nightly if not already installed
rustup install nightly

# Run tests with AddressSanitizer
cd core
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test --target x86_64-unknown-linux-gnu

# Or use the provided script
./scripts/run-asan-tests.sh

# Run specific test with ASan
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test --target x86_64-unknown-linux-gnu audio_engine_tests
```

### 2. Miri (Undefined Behavior Detection)
Detects undefined behavior in Rust code.

**Usage:**
```bash
# Install Miri
rustup +nightly component add miri

# Run Miri tests
cd core
cargo +nightly miri test --lib

# Or use cargo alias
cargo +nightly test-miri
```

### 3. MemorySanitizer (MSan)
Detects reads of uninitialized memory.

**Usage:**
```bash
# Run tests with MemorySanitizer
cd core
RUSTFLAGS="-Zsanitizer=memory" cargo +nightly test --target x86_64-unknown-linux-gnu
```

### 4. Valgrind
External tool for memory error detection and profiling.

**Installation:**
```bash
# Ubuntu/Debian
sudo apt-get install valgrind

# macOS
brew install valgrind
```

**Usage:**
```bash
# Build tests first
cd core
cargo build --tests

# Run with Valgrind
valgrind --leak-check=full --error-exitcode=1 target/debug/deps/contextune_core-*
```

## Cargo Aliases

The project includes convenient cargo aliases in `core/.cargo/config.toml`:

```bash
# AddressSanitizer tests
cargo test-asan

# Miri tests
cargo test-miri

# Audio quality tests
cargo test-audio

# CUE parser tests
cargo test-cue

# FFI safety tests
cargo test-ffi

# All integration tests
cargo test-integration
```

## Environment Variables

### AddressSanitizer Options
```bash
export ASAN_OPTIONS="detect_leaks=1:abort_on_error=1:detect_stack_use_after_return=1:check_initialization_order=1:strict_init_order=1"
```

### MemorySanitizer Options
```bash
export MSAN_OPTIONS="abort_on_error=1:print_stats=1"
```

## CI Integration

Memory safety tests are automatically run in GitHub Actions:

- **AddressSanitizer**: Runs on every PR and push to main
- **Miri**: Runs on every PR and push to main
- **Valgrind**: Runs on scheduled basis

## Common Issues and Solutions

### 1. AddressSanitizer False Positives
Some audio libraries may trigger false positives. Use suppression files if needed:

```bash
export ASAN_OPTIONS="$ASAN_OPTIONS:suppressions=asan_suppressions.txt"
```

### 2. Miri Limitations
Miri cannot run tests that:
- Use FFI calls to C libraries
- Perform actual audio I/O
- Use inline assembly

For these cases, focus on unit tests of pure Rust code.

### 3. Performance Impact
Sanitizers significantly slow down execution. Use them for testing, not production builds.

## Best Practices

1. **Run regularly**: Include memory safety tests in your development workflow
2. **Fix immediately**: Address any issues found by sanitizers promptly
3. **Test incrementally**: Run sanitizers on new code before integration
4. **Use multiple tools**: Different tools catch different types of issues
5. **Document suppressions**: If you must suppress warnings, document why

## Integration with Development Workflow

```bash
# Before committing new code
cargo test                    # Standard tests
cargo +nightly test-miri     # Check for undefined behavior
./scripts/run-asan-tests.sh  # Check for memory errors

# Before releasing
cargo test-audio              # Audio quality verification
cargo test-ffi               # FFI safety tests
```

## Troubleshooting

### Nightly Toolchain Issues
```bash
# Update nightly toolchain
rustup update nightly

# Install required components
rustup +nightly component add miri rust-src
```

### Target Issues
```bash
# Add Linux target for cross-compilation
rustup target add x86_64-unknown-linux-gnu
```

### Audio Dependencies
```bash
# Install audio system dependencies (Linux)
sudo apt-get install libasound2-dev pulseaudio
```

For more detailed information, see the main project documentation and CI configuration files.