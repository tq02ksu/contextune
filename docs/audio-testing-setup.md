# Audio Testing Setup Guide

This guide explains how to set up your local environment for audio quality testing.

## Overview

The Contextune project includes comprehensive audio quality tests that verify:
- Bit-perfect playback
- Frequency response accuracy
- THD+N (Total Harmonic Distortion + Noise) measurements
- Audio format conversion accuracy

These tests can run without physical audio hardware by using virtual audio devices.

## Prerequisites

### All Platforms

- Rust toolchain (stable or nightly)
- Cargo package manager

### Platform-Specific Requirements

#### Linux (Ubuntu/Debian)

```bash
# Install audio dependencies
sudo apt-get update
sudo apt-get install -y libasound2-dev pulseaudio pulseaudio-utils

# Install development tools
sudo apt-get install -y build-essential pkg-config
```

#### macOS

```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Optional: Install BlackHole for virtual audio routing
brew install blackhole-2ch
```

#### Windows

- Install Visual Studio Build Tools or Visual Studio with C++ development tools
- Audio drivers are included with Windows

## Setup Instructions

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/contextune.git
cd contextune
```

### 2. Generate Reference Audio Files

```bash
cd core
cargo run --example generate_test_audio
```

This will generate reference audio files in `core/test_data/reference/`:
- Sine waves at various frequencies (100Hz, 440Hz, 1kHz, 5kHz, 10kHz)
- White noise
- Impulse signals
- Silence
- Frequency sweeps

Total size: approximately 74MB

### 3. Setup Virtual Audio (Linux Only)

For Linux systems, set up a PulseAudio null sink for testing:

```bash
# Run setup script
./scripts/setup_audio.sh
```

The script will:
- Detect your operating system
- Install PulseAudio if needed (on Linux)
- Create a virtual audio sink for testing
- Provide helpful next steps

**Options:**

```bash
# Run in CI mode (minimal output)
./scripts/setup_audio.sh --ci

# Cleanup audio devices
./scripts/setup_audio.sh --cleanup

# Show help
./scripts/setup_audio.sh --help
```

**Manual setup (if needed):**

```bash
# Start PulseAudio
pulseaudio --start

# Create null sink
pactl load-module module-null-sink \
  sink_name=contextune_test_sink \
  sink_properties=device.description="Contextune_Test_Sink" \
  rate=48000 \
  channels=2

# Verify
pactl list short sinks
```

### 4. Run Audio Tests

```bash
cd core

# Run all audio quality tests
cargo test --test audio_quality
cargo test --test bit_perfect_verification
cargo test --test frequency_response
cargo test --test thd_n_measurement

# Or run all tests at once
cargo test
```

## Test Suites

### Bit-Perfect Verification Tests

Verifies that audio data is not modified during processing:

```bash
cargo test --test bit_perfect_verification -- --nocapture
```

Tests include:
- Audio checksum calculation
- WAV file reading accuracy
- Sample-by-sample comparison
- Sine wave properties verification
- White noise statistical properties

### Frequency Response Tests

Analyzes frequency content using FFT:

```bash
cargo test --test frequency_response -- --nocapture
```

Tests include:
- Frequency detection accuracy (±5Hz)
- Signal-to-noise ratio (SNR > 60dB)
- White noise spectrum flatness
- Impulse response characteristics
- Frequency sweep coverage

### THD+N Measurement Tests

Measures total harmonic distortion plus noise:

```bash
cargo test --test thd_n_measurement -- --nocapture
```

Tests include:
- THD+N < 1% for pure sine waves
- THD+N < -40dB in decibels
- Harmonic distortion analysis
- Different sample rates (44.1/48/96/192 kHz)
- Different bit depths (16/24 bit)

## Expected Results

### Bit-Perfect Tests
- All samples should match exactly
- Checksums should be deterministic
- RMS values within 1% of expected

### Frequency Response Tests
- Frequency detection within ±5Hz
- SNR > 60dB for pure tones
- Flat spectrum for white noise (±20%)

### THD+N Tests
- THD+N < 0.01% (typical: ~0.002%)
- THD+N < -80dB for 16-bit audio
- THD+N < -94dB achieved in practice

## Troubleshooting

### Linux: PulseAudio Not Starting

```bash
# Kill existing PulseAudio instances
pulseaudio --kill

# Start fresh
pulseaudio --start --log-level=debug

# Check status
pulseaudio --check && echo "Running" || echo "Not running"
```

### macOS: No Audio Device Found

```bash
# List audio devices
system_profiler SPAudioDataType

# Install BlackHole for virtual audio
brew install blackhole-2ch

# Configure in Audio MIDI Setup app
open "/Applications/Utilities/Audio MIDI Setup.app"
```

### Windows: Build Errors

Ensure Visual Studio Build Tools are installed with C++ support:
- Download from: https://visualstudio.microsoft.com/downloads/
- Select "Desktop development with C++"

### Test Files Not Found

If tests report missing files:

```bash
cd core
cargo run --example generate_test_audio
```

Verify files exist:

```bash
ls -lh core/test_data/reference/
```

## CI/CD Integration

The audio tests are automatically run in GitHub Actions CI:

- **Linux**: Uses PulseAudio null sink
- **macOS**: Uses default CoreAudio device
- **Windows**: Uses default audio device

See `.github/workflows/ci.yml` for the complete CI configuration.

## Performance Considerations

### Test Execution Time

- Bit-perfect tests: ~0.2 seconds
- Frequency response tests: ~1.7 seconds
- THD+N tests: ~1.5 seconds
- Total: ~3.5 seconds

### Disk Space

- Reference audio files: ~74MB
- Test artifacts: ~10MB
- Total: ~84MB

### Memory Usage

- FFT operations: ~50MB peak
- Audio file loading: ~20MB
- Total: ~70MB peak

## Advanced Configuration

### Custom Test Frequencies

Edit `core/examples/generate_test_audio.rs` to add custom test frequencies:

```rust
let test_frequencies = [
    (100.0, "100Hz"),
    (440.0, "440Hz"),
    (1000.0, "1kHz"),
    // Add your custom frequencies here
    (2000.0, "2kHz"),
];
```

### Custom Sample Rates

```rust
let sample_rates = [44100, 48000, 96000, 192000];
// Add custom sample rates
```

### Custom Bit Depths

```rust
let bit_depths = [16, 24];
// Note: 24-bit requires special handling in tests
```

## References

- [PulseAudio Documentation](https://www.freedesktop.org/wiki/Software/PulseAudio/)
- [BlackHole Audio Driver](https://github.com/ExistentialAudio/BlackHole)
- [Audio Quality Measurement Standards](https://en.wikipedia.org/wiki/Audio_system_measurements)
- [THD+N Measurement](https://en.wikipedia.org/wiki/Total_harmonic_distortion)

## Support

For issues or questions:
- Open an issue on GitHub
- Check existing issues for solutions
- Consult the main README.md for general setup

## License

This documentation is part of the Contextune project and follows the same license.
