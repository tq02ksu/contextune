#!/usr/bin/env python3
"""
Benchmark Analysis and Regression Detection

This script analyzes Criterion benchmark results and detects performance regressions.
It parses the JSON output from Criterion and provides detailed comparison reports.

Usage:
    python3 scripts/analyze-benchmarks.py [--baseline BASELINE_DIR] [--current CURRENT_DIR] [--threshold PERCENT]
"""

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
from enum import Enum


class ChangeType(Enum):
    """Type of performance change"""
    REGRESSION = "regression"
    IMPROVEMENT = "improvement"
    STABLE = "stable"
    NEW = "new"
    MISSING = "missing"


@dataclass
class BenchmarkResult:
    """Benchmark result data"""
    name: str
    mean: float
    std_dev: float
    median: float
    unit: str = "ns"


@dataclass
class Comparison:
    """Comparison between baseline and current"""
    name: str
    baseline: Optional[BenchmarkResult]
    current: Optional[BenchmarkResult]
    change_percent: float
    change_type: ChangeType


class BenchmarkAnalyzer:
    """Analyzes benchmark results and detects regressions"""

    def __init__(self, threshold: float = 5.0):
        """
        Initialize analyzer
        
        Args:
            threshold: Regression threshold percentage (default: 5%)
        """
        self.threshold = threshold

    def load_criterion_results(self, criterion_dir: Path) -> Dict[str, BenchmarkResult]:
        """
        Load benchmark results from Criterion output directory
        
        Args:
            criterion_dir: Path to criterion output directory
            
        Returns:
            Dictionary mapping benchmark names to results
        """
        results = {}
        
        if not criterion_dir.exists():
            return results
        
        # Iterate through benchmark directories
        for bench_dir in criterion_dir.iterdir():
            if not bench_dir.is_dir():
                continue
            
            # Look for estimates.json
            estimates_file = bench_dir / "base" / "estimates.json"
            if not estimates_file.exists():
                # Try without base subdirectory
                estimates_file = bench_dir / "estimates.json"
            
            if estimates_file.exists():
                try:
                    with open(estimates_file, 'r') as f:
                        data = json.load(f)
                    
                    # Extract mean and std_dev
                    mean = data.get("mean", {}).get("point_estimate", 0)
                    std_dev = data.get("std_dev", {}).get("point_estimate", 0)
                    median = data.get("median", {}).get("point_estimate", 0)
                    
                    results[bench_dir.name] = BenchmarkResult(
                        name=bench_dir.name,
                        mean=mean,
                        std_dev=std_dev,
                        median=median
                    )
                except (json.JSONDecodeError, KeyError) as e:
                    print(f"Warning: Failed to parse {estimates_file}: {e}", file=sys.stderr)
        
        return results

    def compare_results(
        self,
        baseline: Dict[str, BenchmarkResult],
        current: Dict[str, BenchmarkResult]
    ) -> List[Comparison]:
        """
        Compare baseline and current results
        
        Args:
            baseline: Baseline benchmark results
            current: Current benchmark results
            
        Returns:
            List of comparisons
        """
        comparisons = []
        all_benchmarks = set(baseline.keys()) | set(current.keys())
        
        for name in sorted(all_benchmarks):
            baseline_result = baseline.get(name)
            current_result = current.get(name)
            
            if baseline_result and current_result:
                # Calculate percentage change
                change = ((current_result.mean - baseline_result.mean) / baseline_result.mean) * 100
                
                # Determine change type
                if abs(change) < self.threshold:
                    change_type = ChangeType.STABLE
                elif change > 0:
                    change_type = ChangeType.REGRESSION
                else:
                    change_type = ChangeType.IMPROVEMENT
                
                comparisons.append(Comparison(
                    name=name,
                    baseline=baseline_result,
                    current=current_result,
                    change_percent=change,
                    change_type=change_type
                ))
            elif current_result:
                # New benchmark
                comparisons.append(Comparison(
                    name=name,
                    baseline=None,
                    current=current_result,
                    change_percent=0.0,
                    change_type=ChangeType.NEW
                ))
            else:
                # Missing benchmark
                comparisons.append(Comparison(
                    name=name,
                    baseline=baseline_result,
                    current=None,
                    change_percent=0.0,
                    change_type=ChangeType.MISSING
                ))
        
        return comparisons

    def format_time(self, nanoseconds: float) -> str:
        """Format time in appropriate unit"""
        if nanoseconds < 1000:
            return f"{nanoseconds:.2f} ns"
        elif nanoseconds < 1_000_000:
            return f"{nanoseconds / 1000:.2f} µs"
        elif nanoseconds < 1_000_000_000:
            return f"{nanoseconds / 1_000_000:.2f} ms"
        else:
            return f"{nanoseconds / 1_000_000_000:.2f} s"

    def print_report(self, comparisons: List[Comparison]) -> bool:
        """
        Print comparison report
        
        Args:
            comparisons: List of comparisons
            
        Returns:
            True if regressions detected, False otherwise
        """
        # Categorize comparisons
        regressions = [c for c in comparisons if c.change_type == ChangeType.REGRESSION]
        improvements = [c for c in comparisons if c.change_type == ChangeType.IMPROVEMENT]
        stable = [c for c in comparisons if c.change_type == ChangeType.STABLE]
        new = [c for c in comparisons if c.change_type == ChangeType.NEW]
        missing = [c for c in comparisons if c.change_type == ChangeType.MISSING]
        
        print("\n" + "=" * 80)
        print("BENCHMARK REGRESSION ANALYSIS")
        print("=" * 80)
        
        # Summary
        print(f"\nSummary:")
        print(f"  Total benchmarks:  {len(comparisons)}")
        print(f"  Regressions:       {len(regressions)} ❌")
        print(f"  Improvements:      {len(improvements)} ✅")
        print(f"  Stable:            {len(stable)} ➖")
        print(f"  New:               {len(new)} 🆕")
        print(f"  Missing:           {len(missing)} ⚠️")
        print(f"  Threshold:         ±{self.threshold}%")
        
        # Regressions
        if regressions:
            print("\n" + "-" * 80)
            print("REGRESSIONS (slower performance):")
            print("-" * 80)
            for comp in sorted(regressions, key=lambda c: c.change_percent, reverse=True):
                baseline_time = self.format_time(comp.baseline.mean)
                current_time = self.format_time(comp.current.mean)
                print(f"  ❌ {comp.name}")
                print(f"     Baseline: {baseline_time}")
                print(f"     Current:  {current_time}")
                print(f"     Change:   +{comp.change_percent:.2f}% (slower)")
                print()
        
        # Improvements
        if improvements:
            print("\n" + "-" * 80)
            print("IMPROVEMENTS (faster performance):")
            print("-" * 80)
            for comp in sorted(improvements, key=lambda c: c.change_percent):
                baseline_time = self.format_time(comp.baseline.mean)
                current_time = self.format_time(comp.current.mean)
                print(f"  ✅ {comp.name}")
                print(f"     Baseline: {baseline_time}")
                print(f"     Current:  {current_time}")
                print(f"     Change:   {comp.change_percent:.2f}% (faster)")
                print()
        
        # New benchmarks
        if new:
            print("\n" + "-" * 80)
            print("NEW BENCHMARKS:")
            print("-" * 80)
            for comp in new:
                current_time = self.format_time(comp.current.mean)
                print(f"  🆕 {comp.name}: {current_time}")
        
        # Missing benchmarks
        if missing:
            print("\n" + "-" * 80)
            print("MISSING BENCHMARKS:")
            print("-" * 80)
            for comp in missing:
                baseline_time = self.format_time(comp.baseline.mean)
                print(f"  ⚠️  {comp.name}: {baseline_time} (baseline)")
        
        print("\n" + "=" * 80)
        
        return len(regressions) > 0


def main():
    parser = argparse.ArgumentParser(
        description="Analyze Criterion benchmark results and detect regressions"
    )
    parser.add_argument(
        "--baseline",
        type=Path,
        default=Path("target/criterion-baseline"),
        help="Path to baseline criterion directory"
    )
    parser.add_argument(
        "--current",
        type=Path,
        default=Path("target/criterion"),
        help="Path to current criterion directory"
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=5.0,
        help="Regression threshold percentage (default: 5.0)"
    )
    
    args = parser.parse_args()
    
    # Check if directories exist
    if not args.baseline.exists():
        print(f"Error: Baseline directory not found: {args.baseline}", file=sys.stderr)
        print("Run './scripts/benchmark-regression.sh baseline' first", file=sys.stderr)
        sys.exit(1)
    
    if not args.current.exists():
        print(f"Error: Current directory not found: {args.current}", file=sys.stderr)
        print("Run benchmarks first: cargo bench", file=sys.stderr)
        sys.exit(1)
    
    # Analyze benchmarks
    analyzer = BenchmarkAnalyzer(threshold=args.threshold)
    
    print("Loading baseline results...")
    baseline = analyzer.load_criterion_results(args.baseline)
    print(f"Loaded {len(baseline)} baseline benchmarks")
    
    print("Loading current results...")
    current = analyzer.load_criterion_results(args.current)
    print(f"Loaded {len(current)} current benchmarks")
    
    if not baseline and not current:
        print("Error: No benchmark results found", file=sys.stderr)
        sys.exit(1)
    
    # Compare results
    comparisons = analyzer.compare_results(baseline, current)
    
    # Print report
    has_regressions = analyzer.print_report(comparisons)
    
    # Exit with error if regressions detected
    if has_regressions:
        sys.exit(1)
    else:
        print("\n✅ No performance regressions detected!")
        sys.exit(0)


if __name__ == "__main__":
    main()
