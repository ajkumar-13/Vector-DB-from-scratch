# Post #11: Vector Math for Developers — Linear Algebra Basics

> **Series:** Building a Vector Database from Scratch in Rust  
> **Reading Time:** ~15 minutes  
> **Difficulty:** Beginner-Intermediate  

---

## 📁 Folder Contents

| File | Purpose |
|------|---------|
| [blog.md](blog.md) | Main post: vectors, dot product, cosine similarity |
| [code/vector-math.rs](code/vector-math.rs) | Core math implementations with tests |
| [code/similarity-demo.rs](code/similarity-demo.rs) | Interactive demo comparing metrics |
| [diagrams/mermaid-diagrams.md](diagrams/mermaid-diagrams.md) | Visual guides for vector operations |

---

## 🎯 What You'll Learn

1. **What is a Vector?** — Direction + Magnitude, not just `Vec<f32>`
2. **Magnitude (L2 Norm)** — The length of an arrow
3. **Dot Product** — The foundation of deep learning
4. **Cosine Similarity** — Why it's the gold standard for semantic search
5. **Euclidean Distance** — When to use it (and when not to)
6. **Normalization** — The optimization trick that makes search faster

---

## 🧮 The Core Formulas

### Magnitude (L2 Norm)

$$\|v\| = \sqrt{\sum_{i=1}^{n} v_i^2}$$

```rust
fn magnitude(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}
```

### Dot Product

$$a \cdot b = \sum_{i=1}^{n} a_i \times b_i$$

```rust
fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
```

### Cosine Similarity

$$\cos(\theta) = \frac{a \cdot b}{\|a\| \times \|b\|}$$

```rust
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    dot_product(a, b) / (magnitude(a) * magnitude(b))
}
```

---

## 📊 Similarity Metrics Comparison

| Metric | Range | Best For | Sensitive To |
|--------|-------|----------|--------------|
| **Dot Product** | $(-\infty, +\infty)$ | Pre-normalized vectors | Magnitude |
| **Cosine Similarity** | $[-1, 1]$ | Semantic search | Direction only |
| **Euclidean Distance** | $[0, +\infty)$ | Spatial data | Both |

---

## 🚀 The Normalization Trick

**Problem:** Computing `sqrt()` is expensive. Doing it for every search wastes CPU.

**Solution:** Normalize vectors on insert (make $\|v\| = 1$).

```rust
// On insert:
let normalized = normalize(&vector);  // Once

// On search:
let similarity = dot_product(&query, &stored);  // Fast! No sqrt needed
```

When both vectors are normalized:
$$\cos(\theta) = a \cdot b$$

---

## 🔗 Dependencies

- **Post #10:** Concurrent storage engine
- **Next:** Post #12 (Brute Force Search)

---

## 🚀 Next Up

**Post #12:** Brute Force Search — Implementing exact k-NN and benchmarking against 1M vectors
