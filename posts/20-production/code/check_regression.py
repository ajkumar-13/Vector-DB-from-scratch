#!/usr/bin/env python3
"""
Performance regression checker for CI/CD pipeline.

Compares current benchmark results to baseline and fails if
there's a significant regression (> 10% slower).
"""

import json
import sys
from pathlib import Path


def load_benchmark(path: str) -> dict:
    """Load benchmark results from Criterion JSON"""
    with open(path) as f:
        return json.load(f)


def compare_benchmarks(baseline_path: str, current_path: str, threshold: float = 0.10) -> bool:
    """
    Compare two benchmark results
    
    Args:
        baseline_path: Path to baseline benchmark
        current_path: Path to current benchmark
        threshold: Maximum allowed regression (default 10%)
    
    Returns:
        True if no regression, False if regression detected
    """
    baseline = load_benchmark(baseline_path)
    current = load_benchmark(current_path)
    
    baseline_mean = baseline['mean']['point_estimate']
    current_mean = current['mean']['point_estimate']
    
    # Calculate percentage change
    regression = (current_mean - baseline_mean) / baseline_mean * 100
    
    print(f"Baseline:  {baseline_mean:.2f}ns")
    print(f"Current:   {current_mean:.2f}ns")
    print(f"Change:    {regression:+.1f}%")
    print(f"Threshold: {threshold * 100:.0f}%")
    print()
    
    if regression > threshold * 100:
        print(f"❌ Performance regression detected: {regression:.1f}%")
        print(f"   This exceeds the {threshold * 100:.0f}% threshold.")
        return False
    elif regression > 0:
        print(f"⚠️  Performance slightly slower: {regression:.1f}%")
        print(f"   Still within {threshold * 100:.0f}% threshold.")
        return True
    else:
        print(f"✅ Performance improved: {regression:.1f}%")
        return True


def main():
    if len(sys.argv) < 3:
        print("Usage: check_regression.py <baseline.json> <current.json> [threshold]")
        sys.exit(1)
    
    baseline_path = sys.argv[1]
    current_path = sys.argv[2]
    threshold = float(sys.argv[3]) if len(sys.argv) > 3 else 0.10
    
    # Check if files exist
    if not Path(baseline_path).exists():
        print(f"Error: Baseline file not found: {baseline_path}")
        sys.exit(1)
    
    if not Path(current_path).exists():
        print(f"Error: Current benchmark file not found: {current_path}")
        sys.exit(1)
    
    # Compare benchmarks
    passed = compare_benchmarks(baseline_path, current_path, threshold)
    
    sys.exit(0 if passed else 1)


if __name__ == "__main__":
    main()
