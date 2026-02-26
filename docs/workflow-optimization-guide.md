# GitHub Actions Workflow Optimization Guide

## Quick Reference

This guide provides actionable steps to optimize the Contexture project's GitHub Actions workflows based on a comprehensive analysis.

**Related Documents**:
- [Detailed Analysis (Chinese)](./workflow-analysis.md) - 完整的中文分析报告
- [Current Workflows README](../.github/workflows/README.md)

---

## Executive Summary

### Current State
- **4 workflows**: CI, Security, Performance, Release
- **Average CI time**: ~77 minutes
- **Monthly cost**: ~$280
- **Key issues**: Workflow duplication, long dependency chains, excessive `continue-on-error`

### Optimization Potential
| Metric | Current | After Phase 1 | After Phase 3 | Improvement |
|--------|---------|---------------|---------------|-------------|
| CI Time | 77 min | 55 min | 35 min | **-55%** |
| Monthly Cost | $280 | $190 | $122 | **-56%** |
| PR Feedback | 80 min | 60 min | 40 min | **-50%** |

---

## Phase 1: Immediate Optimizations (Week 1-2)

### 1.1 Eliminate Workflow Duplication

**Problem**: Security audit and performance benchmarks run twice (in CI and dedicated workflows).

**Solution**: Remove duplicates from CI, keep in specialized workflows.

<details>
<summary>Implementation Steps</summary>

**Step 1**: Update `ci.yml`

```yaml
# Remove these jobs from ci.yml:
# - security-audit
# - performance-benchmarks

# Add lightweight security check instead:
  quick-security-check:
    name: Quick Security Check
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Cache Cargo
        uses: actions/cache@v4
        with:
          path: ~/.cargo/bin/
          key: cargo-bin-${{ runner.os }}
      
      - name: Check for known vulnerabilities
        run: |
          cargo install cargo-audit --locked
          cargo audit --deny warnings
```

**Step 2**: Update `security.yml` triggers

```yaml
on:
  schedule:
    - cron: '0 2 * * *'  # Keep daily run
  workflow_dispatch:     # Keep manual trigger
  # Remove push and pull_request triggers to avoid duplication
```

**Expected Savings**: 15-20 minutes per CI run, ~30% resource reduction

</details>

### 1.2 Optimize Dependency Chains

**Problem**: `plugin-tests` waits for `ffi-integration-tests` which waits for `rust-tests` (52 min serial).

**Solution**: Make `plugin-tests` depend directly on `rust-tests`.

<details>
<summary>Implementation Steps</summary>

```yaml
# Before:
plugin-tests:
  needs: ffi-integration-tests  # Waits for FFI tests

# After:
plugin-tests:
  name: IntelliJ Plugin Tests
  runs-on: ubuntu-latest
  needs: rust-tests  # Direct dependency
  
  steps:
    - name: Checkout code
      uses: actions/checkout@v4
    
    - name: Setup Java
      uses: actions/setup-java@v3
      with:
        distribution: 'temurin'
        java-version: '17'
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Build Rust Core
      run: cargo build --release  # Rebuild in plugin-tests
    
    - name: Run Plugin Tests
      run: |
        cd intellij-plugin
        ./gradlew test
```

**Expected Savings**: 10-15 minutes per CI run

</details>

### 1.3 Split Lint from Test Matrix

**Problem**: `cargo fmt` and `clippy` run 3 times (Linux, Windows, macOS) but only need to run once.

**Solution**: Create separate lint job that runs only on Linux.

<details>
<summary>Implementation Steps</summary>

```yaml
jobs:
  lint:
    name: Code Quality Checks
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      
      - name: Cache Cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: lint-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Check Formatting
        run: cargo fmt --all -- --check
      
      - name: Clippy Lints
        run: cargo clippy --all-targets --all-features -- -D warnings

  rust-tests:
    name: Rust Tests
    runs-on: ${{ matrix.os }}
    needs: lint  # Ensure lint passes first
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      # Remove fmt and clippy steps
      
      - name: Run Unit Tests
        run: cargo test --all-features --verbose
      
      - name: Run Doc Tests
        run: cargo test --doc --all-features
```

**Expected Savings**: ~10 minutes total (avoids duplicate lint on Windows/macOS)

</details>

### 1.4 Review continue-on-error

**Problem**: Too many critical steps have `continue-on-error: true`, potentially masking real failures.

**Solution**: Remove unnecessary `continue-on-error` and document reasons for keeping it.

<details>
<summary>Implementation Steps</summary>

**Audit all `continue-on-error` usage:**

| Job/Step | Current | Should Be | Reason |
|----------|---------|-----------|--------|
| ffi-integration-tests | ✓ | ❌ | Must fail if FFI broken |
| plugin-tests | ✓ | ❌ | Must fail if plugin broken |
| audio-quality-tests (first test) | ✓ | ✓ | CI environment lacks audio hardware |
| memory-safety-checks | ✓ | ⚠️ | Review each step individually |
| performance-benchmarks | ✓ | ❌ | Should fail on regressions |

**Example fix:**

```yaml
# Remove from FFI tests
ffi-integration-tests:
  steps:
    - name: Build Rust Library
      run: cargo build --release --lib
      # continue-on-error: true  # REMOVED

    - name: Run FFI Tests
      run: cargo test --test ffi_integration
      # continue-on-error: true  # REMOVED

# Keep with documentation
audio-quality-tests:
  steps:
    - name: Run Audio Quality Tests
      run: cargo test --test audio_quality -- --nocapture
      continue-on-error: true
      # REASON: Audio tests may be unstable in CI without physical audio devices
      # These tests are critical for releases but not for PR validation
```

</details>

### 1.5 Upgrade Actions Versions

**Problem**: Inconsistent action versions (mix of v3 and v4).

**Solution**: Standardize on latest versions.

<details>
<summary>Implementation Steps</summary>

**Find and replace:**

```bash
# Update cache actions
find .github/workflows -name "*.yml" -exec sed -i 's/actions\/cache@v3/actions\/cache@v4/g' {} \;

# Update download-artifact actions
find .github/workflows -name "*.yml" -exec sed -i 's/actions\/download-artifact@v3/actions\/download-artifact@v4/g' {} \;
```

**Manual verification needed for:**
- `actions/upload-artifact@v4` - Already using v4 ✓
- `actions/checkout@v4` - Already using v4 ✓

</details>

---

## Phase 2: Short-term Optimizations (Week 3-4)

### 2.1 Improve Cache Strategy

**Goal**: Increase cache hit rate and reduce build times.

<details>
<summary>Implementation Steps</summary>

```yaml
- name: Cache Cargo
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/bin/
      ~/.cargo/registry/index/
      ~/.cargo/registry/cache/
      ~/.cargo/git/db/
      target/
    # Improved key with more context
    key: ${{ runner.os }}-${{ matrix.rust || 'stable' }}-cargo-${{ hashFiles('**/Cargo.lock') }}-${{ github.workflow }}
    restore-keys: |
      ${{ runner.os }}-${{ matrix.rust || 'stable' }}-cargo-${{ hashFiles('**/Cargo.lock') }}-
      ${{ runner.os }}-${{ matrix.rust || 'stable' }}-cargo-
```

**Benefits**:
- Separate caches for different workflows
- Include Rust version in key
- Better restore key hierarchy

</details>

### 2.2 Add Concurrency Control

**Goal**: Cancel outdated workflow runs when new commits are pushed.

<details>
<summary>Implementation Steps</summary>

Add to all workflow files:

```yaml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

# Add concurrency control
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
```

**Behavior**:
- PRs: Cancel old runs when new commit pushed
- Main/Develop: Do not cancel (allow all runs to complete)

</details>

### 2.3 Optimize Artifact Retention

**Goal**: Reduce storage costs while keeping important artifacts.

<details>
<summary>Implementation Steps</summary>

**Categorize artifacts by importance:**

| Artifact Type | Current Retention | Recommended | Reason |
|--------------|-------------------|-------------|---------|
| Test results | 30 days | 7 days | Short-term debugging only |
| Benchmark results | 90 days | 90 days | Historical tracking |
| Security reports | 30 days | 30 days | Compliance |
| Coverage reports | 30 days | 14 days | Recent trends |
| Release binaries | 90 days | 90 days | Distribution |

**Example:**

```yaml
- name: Upload Test Results
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: test-results-${{ matrix.os }}-${{ github.run_number }}
    path: target/test-results/
    retention-days: 7  # Reduced from 30
```

</details>

### 2.4 Add Failure Notifications

**Goal**: Get immediate alerts when CI fails.

<details>
<summary>Implementation Steps</summary>

Add to `ci.yml`:

```yaml
  notify-on-failure:
    name: Notify on Failure
    runs-on: ubuntu-latest
    needs: [rust-tests, audio-quality-tests, code-coverage]
    if: failure()
    
    steps:
      - name: Create Issue on Failure
        uses: actions/github-script@v7
        with:
          script: |
            const issue = await github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: `CI Failure: ${context.workflow} on ${context.ref}`,
              body: `CI workflow failed on ${context.ref}\n\n` +
                    `Run: ${context.serverUrl}/${context.repo.owner}/${context.repo.repo}/actions/runs/${context.runId}\n\n` +
                    `Commit: ${context.sha}\n` +
                    `Author: ${context.actor}`,
              labels: ['ci-failure', 'automated']
            });
            
            console.log(`Created issue #${issue.data.number}`);
```

**Alternative**: Use Slack/Discord webhooks for team notifications

</details>

---

## Phase 3: Long-term Optimizations (Month 2-3)

### 3.1 Implement Incremental Builds

**Goal**: Only build and test changed code.

<details>
<summary>Research and Implementation</summary>

**Tools to evaluate:**
- `cargo-nextest` - Faster test execution
- `sccache` - Distributed compilation cache
- Workspace-aware builds

**Example with cargo-nextest:**

```yaml
- name: Install cargo-nextest
  run: cargo install cargo-nextest --locked

- name: Run tests with nextest
  run: cargo nextest run --all-features
  # nextest is 3x faster than cargo test for large test suites
```

**Expected improvement**: 30-50% faster test execution

</details>

### 3.2 Implement Test Selection

**Goal**: Run only tests affected by changes.

<details>
<summary>Implementation Steps</summary>

```yaml
- name: Detect Changed Components
  id: changes
  run: |
    # Detect which components changed
    if git diff --name-only ${{ github.event.before }} ${{ github.sha }} | grep "^core/"; then
      echo "core_changed=true" >> $GITHUB_OUTPUT
    else
      echo "core_changed=false" >> $GITHUB_OUTPUT
    fi
    
    if git diff --name-only ${{ github.event.before }} ${{ github.sha }} | grep "^intellij-plugin/"; then
      echo "plugin_changed=true" >> $GITHUB_OUTPUT
    else
      echo "plugin_changed=false" >> $GITHUB_OUTPUT
    fi

- name: Run Core Tests
  if: steps.changes.outputs.core_changed == 'true' || github.event_name == 'push'
  run: cargo test --all-features

- name: Run Plugin Tests
  if: steps.changes.outputs.plugin_changed == 'true' || github.event_name == 'push'
  run: |
    cd intellij-plugin
    ./gradlew test
```

**Note**: Always run all tests on main branch pushes

</details>

### 3.3 Tiered Testing Strategy

**Goal**: Run different test levels based on context.

<details>
<summary>Strategy Design</summary>

**Test Tiers:**

| Tier | Duration | When to Run | Tests Included |
|------|----------|-------------|----------------|
| Fast | < 5 min | Every PR commit | Lint + Unit tests |
| Standard | < 30 min | PR ready for review | + Integration tests |
| Full | < 60 min | Merge to develop | + Audio quality + Benchmarks |
| Extended | > 60 min | Merge to main / Nightly | + Memory safety + Fuzz tests |

**Implementation:**

```yaml
# .github/workflows/ci-fast.yml
name: Fast CI
on:
  pull_request:
  # Only run for draft PRs or early commits

jobs:
  fast-tests:
    runs-on: ubuntu-latest
    steps:
      - name: Lint and Unit Tests
        run: |
          cargo fmt --check
          cargo clippy
          cargo test --lib

# .github/workflows/ci-full.yml
name: Full CI
on:
  pull_request:
    types: [ready_for_review]
  push:
    branches: [main, develop]
  # Run complete test suite
```

</details>

---

## Implementation Checklist

### Phase 1 (Week 1-2)

- [ ] **Eliminate Duplication** (4 hours)
  - [ ] Remove `security-audit` job from `ci.yml`
  - [ ] Remove `performance-benchmarks` job from `ci.yml`
  - [ ] Update `security.yml` triggers
  - [ ] Add lightweight security check to `ci.yml`
  - [ ] Test changes on feature branch
  - [ ] Monitor 5 CI runs for issues

- [ ] **Optimize Dependencies** (2 hours)
  - [ ] Update `plugin-tests` dependencies
  - [ ] Add Rust build step to `plugin-tests`
  - [ ] Test plugin tests still pass
  - [ ] Verify time savings

- [ ] **Split Lint Job** (2 hours)
  - [ ] Create separate `lint` job
  - [ ] Remove lint from `rust-tests` matrix
  - [ ] Update job dependencies
  - [ ] Verify all checks still run

- [ ] **Review continue-on-error** (3 hours)
  - [ ] Audit all usages
  - [ ] Remove from FFI and plugin tests
  - [ ] Add documentation comments
  - [ ] Test stricter failure handling
  - [ ] Monitor failure rates

- [ ] **Upgrade Actions** (1 hour)
  - [ ] Update to actions/cache@v4
  - [ ] Update to actions/download-artifact@v4
  - [ ] Test all updated actions

**Phase 1 Success Criteria:**
- [ ] CI time reduced by >20%
- [ ] No increase in false failures
- [ ] All tests still running
- [ ] Team satisfied with changes

### Phase 2 (Week 3-4)

- [ ] **Improve Caching** (3 hours)
  - [ ] Update cache keys
  - [ ] Test cache hit rates
  - [ ] Monitor build times

- [ ] **Add Concurrency** (1 hour)
  - [ ] Add to all workflows
  - [ ] Test cancellation behavior
  - [ ] Verify main branch not affected

- [ ] **Optimize Artifacts** (2 hours)
  - [ ] Review all retention periods
  - [ ] Update retention days
  - [ ] Monitor storage usage

- [ ] **Add Notifications** (2 hours)
  - [ ] Implement failure notifications
  - [ ] Configure notification targets
  - [ ] Test notification triggers

- [ ] **Update Documentation** (4 hours)
  - [ ] Update workflow README
  - [ ] Document all changes
  - [ ] Create troubleshooting guide

**Phase 2 Success Criteria:**
- [ ] Cache hit rate >80%
- [ ] Notifications working
- [ ] Documentation updated
- [ ] Storage costs reduced

### Phase 3 (Month 2-3)

- [ ] **Incremental Builds** (2 weeks)
  - [ ] Research tools
  - [ ] Implement cargo-nextest
  - [ ] Test performance improvements
  - [ ] Roll out gradually

- [ ] **Test Selection** (1 week)
  - [ ] Implement change detection
  - [ ] Add conditional test execution
  - [ ] Ensure safety (always test main)
  - [ ] Monitor effectiveness

- [ ] **Tiered Testing** (1 week)
  - [ ] Design tier strategy
  - [ ] Create workflow variants
  - [ ] Update triggers
  - [ ] Train team on new flow

**Phase 3 Success Criteria:**
- [ ] CI time reduced by >50%
- [ ] Test coverage maintained
- [ ] Developer experience improved
- [ ] Cost reduction achieved

---

## Monitoring and Validation

### Metrics to Track

**Before Optimization (Baseline):**
```yaml
# Record these metrics before starting
ci_time_average: 77 minutes
ci_time_p95: 90 minutes
failure_rate: X%
cache_hit_rate: X%
monthly_cost: $280
pr_feedback_time: 80 minutes
```

**After Each Phase:**
- CI execution time (average, p95)
- Test failure rate
- Cache hit rate
- Monthly GitHub Actions cost
- Time to PR feedback
- Developer satisfaction score

### Validation Checklist

After each optimization:
- [ ] All tests still pass
- [ ] No new flaky tests
- [ ] Failure detection not degraded
- [ ] Team can understand changes
- [ ] Documentation updated
- [ ] Metrics show improvement

### Rollback Procedure

If optimization causes issues:

1. **Immediate Rollback**
   ```bash
   git revert <commit-hash>
   git push origin main
   ```

2. **Diagnose Issue**
   - Check workflow logs
   - Review error messages
   - Collect team feedback

3. **Fix and Retry**
   - Fix identified issue
   - Test in feature branch
   - Re-deploy gradually

---

## Cost-Benefit Analysis

### Investment Required

| Phase | Development Time | Cost |
|-------|-----------------|------|
| Phase 1 | ~12 hours | ~$1,200 |
| Phase 2 | ~12 hours | ~$1,200 |
| Phase 3 | ~80 hours | ~$8,000 |
| **Total** | **~104 hours** | **~$10,400** |

### Expected Returns

| Metric | Monthly Savings |
|--------|-----------------|
| GitHub Actions costs | $158/month |
| Developer time saved | ~$500/month |
| **Total** | **~$658/month** |

**ROI**: Break even in **3-4 months**

### Additional Benefits (Not Monetized)

- Faster PR feedback loop
- Improved developer experience
- More reliable CI system
- Better security posture
- Easier to maintain and extend

---

## FAQ

<details>
<summary>Q: Will these changes break existing workflows?</summary>

A: Changes are designed to be backward compatible. Each phase includes testing and validation steps. We recommend implementing changes gradually and monitoring results.
</details>

<details>
<summary>Q: Can we cherry-pick specific optimizations?</summary>

A: Yes! Optimizations are largely independent. Start with high-impact, low-risk changes like eliminating duplication.
</details>

<details>
<summary>Q: How do we handle flaky tests?</summary>

A: Document flaky tests with `continue-on-error` and a comment explaining why. Track flaky tests in issues and prioritize fixes. Don't hide real failures.
</details>

<details>
<summary>Q: What if CI time increases after optimization?</summary>

A: Investigate immediately. Check cache hit rates, network issues, or GitHub Actions incidents. Have rollback plan ready.
</details>

<details>
<summary>Q: Should we implement all phases?</summary>

A: Phase 1 is strongly recommended. Phases 2-3 depend on project growth and team capacity. Evaluate ROI at each phase.
</details>

---

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI Best Practices](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [Cargo Caching Guide](https://github.com/actions/cache/blob/main/examples.md#rust---cargo)
- [cargo-nextest](https://nexte.st/)
- [Criterion.rs Benchmarking](https://github.com/bheisler/criterion.rs)

---

## Contact

For questions or suggestions about this optimization guide:
- Open a GitHub Issue
- Contact the DevOps team
- Discuss in team meetings

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-25  
**Author**: CI/CD Optimization Team
