# Post #16: Benchmarking the Search Engine

**Series:** Building a Vector Database from Scratch in Rust  
**Post:** 16 of 20  
**Reading Time:** ~15 minutes

## Overview

After building both Brute Force (Post #12) and HNSW (Posts #13-15) search engines, it's time to answer the critical question: **How much faster is HNSW, really?**

This post conducts a rigorous head-to-head comparison across three dataset sizes (10K, 100K, 1M vectors) measuring latency, recall, throughput, build time, and memory usage.

## Key Concepts

| Concept | Description |
|---------|-------------|
| **The Tipping Point** | Around 50-100K vectors, where HNSW becomes essential (Brute Force hits 15-20ms latency) |
| **Pareto Frontier** | The recall vs latency trade-off curve - shows diminishing returns beyond 99% recall |
| **P95/P99 Latency** | Tail latency metrics (95th/99th percentile) - critical for SLA guarantees |
| **Build Time Tax** | HNSW's indexing cost: 2-5 minutes for 1M vectors vs 0s for Brute Force |
| **Memory Overhead** | HNSW uses 30-40% more RAM than vectors alone due to graph structure |

## Benchmark Results Summary

### Small Scale (10K vectors, 768 dims)
- **Brute Force:** 1.5ms search, 0s build, 100% recall
- **HNSW:** 0.3ms search, 0.5s build, 99.9% recall
- **Verdict:** Brute Force simpler, HNSW barely faster

### Medium Scale (100K vectors, 768 dims)
- **Brute Force:** 15ms search (becoming problematic)
- **HNSW:** 0.8ms search, **19x faster**
- **Verdict:** The tipping point - HNSW now essential

### Large Scale (1M vectors, 768 dims)
- **Brute Force:** 150ms search, ~7 QPS
- **HNSW:** 2ms search, ~500 QPS, **75x faster**
- **Verdict:** Brute Force unusable, HNSW dominant

## Files in This Post

```
post-16-benchmarking/
├── README.md                      # This file
├── blog.md                        # Full explanation
├── code/
│   ├── benchmark-harness.rs       # Reusable benchmarking framework
│   └── comparison-suite.rs        # Complete Brute Force vs HNSW comparison
└── diagrams/
    └── mermaid-diagrams.md        # Performance visualizations
```

## Key Takeaways

1. **Use Brute Force for < 50K vectors** - simpler, zero overhead, accurate
2. **Use HNSW for > 100K vectors** - exponential speedup at scale
3. **There is no free lunch** - HNSW costs build time (minutes) and memory (+30-40%)
4. **98-99% recall is often "good enough"** - last 1% costs 2-10x more latency
5. **P99 latency matters** - average is misleading for production SLAs

## Connection to Other Posts

- **[Post #12: Brute Force Search](../post-12-brute-force/blog.md)** - The baseline implementation
- **[Post #15: HNSW Search Algorithm](../post-15-hnsw-impl-2/blog.md)** - The fast approximate search
- **Post #17: Persistence (Next)** - Saving HNSW graphs to disk to avoid rebuild cost

## The Problem We Discover

After benchmarking, we realize HNSW has a critical flaw: **It lives entirely in RAM.**

If the server restarts with 1M vectors:
- Brute Force: Loads instantly via mmap
- HNSW: Must rebuild the entire graph (2-5 minutes!)

This is unacceptable for production. **Post #17** will solve this by implementing HNSW serialization to disk.
