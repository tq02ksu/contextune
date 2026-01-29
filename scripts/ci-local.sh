#!/bin/bash

# Local CI Script
# Run the same checks that GitHub Actions runs, locally

set -e

echo "🚀 Running local CI checks..."
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ $1 passed${NC}"
    else
        echo -e "${RED}✗ $1 failed${NC}"
        exit 1
    fi
}

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust is not installed${NC}"
    echo "Install from: https://rustup.rs/"
    exit 1
fi

echo "📋 Step 1: Checking code formatting..."
cargo fmt --all -- --check
print_status "Format check"
echo ""

echo "🔍 Step 2: Running Clippy lints..."
cargo clippy --all-targets --all-features -- -D warnings
print_status "Clippy"
echo ""

echo "🧪 Step 3: Running unit tests..."
cargo test --all-features
print_status "Unit tests"
echo ""

echo "📚 Step 4: Running doc tests..."
cargo test --doc --all-features
print_status "Doc tests"
echo ""

# Optional: Run benchmarks if requested
if [ "$1" == "--bench" ]; then
    echo "⚡ Step 5: Running benchmarks..."
    cargo bench
    print_status "Benchmarks"
    echo ""
fi

# Optional: Run coverage if tarpaulin is installed
if command -v cargo-tarpaulin &> /dev/null; then
    if [ "$1" == "--coverage" ] || [ "$2" == "--coverage" ]; then
        echo "📊 Step 6: Generating coverage report..."
        cargo tarpaulin --out Html --output-dir coverage
        print_status "Coverage"
        echo -e "${YELLOW}Coverage report: coverage/index.html${NC}"
        echo ""
    fi
else
    if [ "$1" == "--coverage" ] || [ "$2" == "--coverage" ]; then
        echo -e "${YELLOW}⚠ Tarpaulin not installed. Skipping coverage.${NC}"
        echo "Install with: cargo install cargo-tarpaulin"
        echo ""
    fi
fi

# Optional: Run security audit if cargo-audit is installed
if command -v cargo-audit &> /dev/null; then
    echo "🔒 Step 7: Running security audit..."
    cd core
    cargo audit
    cd ..
    print_status "Security audit"
    echo ""
else
    echo -e "${YELLOW}⚠ cargo-audit not installed. Skipping security check.${NC}"
    echo "Install with: cargo install cargo-audit"
    echo ""
fi

# Optional: Run dependency check if cargo-deny is installed
if command -v cargo-deny &> /dev/null; then
    echo "🚫 Step 8: Running dependency check..."
    cd core
    cargo deny-check --hide-inclusion-graph
    cd ..
    print_status "Dependency check"
    echo ""
else
    echo -e "${YELLOW}⚠ cargo-deny not installed. Skipping dependency check.${NC}"
    echo "Install with: cargo install cargo-deny"
    echo ""
fi

echo -e "${GREEN}✅ All checks passed!${NC}"
echo ""
echo "Usage:"
echo "  ./scripts/ci-local.sh           # Run basic checks"
echo "  ./scripts/ci-local.sh --bench   # Include benchmarks"
echo "  ./scripts/ci-local.sh --coverage # Include coverage report"
