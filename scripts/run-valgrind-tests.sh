#!/bin/bash
# Script to run tests with Valgrind
# Usage: ./scripts/run-valgrind-tests.sh [test-name]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Running tests with Valgrind...${NC}"

# Check if Valgrind is available
if ! command -v valgrind &> /dev/null; then
    echo -e "${RED}Error: Valgrind is not installed${NC}"
    case "$(uname -s)" in
        Linux*)
            echo "Install with: sudo apt-get install valgrind"
            ;;
        Darwin*)
            echo "Install with: brew install valgrind"
            echo -e "${YELLOW}Note: Valgrind support on macOS is limited${NC}"
            ;;
        *)
            echo "Please install Valgrind for your platform"
            ;;
    esac
    exit 1
fi

# Change to core directory
cd "$(dirname "$0")/../core"

# Build tests first
echo -e "${YELLOW}Building tests...${NC}"
cargo build --tests

# Find test executables
TEST_DIR="target/debug/deps"
if [ ! -d "$TEST_DIR" ]; then
    echo -e "${RED}Error: Test directory not found. Make sure tests are built.${NC}"
    exit 1
fi

# Valgrind options
VALGRIND_OPTS="--leak-check=full --show-leak-kinds=all --track-origins=yes --error-exitcode=1"

if [ $# -eq 1 ]; then
    TEST_NAME="$1"
    echo -e "${YELLOW}Running specific test: $TEST_NAME${NC}"
    
    # Find the test executable
    TEST_EXEC=$(find "$TEST_DIR" -name "*$TEST_NAME*" -type f -executable | head -1)
    if [ -z "$TEST_EXEC" ]; then
        echo -e "${RED}Error: Test executable for '$TEST_NAME' not found${NC}"
        exit 1
    fi
    
    echo -e "${YELLOW}Running: valgrind $VALGRIND_OPTS $TEST_EXEC${NC}"
    valgrind $VALGRIND_OPTS "$TEST_EXEC"
else
    echo -e "${YELLOW}Running all unit tests with Valgrind...${NC}"
    
    # Find the main library test executable
    LIB_TEST=$(find "$TEST_DIR" -name "contexture_core-*" -type f -executable | head -1)
    if [ -n "$LIB_TEST" ]; then
        echo -e "${YELLOW}Running library tests: $LIB_TEST${NC}"
        valgrind $VALGRIND_OPTS "$LIB_TEST"
    else
        echo -e "${RED}Error: Library test executable not found${NC}"
        exit 1
    fi
    
    # Run integration tests
    echo -e "${YELLOW}Running integration tests...${NC}"
    for test_exec in "$TEST_DIR"/*; do
        if [ -x "$test_exec" ] && [[ "$test_exec" != *"contexture_core"* ]]; then
            echo -e "${YELLOW}Running: $(basename "$test_exec")${NC}"
            if ! valgrind $VALGRIND_OPTS "$test_exec"; then
                echo -e "${RED}Test failed: $(basename "$test_exec")${NC}"
            fi
        fi
    done
fi

echo -e "${GREEN}Valgrind tests completed!${NC}"