# Post #10: Concurrency Control — Managing State with Arc, RwLock, and Mutex

> **Series:** Building a Vector Database from Scratch in Rust  
> **Reading Time:** ~15 minutes  
> **Difficulty:** Intermediate  

---

## 📁 Folder Contents

| File | Purpose |
|------|---------|
| [blog.md](blog.md) | Main post: Arc, RwLock, async concurrency patterns |
| [code/shared-state.rs](code/shared-state.rs) | Arc + RwLock example with VectorStore |
| [code/axum-server.rs](code/axum-server.rs) | Complete concurrent HTTP server |
| [diagrams/mermaid-diagrams.md](diagrams/mermaid-diagrams.md) | Visual guides for lock behavior |

---

## 🎯 What You'll Learn

1. **The Single-Threaded Problem** — Why `&mut self` doesn't work in async handlers
2. **`Arc` (Atomic Reference Counting)** — Shared ownership across threads
3. **`Mutex` vs `RwLock`** — When to use which
4. **Tokio's async locks** — Why `tokio::sync::RwLock` instead of `std::sync::RwLock`
5. **Deadlock traps** — The "upgrade" deadlock and how to avoid it
6. **Background tasks** — Compaction running without blocking requests

---

## 🔒 Lock Comparison

| Lock Type | Readers | Writers | Use Case |
|-----------|---------|---------|----------|
| `Mutex` | 1 at a time | 1 at a time | Simple critical sections |
| `RwLock` | Many concurrent | 1 exclusive | Read-heavy workloads |

**Vector DB = Read-Heavy → Use `RwLock`**

---

## 🏗️ The Shared State Pattern

```rust
// Old: Single owner
let mut db = VectorStore::new();

// New: Shared across threads
use std::sync::Arc;
use tokio::sync::RwLock;

type SharedVectorStore = Arc<RwLock<VectorStore>>;
let db = Arc::new(RwLock::new(VectorStore::new()));
```

---

## ⚠️ Common Pitfalls

### 1. The "Upgrade" Deadlock

```rust
// ❌ DEADLOCK: Holding read lock while requesting write lock
let db = store.read().await;
if !db.contains("id") {
    let mut w_db = store.write().await;  // Waits forever!
}
```

### 2. Using `std::sync` in Async Code

```rust
// ❌ BAD: Blocks the entire runtime
use std::sync::RwLock;

// ✅ GOOD: Yields to other tasks
use tokio::sync::RwLock;
```

### 3. Holding Locks Across `.await`

```rust
// ❌ BAD: Lock held during I/O
let db = store.read().await;
expensive_io_operation().await;  // Lock still held!

// ✅ GOOD: Release lock before I/O
let data = {
    let db = store.read().await;
    db.get("id").clone()
};  // Lock dropped here
expensive_io_operation().await;
```

---

## 📊 Concurrency Model

```
┌─────────────────────────────────────────────────────────┐
│                    Arc<RwLock<VectorStore>>             │
├─────────────────────────────────────────────────────────┤
│                                                         │
│   Reader 1 ──┐                                          │
│   Reader 2 ──┼──▶ .read().await  ──▶ [Concurrent OK]   │
│   Reader 3 ──┘                                          │
│                                                         │
│   Writer ────────▶ .write().await ──▶ [Exclusive]       │
│                                                         │
│   Background ────▶ .write().await ──▶ [Compaction]     │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## 🚀 Performance Impact

| Scenario | Mutex | RwLock |
|----------|-------|--------|
| 100 concurrent searches | Serialized (slow) | Parallel (fast) |
| 1 insert + 99 searches | All wait | Only insert waits |
| Insert during compaction | Both wait | Both wait (unavoidable) |

---

## 🔗 Dependencies

- **Post #5:** Axum HTTP server setup
- **Post #9:** VectorStore with compaction

---

## 🚀 Next Up

**Post #11:** Vector Math for Developers — Linear algebra basics for similarity search
