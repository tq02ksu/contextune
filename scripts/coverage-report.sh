#!/bin/bash

# Comprehensive Coverage Report Generator for Contextune Music Player Plugin
# This script generates coverage reports, checks thresholds, and provides detailed analysis

set -e

# Configuration
COVERAGE_THRESHOLD=85
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORE_DIR="${WORKSPACE_ROOT}/core"
COVERAGE_DIR="${WORKSPACE_ROOT}/target/coverage"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default options
GENERATE_HTML=true
GENERATE_XML=true
CHECK_THRESHOLD=true
OPEN_REPORT=false
CI_MODE=false
EXCLUDE_SLOW_TESTS=false

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help              Show this help message"
    echo "  --html-only             Generate only HTML report"
    echo "  --xml-only              Generate only XML report"
    echo "  --no-threshold          Skip threshold checking"
    echo "  --open                  Open HTML report in browser after generation"
    echo "  --ci                    CI mode (XML only, no colors, exit on threshold failure)"
    echo "  --exclude-slow          Exclude slow tests (memory leak tests)"
    echo "  --threshold N           Set custom coverage threshold (default: 85)"
    echo ""
    echo "Examples:"
    echo "  $0                      Generate both HTML and XML reports with threshold check"
    echo "  $0 --html-only --open   Generate HTML report and open in browser"
    echo "  $0 --ci                 CI mode for automated builds"
    echo "  $0 --exclude-slow       Skip memory leak tests for faster execution"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        --html-only)
            GENERATE_XML=false
            shift
            ;;
        --xml-only)
            GENERATE_HTML=false
            shift
            ;;
        --no-threshold)
            CHECK_THRESHOLD=false
            shift
            ;;
        --open)
            OPEN_REPORT=true
            shift
            ;;
        --ci)
            CI_MODE=true
            GENERATE_HTML=false
            GENERATE_XML=true
            CHECK_THRESHOLD=true
            shift
            ;;
        --exclude-slow)
            EXCLUDE_SLOW_TESTS=true
            shift
            ;;
        --threshold)
            COVERAGE_THRESHOLD="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Disable colors in CI mode
if [[ "$CI_MODE" == "true" ]]; then
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    CYAN=''
    NC=''
fi

echo "🔍 Generating coverage report for Contextune Music Player Plugin..."
echo "📁 Workspace: ${WORKSPACE_ROOT}"
echo "🎯 Coverage threshold: ${COVERAGE_THRESHOLD}%"
echo ""

# Change to core directory
cd "${CORE_DIR}"

# Check if Tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo -e "${YELLOW}⚠️  cargo-tarpaulin not found. Installing...${NC}"
    cargo install cargo-tarpaulin --features vendored-openssl
fi

# Create coverage directory
mkdir -p "${COVERAGE_DIR}"

# Build Tarpaulin command
TARPAULIN_CMD="cargo tarpaulin"
TARPAULIN_ARGS="--output-dir ${COVERAGE_DIR} --exclude-files 'target/*' --exclude-files 'core/fuzz/*' --exclude-files 'core/benches/*' --exclude-files 'core/examples/*'"

# Add output formats
OUTPUT_FORMATS=""
if [[ "$GENERATE_HTML" == "true" ]]; then
    OUTPUT_FORMATS="${OUTPUT_FORMATS} --out Html"
fi
if [[ "$GENERATE_XML" == "true" ]]; then
    OUTPUT_FORMATS="${OUTPUT_FORMATS} --out Xml"
fi

# Add test exclusions for slow tests
if [[ "$EXCLUDE_SLOW_TESTS" == "true" ]]; then
    TARPAULIN_ARGS="${TARPAULIN_ARGS} --exclude-files '**/memory_leak_detection.rs'"
    echo -e "${YELLOW}⚠️  Excluding slow memory leak tests${NC}"
fi

# Run coverage analysis
echo "🧪 Running coverage analysis..."
echo -e "${CYAN}Command: ${TARPAULIN_CMD} ${OUTPUT_FORMATS} ${TARPAULIN_ARGS}${NC}"
echo ""

eval "${TARPAULIN_CMD} ${OUTPUT_FORMATS} ${TARPAULIN_ARGS}"

# Check if reports were generated
REPORTS_GENERATED=()
HTML_REPORT="${COVERAGE_DIR}/tarpaulin-report.html"
XML_REPORT="${COVERAGE_DIR}/cobertura.xml"

if [[ "$GENERATE_HTML" == "true" && -f "$HTML_REPORT" ]]; then
    REPORTS_GENERATED+=("HTML: $HTML_REPORT")
fi

if [[ "$GENERATE_XML" == "true" && -f "$XML_REPORT" ]]; then
    REPORTS_GENERATED+=("XML: $XML_REPORT")
fi

if [[ ${#REPORTS_GENERATED[@]} -eq 0 ]]; then
    echo -e "${RED}❌ No coverage reports were generated!${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}✅ Coverage analysis completed!${NC}"
echo ""

# Parse coverage from XML report if available
if [[ -f "$XML_REPORT" ]]; then
    COVERAGE_RATE=$(grep -o 'line-rate="[0-9.]*"' "$XML_REPORT" | head -1 | grep -o '[0-9.]*')
    COVERAGE_PERCENT=$(echo "$COVERAGE_RATE * 100" | bc -l)
    COVERAGE_INT=$(echo "$COVERAGE_PERCENT" | cut -d. -f1)
    
    echo "📊 Coverage Results:"
    echo "   Current coverage: ${COVERAGE_PERCENT}%"
    
    if [[ "$CHECK_THRESHOLD" == "true" ]]; then
        echo "   Required threshold: ${COVERAGE_THRESHOLD}%"
        
        if [[ "$COVERAGE_INT" -ge "$COVERAGE_THRESHOLD" ]]; then
            echo -e "${GREEN}   ✅ Coverage meets threshold!${NC}"
            THRESHOLD_MET=true
        else
            DEFICIT=$((COVERAGE_THRESHOLD - COVERAGE_INT))
            echo -e "${RED}   ❌ Coverage below threshold by ${DEFICIT} percentage points${NC}"
            THRESHOLD_MET=false
        fi
    fi
    echo ""
fi

# Show generated reports
echo "📄 Generated Reports:"
for report in "${REPORTS_GENERATED[@]}"; do
    echo "   ${report}"
done
echo ""

# Show coverage breakdown if XML report exists
if [[ -f "$XML_REPORT" ]]; then
    echo "📈 Coverage Breakdown:"
    
    # Extract package coverage information
    while IFS= read -r line; do
        if [[ $line =~ \<package.*name=\"([^\"]+)\".*line-rate=\"([0-9.]+)\" ]]; then
            package_name="${BASH_REMATCH[1]}"
            package_rate="${BASH_REMATCH[2]}"
            package_percent=$(echo "$package_rate * 100" | bc -l | cut -d. -f1)
            
            if [[ $package_percent -ge 90 ]]; then
                color=$GREEN
            elif [[ $package_percent -ge 70 ]]; then
                color=$YELLOW
            else
                color=$RED
            fi
            
            echo -e "   ${color}${package_name}: ${package_percent}%${NC}"
        fi
    done < "$XML_REPORT"
    echo ""
fi

# Provide improvement suggestions if threshold not met
if [[ "$CHECK_THRESHOLD" == "true" && "$THRESHOLD_MET" == "false" ]]; then
    echo "💡 Tips to improve coverage:"
    echo "   • Add unit tests for uncovered functions"
    echo "   • Add integration tests for complex workflows"
    echo "   • Remove dead code or mark it as unreachable"
    echo "   • Consider adding property-based tests"
    echo "   • Review uncovered lines in the HTML report"
    echo ""
fi

# Open HTML report if requested
if [[ "$OPEN_REPORT" == "true" && -f "$HTML_REPORT" ]]; then
    echo "🌐 Opening HTML report in browser..."
    if command -v open &> /dev/null; then
        open "$HTML_REPORT"
    elif command -v xdg-open &> /dev/null; then
        xdg-open "$HTML_REPORT"
    elif command -v start &> /dev/null; then
        start "$HTML_REPORT"
    else
        echo -e "${YELLOW}⚠️  Could not open browser automatically. Please open: ${HTML_REPORT}${NC}"
    fi
fi

# Exit with appropriate code
if [[ "$CHECK_THRESHOLD" == "true" && "$THRESHOLD_MET" == "false" ]]; then
    if [[ "$CI_MODE" == "true" ]]; then
        echo "Coverage below threshold in CI mode"
        exit 1
    else
        echo -e "${YELLOW}⚠️  Coverage below threshold, but continuing...${NC}"
        exit 0
    fi
else
    echo -e "${GREEN}🎉 Coverage report generation completed successfully!${NC}"
    exit 0
fi