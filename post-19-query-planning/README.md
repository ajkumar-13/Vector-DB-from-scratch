# Post #19: Query Planning - Designing an Optimizer for Hybrid Search

## Overview

In [Post #18](../post-18-tantivy-hybrid/blog.md), we integrated Tantivy and implemented hybrid search with pre-filtering. But there's a critical problem: **not all queries benefit from the same execution strategy**.

This post teaches you how to build a **Cost-Based Optimizer (CBO)** that analyzes query characteristics and automatically chooses the fastest execution path.

## What You'll Learn

1. **The "It Depends" Problem** - Why one strategy doesn't fit all queries
2. **Selectivity Estimation** - Fast counting with Tantivy
3. **Three Execution Paths** - Vector-first, Filter-first, Brute force
4. **Cost Model Design** - Mathematical foundations for query planning
5. **The Oversampling Problem** - Statistical challenges in post-filtering
6. **Adaptive Query Execution** - Dynamic strategy selection

## Key Concepts

| Concept | Description |
|---------|-------------|
| **Selectivity (s)** | Percentage of documents matching a filter (0.0 to 1.0) |
| **Pre-Filtering** | Apply filter first, then search constrained HNSW |
| **Post-Filtering** | Search HNSW first, then filter results |
| **Brute Force Scan** | Skip HNSW, scan only filtered documents |
| **k-Expansion** | Oversample HNSW results to account for filtering |
| **Cost Model** | Mathematical formula estimating query execution time |

## The Problem

```rust
// Query 1: Broad filter (95% match)
"category:shoes AND in_stock:true"  // Pre-filtering adds overhead!

// Query 2: Narrow filter (0.1% match)  
"product_id:ABC123"  // HNSW graph becomes disconnected!

// Query 3: Medium filter (10% match)
"price:<100 AND brand:Nike"  // Pre-filtering is perfect!
```

**Challenge:** How do we automatically choose the right strategy?

## The Three Strategies

### Strategy A: Vector-First (Post-Filtering)

```
1. Search HNSW → Get top-K*2 results
2. Filter results → Keep matches
3. Return top-K

Best for: Selectivity > 50%
Risk: May filter out all results (recall = 0)
```

### Strategy B: Filter-First (Pre-Filtering)

```
1. Run Tantivy → Get bitmask
2. Search HNSW with bitmask → Get top-K
3. Return results

Best for: Selectivity 1-50%
Risk: Disconnected graph at low selectivity
```

### Strategy C: Brute Force

```
1. Run Tantivy → Get matching IDs
2. Compute distances for ALL matches
3. Sort and return top-K

Best for: Selectivity < 1%
Risk: Slow if too many matches
```

## The Cost Model

**Selectivity (s):** Fraction of documents matching filter

$$s = \frac{\text{matching\_docs}}{\text{total\_docs}}$$

**Cost Functions:**

| Strategy | Cost Formula | Best When |
|----------|--------------|-----------|
| **Brute Force** | $C_{brute} = s \cdot N \cdot C_{dist}$ | $s < 0.01$ |
| **Filter-First** | $C_{filter} = C_{tantivy} + C_{hnsw} \cdot f(s)$ | $0.01 \leq s \leq 0.5$ |
| **Vector-First** | $C_{vector} = C_{hnsw} + k' \cdot C_{check}$ | $s > 0.5$ |

Where:
- $N$ = total documents
- $C_{dist}$ = cost of distance computation
- $f(s)$ = connectivity factor (increases as $s$ decreases)
- $k'$ = expanded k for oversampling

## The Optimizer Algorithm

```rust
pub fn plan(filter_query: &str) -> ExecutionPlan {
    let s = estimate_selectivity(filter_query);
    
    if s < 0.01 {
        return ExecutionPlan::BruteForce;
    } else if s > 0.5 {
        let k_expansion = (1.0 / s * 1.5).ceil() as usize;
        return ExecutionPlan::VectorFirst { k_expansion };
    } else {
        return ExecutionPlan::FilterFirst;
    }
}
```

## Performance Results

**Benchmark: 1M vectors, varying selectivity**

| Selectivity | Strategy | Latency | Notes |
|-------------|----------|---------|-------|
| 0.1% | **Brute Force** | 0.5ms | Only 1K docs to scan |
| 1.0% | Filter-First | 3.2ms | HNSW still connected |
| 5.0% | Filter-First | 3.0ms | Optimal balance |
| 50.0% | **Vector-First** | 2.0ms | Post-filter efficient |
| 95.0% | **Vector-First** | 1.8ms | Minimal filtering |

**Key Insight:** Wrong strategy can be 100× slower!

## Files in This Post

- **[blog.md](blog.md)**: Full 15-minute explanation
- **[code/query-planner.rs](code/query-planner.rs)**: Cost-based optimizer implementation
- **[code/execution-engine.rs](code/execution-engine.rs)**: Strategy execution logic
- **[code/benchmarks.rs](code/benchmarks.rs)**: Performance comparison suite
- **[diagrams/mermaid-diagrams.md](diagrams/mermaid-diagrams.md)**: Visual decision trees

## Connection to Other Posts

**Builds on:**
- [Post #17: Inverted Indexes](../post-17-inverted-indexes/blog.md) - Filter theory
- [Post #18: Tantivy Integration](../post-18-tantivy-hybrid/blog.md) - Hybrid search implementation

**Leads to:**
- Post #20: Production Hardening - Final optimizations and deployment

## Quick Example

```rust
let planner = QueryPlanner::new(0.01, 0.5);  // Thresholds

// Query 1: Rare product
let plan1 = planner.plan(&index, "product_id:XYZ789");
// → BruteForce { matches: [42] }

// Query 2: Common category
let plan2 = planner.plan(&index, "category:shoes");
// → VectorFirst { k_expansion: 20 }

// Query 3: Medium filter
let plan3 = planner.plan(&index, "price:<100 AND brand:Nike");
// → FilterFirst { bitmask: [true, false, true, ...] }

// Execute optimized plan
let results = execute(&plan, &query_vector);
```

## The Oversampling Problem

When using Vector-First strategy, we need to oversample:

$$k' = k \cdot \frac{1}{s} \cdot \text{safety\_factor}$$

**Example:**
- User wants k=10 results
- Filter matches s=20% of docs
- Expected matches: $10 \cdot 0.2 = 2$ (not enough!)
- Need to fetch: $k' = 10 / 0.2 \cdot 1.5 = 75$ candidates

**Problem:** If $k'$ becomes too large (e.g., > 1000), switch to Filter-First instead.

## Key Takeaways

1. **No One-Size-Fits-All** - Query strategy must adapt to data characteristics
2. **Selectivity is King** - This one metric determines optimal strategy
3. **Fast Estimation** - Tantivy's count query is O(1) with term dictionary
4. **Safety Margins** - Oversampling ensures we get enough results
5. **Automatic Optimization** - Users don't need to know which strategy to use

## Production Considerations

**Before deploying:**
- [ ] Tune selectivity thresholds (0.01 and 0.5) for your workload
- [ ] Add query result caching (same filter → same bitmask)
- [ ] Monitor strategy distribution in production
- [ ] Add fallback logic (if HNSW fails, try brute force)
- [ ] Profile oversampling safety factor (1.5× vs 2.0×)
- [ ] Implement query stats collection for analysis

## Advanced Topics

### Dynamic Threshold Tuning

Learn from query history to adjust thresholds:

```rust
let optimal_threshold = analyze_query_log(last_1000_queries);
planner.update_thresholds(optimal_threshold);
```

### Multi-Filter Queries

Handle complex filters with multiple selectivities:

```rust
// (filter1 AND filter2) OR filter3
// Need to estimate combined selectivity
let s_combined = estimate_compound_selectivity(ast);
```

### Cost Model Refinement

Measure actual costs and update model:

```rust
let actual_latency = execute_and_measure(&plan);
cost_model.update(s, strategy, actual_latency);
```

## Next Steps

After completing this post, you'll have an intelligent query optimizer. The final post covers:
- **Post #20**: Production hardening (quantization, compression, deployment)

You now have a complete, production-ready hybrid search engine with automatic query optimization!
