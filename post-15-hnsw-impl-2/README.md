# Post #15: Implementing HNSW Part 2 — Search Algorithm and Parameter Tuning

**Reading Time:** ~20 minutes

## Overview

This post completes the HNSW implementation by adding the search algorithm. We implement the two-phase zoom-in process (greedy descent + beam search), tune the critical hyperparameters (M, ef_construction, ef_search), and benchmark recall vs latency trade-offs to prove HNSW beats brute force by orders of magnitude.

## What You'll Learn

- **Two-Phase Search Algorithm**: Greedy descent through layers + beam search at ground level
- **Hyperparameter Tuning**: How M, ef_construction, and ef_search affect quality and speed
- **Recall Measurement**: Comparing HNSW results against brute force ground truth
- **Performance Benchmarking**: Recall@K curves and latency analysis
- **Runtime Tuning**: Adjusting ef_search per-query for speed/accuracy trade-offs
- **Search Visualization**: Understanding which nodes are visited during traversal

## Key Concepts

| Concept | Description | Impact |
|---------|-------------|--------|
| Greedy Descent | Single-candidate search from top to Layer 1 | Fast navigation to neighborhood |
| Beam Search | Multi-candidate search at Layer 0 | Prevents local minima, improves recall |
| Recall@K | % overlap with brute force top-K | Measures approximation quality |
| ef_search | Beam width during query | Tunable at runtime! |
| M | Max connections per node | Set at build time, affects graph quality |
| ef_construction | Beam width during insertion | Affects build time and graph quality |

## Performance Results

After this post, you'll achieve:

| Dataset | HNSW (ef=100) | Brute Force | Speedup | Recall |
|---------|---------------|-------------|---------|--------|
| 10K vectors | 0.15ms | 1.5ms | 10x | 99% |
| 100K vectors | 0.5ms | 15ms | 30x | 99% |
| 1M vectors | 2.0ms | 150ms | 75x | 98% |

## Connection to Other Posts

- **Post #12**: Brute force baseline for ground truth comparison
- **Post #13**: Theory foundation (skip lists, beam search concept)
- **Post #14**: Graph construction (we built the index to search)
- **Post #16**: Benchmarking at scale (comprehensive performance analysis)

## Files in This Post

```
post-15-hnsw-impl-2/
├── README.md (this file)
├── blog.md (full content)
├── code/
│   ├── search-impl.rs (complete search algorithm)
│   └── benchmark-recall.rs (recall measurement and tuning)
└── diagrams/
    └── mermaid-diagrams.md (10 diagrams)
```

## What's NOT Covered (Yet)

- Disk persistence (Post #16)
- Concurrent search (Post #16)
- Production optimizations (Post #20)
- Quantization for memory savings (Post #20)

---

**Next:** Read [blog.md](blog.md) for the full implementation walkthrough.
