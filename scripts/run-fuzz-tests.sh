#!/bin/bash
# Script to run fuzzing tests for FFI interfaces
# Usage: ./scripts/run-fuzz-tests.sh [target] [duration]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

TARGET=${1:-ffi_safety}
DURATION=${2:-60}  # Default 60 seconds

echo -e "${YELLOW}Running fuzzing tests for target: $TARGET${NC}"
echo -e "${YELLOW}Duration: ${DURATION} seconds${NC}"

# Check if cargo-fuzz is installed
if ! command -v cargo-fuzz &> /dev/null; then
    echo -e "${RED}Error: cargo-fuzz is not installed${NC}"
    echo "Install with: cargo install cargo-fuzz"
    exit 1
fi

# Check if nightly Rust is available
if ! rustup toolchain list | grep -q nightly; then
    echo -e "${RED}Error: Nightly Rust toolchain is required for fuzzing${NC}"
    echo "Install with: rustup install nightly"
    exit 1
fi

# Change to core directory
cd "$(dirname "$0")/../core"

# Check if target exists
if [ ! -f "fuzz/fuzz_targets/${TARGET}.rs" ]; then
    echo -e "${RED}Error: Fuzz target '${TARGET}' not found${NC}"
    echo "Available targets:"
    ls fuzz/fuzz_targets/*.rs 2>/dev/null | sed 's/.*\///;s/\.rs$//' || echo "No fuzz targets found"
    exit 1
fi

echo -e "${YELLOW}Building fuzz target with nightly...${NC}"
cargo +nightly fuzz build "$TARGET"

echo -e "${YELLOW}Running fuzz test...${NC}"

# Use gtimeout on macOS if available, otherwise use a background process approach
if command -v gtimeout &> /dev/null; then
    gtimeout "${DURATION}s" cargo +nightly fuzz run "$TARGET" || {
        exit_code=$?
        if [ $exit_code -eq 124 ]; then
            echo -e "${GREEN}Fuzzing completed successfully (timeout reached)${NC}"
        else
            echo -e "${RED}Fuzzing failed with exit code: $exit_code${NC}"
            exit $exit_code
        fi
    }
elif command -v timeout &> /dev/null; then
    timeout "${DURATION}s" cargo +nightly fuzz run "$TARGET" || {
        exit_code=$?
        if [ $exit_code -eq 124 ]; then
            echo -e "${GREEN}Fuzzing completed successfully (timeout reached)${NC}"
        else
            echo -e "${RED}Fuzzing failed with exit code: $exit_code${NC}"
            exit $exit_code
        fi
    }
else
    # Fallback: run in background and kill after duration
    echo -e "${YELLOW}Running fuzzing for ${DURATION} seconds (no timeout command available)...${NC}"
    cargo +nightly fuzz run "$TARGET" &
    FUZZ_PID=$!
    sleep "$DURATION"
    kill $FUZZ_PID 2>/dev/null || true
    wait $FUZZ_PID 2>/dev/null || true
    echo -e "${GREEN}Fuzzing completed successfully (manual timeout)${NC}"
fi

echo -e "${GREEN}Fuzzing tests completed!${NC}"

# Show any crashes found
if [ -d "fuzz/artifacts/$TARGET" ] && [ "$(ls -A fuzz/artifacts/$TARGET 2>/dev/null)" ]; then
    echo -e "${YELLOW}Crashes found in fuzz/artifacts/$TARGET:${NC}"
    ls -la "fuzz/artifacts/$TARGET"
else
    echo -e "${GREEN}No crashes found!${NC}"
fi