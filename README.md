<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust"/>
  <img src="https://img.shields.io/badge/License-MIT-green.svg?style=for-the-badge" alt="MIT License"/>
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=for-the-badge" alt="PRs Welcome"/>
  <img src="https://img.shields.io/badge/Status-In%20Progress-yellow?style=for-the-badge" alt="Status"/>
</p>

<h1 align="center">🦀 Building a Vector Database from Scratch in Rust</h1>

<p align="center">
  <strong>A comprehensive 20-part tutorial series that builds a production-grade vector database from the ground up.</strong>
</p>

<p align="center">
  Learn systems programming, database internals, and AI infrastructure by building real software.
</p>

---

## What Is This?

This repository contains a **complete tutorial series** for building a vector database from scratch using Rust. By the end, you'll have built a working system with:

- ⚡ **Sub-millisecond search** using HNSW graphs
- 💾 **Crash recovery** with Write-Ahead Logging (WAL)
- 🗺️ **Memory-mapped storage** for efficient disk I/O
- 🔍 **Hybrid search** combining vectors + metadata filters
- 🌐 **Async HTTP API** with Axum and Tokio
- 📦 **Production-ready** Docker deployment

**No Rust experience required**,  we learn together, concept by concept.

---

## Why Build a Vector Database?

Vector databases power modern AI applications:

- **RAG (Retrieval-Augmented Generation)** — Give LLMs long-term memory
- **Semantic Search** — Find by meaning, not keywords
- **Recommendation Systems** — "Users who liked X also liked Y"
- **Image/Audio Search** — Find similar media content

Everyone *uses* vector databases, but few understand how they work. This series changes that.

---

## Quick Start

### Prerequisites

- **Rust 1.70+** (we recommend 1.90+)
- **Git**
- Basic programming experience (any language)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/vectordb-from-scratch.git
cd vectordb-from-scratch

# Run Post #1 example (cosine similarity)
cd post-01-the-blueprint/code
cargo run --bin cosine-similarity-preview

# Run Post #2 example (async runtime)
cd ../../post-02-setting-up-the-forge/code
cargo run --bin hello-async
```

### Running the Main Project

```bash
cd vectordb
cargo run
```

---

## 📚 Series Structure

### Phase 1: Foundation (Posts 1-4)
| Post | Title | Topics |
|------|-------|--------|
| 01 | [The Blueprint](posts/01-the-blueprint/index.md) | Architecture design, vectors, embeddings, cosine similarity |
| 02 | [Setting Up the Forge](posts/02-setting-up-the-forge/index.md) | Rust toolchain, VS Code, async runtime |
| 03 | [Ownership & Borrowing](posts/03-ownership-borrowing-memory/index.md) | Memory safety, the borrow checker |
| 04 | [Structs, Enums & Errors](posts/04-structs-enums-error-handling/index.md) | Domain modeling, Result type |

### Phase 2: Storage Layer (Posts 5-10)
| Post | Title | Topics |
|------|-------|--------|
| 05 | [Async & Axum](posts/05-async-axum/index.md) | HTTP server, JSON endpoints |
| 06 | [Binary File Formats](posts/06-binary-file-formats/index.md) | Custom segment format, endianness |
| 07 | [Memory-Mapped Files](posts/07-mmap/index.md) | Zero-copy I/O with `memmap2` |
| 08 | [Write-Ahead Logging](posts/08-wal/index.md) | Durability, crash safety |
| 09 | [Crash Recovery](posts/09-crash-recovery/index.md) | Replaying WAL, consistency |
| 10 | [Concurrency](posts/10-concurrency/index.md) | RwLock, Arc, thread safety |

### Phase 3: Vector Search (Posts 11-16)
| Post | Title | Topics |
|------|-------|--------|
| 11 | [Vector Math](posts/11-vector-math/index.md) | Dot product, cosine distance, norms |
| 12 | [Brute Force Search](posts/12-brute-force/index.md) | Linear scan, baseline performance |
| 12.5 | [Heaps Deep Dive](posts/12.5-heaps-deep-dive/index.md) | Priority queues, top-k selection |
| 13 | [HNSW Introduction](posts/13-hnsw-intro/index.md) | Navigable small world graphs |
| 14 | [HNSW Implementation I](posts/14-hnsw-impl-1/index.md) | Graph construction, insertion |
| 15 | [HNSW Implementation II](posts/15-hnsw-impl-2/index.md) | Search algorithm, parameters |
| 16 | [Benchmarking](posts/16-benchmarking/index.md) | Criterion, performance testing |

### Phase 4: Hybrid Search (Posts 17-19)
| Post | Title | Topics |
|------|-------|--------|
| 17 | [Inverted Indexes](posts/17-inverted-indexes/index.md) | Text search fundamentals |
| 18 | [Tantivy Integration](posts/18-tantivy-hybrid/index.md) | Metadata filtering, hybrid queries |
| 19 | [Query Planning](posts/19-query-planning/index.md) | Cost-based optimization |

### Phase 5: Production (Post 20)
| Post | Title | Topics |
|------|-------|--------|
| 20 | [Production Hardening](posts/20-production/index.md) | Quantization, Docker, CI/CD, SIMD |

---

## 📁 Project Structure

```
vectordb-from-scratch/
├── README.md                 # You are here
├── vectordb/                 # Main database implementation
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── storage/          # WAL, segments, mmap
│       ├── engine/           # HNSW, query planning
│       └── transport/        # HTTP API (Axum)
│
├── post-01-the-blueprint/    # Each post has its own directory
│   ├── blog.md               # The tutorial content
│   ├── README.md             # Post-specific instructions
│   ├── code/                 # Standalone runnable examples
│   │   ├── Cargo.toml
│   │   └── *.rs
│   └── diagrams/             # Mermaid diagrams
│
├── post-02-setting-up-the-forge/
│   └── ...
│
└── ... (posts 03-20)
```

Each post's `code/` directory is **completely self-contained**,  you can run examples without affecting the main project.

---

## 🛠️ Tech Stack

| Component | Technology | Why |
|-----------|------------|-----|
| Language | **Rust** | Memory safety + C-level performance |
| Async Runtime | **Tokio** | Industry-standard, battle-tested |
| HTTP Framework | **Axum** | Ergonomic, fast, Tokio-native |
| Serialization | **Serde** | The gold standard for Rust |
| Memory Mapping | **memmap2** | Zero-copy disk I/O |
| Text Search | **Tantivy** | Rust's Lucene equivalent |
| Benchmarking | **Criterion** | Statistical benchmarking |

---

## 📊 Performance

Results from Post #20 (1M vectors, 768 dimensions):

| Metric | Before Optimization | After Optimization |
|--------|--------------------|--------------------|
| Memory | 4.2 GB | 1.1 GB (4× reduction) |
| Latency (p50) | 2.1 ms | 0.8 ms (2.6× faster) |
| Throughput | 476 qps | 1,250 qps |
| Recall@10 | 100% | 96.8% |

---

## ❓ FAQ

<details>
<summary><strong>Do I need to know Rust?</strong></summary>

**No!** We teach Rust concepts as we need them. If you can program in any language, you can follow along.
</details>

<details>
<summary><strong>Why Rust instead of Python/Go/C++?</strong></summary>

- **Python**: Too slow for database internals (10-100× slower)
- **Go**: GC pauses are problematic for latency-sensitive systems
- **C++**: Memory safety bugs cause ~70% of security vulnerabilities
- **Rust**: C++ performance with memory safety guarantees
</details>

<details>
<summary><strong>Why Edition 2021 instead of 2024?</strong></summary>

Edition 2021 is stable, battle-tested, and has full ecosystem support. Edition 2024 adds minor syntax changes that don't affect our project. Both editions are backwards compatible. [Learn more](https://doc.rust-lang.org/edition-guide/)
</details>

<details>
<summary><strong>How long does the series take?</strong></summary>

Each post takes 30-60 minutes to read and implement. The full series is ~20-30 hours of focused learning.
</details>

<details>
<summary><strong>Can I use this code in production?</strong></summary>

**Yes, with the right hardening!** Post #20 covers production deployment including quantization, Docker, CI/CD, and SIMD optimizations. The architecture is designed for real-world use. Start with the tutorial, understand every component, then extend it for your specific needs. You'll have full control over your vector database — no vendor lock-in!
</details>

---

## 🤝 Contributing

We welcome contributions! Here's how you can help:

### Types of Contributions

- 🐛 **Bug fixes** — Found an error in the code or text? PRs welcome!
- 📝 **Typo fixes** — Even small improvements help
- 🎨 **Diagram improvements** — Better visualizations
- 🌐 **Translations** — Help make this accessible to more people
- 💡 **Suggestions** — Open an issue with ideas

### Guidelines

1. **Fork** the repository
2. **Create a branch** for your feature (`git checkout -b fix/typo-in-post-3`)
3. **Make your changes** and test them
4. **Submit a PR** with a clear description

### Code Style

```bash
# Before submitting, run:
cargo fmt      # Format code
cargo clippy   # Lint
cargo test     # Run tests
```

---

## 📖 Learning Resources

### Rust
- [The Rust Book](https://doc.rust-lang.org/book/) — Official introduction
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) — Learn by doing
- [Rustlings](https://github.com/rust-lang/rustlings) — Interactive exercises

### Vector Databases
- [HNSW Paper](https://arxiv.org/abs/1603.09320) — The algorithm behind most vector DBs
- [Pinecone Learning Center](https://www.pinecone.io/learn/) — Great conceptual explanations
- [Weaviate Blog](https://weaviate.io/blog) — Deep dives into vector search

### Systems Programming
- [Database Internals](https://www.databass.dev/) — Alex Petrov's excellent book
- [Designing Data-Intensive Applications](https://dataintensive.net/) — The DDIA Bible

---

## 📜 License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.

You are free to:
- ✅ Use this code commercially
- ✅ Modify and distribute
- ✅ Use for private projects

---

## 🙏 Acknowledgments

- The Rust community for incredible tooling
- [Qdrant](https://qdrant.tech/), [Milvus](https://milvus.io/), and [Pinecone](https://www.pinecone.io/) for inspiration
- Everyone who contributed feedback and corrections

---

<p align="center">
  <strong>Built with 🦀 and ❤️</strong>
</p>

<p align="center">
  <a href="https://github.com/yourusername/vectordb-from-scratch/issues">Report Bug</a> •
  <a href="https://github.com/yourusername/vectordb-from-scratch/issues">Request Feature</a>
</p>
