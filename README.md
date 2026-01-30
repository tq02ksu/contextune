# Contexture

[![CI](https://github.com/tq02ksu/contextune/workflows/CI/badge.svg)](https://github.com/tq02ksu/contextune/actions)
[![codecov](https://codecov.io/gh/tq02ksu/contextune/branch/main/graph/badge.svg?token=YOUR_CODECOV_TOKEN)](https://codecov.io/gh/tq02ksu/contextune)
[![Coverage Status](https://img.shields.io/codecov/c/github/tq02ksu/contextune/main.svg)](https://codecov.io/gh/tq02ksu/contextune)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

An intelligent music player plugin for IDEs with high-fidelity audio playback and AI-powered music discovery.

## Features

- 🎵 **HiFi Audio Playback**: Bit-perfect audio with support for FLAC, WAV, ALAC, MP3, AAC, and more
- 🎼 **CUE Sheet Support**: Play albums stored as single files with CUE sheets
- 🤖 **AI-Powered Recommendations**: Context-aware music suggestions based on your coding activity
- 💬 **Natural Language Interface**: Chat with the music assistant to find the perfect soundtrack
- 📚 **Smart Library Management**: Automatic music classification and organization
- 🎨 **Rich Context**: Get background information and generated visuals for your music
- 🔌 **IDE Integration**: Seamless integration with IntelliJ IDEA (VS Code support planned)

## Architecture

Contexture uses a hybrid architecture:
- **Rust Core**: High-performance audio engine with bit-perfect playback
- **IDE Plugin**: Native integration with your development environment
- **AI Layer**: Local AI processing for privacy-preserving recommendations

## Getting Started

### Prerequisites

- Rust 1.70+ (for building the core)
- Java 17+ (for the IntelliJ plugin)
- IntelliJ IDEA 2023.1+ (for running the plugin)

### Building

```bash
# Build the Rust core
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Running Locally

```bash
# Run local CI checks
./scripts/ci-local.sh

# With coverage
./scripts/ci-local.sh --coverage

# With benchmarks
./scripts/ci-local.sh --bench
```

## Project Structure

```
contextune/
├── core/             # Rust audio core library
├── intellij-plugin/  # IntelliJ Platform plugin
├── scripts/          # Build and utility scripts
├── test_data/        # Test data and reference files
└── docs/             # Documentation
```

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test audio_quality
cargo test --test ffi_integration

# Generate coverage report
cargo coverage

# Generate HTML coverage report and open in browser
cargo coverage-html

# CI mode coverage (XML only)
cargo coverage-ci
```

### Code Coverage

The project maintains a minimum code coverage threshold of 85%. Coverage reports are generated using [Tarpaulin](https://github.com/xd009642/tarpaulin) and uploaded to [Codecov](https://codecov.io).

Available coverage commands:
- `cargo coverage` - Generate HTML and XML reports with threshold checking
- `cargo coverage-html` - Generate HTML report and open in browser
- `cargo coverage-ci` - CI mode (XML only, fails on threshold violation)
- `cargo coverage-check` - Generate reports without threshold checking

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Security audit
cargo audit
```

## Documentation

- [Architecture](docs/architecture.md)
- [API Documentation](docs/api.md)
- [CI/CD Guide](docs/ci-cd.md)
- [Development Guide](docs/development.md)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
