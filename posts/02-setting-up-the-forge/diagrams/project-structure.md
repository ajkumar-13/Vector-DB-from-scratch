# Project Structure Diagrams

## Initial Structure (After `cargo new`)

```
vectordb/
├── Cargo.toml          # Project manifest (dependencies, metadata)
│                       # Think of it as package.json or requirements.txt
│
├── Cargo.lock          # Exact dependency versions (auto-generated)
│                       # Commit this to git for reproducible builds
│
├── .gitignore          # Ignores /target directory (build artifacts)
│
└── src/
    └── main.rs         # Entry point for binary crate
                        # The fn main() function lives here
```

## Full Structure (What We'll Build)

```
vectordb/
│
├── Cargo.toml              # Project manifest
├── Cargo.lock              # Lockfile
├── README.md               # Project documentation
│
├── src/
│   │
│   ├── main.rs             # Entry point: CLI parsing, starts server
│   │
│   ├── lib.rs              # Library root: re-exports public API
│   │                       # Makes vectordb usable as a library too
│   │
│   ├── transport/          # ═══ LAYER 1: HTTP Interface ═══
│   │   ├── mod.rs          # Module declaration
│   │   ├── routes.rs       # Axum route definitions
│   │   ├── handlers.rs     # Request handlers
│   │   └── dto.rs          # Data Transfer Objects (JSON shapes)
│   │
│   ├── engine/             # ═══ LAYER 2: Core Engine ═══
│   │   ├── mod.rs          # Module declaration
│   │   ├── query.rs        # Query planning & execution
│   │   ├── vector_index.rs # HNSW graph for similarity search
│   │   └── metadata_index.rs # Tantivy integration for filtering
│   │
│   └── storage/            # ═══ LAYER 3: Persistence ═══
│       ├── mod.rs          # Module declaration
│       ├── wal.rs          # Write-Ahead Log implementation
│       ├── segment.rs      # Memory-mapped segment files
│       └── recovery.rs     # Crash recovery logic
│
├── tests/                  # Integration tests
│   ├── api_tests.rs        # HTTP API tests
│   └── fixtures/           # Test data
│       └── sample_vectors.json
│
├── benches/                # Performance benchmarks
│   ├── search_bench.rs     # Search latency benchmarks
│   └── insert_bench.rs     # Ingestion throughput benchmarks
│
├── examples/               # Example usage
│   └── basic_usage.rs      # cargo run --example basic_usage
│
└── target/                 # Build artifacts (gitignored)
    ├── debug/              # Debug builds
    └── release/            # Optimized builds
```

## Module Dependency Graph

```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                             │
│                    (Entry Point, CLI)                       │
└─────────────────────────────────┬───────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────┐
│                          lib.rs                             │
│                  (Public API, Re-exports)                   │
└────────┬─────────────────────┬─────────────────────┬────────┘
         │                     │                     │
         ▼                     ▼                     ▼
┌─────────────────┐   ┌─────────────────┐    ┌─────────────────┐
│   transport/    │   │    engine/      │    │    storage/     │
│                 │   │                 │    │                 │
│  routes.rs      │──▶│  query.rs       │──▶│  wal.rs         │
│  handlers.rs    │   │  vector_index.rs│    │  segment.rs     │
│  dto.rs         │   │  metadata_index │    │  recovery.rs    │
└─────────────────┘   └─────────────────┘    └─────────────────┘
     HTTP Layer            Core Logic            Persistence
```

## File Naming Conventions

| Pattern | Meaning | Example |
|---------|---------|---------|
| `mod.rs` | Module declaration file | `src/engine/mod.rs` |
| `lib.rs` | Library crate root | `src/lib.rs` |
| `main.rs` | Binary crate root | `src/main.rs` |
| `*_test.rs` | Unit tests (usually inline) | `#[cfg(test)] mod tests` |
| `tests/*.rs` | Integration tests | `tests/api_tests.rs` |
| `benches/*.rs` | Benchmarks | `benches/search_bench.rs` |
| `examples/*.rs` | Runnable examples | `examples/basic_usage.rs` |
