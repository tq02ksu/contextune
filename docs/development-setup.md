# Development Environment Setup Guide

This guide will help you set up a complete development environment for the Contextune Music Player Plugin.

## Table of Contents

- [System Requirements](#system-requirements)
- [Core Dependencies](#core-dependencies)
- [Platform-Specific Setup](#platform-specific-setup)
- [IDE Configuration](#ide-configuration)
- [Development Tools](#development-tools)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)

## System Requirements

### Minimum Requirements
- **OS**: Windows 10+, macOS 10.15+, or Linux (Ubuntu 20.04+ recommended)
- **RAM**: 8GB (16GB recommended for development)
- **Storage**: 5GB free space
- **Network**: Internet connection for downloading dependencies

### Recommended Specifications
- **OS**: Latest stable versions
- **RAM**: 16GB or more
- **CPU**: Multi-core processor (4+ cores recommended)
- **Storage**: SSD with 10GB+ free space

## Core Dependencies

### 1. Rust Toolchain

Install Rust using rustup (recommended):

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the on-screen instructions, then restart your shell

# Verify installation
rustc --version
cargo --version

# Install additional components
rustup component add rustfmt clippy
rustup install nightly
rustup +nightly component add miri rust-src
```

### 2. Git

```bash
# Linux (Ubuntu/Debian)
sudo apt-get install git

# macOS (with Homebrew)
brew install git

# Windows
# Download from https://git-scm.com/download/win
```

### 3. Build Tools

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install build-essential pkg-config
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Or install Xcode from the App Store
```

#### Windows
```bash
# Install Visual Studio Build Tools or Visual Studio Community
# Download from https://visualstudio.microsoft.com/downloads/
```
## Platform-Specific Setup

### Linux Setup

#### Audio Dependencies
```bash
# ALSA development libraries
sudo apt-get install libasound2-dev

# PulseAudio development libraries
sudo apt-get install libpulse-dev pulseaudio

# Additional audio tools
sudo apt-get install pulseaudio-utils pavucontrol

# Optional: JACK support
sudo apt-get install libjack-jackd2-dev
```

#### System Libraries
```bash
# SSL/TLS support
sudo apt-get install libssl-dev

# Additional development tools
sudo apt-get install valgrind gdb lldb

# For cross-compilation (optional)
sudo apt-get install gcc-multilib
```

#### Audio System Configuration
```bash
# Start PulseAudio (if not running)
pulseaudio --start

# Set up null sink for testing
pactl load-module module-null-sink sink_name=test_sink

# Verify audio setup
pactl list sinks | grep -E "(Name|Description)"
```

### macOS Setup

#### Homebrew (Package Manager)
```bash
# Install Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Update Homebrew
brew update
```

#### Development Tools
```bash
# Additional development tools
brew install llvm gdb

# Audio tools (optional)
brew install sox portaudio

# For cross-compilation (optional)
brew install mingw-w64
```

#### System Configuration
```bash
# Grant microphone access (required for audio tests)
# Go to System Preferences > Security & Privacy > Privacy > Microphone
# Add Terminal or your IDE to the list

# Verify audio system
system_profiler SPAudioDataType
```

### Windows Setup

#### Visual Studio Build Tools
1. Download Visual Studio Installer
2. Install "C++ build tools" workload
3. Include Windows 10/11 SDK
4. Include CMake tools (optional)

#### Windows Subsystem for Linux (Optional)
```powershell
# Enable WSL2
wsl --install

# Install Ubuntu
wsl --install -d Ubuntu

# Follow Linux setup instructions within WSL
```

#### Audio Configuration
```powershell
# Verify audio services
sc query audiosrv

# Check audio devices
Get-WmiObject -Class Win32_SoundDevice
```

## IDE Configuration

### Visual Studio Code (Recommended)

#### Installation
```bash
# Linux (Ubuntu/Debian)
wget -qO- https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > packages.microsoft.gpg
sudo install -o root -g root -m 644 packages.microsoft.gpg /etc/apt/trusted.gpg.d/
sudo sh -c 'echo "deb [arch=amd64,arm64,armhf signed-by=/etc/apt/trusted.gpg.d/packages.microsoft.gpg] https://packages.microsoft.com/repos/code stable main" > /etc/apt/sources.list.d/vscode.list'
sudo apt-get update
sudo apt-get install code

# macOS
brew install --cask visual-studio-code

# Windows
# Download from https://code.visualstudio.com/
```

#### Essential Extensions
```bash
# Install via command line
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
code --install-extension serayuzgur.crates
code --install-extension tamasfe.even-better-toml
code --install-extension ms-vscode.cmake-tools
```

#### VS Code Settings
Create `.vscode/settings.json`:
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.procMacro.enable": true,
    "files.watcherExclude": {
        "**/target/**": true
    },
    "search.exclude": {
        "**/target": true,
        "**/Cargo.lock": true
    }
}
```

#### Launch Configuration
Create `.vscode/launch.json`:
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug unit tests",
            "cargo": {
                "args": ["test", "--no-run", "--bin=contextune"],
                "filter": {
                    "name": "contextune",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}/core"
        }
    ]
}
```

### IntelliJ IDEA / CLion

#### Installation
```bash
# Install via JetBrains Toolbox (recommended)
# Download from https://www.jetbrains.com/toolbox-app/

# Or install directly
# IntelliJ IDEA: https://www.jetbrains.com/idea/
# CLion: https://www.jetbrains.com/clion/
```

#### Plugins
- Rust Plugin
- TOML Plugin
- Markdown Plugin
- Git Integration

### Vim/Neovim (Advanced)

#### Rust Support
```bash
# Install rust.vim
git clone https://github.com/rust-lang/rust.vim ~/.vim/pack/plugins/start/rust.vim

# Or use a plugin manager like vim-plug
# Add to .vimrc:
# Plug 'rust-lang/rust.vim'
# Plug 'neoclide/coc.nvim', {'branch': 'release'}
```

## Development Tools

### Essential Cargo Tools
```bash
# Code formatting and linting
rustup component add rustfmt clippy

# Security auditing
cargo install cargo-audit cargo-deny

# Code coverage
cargo install cargo-tarpaulin

# Benchmarking
cargo install cargo-criterion

# Fuzzing
cargo install cargo-fuzz

# Cross-compilation
cargo install cross

# File watching
cargo install cargo-watch

# Documentation tools
cargo install cargo-doc
```

### Optional Tools
```bash
# Advanced debugging
cargo install cargo-expand  # Macro expansion
cargo install cargo-asm     # Assembly inspection
cargo install cargo-bloat   # Binary size analysis

# Performance profiling
cargo install cargo-profdata
cargo install flamegraph

# Supply chain security
cargo install cargo-supply-chain

# License checking
cargo install cargo-license
```

### Git Configuration
```bash
# Set up Git identity
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# Useful Git aliases
git config --global alias.co checkout
git config --global alias.br branch
git config --global alias.ci commit
git config --global alias.st status

# Set up Git hooks (optional)
# Copy pre-commit hooks from scripts/ directory
```
## Java Development (for Plugin)

### Java Development Kit
```bash
# Install OpenJDK 17 (recommended)
# Linux
sudo apt-get install openjdk-17-jdk

# macOS
brew install openjdk@17

# Windows
# Download from https://adoptium.net/
```

### IntelliJ IDEA Setup
```bash
# Install IntelliJ IDEA Community or Ultimate
# Download from https://www.jetbrains.com/idea/

# Required plugins:
# - Kotlin (usually pre-installed)
# - Gradle
# - Git Integration
```

### Gradle Configuration
```bash
# Verify Gradle installation (comes with IntelliJ)
./gradlew --version

# Set up Gradle wrapper (if needed)
gradle wrapper --gradle-version 8.0
```

## Project Setup

### Clone Repository
```bash
# Clone the repository
git clone https://github.com/your-org/contextune.git
cd contextune

# Set up Git hooks
cp scripts/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit
```

### Initial Build
```bash
# Build Rust core
cd core
cargo build

# Run tests to verify setup
cargo test

# Build release version
cargo build --release

# Return to project root
cd ..
```

### Plugin Development Setup
```bash
# Navigate to plugin directory
cd plugin

# Build plugin
./gradlew build

# Run plugin tests
./gradlew test

# Return to project root
cd ..
```

## Verification

### Verify Rust Setup
```bash
cd core

# Check Rust version
rustc --version
cargo --version

# Verify components
rustup component list --installed

# Test basic compilation
cargo check

# Run formatter
cargo fmt --check

# Run linter
cargo clippy

# Run tests
cargo test --lib
```

### Verify Audio Setup
```bash
# Linux: Check PulseAudio
pulseaudio --check -v
pactl info

# macOS: Check CoreAudio
system_profiler SPAudioDataType | head -20

# Windows: Check audio services
sc query audiosrv
```

### Verify Development Tools
```bash
# Check installed tools
cargo --list | grep -E "(audit|deny|tarpaulin|fuzz)"

# Test security tools
cargo audit --version
cargo deny --version

# Test coverage tool
cargo tarpaulin --version
```

### Run Full Test Suite
```bash
# Run local CI checks
./scripts/ci-local.sh

# This should pass without errors if setup is correct
```

## Environment Variables

### Required Environment Variables
```bash
# Add to your shell profile (.bashrc, .zshrc, etc.)

# Rust configuration
export RUST_BACKTRACE=1
export CARGO_TERM_COLOR=always

# Audio configuration (Linux)
export PULSE_RUNTIME_PATH=/run/user/$(id -u)/pulse

# Development configuration
export RUST_LOG=debug  # For verbose logging during development
```

### Optional Environment Variables
```bash
# Performance tuning
export CARGO_BUILD_JOBS=4  # Parallel build jobs
export RUST_MIN_STACK=8388608  # Increase stack size for tests

# Testing configuration
export SKIP_SLOW_TESTS=1  # Skip slow tests during development
export TEST_TIMEOUT=60    # Test timeout in seconds
```

## Troubleshooting

### Common Issues

#### Rust Installation Issues
```bash
# Update Rust
rustup update

# Reinstall Rust
rustup self uninstall
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Fix PATH issues
source ~/.cargo/env
```

#### Audio Issues on Linux
```bash
# Restart PulseAudio
pulseaudio --kill
pulseaudio --start

# Check audio group membership
groups $USER | grep audio

# Add user to audio group
sudo usermod -a -G audio $USER
# Log out and back in
```

#### Build Issues
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Check for conflicting versions
cargo tree --duplicates

# Rebuild from scratch
rm -rf target/
cargo build
```

#### Permission Issues
```bash
# Linux: Fix cargo permissions
sudo chown -R $USER:$USER ~/.cargo

# macOS: Fix Homebrew permissions
sudo chown -R $(whoami) /usr/local/var/homebrew

# Windows: Run as administrator
# Right-click terminal -> "Run as administrator"
```

### Getting Help

#### Documentation
- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Rust Reference](https://doc.rust-lang.org/reference/)
- [Project Documentation](./README.md)

#### Community Resources
- [Rust Users Forum](https://users.rust-lang.org/)
- [Rust Discord](https://discord.gg/rust-lang)
- [Stack Overflow](https://stackoverflow.com/questions/tagged/rust)

#### Project-Specific Help
- [GitHub Issues](https://github.com/your-org/contextune/issues)
- [Contributing Guidelines](../CONTRIBUTING.md)
- [Security Policy](../SECURITY.md)

## Next Steps

After completing the setup:

1. **Read the Documentation**
   - [Architecture Overview](./architecture.md)
   - [Local Testing Procedures](./local-testing.md)
   - [Security Guidelines](./security-guidelines.md)

2. **Explore the Codebase**
   - Start with `core/src/lib.rs`
   - Review the module structure
   - Run example programs

3. **Make Your First Contribution**
   - Pick a "good first issue" from GitHub
   - Follow the contributing guidelines
   - Submit a pull request

4. **Set Up Continuous Development**
   - Configure file watching: `cargo watch -x test`
   - Set up pre-commit hooks
   - Integrate with your IDE's debugging tools

---

Welcome to Contextune development! If you encounter any issues with this setup guide, please open an issue on GitHub.