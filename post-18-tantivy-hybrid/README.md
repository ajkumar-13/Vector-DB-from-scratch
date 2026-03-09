# Post #18: The Hybrid Engine - Integrating Tantivy for High-Speed Metadata Filtering

## Overview

In [Post #17](../post-17-inverted-indexes/blog.md), we built a simple inverted index from scratch to understand the fundamentals of keyword filtering. Now we level up by integrating **Tantivy**, a production-grade full-text search engine library written in Rust.

This post demonstrates how to build a **hybrid search engine** that combines:
- **Vector search** (HNSW) for semantic similarity
- **Full-text search** (Tantivy) for exact metadata filtering

## What You'll Learn

1. **Why use Tantivy** instead of building everything from scratch
2. **The Sidecar Pattern** for running dual indexes side-by-side
3. **ID Alignment** strategies between HNSW and Tantivy
4. **Bitmask generation** for O(1) filter checks during graph traversal
5. **Filtered HNSW search** that respects metadata constraints
6. **Performance characteristics** of pre-filtering vs post-filtering

## Key Concepts

| Concept | Description |
|---------|-------------|
| **Tantivy** | Rust-native full-text search library (like Lucene) |
| **Schema** | Defines indexed fields (TEXT, FAST, STORED) |
| **Bitmask** | Vec<bool> for O(1) membership testing during HNSW traversal |
| **Pre-filtering** | Apply metadata filters before vector search |
| **Post-filtering** | Filter results after vector search (lossy) |
| **ID Alignment** | Keeping HNSW point IDs synchronized with Tantivy documents |

## The Problem

Pure vector search can't enforce hard constraints:

```rust
// Query: "Find running shoes under $100"
// Problem: HNSW returns semantically similar items regardless of price
// Result: [Nike Air Max ($120), Adidas Boost ($180), ...]
// Only 1 out of 10 results might be under $100!
```

**Solution:** Use Tantivy to pre-filter documents, then search HNSW within that subset.

## The Architecture

```
User Query: vector + "price < 100 AND category = shoes"
    ↓
1. Tantivy: Parse query → Get matching doc IDs → [5, 12, 23, 41...]
    ↓
2. Convert to Bitmask: bitmask[5]=true, bitmask[12]=true...
    ↓
3. HNSW: Search constrained to bitmask (skip non-matching nodes)
    ↓
4. Results: Top-K vectors that also match filters
```

## Files in This Post

- **[blog.md](blog.md)**: Full 20-minute explanation
- **[code/tantivy-integration.rs](code/tantivy-integration.rs)**: MetadataIndex wrapper with schema setup
- **[code/hybrid-search.rs](code/hybrid-search.rs)**: Complete hybrid search implementation
- **[diagrams/mermaid-diagrams.md](diagrams/mermaid-diagrams.md)**: Visual explanations

## Connection to Other Posts

**Builds on:**
- [Post #15: HNSW Implementation](../post-15-hnsw-impl-2/blog.md) - Modifies search to accept filters
- [Post #17: Inverted Indexes](../post-17-inverted-indexes/blog.md) - Theory behind metadata filtering

**Leads to:**
- Post #19: Query Planning and Optimization
- Post #20: Production Hardening

## Quick Example

```rust
use tantivy::schema::*;
use tantivy::Index;

// Setup Tantivy schema
let mut schema_builder = Schema::builder();
schema_builder.add_u64_field("point_id", INDEXED | FAST | STORED);
schema_builder.add_json_field("metadata", TEXT | STORED);

// Index a document
let mut doc = Document::default();
doc.add_u64(point_id_field, 42);
doc.add_json_object(metadata_field, json!({
    "category": "shoes",
    "price": 89.99,
    "brand": "Nike"
}));

// Query: "Find Nike shoes under $100"
let query = "category:shoes AND brand:Nike AND price:<100";
let bitmask = metadata_index.search_to_bitmask(query);

// Hybrid search
let results = hnsw.search_with_filter(&query_vector, k=10, bitmask);
```

## Why Tantivy?

Building a production full-text search engine requires:

| Feature | Why It's Hard | Tantivy Solution |
|---------|---------------|------------------|
| **BM25 Scoring** | Complex TF-IDF calculations | Built-in |
| **Tokenization** | Unicode, stemming, stop words | Multiple languages |
| **Compression** | Roaring bitmaps, SIMD codecs | Optimized |
| **Range Queries** | Efficient numeric filtering | Fast fields |
| **Boolean Logic** | Complex AND/OR/NOT combinations | Query parser |

Instead of spending months rebuilding Elasticsearch, we integrate Tantivy in ~200 lines of code.

## Performance Characteristics

**Benchmark: 1M vectors, filter matches 10% of data**

| Approach | Latency | Recall | Notes |
|----------|---------|--------|-------|
| Post-filtering | 2ms | 25% | Fast but lossy |
| Brute force + filter | 150ms | 100% | Perfect but slow |
| **Hybrid (pre-filter)** | **3ms** | **100%** | Best of both |

**Breakdown:**
- Tantivy query: ~0.5ms
- Bitmask generation: ~0.5ms
- HNSW search (filtered): ~2ms

## Key Takeaways

1. **Don't reinvent wheels** - Tantivy is battle-tested and fast
2. **Bitmasks are the bridge** - O(1) lookup during graph traversal
3. **ID alignment is critical** - Keep HNSW and Tantivy in sync
4. **Pre-filtering wins** - No false negatives, minimal overhead
5. **Graph connectivity matters** - Very restrictive filters can disconnect the graph

## Next Steps

After completing this post, you'll have a hybrid search engine. Next challenge:
- **Post #19**: Query planning (when to filter-first vs vector-first?)
- **Post #20**: Production hardening (updates, deletions, persistence)
