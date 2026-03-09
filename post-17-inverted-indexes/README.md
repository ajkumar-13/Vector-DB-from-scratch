# Post #17: Inverted Indexes Explained

**Series:** Building a Vector Database from Scratch in Rust  
**Post:** 17 of 20  
**Reading Time:** ~15 minutes

## Overview

We've built a powerful HNSW vector search engine, but it has a fundamental limitation: **it's fuzzy**. Vector search finds semantically similar items but cannot enforce hard constraints like "price < $100" or "color = blue".

This post introduces **Inverted Indexes** - the data structure behind text search engines and metadata filtering. We'll learn how to combine exact keyword matching with approximate vector search to create a true hybrid search system.

## Key Concepts

| Concept | Description |
|---------|-------------|
| **Forward Index** | Document → Terms mapping (natural storage format, O(N) search) |
| **Inverted Index** | Term → Documents mapping (optimized for search, O(1) or O(log N)) |
| **Postings List** | Sorted list of document IDs containing a specific term |
| **Boolean Algebra** | Combining queries with AND/OR/NOT operations |
| **Two-Pointer Intersection** | Linear-time algorithm for merging sorted lists |
| **Pre-Filtering** | Apply filters before vector search (constrain search space) |
| **Post-Filtering** | Apply filters after vector search (simpler but less accurate) |

## The Problem

**Vector Search Limitations:**

```
Query: "running shoes under $100"

Vector Search alone:
  ✗ Finds visually similar shoes (might be $500)
  ✗ Finds hiking boots (shoe-like but wrong category)
  ✗ Cannot enforce price < $100 constraint

Hybrid Search (Vectors + Filters):
  ✓ Finds semantically similar "running shoes"
  ✓ Enforces price < $100
  ✓ Filters by category = "shoes"
```

## What We'll Build

1. **Inverted Index Implementation** - Term → DocID mapping with sorted postings lists
2. **Boolean Query Engine** - Efficient AND/OR/NOT operations on posting lists
3. **Tokenization Pipeline** - Text normalization and term extraction
4. **Set Intersection Algorithms** - Two-pointer merge for sorted lists

## Files in This Post

```
post-17-inverted-indexes/
├── README.md                      # This file
├── blog.md                        # Full explanation
├── code/
│   ├── inverted-index.rs          # Core inverted index implementation
│   └── boolean-ops.rs             # Set operations and query execution
└── diagrams/
    └── mermaid-diagrams.md        # Visual explanations
```

## Example Usage

```rust
let mut index = InvertedIndex::new();

// Index documents
index.add_document(1, "tags", vec!["blue", "shoes"]);
index.add_document(2, "tags", vec!["red", "shoes"]);
index.add_document(3, "tags", vec!["blue", "hat"]);

// Query: shoes AND blue
let shoes_docs = index.search("shoes");  // [1, 2]
let blue_docs = index.search("blue");    // [1, 3]
let result = intersect(&shoes_docs, &blue_docs);  // [1]
```

## Connection to Other Posts

- **[Post #12-15: HNSW Search](../post-15-hnsw-impl-2/blog.md)** - The vector search engine we'll enhance
- **[Post #16: Benchmarking](../post-16-benchmarking/blog.md)** - Performance comparison
- **Post #18: Filtered HNSW (Next)** - Combining inverted indexes with HNSW using bitmasks
- **Post #19: Query Planning** - Choosing between pre-filtering and post-filtering strategies

## The Hybrid Search Challenge

We now have two engines:
1. **HNSW:** Fast approximate vector search (O(log N))
2. **Inverted Index:** Exact keyword matching (O(1) lookup)

**How do we combine them?**

### Strategy 1: Post-Filtering
```
1. HNSW search → Top 100 results
2. Filter by metadata → Keep only matches
Risk: What if none of the top 100 match? Zero results!
```

### Strategy 2: Pre-Filtering (Better!)
```
1. Inverted index → All docs matching filters
2. HNSW search within that subset
Challenge: How to constrain HNSW graph traversal?
```

**Post #18** will solve this using bitmasks to guide HNSW search through only the allowed documents.

## Key Takeaways

1. **Inverted indexes invert the natural storage order** - Store by term, not by document
2. **Sorted postings lists enable fast set operations** - Intersection in O(n + m) time
3. **Boolean algebra is fundamental to search** - AND/OR/NOT compose complex queries
4. **Hybrid search requires careful coordination** - Can't just "glue together" two engines
5. **Pre-filtering is more accurate than post-filtering** - But requires deeper integration

## The Road Ahead

With inverted indexes, we can now:
- Filter by exact keywords ("blue", "shoes")
- Enforce range constraints (price < 100)
- Combine multiple conditions (color=blue AND price<100)

**Next challenge:** Modify HNSW to respect these filters during graph traversal, creating a true hybrid search engine that combines semantic understanding with logical precision.
