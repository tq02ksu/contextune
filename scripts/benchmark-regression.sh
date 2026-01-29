#!/usr/bin/env bash
# Performance regression detection script
#
# This script runs benchmarks and compares results against a baseline
# to detect performance regressions.
#
# Usage:
#   ./scripts/benchmark-regression.sh [baseline|compare|update]
#
# Commands:
#   baseline - Run benchmarks and save as baseline
#   compare  - Run benchmarks and compare against baseline
#   update   - Update baseline with current results

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BASELINE_DIR="target/criterion-baseline"
CURRENT_DIR="target/criterion"
REGRESSION_THRESHOLD=5 # 5% regression threshold
REPORT_FILE="target/benchmark-report.txt"

# Print colored message
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if criterion is available
check_dependencies() {
    if ! cargo bench --help &> /dev/null; then
        print_error "cargo bench is not available"
        exit 1
    fi
}

# Run benchmarks
run_benchmarks() {
    local output_dir=$1
    print_info "Running benchmarks..."
    
    # Run benchmarks with criterion
    CRITERION_HOME="$output_dir" cargo bench --bench audio_pipeline 2>&1 | tee "$REPORT_FILE"
    
    print_success "Benchmarks completed"
}

# Save baseline
save_baseline() {
    print_info "Saving baseline benchmarks..."
    
    # Remove old baseline if exists
    if [ -d "$BASELINE_DIR" ]; then
        print_warning "Removing old baseline"
        rm -rf "$BASELINE_DIR"
    fi
    
    # Create baseline directory
    mkdir -p "$BASELINE_DIR"
    
    # Run benchmarks and save to baseline
    run_benchmarks "$BASELINE_DIR"
    
    print_success "Baseline saved to $BASELINE_DIR"
}

# Compare benchmarks
compare_benchmarks() {
    print_info "Comparing benchmarks against baseline..."
    
    if [ ! -d "$BASELINE_DIR" ]; then
        print_error "No baseline found. Run './scripts/benchmark-regression.sh baseline' first"
        exit 1
    fi
    
    # Run current benchmarks
    run_benchmarks "$CURRENT_DIR"
    
    # Use Python analyzer if available
    if command -v python3 &> /dev/null; then
        print_info "Using Python analyzer for detailed comparison..."
        python3 scripts/analyze-benchmarks.py \
            --baseline "$BASELINE_DIR" \
            --current "$CURRENT_DIR" \
            --threshold "$REGRESSION_THRESHOLD"
        
        local exit_code=$?
        if [ $exit_code -eq 0 ]; then
            print_success "No significant regressions detected"
        else
            print_error "Performance regressions detected!"
            exit 1
        fi
    else
        # Fallback to simple comparison
        print_warning "Python3 not found, using simplified comparison"
        
        local regressions=0
        local improvements=0
        
        # Parse benchmark results (simplified)
        print_info "Regression analysis:"
        echo "----------------------------------------"
        
        if [ -f "$REPORT_FILE" ]; then
            print_info "Benchmark report saved to $REPORT_FILE"
            
            # Check for obvious performance issues in the report
            if grep -q "Performance has regressed" "$REPORT_FILE" 2>/dev/null; then
                print_warning "Potential regressions detected in report"
                regressions=1
            fi
            
            if grep -q "Performance has improved" "$REPORT_FILE" 2>/dev/null; then
                print_success "Performance improvements detected"
                improvements=1
            fi
        fi
        
        echo "----------------------------------------"
        
        if [ $regressions -gt 0 ]; then
            print_error "Performance regressions detected!"
            print_info "Review the report at $REPORT_FILE"
            print_info "Install Python3 for detailed analysis"
            exit 1
        else
            print_success "No obvious regressions detected"
            print_info "Install Python3 for detailed analysis"
        fi
    fi
}

# Update baseline with current results
update_baseline() {
    print_info "Updating baseline with current results..."
    
    if [ ! -d "$CURRENT_DIR" ]; then
        print_error "No current benchmark results found. Run benchmarks first."
        exit 1
    fi
    
    # Backup old baseline
    if [ -d "$BASELINE_DIR" ]; then
        local backup_dir="${BASELINE_DIR}.backup.$(date +%Y%m%d_%H%M%S)"
        print_info "Backing up old baseline to $backup_dir"
        mv "$BASELINE_DIR" "$backup_dir"
    fi
    
    # Copy current to baseline
    cp -r "$CURRENT_DIR" "$BASELINE_DIR"
    
    print_success "Baseline updated"
}

# Show usage
show_usage() {
    cat << EOF
Performance Regression Detection Script

Usage:
    $0 [command]

Commands:
    baseline    Run benchmarks and save as baseline
    compare     Run benchmarks and compare against baseline (default)
    update      Update baseline with current results
    help        Show this help message

Examples:
    # Save initial baseline
    $0 baseline

    # Compare current performance against baseline
    $0 compare

    # Update baseline after verifying improvements
    $0 update

Configuration:
    Regression threshold: ${REGRESSION_THRESHOLD}%
    Baseline directory:   ${BASELINE_DIR}
    Current directory:    ${CURRENT_DIR}
    Report file:          ${REPORT_FILE}

EOF
}

# Main
main() {
    local command=${1:-compare}
    
    check_dependencies
    
    case "$command" in
        baseline)
            save_baseline
            ;;
        compare)
            compare_benchmarks
            ;;
        update)
            update_baseline
            ;;
        help|--help|-h)
            show_usage
            ;;
        *)
            print_error "Unknown command: $command"
            show_usage
            exit 1
            ;;
    esac
}

main "$@"
