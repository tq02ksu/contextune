#!/bin/bash
# Audio environment setup for testing
# Works for both local development and CI environments
#
# Usage:
#   ./scripts/setup_audio.sh           # Auto-detect and setup
#   ./scripts/setup_audio.sh --ci      # CI mode (minimal output)
#   ./scripts/setup_audio.sh --cleanup # Cleanup audio devices

set -e

# Parse arguments
CI_MODE=false
CLEANUP_MODE=false

for arg in "$@"; do
    case $arg in
        --ci)
            CI_MODE=true
            ;;
        --cleanup)
            CLEANUP_MODE=true
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --ci       Run in CI mode (minimal output)"
            echo "  --cleanup  Cleanup audio devices and exit"
            echo "  --help     Show this help message"
            exit 0
            ;;
    esac
done

# Logging functions
log_info() {
    if [ "$CI_MODE" = false ]; then
        echo "ℹ️  $1"
    else
        echo "[INFO] $1"
    fi
}

log_success() {
    if [ "$CI_MODE" = false ]; then
        echo "✓ $1"
    else
        echo "[OK] $1"
    fi
}

log_error() {
    if [ "$CI_MODE" = false ]; then
        echo "❌ $1" >&2
    else
        echo "[ERROR] $1" >&2
    fi
}

log_warn() {
    if [ "$CI_MODE" = false ]; then
        echo "⚠️  $1"
    else
        echo "[WARN] $1"
    fi
}

# Cleanup function for Linux
cleanup_linux_audio() {
    log_info "Cleaning up PulseAudio null sink..."
    
    # Unload null sink module
    pactl unload-module module-null-sink 2>/dev/null || true
    
    # Stop PulseAudio if we started it
    if [ "$CI_MODE" = true ]; then
        pulseaudio --kill 2>/dev/null || true
    fi
    
    log_success "Linux audio cleanup complete"
}

# Setup Linux audio
setup_linux_audio() {
    log_info "Setting up PulseAudio null sink for Linux..."
    
    # Check if PulseAudio is installed
    if ! command -v pulseaudio &> /dev/null; then
        log_warn "PulseAudio not found"
        
        if [ "$CI_MODE" = true ]; then
            log_info "Installing PulseAudio..."
            sudo apt-get update -qq
            sudo apt-get install -y -qq pulseaudio pulseaudio-utils
        else
            log_error "Please install PulseAudio:"
            echo "  Ubuntu/Debian: sudo apt-get install pulseaudio pulseaudio-utils"
            echo "  Fedora/RHEL:   sudo yum install pulseaudio pulseaudio-utils"
            echo "  Arch:          sudo pacman -S pulseaudio"
            exit 1
        fi
    fi
    
    # Start PulseAudio if not running
    if ! pulseaudio --check 2>/dev/null; then
        log_info "Starting PulseAudio daemon..."
        
        if [ "$CI_MODE" = true ]; then
            # CI mode: start with minimal logging
            pulseaudio --start --exit-idle-time=-1 --log-level=error 2>/dev/null || true
        else
            # Local mode: normal startup
            pulseaudio --start --exit-idle-time=-1 2>/dev/null || true
        fi
        
        sleep 2
    fi
    
    # Load null sink module
    log_info "Creating null sink device..."
    
    # Remove existing null sink if present
    pactl unload-module module-null-sink 2>/dev/null || true
    
    # Load new null sink
    if pactl load-module module-null-sink \
        sink_name=contexture_test_sink \
        sink_properties=device.description="Contexture_Test_Sink" \
        rate=48000 \
        channels=2 >/dev/null 2>&1; then
        log_success "Null sink created: contexture_test_sink"
    else
        log_warn "Could not create null sink (may already exist)"
    fi
    
    # Set as default sink (optional, only in CI)
    if [ "$CI_MODE" = true ]; then
        pactl set-default-sink contexture_test_sink 2>/dev/null || true
    fi
    
    # Verify setup
    if [ "$CI_MODE" = false ]; then
        log_info "Available audio sinks:"
        pactl list short sinks | grep -E "(contexture_test_sink|RUNNING)" || pactl list short sinks
    fi
    
    log_success "Linux audio environment ready"
}

# Setup macOS audio
setup_macos_audio() {
    log_info "Configuring macOS audio environment..."
    
    # Check for BlackHole virtual audio device
    if system_profiler SPAudioDataType 2>/dev/null | grep -q "BlackHole"; then
        log_success "BlackHole virtual audio device detected"
    else
        if [ "$CI_MODE" = false ]; then
            log_warn "BlackHole not installed (optional for better testing)"
            echo ""
            echo "To install BlackHole virtual audio device:"
            echo "  brew install blackhole-2ch"
            echo ""
            echo "Using default CoreAudio device for now."
        else
            log_info "Using default CoreAudio device"
        fi
    fi
    
    log_success "macOS audio environment ready"
}

# Setup Windows audio
setup_windows_audio() {
    log_info "Configuring Windows audio environment..."
    
    # Windows always has default audio devices
    log_info "Using Windows default audio device"
    
    if [ "$CI_MODE" = false ]; then
        log_warn "For virtual audio on Windows, consider:"
        echo "  - VB-CABLE: https://vb-audio.com/Cable/"
        echo "  - Virtual Audio Cable: https://vac.muzychenko.net/"
    fi
    
    log_success "Windows audio environment ready"
}

# Main setup function
setup_audio() {
    if [ "$CI_MODE" = true ]; then
        echo "=== Audio Setup (CI Mode) ==="
    else
        echo ""
        echo "╔════════════════════════════════════════╗"
        echo "║  Contexture Audio Environment Setup   ║"
        echo "╚════════════════════════════════════════╝"
        echo ""
    fi
    
    # Detect OS
    OS="$(uname -s)"
    case "${OS}" in
        Linux*)
            log_info "Detected: Linux"
            setup_linux_audio
            ;;
        Darwin*)
            log_info "Detected: macOS"
            setup_macos_audio
            ;;
        MINGW*|MSYS*|CYGWIN*)
            log_info "Detected: Windows"
            setup_windows_audio
            ;;
        *)
            log_error "Unknown OS: ${OS}"
            exit 1
            ;;
    esac
    
    echo ""
    log_success "Audio environment setup complete!"
    
    if [ "$CI_MODE" = false ]; then
        echo ""
        echo "Next steps:"
        echo "  1. Generate test audio files:"
        echo "     cd core && cargo run --example generate_test_audio"
        echo ""
        echo "  2. Run audio tests:"
        echo "     cd core && cargo test"
        echo ""
    fi
}

# Main execution
if [ "$CLEANUP_MODE" = true ]; then
    # Cleanup mode
    OS="$(uname -s)"
    case "${OS}" in
        Linux*)
            cleanup_linux_audio
            ;;
        *)
            log_info "No cleanup needed for ${OS}"
            ;;
    esac
else
    # Setup mode
    setup_audio
    
    # Register cleanup on exit (Linux only)
    if [[ "$(uname -s)" == "Linux" ]] && [ "$CI_MODE" = true ]; then
        trap cleanup_linux_audio EXIT
    fi
fi
