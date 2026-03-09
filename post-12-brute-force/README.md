# Post #12: The Brute Force Engine — Implementing Exact Nearest Neighbor Search (k-NN)

**Series:** Building a Vector Database from Scratch in Rust  
**Reading Time:** ~15 minutes  
**Phase:** Search Engine (Post 11-15)

---

## Overview

This post implements the **k-Nearest Neighbor (k-NN)** search algorithm using brute force (flat search). We scan every vector in the database to find the top-k most similar matches.

**Why start with brute force?**

1. **Baseline:** Provides ground truth (100% recall) for testing approximate algorithms
2. **Simplicity:** Easy to implement and debug
3. **Small-scale viability:** Works well for < 1M vectors

---

## Key Concepts

### The Algorithm

1. **Heap-based selection:** Use a fixed-size min-heap to track top-k results
2. **Unified scan:** Query both MemTable (in-memory) and Segments (mmap'd)
3. **Tombstone filtering:** Skip deleted vectors during search

### Complexity Analysis

| Operation | Time | Space |
|-----------|------|-------|
| Search | O(N·D) | O(k) |
| Insert | O(1) | O(D) |

Where:
- **N** = total vectors
- **D** = vector dimensions
- **k** = number of results

### Performance Reality

| Vector Count | Memory | Search Time (768D) |
|--------------|--------|-------------------|
| 10,000 | 30 MB | ~1 ms |
| 100,000 | 300 MB | ~15 ms |
| 1,000,000 | 3 GB | ~150 ms |
| 10,000,000 | 30 GB | ~1.5 sec |

**The Wall:** Linear search doesn't scale to billions of vectors → leads to HNSW (Post #13).

---

## Code Structure

```
post-12-brute-force/
├── README.md (this file)
├── blog.md (full explanation)
├── code/
│   ├── brute-force-search.rs  # Heap-based k-NN implementation
│   └── benchmark.rs            # Performance testing at scale
└── diagrams/
    └── mermaid-diagrams.md     # Heap visualization, search flow
```

---

## Key Implementation Details

### 1. Min-Heap with Reverse

Rust's `BinaryHeap` is a max-heap, so we use `std::cmp::Reverse` to get min-heap behavior:

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

let mut heap: BinaryHeap<Reverse<Candidate>> = BinaryHeap::new();

// Push candidate if better than worst in heap
if heap.len() < k {
    heap.push(Reverse(candidate));
} else if candidate.score > heap.peek().unwrap().0.score {
    heap.pop();
    heap.push(Reverse(candidate));
}
```

### 2. Candidate Ordering

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub id: String,
    pub score: f32,
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // Handle NaN by treating as -infinity
        self.score.partial_cmp(&other.score).unwrap_or(Ordering::Less)
    }
}
```

### 3. Unified Search Across Storage Layers

```rust
// Scan MemTable
for (id, vector) in &self.memtable {
    let score = cosine_similarity(query, vector);
    push_candidate(&mut heap, Candidate { id: id.clone(), score }, k);
}

// Scan Segments (mmap'd files)
for segment in &self.segments {
    for (i, vector) in segment.iter().enumerate() {
        if !self.tombstones.contains(&id) { // Check deletions
            let score = cosine_similarity(query, vector);
            push_candidate(&mut heap, Candidate { id, score }, k);
        }
    }
}
```

---

## When to Use Brute Force

**✅ Good for:**
- Development/testing (ground truth baseline)
- Small datasets (< 100k vectors)
- Perfect accuracy requirements (medical, legal)
- Low query volume applications

**❌ Not suitable for:**
- Large-scale production (> 1M vectors)
- Real-time search (< 10ms latency)
- High concurrency workloads

---

## Connection to Other Posts

- **Post #11 (Vector Math):** Uses cosine_similarity and dot_product
- **Post #7 (Mmap):** Iterates over memory-mapped segments
- **Post #9 (Recovery):** Handles tombstones for deleted vectors
- **Post #13 (HNSW):** This becomes the "verify" step for approximate search

---

## Files

- **[blog.md](blog.md)** — Full tutorial with code walkthrough
- **[code/brute-force-search.rs](code/brute-force-search.rs)** — Complete k-NN implementation with heap
- **[code/benchmark.rs](code/benchmark.rs)** — Scalability tests (10k → 10M vectors)
- **[diagrams/mermaid-diagrams.md](diagrams/mermaid-diagrams.md)** — 9 diagrams showing heap mechanics

---

## Next Steps

**Post #13:** Introduction to HNSW — The graph-based algorithm that reduces search from O(N) to O(log N).

---

*Part of the "Building a Vector Database from Scratch in Rust" series.*
