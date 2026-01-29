#!/bin/bash
# Script to run tests with MemorySanitizer
# Usage: ./scripts/run-msan-tests.sh [test-name]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Running tests with MemorySanitizer...${NC}"

# Check if nightly Rust is available
if ! rustup toolchain list | grep -q nightly; then
    echo -e "${RED}Error: Nightly Rust toolchain is required for MemorySanitizer${NC}"
    echo "Install with: rustup install nightly"
    exit 1
fi

# Detect platform and set appropriate target
case "$(uname -s)" in
    Linux*)
        TARGET="x86_64-unknown-linux-gnu"
        ;;
    Darwin*)
        echo -e "${YELLOW}Warning: MemorySanitizer is not supported on macOS${NC}"
        echo -e "${YELLOW}Falling back to AddressSanitizer...${NC}"
        TARGET="x86_64-apple-darwin"
        export RUSTFLAGS="-Zsanitizer=address"
        ;;
    CYGWIN*|MINGW*|MSYS*)
        echo -e "${RED}Error: MemorySanitizer is not supported on Windows${NC}"
        exit 1
        ;;
    *)
        echo -e "${RED}Error: Unsupported platform${NC}"
        exit 1
        ;;
esac

echo -e "${YELLOW}Using target: $TARGET${NC}"

# Set MemorySanitizer environment variables (Linux only)
if [[ "$TARGET" == "x86_64-unknown-linux-gnu" ]]; then
    export RUSTFLAGS="-Zsanitizer=memory"
    export MSAN_OPTIONS="abort_on_error=1:print_stats=1"
else
    # Use AddressSanitizer as fallback
    export ASAN_OPTIONS="detect_leaks=1:abort_on_error=1:detect_stack_use_after_return=1"
fi

# Change to core directory
cd "$(dirname "$0")/../core"

# Run specific test if provided, otherwise run all tests
if [ $# -eq 1 ]; then
    TEST_NAME="$1"
    echo -e "${YELLOW}Running test: $TEST_NAME${NC}"
    cargo +nightly test --target "$TARGET" "$TEST_NAME" -- --nocapture
else
    echo -e "${YELLOW}Running all tests with MemorySanitizer...${NC}"
    cargo +nightly test --target "$TARGET" -- --nocapture
fi

echo -e "${GREEN}MemorySanitizer tests completed!${NC}"