#!/bin/bash

# Security Scanning Script for Contextune Music Player Plugin
# Comprehensive security analysis for local development

set -e

# Configuration
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORE_DIR="${WORKSPACE_ROOT}/core"
RESULTS_DIR="${WORKSPACE_ROOT}/target/security-results"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default options
RUN_AUDIT=true
RUN_DENY=true
RUN_SUPPLY_CHAIN=true
VERBOSE=false
UPDATE_DB=true

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help              Show this help message"
    echo "  --audit-only            Run only cargo-audit"
    echo "  --deny-only             Run only cargo-deny"
    echo "  --supply-chain-only     Run only supply chain analysis"
    echo "  --no-update             Skip updating advisory database"
    echo "  --verbose               Enable verbose output"
    echo ""
    echo "Examples:"
    echo "  $0                      Run all security scans"
    echo "  $0 --audit-only         Run only vulnerability audit"
    echo "  $0 --verbose            Run with detailed output"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        --audit-only)
            RUN_DENY=false
            RUN_SUPPLY_CHAIN=false
            shift
            ;;
        --deny-only)
            RUN_AUDIT=false
            RUN_SUPPLY_CHAIN=false
            shift
            ;;
        --supply-chain-only)
            RUN_AUDIT=false
            RUN_DENY=false
            shift
            ;;
        --no-update)
            UPDATE_DB=false
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

echo -e "${BLUE}🔒 Security Scanning for Contextune Music Player Plugin${NC}"
echo "📁 Workspace: ${WORKSPACE_ROOT}"
echo ""

# Create results directory
mkdir -p "${RESULTS_DIR}"

# Change to core directory
cd "${CORE_DIR}"

# Function to check if a tool is installed
check_tool() {
    local tool=$1
    local install_cmd=$2
    
    if ! command -v "$tool" &> /dev/null; then
        echo -e "${YELLOW}⚠️  $tool not found. Installing...${NC}"
        eval "$install_cmd"
        if ! command -v "$tool" &> /dev/null; then
            echo -e "${RED}❌ Failed to install $tool${NC}"
            return 1
        fi
    fi
    return 0
}

# Run cargo-audit
if [[ "$RUN_AUDIT" == "true" ]]; then
    echo -e "${CYAN}🔍 Running Vulnerability Audit...${NC}"
    
    if check_tool "cargo-audit" "cargo install cargo-audit --locked"; then
        # Update advisory database
        if [[ "$UPDATE_DB" == "true" ]]; then
            echo "📥 Updating advisory database..."
            cargo audit --update
        fi
        
        # Run audit
        echo "🔍 Scanning for known vulnerabilities..."
        if [[ "$VERBOSE" == "true" ]]; then
            cargo audit --color always
        else
            cargo audit --color always --quiet
        fi
        
        # Generate JSON report
        cargo audit --json > "${RESULTS_DIR}/audit-results.json" 2>/dev/null || true
        
        echo -e "${GREEN}✅ Vulnerability audit completed${NC}"
    else
        echo -e "${RED}❌ Skipping vulnerability audit - tool installation failed${NC}"
    fi
    echo ""
fi

# Run cargo-deny
if [[ "$RUN_DENY" == "true" ]]; then
    echo -e "${CYAN}🚫 Running Dependency Policy Check...${NC}"
    
    if check_tool "cargo-deny" "cargo install cargo-deny --locked"; then
        echo "📋 Checking dependency policies..."
        
        # Check all policies
        if [[ "$VERBOSE" == "true" ]]; then
            cargo deny-check
        else
            cargo deny-check --hide-inclusion-graph
        fi
        
        # Generate JSON report
        cargo deny-check --format json > "${RESULTS_DIR}/deny-results.json" 2>/dev/null || true
        
        echo -e "${GREEN}✅ Dependency policy check completed${NC}"
    else
        echo -e "${RED}❌ Skipping dependency policy check - tool installation failed${NC}"
    fi
    echo ""
fi

# Run supply chain analysis
if [[ "$RUN_SUPPLY_CHAIN" == "true" ]]; then
    echo -e "${CYAN}🔗 Running Supply Chain Analysis...${NC}"
    
    echo "📦 Analyzing dependency tree..."
    
    # Create supply chain report
    SUPPLY_CHAIN_REPORT="${RESULTS_DIR}/supply-chain-report.md"
    echo "# Supply Chain Security Report" > "$SUPPLY_CHAIN_REPORT"
    echo "Generated on: $(date)" >> "$SUPPLY_CHAIN_REPORT"
    echo "" >> "$SUPPLY_CHAIN_REPORT"
    
    # List all dependencies with sources
    echo "## All Dependencies" >> "$SUPPLY_CHAIN_REPORT"
    echo '```' >> "$SUPPLY_CHAIN_REPORT"
    cargo tree --format "{p} from {r}" >> "$SUPPLY_CHAIN_REPORT"
    echo '```' >> "$SUPPLY_CHAIN_REPORT"
    echo "" >> "$SUPPLY_CHAIN_REPORT"
    
    # Check for git/path dependencies
    echo "## Non-Registry Dependencies" >> "$SUPPLY_CHAIN_REPORT"
    echo '```' >> "$SUPPLY_CHAIN_REPORT"
    cargo tree --format "{p} {r}" | grep -E "(git|path)" >> "$SUPPLY_CHAIN_REPORT" || echo "None found" >> "$SUPPLY_CHAIN_REPORT"
    echo '```' >> "$SUPPLY_CHAIN_REPORT"
    echo "" >> "$SUPPLY_CHAIN_REPORT"
    
    # Count dependencies by source
    echo "## Dependency Statistics" >> "$SUPPLY_CHAIN_REPORT"
    TOTAL_DEPS=$(cargo tree --format "{p}" | wc -l)
    REGISTRY_DEPS=$(cargo tree --format "{r}" | grep -c "registry" || echo "0")
    GIT_DEPS=$(cargo tree --format "{r}" | grep -c "git" || echo "0")
    PATH_DEPS=$(cargo tree --format "{r}" | grep -c "path" || echo "0")
    
    echo "- Total dependencies: $TOTAL_DEPS" >> "$SUPPLY_CHAIN_REPORT"
    echo "- Registry dependencies: $REGISTRY_DEPS" >> "$SUPPLY_CHAIN_REPORT"
    echo "- Git dependencies: $GIT_DEPS" >> "$SUPPLY_CHAIN_REPORT"
    echo "- Path dependencies: $PATH_DEPS" >> "$SUPPLY_CHAIN_REPORT"
    
    if [[ "$VERBOSE" == "true" ]]; then
        echo "📊 Dependency statistics:"
        echo "   Total: $TOTAL_DEPS"
        echo "   Registry: $REGISTRY_DEPS"
        echo "   Git: $GIT_DEPS"
        echo "   Path: $PATH_DEPS"
    fi
    
    echo -e "${GREEN}✅ Supply chain analysis completed${NC}"
    echo ""
fi

# Generate summary report
SUMMARY_REPORT="${RESULTS_DIR}/security-summary.md"
echo "# Security Scan Summary" > "$SUMMARY_REPORT"
echo "Generated on: $(date)" >> "$SUMMARY_REPORT"
echo "" >> "$SUMMARY_REPORT"

if [[ "$RUN_AUDIT" == "true" ]]; then
    echo "## Vulnerability Audit" >> "$SUMMARY_REPORT"
    if [[ -f "${RESULTS_DIR}/audit-results.json" ]]; then
        echo "✅ Completed - see audit-results.json for details" >> "$SUMMARY_REPORT"
    else
        echo "❌ Failed to generate results" >> "$SUMMARY_REPORT"
    fi
    echo "" >> "$SUMMARY_REPORT"
fi

if [[ "$RUN_DENY" == "true" ]]; then
    echo "## Dependency Policy Check" >> "$SUMMARY_REPORT"
    if [[ -f "${RESULTS_DIR}/deny-results.json" ]]; then
        echo "✅ Completed - see deny-results.json for details" >> "$SUMMARY_REPORT"
    else
        echo "❌ Failed to generate results" >> "$SUMMARY_REPORT"
    fi
    echo "" >> "$SUMMARY_REPORT"
fi

if [[ "$RUN_SUPPLY_CHAIN" == "true" ]]; then
    echo "## Supply Chain Analysis" >> "$SUMMARY_REPORT"
    if [[ -f "${RESULTS_DIR}/supply-chain-report.md" ]]; then
        echo "✅ Completed - see supply-chain-report.md for details" >> "$SUMMARY_REPORT"
    else
        echo "❌ Failed to generate results" >> "$SUMMARY_REPORT"
    fi
    echo "" >> "$SUMMARY_REPORT"
fi

# Show results
echo -e "${GREEN}🎉 Security scanning completed!${NC}"
echo ""
echo "📄 Generated reports:"
if [[ -f "$SUMMARY_REPORT" ]]; then
    echo "   📋 Summary: $SUMMARY_REPORT"
fi
if [[ -f "${RESULTS_DIR}/audit-results.json" ]]; then
    echo "   🔍 Audit: ${RESULTS_DIR}/audit-results.json"
fi
if [[ -f "${RESULTS_DIR}/deny-results.json" ]]; then
    echo "   🚫 Policy: ${RESULTS_DIR}/deny-results.json"
fi
if [[ -f "${RESULTS_DIR}/supply-chain-report.md" ]]; then
    echo "   🔗 Supply Chain: ${RESULTS_DIR}/supply-chain-report.md"
fi
echo ""
echo "💡 Tip: Use --verbose for detailed output during scanning"