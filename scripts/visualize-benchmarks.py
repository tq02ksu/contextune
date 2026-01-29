#!/usr/bin/env python3
"""
Benchmark Visualization Tool

This script generates visualizations from Criterion benchmark results.
It creates charts showing performance trends over time.

Usage:
    python3 scripts/visualize-benchmarks.py [--output OUTPUT_DIR] [--format FORMAT]
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, List, Tuple
from dataclasses import dataclass
from datetime import datetime


@dataclass
class BenchmarkData:
    """Benchmark data point"""
    name: str
    timestamp: str
    mean: float
    std_dev: float
    median: float
    unit: str = "ns"


class BenchmarkVisualizer:
    """Visualizes benchmark results"""

    def __init__(self, output_dir: Path):
        """
        Initialize visualizer
        
        Args:
            output_dir: Directory to save visualizations
        """
        self.output_dir = output_dir
        self.output_dir.mkdir(parents=True, exist_ok=True)

    def load_criterion_results(self, criterion_dir: Path) -> Dict[str, BenchmarkData]:
        """
        Load benchmark results from Criterion output directory
        
        Args:
            criterion_dir: Path to criterion output directory
            
        Returns:
            Dictionary mapping benchmark names to data
        """
        results = {}
        timestamp = datetime.now().isoformat()
        
        if not criterion_dir.exists():
            return results
        
        # Recursively find all estimates.json files
        estimates_files = list(criterion_dir.rglob("estimates.json"))
        
        for estimates_file in estimates_files:
            try:
                with open(estimates_file, 'r') as f:
                    data = json.load(f)
                
                mean = data.get("mean", {}).get("point_estimate", 0)
                std_dev = data.get("std_dev", {}).get("point_estimate", 0)
                median = data.get("median", {}).get("point_estimate", 0)
                
                # Construct benchmark name from path
                # e.g., target/criterion/cpu_usage/multi_pass_processing/1000/base/estimates.json
                # -> cpu_usage/multi_pass_processing/1000
                rel_path = estimates_file.relative_to(criterion_dir)
                parts = list(rel_path.parts[:-2])  # Remove 'base' and 'estimates.json'
                
                if not parts:
                    continue
                
                bench_name = '/'.join(parts)
                
                # Only keep 'base' results (not 'new' or 'change')
                if 'base' in estimates_file.parts:
                    results[bench_name] = BenchmarkData(
                        name=bench_name,
                        timestamp=timestamp,
                        mean=mean,
                        std_dev=std_dev,
                        median=median
                    )
            except (json.JSONDecodeError, KeyError, ValueError) as e:
                print(f"Warning: Failed to parse {estimates_file}: {e}", file=sys.stderr)
        
        return results

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

    def generate_html_report(self, results: Dict[str, BenchmarkData]) -> str:
        """
        Generate HTML report with embedded charts
        
        Args:
            results: Benchmark results
            
        Returns:
            HTML content
        """
        # Sort benchmarks by category
        categories = {}
        for name, data in results.items():
            category = name.split('/')[0] if '/' in name else 'other'
            if category not in categories:
                categories[category] = []
            categories[category].append((name, data))
        
        # Generate HTML
        html = """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Benchmark Results - Contexture</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            line-height: 1.6;
            color: #333;
            background: #f5f5f5;
            padding: 20px;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        
        h1 {{
            color: #2c3e50;
            margin-bottom: 10px;
            font-size: 2em;
        }}
        
        .timestamp {{
            color: #7f8c8d;
            margin-bottom: 30px;
            font-size: 0.9em;
        }}
        
        .summary {{
            background: #ecf0f1;
            padding: 20px;
            border-radius: 6px;
            margin-bottom: 30px;
        }}
        
        .summary h2 {{
            color: #34495e;
            margin-bottom: 15px;
            font-size: 1.3em;
        }}
        
        .stats {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
        }}
        
        .stat {{
            background: white;
            padding: 15px;
            border-radius: 4px;
            border-left: 4px solid #3498db;
        }}
        
        .stat-label {{
            color: #7f8c8d;
            font-size: 0.85em;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }}
        
        .stat-value {{
            color: #2c3e50;
            font-size: 1.8em;
            font-weight: bold;
            margin-top: 5px;
        }}
        
        .category {{
            margin-bottom: 40px;
        }}
        
        .category h2 {{
            color: #2c3e50;
            margin-bottom: 20px;
            padding-bottom: 10px;
            border-bottom: 2px solid #3498db;
            font-size: 1.5em;
        }}
        
        .benchmark-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
            gap: 20px;
        }}
        
        .benchmark-card {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 6px;
            border: 1px solid #e1e4e8;
            transition: transform 0.2s, box-shadow 0.2s;
        }}
        
        .benchmark-card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 4px 8px rgba(0,0,0,0.1);
        }}
        
        .benchmark-name {{
            font-weight: 600;
            color: #2c3e50;
            margin-bottom: 15px;
            font-size: 1.1em;
            word-break: break-word;
        }}
        
        .benchmark-metrics {{
            display: flex;
            flex-direction: column;
            gap: 8px;
        }}
        
        .metric {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 8px;
            background: white;
            border-radius: 4px;
        }}
        
        .metric-label {{
            color: #7f8c8d;
            font-size: 0.9em;
        }}
        
        .metric-value {{
            color: #2c3e50;
            font-weight: 600;
            font-family: 'Courier New', monospace;
        }}
        
        .footer {{
            margin-top: 40px;
            padding-top: 20px;
            border-top: 1px solid #e1e4e8;
            text-align: center;
            color: #7f8c8d;
            font-size: 0.9em;
        }}
        
        @media (max-width: 768px) {{
            .container {{
                padding: 20px;
            }}
            
            .benchmark-grid {{
                grid-template-columns: 1fr;
            }}
            
            .stats {{
                grid-template-columns: 1fr;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🎵 Contexture Benchmark Results</h1>
        <div class="timestamp">Generated: {timestamp}</div>
        
        <div class="summary">
            <h2>Summary</h2>
            <div class="stats">
                <div class="stat">
                    <div class="stat-label">Total Benchmarks</div>
                    <div class="stat-value">{total_benchmarks}</div>
                </div>
                <div class="stat">
                    <div class="stat-label">Categories</div>
                    <div class="stat-value">{total_categories}</div>
                </div>
                <div class="stat">
                    <div class="stat-label">Fastest</div>
                    <div class="stat-value">{fastest_time}</div>
                </div>
                <div class="stat">
                    <div class="stat-label">Slowest</div>
                    <div class="stat-value">{slowest_time}</div>
                </div>
            </div>
        </div>
"""
        
        # Calculate summary stats
        all_times = [data.mean for data in results.values()]
        fastest_time = self.format_time(min(all_times)) if all_times else "N/A"
        slowest_time = self.format_time(max(all_times)) if all_times else "N/A"
        
        html = html.format(
            timestamp=datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            total_benchmarks=len(results),
            total_categories=len(categories),
            fastest_time=fastest_time,
            slowest_time=slowest_time
        )
        
        # Add categories
        for category, benchmarks in sorted(categories.items()):
            html += f"""
        <div class="category">
            <h2>{category.replace('_', ' ').title()}</h2>
            <div class="benchmark-grid">
"""
            
            for name, data in sorted(benchmarks):
                display_name = name.split('/')[-1] if '/' in name else name
                html += f"""
                <div class="benchmark-card">
                    <div class="benchmark-name">{display_name}</div>
                    <div class="benchmark-metrics">
                        <div class="metric">
                            <span class="metric-label">Mean</span>
                            <span class="metric-value">{self.format_time(data.mean)}</span>
                        </div>
                        <div class="metric">
                            <span class="metric-label">Median</span>
                            <span class="metric-value">{self.format_time(data.median)}</span>
                        </div>
                        <div class="metric">
                            <span class="metric-label">Std Dev</span>
                            <span class="metric-value">{self.format_time(data.std_dev)}</span>
                        </div>
                    </div>
                </div>
"""
            
            html += """
            </div>
        </div>
"""
        
        # Footer
        html += """
        <div class="footer">
            <p>Generated by Contexture Benchmark Visualizer</p>
            <p>Powered by Criterion.rs</p>
        </div>
    </div>
</body>
</html>
"""
        
        return html

    def generate_markdown_report(self, results: Dict[str, BenchmarkData]) -> str:
        """
        Generate Markdown report
        
        Args:
            results: Benchmark results
            
        Returns:
            Markdown content
        """
        # Sort benchmarks by category
        categories = {}
        for name, data in results.items():
            category = name.split('/')[0] if '/' in name else 'other'
            if category not in categories:
                categories[category] = []
            categories[category].append((name, data))
        
        md = f"""# Benchmark Results

**Generated:** {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}

## Summary

- **Total Benchmarks:** {len(results)}
- **Categories:** {len(categories)}

"""
        
        # Add categories
        for category, benchmarks in sorted(categories.items()):
            md += f"\n## {category.replace('_', ' ').title()}\n\n"
            md += "| Benchmark | Mean | Median | Std Dev |\n"
            md += "|-----------|------|--------|----------|\n"
            
            for name, data in sorted(benchmarks):
                display_name = name.split('/')[-1] if '/' in name else name
                md += f"| {display_name} | {self.format_time(data.mean)} | "
                md += f"{self.format_time(data.median)} | {self.format_time(data.std_dev)} |\n"
        
        return md

    def save_results_json(self, results: Dict[str, BenchmarkData]) -> None:
        """
        Save results as JSON for historical tracking
        
        Args:
            results: Benchmark results
        """
        output_file = self.output_dir / "latest.json"
        
        data = {
            "timestamp": datetime.now().isoformat(),
            "benchmarks": {
                name: {
                    "mean": bench.mean,
                    "median": bench.median,
                    "std_dev": bench.std_dev,
                    "unit": bench.unit
                }
                for name, bench in results.items()
            }
        }
        
        with open(output_file, 'w') as f:
            json.dump(data, f, indent=2)
        
        print(f"Saved JSON results to {output_file}")
        
        # Also append to history
        history_file = self.output_dir / "history.jsonl"
        with open(history_file, 'a') as f:
            f.write(json.dumps(data) + '\n')
        
        print(f"Appended to history: {history_file}")


def main():
    parser = argparse.ArgumentParser(
        description="Generate visualizations from Criterion benchmark results"
    )
    parser.add_argument(
        "--criterion-dir",
        type=Path,
        default=Path("target/criterion"),
        help="Path to criterion directory (default: target/criterion)"
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("target/benchmark-reports"),
        help="Output directory for reports (default: target/benchmark-reports)"
    )
    parser.add_argument(
        "--format",
        choices=["html", "markdown", "json", "all"],
        default="all",
        help="Output format (default: all)"
    )
    
    args = parser.parse_args()
    
    # Check if criterion directory exists
    if not args.criterion_dir.exists():
        print(f"Error: Criterion directory not found: {args.criterion_dir}", file=sys.stderr)
        print("Run benchmarks first: cargo bench", file=sys.stderr)
        sys.exit(1)
    
    # Create visualizer
    visualizer = BenchmarkVisualizer(args.output)
    
    # Load results
    print(f"Loading benchmark results from {args.criterion_dir}...")
    results = visualizer.load_criterion_results(args.criterion_dir)
    
    if not results:
        print("Error: No benchmark results found", file=sys.stderr)
        sys.exit(1)
    
    print(f"Loaded {len(results)} benchmarks")
    
    # Generate reports
    if args.format in ["html", "all"]:
        print("Generating HTML report...")
        html = visualizer.generate_html_report(results)
        html_file = args.output / "index.html"
        with open(html_file, 'w') as f:
            f.write(html)
        print(f"✅ HTML report: {html_file}")
    
    if args.format in ["markdown", "all"]:
        print("Generating Markdown report...")
        md = visualizer.generate_markdown_report(results)
        md_file = args.output / "report.md"
        with open(md_file, 'w') as f:
            f.write(md)
        print(f"✅ Markdown report: {md_file}")
    
    if args.format in ["json", "all"]:
        print("Saving JSON results...")
        visualizer.save_results_json(results)
        print(f"✅ JSON results: {args.output / 'latest.json'}")
    
    print(f"\n✅ All reports generated in {args.output}")


if __name__ == "__main__":
    main()
