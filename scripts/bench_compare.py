#!/usr/bin/env python3
"""Benchmark regression detector for Vidra CI.

Compares current benchmark output against a stored baseline.
Fails if any benchmark regresses by more than the threshold.
"""

import sys
import re

REGRESSION_THRESHOLD = 0.10  # 10% regression triggers failure


def parse_bencher(lines):
    """Parse Rust bencher-format output into a dict of name -> ns/iter."""
    results = {}
    for line in lines:
        m = re.match(r'^test\s+(\S+)\s+.*\s+bench:\s+([\d,]+)\s+ns/iter', line)
        if m:
            name = m.group(1)
            ns = int(m.group(2).replace(',', ''))
            results[name] = ns
    return results


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <baseline.txt> <current.txt>")
        sys.exit(1)

    with open(sys.argv[1]) as f:
        baseline = parse_bencher(f.readlines())
    with open(sys.argv[2]) as f:
        current = parse_bencher(f.readlines())

    if not baseline:
        print("⚠️  No baseline benchmarks found. Skipping comparison.")
        sys.exit(0)

    regressions = []
    print("📊 Benchmark Comparison")
    print(f"{'Benchmark':<50} {'Baseline':>12} {'Current':>12} {'Delta':>10}")
    print("-" * 86)

    for name, base_ns in sorted(baseline.items()):
        if name not in current:
            print(f"  {name:<48} {base_ns:>12,} {'MISSING':>12} {'—':>10}")
            continue

        curr_ns = current[name]
        delta = (curr_ns - base_ns) / base_ns if base_ns > 0 else 0
        marker = ""
        if delta > REGRESSION_THRESHOLD:
            marker = " ❌ REGRESSION"
            regressions.append((name, delta))
        elif delta < -0.05:
            marker = " 🚀 faster"

        print(f"  {name:<48} {base_ns:>12,} {curr_ns:>12,} {delta:>+9.1%}{marker}")

    # Report new benchmarks
    for name in sorted(set(current.keys()) - set(baseline.keys())):
        print(f"  {name:<48} {'NEW':>12} {current[name]:>12,}")

    print()
    if regressions:
        print(f"❌ {len(regressions)} benchmark(s) regressed by >{REGRESSION_THRESHOLD*100:.0f}%:")
        for name, delta in regressions:
            print(f"   - {name}: {delta:+.1%}")
        sys.exit(1)
    else:
        print("✅ No benchmark regressions detected.")


if __name__ == "__main__":
    main()
