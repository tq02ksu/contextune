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

### `benchmark-regression.sh`

Detects performance regressions by comparing benchmarks against a baseline.

**Usage:**

```bash
# Save baseline
./scripts/benchmark-regression.sh baseline

# Compare against baseline
./scripts/benchmark-regression.sh compare

# Update baseline
./scripts/benchmark-regression.sh update

# Show help
./scripts/benchmark-regression.sh help
```

**What it does:**

- Runs Criterion benchmarks
- Compares results against saved baseline
- Detects regressions (>5% slower by default)
- Uses Python analyzer for detailed reports (if available)
- Exits with error if regressions detected

**Configuration:**

- Regression threshold: 5% (configurable in script)
- Baseline directory: `target/criterion-baseline`
- Current directory: `target/criterion`

See [../docs/performance-testing.md](../docs/performance-testing.md) for detailed documentation.

### `analyze-benchmarks.py`

Detailed benchmark analysis and regression detection (Python).

**Usage:**

```bash
# Analyze with defaults
python3 scripts/analyze-benchmarks.py

# Custom threshold
python3 scripts/analyze-benchmarks.py --threshold 3.0

# Custom directories
python3 scripts/analyze-benchmarks.py \
    --baseline target/criterion-baseline \
    --current target/criterion \
    --threshold 5.0

# Show help
python3 scripts/analyze-benchmarks.py --help
```

**What it does:**

- Parses Criterion JSON output
- Compares baseline vs current results
- Categorizes changes (regression/improvement/stable/new/missing)
- Generates detailed reports with statistics
- Exits with error if regressions detected

**Requirements:**

- Python 3.6+
- No external dependencies (uses stdlib only)

See [../docs/performance-testing.md](../docs/performance-testing.md) for detailed documentation.

### `visualize-benchmarks.py`

Generate visual reports from benchmark results.

**Usage:**

```bash
# Generate all formats
python3 scripts/visualize-benchmarks.py

# Generate HTML only
python3 scripts/visualize-benchmarks.py --format html

# Custom directories
python3 scripts/visualize-benchmarks.py \
    --criterion-dir target/criterion \
    --output target/benchmark-reports \
    --format all

# Show help
python3 scripts/visualize-benchmarks.py --help
```

**What it does:**

- Generates HTML report with interactive visualizations
- Generates Markdown report for documentation
- Saves JSON results for historical tracking
- Organizes benchmarks by category
- Shows mean, median, and standard deviation

**Output formats:**

- **HTML**: Interactive web report (`index.html`)
- **Markdown**: Text report for PRs (`report.md`)
- **JSON**: Machine-readable data (`latest.json`, `history.jsonl`)

**Requirements:**

- Python 3.6+
- No external dependencies (uses stdlib only)

See [../docs/performance-testing.md](../docs/performance-testing.md) for detailed documentation.

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
