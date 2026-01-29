# Scripts Directory

This directory contains utility scripts for the Contexture project.

## Available Scripts

### `setup_audio.sh`

Sets up the audio environment for testing. Works on Linux, macOS, and Windows.

**Usage:**

```bash
# Interactive setup (local development)
./scripts/setup_audio.sh

# CI mode (minimal output)
./scripts/setup_audio.sh --ci

# Cleanup audio devices
./scripts/setup_audio.sh --cleanup

# Show help
./scripts/setup_audio.sh --help
```

**What it does:**

- **Linux**: Creates a PulseAudio null sink for testing without physical audio hardware
- **macOS**: Checks for BlackHole virtual audio device (optional)
- **Windows**: Uses default audio device

**Requirements:**

- Linux: PulseAudio (auto-installed in CI mode)
- macOS: No requirements (BlackHole optional)
- Windows: No requirements

### `ci-local.sh`

Runs the full CI pipeline locally before pushing to GitHub.

**Usage:**

```bash
./scripts/ci-local.sh
```

**What it does:**

1. Runs code formatting checks
2. Runs Clippy lints
3. Runs all tests
4. Runs benchmarks (optional)
5. Checks code coverage (optional)

## CI Integration

These scripts are used in GitHub Actions workflows:

- `.github/workflows/ci.yml` - Main CI pipeline
- `.github/workflows/performance.yml` - Performance benchmarks
- `.github/workflows/release.yml` - Release builds

## Development Workflow

### First Time Setup

```bash
# 1. Setup audio environment
./scripts/setup_audio.sh

# 2. Generate test audio files
cd core
cargo run --example generate_test_audio

# 3. Run tests
cargo test
```

### Before Committing

```bash
# Run local CI checks
./scripts/ci-local.sh
```

### Cleanup

```bash
# Remove virtual audio devices (Linux only)
./scripts/setup_audio.sh --cleanup
```

## Troubleshooting

### Linux: PulseAudio Issues

```bash
# Kill and restart PulseAudio
pulseaudio --kill
pulseaudio --start

# Check status
pulseaudio --check && echo "Running" || echo "Not running"

# List audio sinks
pactl list short sinks
```

### macOS: No Audio Device

```bash
# Install BlackHole (optional)
brew install blackhole-2ch

# List audio devices
system_profiler SPAudioDataType
```

### Script Permission Denied

```bash
# Make scripts executable
chmod +x scripts/*.sh
```

## Contributing

When adding new scripts:

1. Add a shebang line: `#!/bin/bash`
2. Add error handling: `set -e`
3. Add help text: `--help` flag
4. Document in this README
5. Make executable: `chmod +x`
6. Test on multiple platforms if possible

## License

These scripts are part of the Contexture project and follow the same license.
