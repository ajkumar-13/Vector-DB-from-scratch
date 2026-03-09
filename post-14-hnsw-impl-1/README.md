# Post #14: Implementing HNSW Part 1 — Building the Graph Structure

**Reading Time:** ~20 minutes

## Overview

This post transitions from HNSW theory (Post #13) to implementation. We build the in-memory graph structure from scratch using Rust's Arena Pattern, implement greedy search traversal, and handle the complex neighbor selection heuristics that make HNSW navigable.

## What You'll Learn

- **Arena Pattern**: Store graph nodes in a `Vec<Node>` instead of pointers
- **Layered Graph Representation**: How to encode hierarchical connections
- **Greedy Search Algorithm**: Traversing a single layer efficiently
- **Probabilistic Layer Assignment**: Exponential decay for natural pyramid structure
- **Neighbor Selection Heuristic**: The diversity algorithm that prevents graph degeneration
- **Bidirectional Edge Management**: Insert, link, and prune connections
- **Edge Pruning Logic**: The hardest part of HNSW - maintaining M connections while preserving navigability

## Key Concepts

| Concept | Description | Complexity |
|---------|-------------|------------|
| Arena Pattern | Store nodes in `Vec`, use `usize` indices as "pointers" | O(1) access |
| Greedy Search | Descend from entry point to nearest neighbor at a layer | O(M × hops) |
| Beam Search | Maintain `ef` candidates to avoid local minima | O(ef × M × hops) |
| Layer Assignment | `-ln(random()) × ml` creates exponential decay | O(1) |
| Neighbor Selection | Diversity heuristic prunes redundant connections | O(M²) per node |
| Edge Pruning | Keep best M connections when node exceeds limit | O(M log M) |

## Connection to Other Posts

- **Post #13**: Theory foundation (skip lists, small worlds, O(log N))
- **Post #12**: Brute force baseline we're replacing
- **Post #15**: Part 2 will add search, tuning, and benchmarks
- **Post #16**: Persistence and serialization to disk

## Files in This Post

```
post-14-hnsw-impl-1/
├── README.md (this file)
├── blog.md (full content)
├── code/
│   ├── hnsw-basic.rs (core data structures and insertion)
│   └── neighbor-selection.rs (diversity heuristic deep-dive)
└── diagrams/
    └── mermaid-diagrams.md (12 diagrams)
```

## Performance Expectations

After this post, you'll have a working HNSW index that can:
- Insert vectors into a hierarchical graph
- Maintain O(log N) navigability through diversity heuristics
- Support variable layer heights (typically 0-5 layers for 1M vectors)

**Build time:** O(N × log N × M) for N vectors
**Memory:** O(N × M × D) where D = dimensions

## What's NOT Covered (Yet)

- Search algorithm (Post #15)
- Parameter tuning (Post #15)
- Disk persistence (Post #16)
- Concurrent insertion (Post #16)
- Production optimizations (Post #20)

---

**Next:** Read [blog.md](blog.md) for the full implementation walkthrough.
