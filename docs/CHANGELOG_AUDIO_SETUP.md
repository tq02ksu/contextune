# Audio Setup Script Optimization

## Changes Made

### Before (2 scripts)
- `scripts/setup_audio_ci.sh` - CI-specific script
- `scripts/ci_setup_audio.sh` - Another CI script (confusing naming)

### After (1 unified script)
- `scripts/setup_audio.sh` - Single script for all use cases

## Improvements

### 1. **Unified Interface**
- Single script handles both local development and CI environments
- Mode selection via `--ci` flag
- Clear, consistent naming

### 2. **Better User Experience**
- **Local mode**: Rich output with emojis and helpful messages
- **CI mode**: Minimal, parseable output for automation
- **Help text**: Built-in `--help` flag

### 3. **Enhanced Features**
- **Cleanup mode**: `--cleanup` flag to remove virtual audio devices
- **Better error handling**: Clear error messages and exit codes
- **Platform detection**: Automatic OS detection and configuration
- **Idempotent**: Safe to run multiple times

### 4. **Improved Logging**
```bash
# Local mode (colorful, helpful)
ℹ️  Detected: macOS
✓ macOS audio environment ready

# CI mode (parseable)
[INFO] Detected: macOS
[OK] macOS audio environment ready
```

### 5. **Smart Defaults**
- Linux: Creates `contextune_test_sink` (descriptive name)
- macOS: Suggests BlackHole but works without it
- Windows: Uses default device with helpful suggestions

### 6. **Better Documentation**
- Inline help text
- Comprehensive README in `scripts/README.md`
- Updated main documentation in `docs/audio-testing-setup.md`

## Usage Examples

### Local Development
```bash
# Setup audio environment
./scripts/setup_audio.sh

# Get help
./scripts/setup_audio.sh --help

# Cleanup when done
./scripts/setup_audio.sh --cleanup
```

### CI/CD
```bash
# In GitHub Actions
./scripts/setup_audio.sh --ci
```

### Manual Testing
```bash
# Setup
./scripts/setup_audio.sh

# Generate test files
cd core && cargo run --example generate_test_audio

# Run tests
cargo test
```

## Benefits

1. **Less Confusion**: One script, clear purpose
2. **Easier Maintenance**: Single source of truth
3. **Better Testing**: Works consistently across environments
4. **Improved DX**: Better developer experience with helpful output
5. **CI-Friendly**: Minimal output mode for automation

## Migration Guide

### For Local Development
```bash
# Old
./scripts/setup_audio_ci.sh

# New
./scripts/setup_audio.sh
```

### For CI/CD
```bash
# Old
./scripts/ci_setup_audio.sh

# New
./scripts/setup_audio.sh --ci
```

## Technical Details

### Script Features
- POSIX-compliant bash
- Error handling with `set -e`
- Trap handlers for cleanup
- Platform-specific logic
- Idempotent operations

### Audio Configuration
- **Linux**: PulseAudio null sink at 48kHz, stereo
- **macOS**: CoreAudio default device
- **Windows**: Default audio device

### Exit Codes
- `0`: Success
- `1`: Error (with descriptive message)

## Testing

Tested on:
- ✅ macOS (Darwin)
- ✅ Linux (Ubuntu via CI)
- ⚠️  Windows (not yet tested, but should work)

## Future Improvements

Potential enhancements:
1. Add `--verbose` flag for debug output
2. Support custom sample rates
3. Add audio device listing
4. Integrate with test runner
5. Add Windows virtual audio device support

## Related Files

- `scripts/setup_audio.sh` - Main script
- `scripts/README.md` - Scripts documentation
- `docs/audio-testing-setup.md` - User guide
- `.github/workflows/ci.yml` - CI integration
