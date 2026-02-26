# GitHub Actions Workflows

This directory contains the CI/CD workflows for the Music Player Plugin project.

## Workflows

### 1. CI Workflow (`ci.yml`)

**Triggers:** Push and Pull Requests to `main` and `develop` branches

**Jobs:**
- **rust-tests**: Runs on Ubuntu, Windows, and macOS with stable and nightly Rust
  - Code formatting check (`cargo fmt`)
  - Linting with Clippy
  - Unit tests
  - Documentation tests

- **audio-quality-tests**: Verifies audio quality standards
  - Generates test audio signals
  - Runs bit-perfect verification tests
  - Validates frequency response
  - Measures THD+N

- **performance-benchmarks**: Tracks performance metrics
  - Runs Criterion benchmarks
  - Checks for performance regressions (>5% threshold)

- **memory-safety-checks**: Ensures memory safety
  - Runs Miri for undefined behavior detection
  - Uses AddressSanitizer
  - Valgrind memory leak detection

- **ffi-integration-tests**: Tests FFI boundaries
  - Cross-platform FFI tests
  - JNI binding validation

- **plugin-tests**: Tests IntelliJ plugin
  - Plugin unit tests
  - Plugin verifier

- **cue-parser-tests**: Validates CUE sheet parsing
  - Comprehensive CUE format tests
  - Sample-accurate seeking tests

- **code-coverage**: Measures test coverage
  - Generates coverage reports with Tarpaulin
  - Uploads to Codecov
  - Enforces 85% minimum coverage

- **security-audit**: Checks for vulnerabilities
  - Runs cargo-audit
  - Runs cargo-deny

### 2. Performance Monitoring Workflow (`performance.yml`)

**Triggers:** 
- Push to `main` branch
- Weekly schedule (Sunday at midnight UTC)

**Purpose:** Tracks performance trends over time and alerts on regressions

### 3. Release Workflow (`release.yml`)

**Triggers:** Push of version tags (e.g., `v1.0.0`)

**Jobs:**
- **build-rust-core**: Builds native libraries for all platforms
  - Linux (x86_64)
  - Windows (x86_64)
  - macOS (x86_64 and ARM64)

- **build-plugin**: Builds IntelliJ plugin with native libraries

- **create-release**: Creates GitHub release with all artifacts

## Configuration

### Required Secrets

None required for basic functionality. Optional:
- `CODECOV_TOKEN`: For private repository coverage reports

### Branch Protection

Recommended branch protection rules for `main`:
- Require status checks to pass before merging
- Require branches to be up to date before merging
- Required status checks:
  - `rust-tests`
  - `code-coverage`
  - `security-audit`

### Permissions

The release workflow requires `contents: write` permission to create releases.

## Local Testing

To run the same checks locally:

```bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --all-features

# Benchmarks
cargo bench

# Coverage
cargo tarpaulin --out Html

# Security audit
cargo audit
```

## Troubleshooting

### Audio Tests Failing

Audio tests require PulseAudio on Linux. Ensure it's installed:
```bash
sudo apt-get install pulseaudio
```

### FFI Tests Failing

FFI tests require Java 17. Ensure it's installed:
```bash
java -version
```

### Performance Regression Alerts

If you receive a performance regression alert:
1. Review the benchmark results in the workflow artifacts
2. Identify which benchmark regressed
3. Profile the code to find the bottleneck
4. Optimize or document the intentional change

## Workflow Analysis and Optimization

📊 **Comprehensive workflow analysis and optimization recommendations are available:**

- **[Workflow Analysis (Chinese)](../../docs/workflow-analysis.md)** - 完整的工作流设计评估与优化建议
- **[Workflow Optimization Guide](../../docs/workflow-optimization-guide.md)** - Actionable implementation guide with step-by-step instructions

### Key Findings

**Current State:**
- 4 main workflows (CI, Security, Performance, Release)
- Average CI time: ~77 minutes
- Monthly cost estimate: ~$280

**Optimization Potential:**
- Reduce CI time by **50-60%** (to ~35 minutes)
- Lower costs by **50-60%** (to ~$122/month)
- Faster PR feedback (from 80 to 40 minutes)

**Priority Improvements:**
1. Eliminate workflow duplication between CI and specialized workflows
2. Optimize job dependency chains
3. Review and reduce excessive `continue-on-error` usage
4. Split lint checks from cross-platform test matrix
5. Improve caching strategy and add concurrency control

See the optimization guide for detailed implementation steps.

## Continuous Improvement

These workflows are designed to evolve with the project. As new features are added:
- Add corresponding tests to the CI workflow
- Update quality gates as needed
- Add new benchmarks for performance-critical code
- Expand security checks as dependencies grow
- Implement optimizations from the workflow analysis recommendations
