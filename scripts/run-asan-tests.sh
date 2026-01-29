#!/bin/bash
# Script to run tests with AddressSanitizer
# Usage: ./scripts/run-asan-tests.sh [test-name]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Running tests with AddressSanitizer...${NC}"

# Check if nightly Rust is available
if ! rustup toolchain list | grep -q nightly; then
    echo -e "${RED}Error: Nightly Rust toolchain is required for AddressSanitizer${NC}"
    echo "Install with: rustup install nightly"
    exit 1
fi

# Detect platform and set appropriate target
case "$(uname -s)" in
    Linux*)
        TARGET="x86_64-unknown-linux-gnu"
        ;;
    Darwin*)
        TARGET="x86_64-apple-darwin"
        ;;
    CYGWIN*|MINGW*|MSYS*)
        TARGET="x86_64-pc-windows-msvc"
        echo -e "${YELLOW}Warning: AddressSanitizer support on Windows is limited${NC}"
        ;;
    *)
        echo -e "${RED}Error: Unsupported platform${NC}"
        exit 1
        ;;
esac

echo -e "${YELLOW}Using target: $TARGET${NC}"

# Set AddressSanitizer environment variables
export RUSTFLAGS="-Zsanitizer=address"
export ASAN_OPTIONS="detect_leaks=1:abort_on_error=1:detect_stack_use_after_return=1:check_initialization_order=1:strict_init_order=1"

# Change to core directory
cd "$(dirname "$0")/../core"

# Run specific test if provided, otherwise run all tests
if [ $# -eq 1 ]; then
    TEST_NAME="$1"
    echo -e "${YELLOW}Running test: $TEST_NAME${NC}"
    cargo +nightly test --target "$TARGET" "$TEST_NAME" -- --nocapture
else
    echo -e "${YELLOW}Running all tests with AddressSanitizer...${NC}"
    cargo +nightly test --target "$TARGET" -- --nocapture
fi

echo -e "${GREEN}AddressSanitizer tests completed!${NC}"