# Post #13: Introduction to HNSW — The Theory of Approximate Nearest Neighbors

**Series:** Building a Vector Database from Scratch in Rust  
**Reading Time:** ~20 minutes  
**Phase:** Search Engine (Post 11-16)

---

## Overview

This post introduces **HNSW (Hierarchical Navigable Small World)**, the algorithm that powers modern vector databases. After experiencing the O(N) wall with brute force search, we explore how graph-based approximate search achieves O(log N) performance.

**The Trade-off:**
- Brute force: 100% accuracy, 150ms for 1M vectors
- HNSW: 99%+ accuracy, 2ms for 1M vectors

---

## Key Concepts

### The Small World Phenomenon

Networks where any node can reach any other node in a small number of hops. By organizing vectors into a proximity graph, we can navigate to similar vectors without checking every node.

### Hierarchical Layers (Skip List Analogy)

HNSW adds multiple layers like highway systems:
- **Layer 0 (Ground):** All vectors, dense connections
- **Layer 1+:** Sparse subsets with long-range links
- **Top Layer:** Few entry points spanning the dataset

### Greedy Search Algorithm

1. Start at entry point (top layer)
2. Check neighbors, move to closest one
3. Drop down layers when no improvement
4. Continue at ground level until local minimum

**Complexity:** O(log N) vs O(N) for brute force

---

## Core Parameters

| Parameter | Purpose | Trade-off |
|-----------|---------|-----------|
| **M** | Max edges per node | Higher = better recall, more memory |
| **ef_construction** | Search depth during build | Higher = better graph, slower indexing |
| **ef_search** | Search depth during query | Higher = better recall, slower search |

---

## Why HNSW Wins

**Performance at scale:**

| Vector Count | Brute Force | HNSW | Speedup |
|--------------|-------------|------|---------|
| 1M | 150 ms | 2 ms | 75x |
| 10M | 1.5 sec | 3 ms | 500x |
| 100M | 15 sec | 5 ms | 3000x |

**Memory efficiency:** Graph structure adds ~20-30% overhead vs raw vectors

---

## Code Structure

```
post-13-hnsw-intro/
├── README.md (this file)
├── blog.md (full theory explanation)
├── code/
│   ├── skip-list-demo.rs      # Skip list analogy
│   └── graph-search-demo.rs   # Greedy search visualization
└── diagrams/
    └── mermaid-diagrams.md    # Layer hierarchy, search path
```

---

## The Problem HNSW Solves

**Brute Force Wall:**
- 1M vectors × 768 dimensions = ~3 billion multiply operations per query
- Linear scaling: 10x more data = 10x slower search
- Unacceptable for real-time applications

**HNSW Solution:**
- Logarithmic scaling: 10x more data = ~1.3x slower search
- Sub-millisecond latency at billion-vector scale
- Tunable accuracy/speed trade-off

---

## Connection to Other Posts

- **Post #12:** Brute force baseline (O(N) search)
- **Post #12.5:** Heap optimization for top-k
- **Post #14:** HNSW implementation (graph building)
- **Post #15:** HNSW search algorithm & tuning
- **Post #16:** Benchmarking brute vs HNSW

---

## Key Insights

1. **Approximate ≠ Bad:** 99% accuracy at 100x speed is the right trade-off
2. **Graphs > Lists:** Navigating connections beats scanning everything
3. **Hierarchy = Speed:** Multiple layers enable logarithmic search
4. **Local Minima Problem:** Solved by starting at top layer (global view)
5. **Tunable at Runtime:** Adjust ef_search per query based on needs

---

## Files

- **[blog.md](blog.md)** — Complete theory with skip list analogy, search algorithm
- **[code/skip-list-demo.rs](code/skip-list-demo.rs)** — Demonstrates hierarchical navigation
- **[code/graph-search-demo.rs](code/graph-search-demo.rs)** — Greedy search visualization
- **[diagrams/mermaid-diagrams.md](diagrams/mermaid-diagrams.md)** — 12 diagrams (layers, search path, complexity)

---

## What's Next

**Post #14:** HNSW Implementation Part 1 — Building the graph structure, layer assignment, edge connections

**Post #15:** HNSW Implementation Part 2 — Search algorithm, parameter tuning, integration with VectorStore

---

*Part of the "Building a Vector Database from Scratch in Rust" series.*
