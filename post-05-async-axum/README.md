# Post #5: The Async Runtime & HTTP Layer

**Series:** Building a Vector Database from Scratch in Rust  
**Topic:** Understanding Tokio, Futures, and Building a Basic HTTP Server with Axum

## Folder Contents

| File | Description |
|------|-------------|
| [blog.md](blog.md) | Main blog post (~2000 words) |
| [code/main-server.rs](code/main-server.rs) | Complete Axum server with all routes |
| [code/models-serde.rs](code/models-serde.rs) | Updated models with Serde derives |
| [code/Cargo.toml](code/Cargo.toml) | Dependencies for web server |
| [diagrams/mermaid-diagrams.md](diagrams/mermaid-diagrams.md) | All Mermaid diagrams for this post |

## Key Concepts

- **Tokio**: Async runtime with multi-threaded scheduler
- **Futures**: Promises for values that will exist later
- **Axum**: Type-safe, macro-free web framework
- **Serde**: JSON serialization/deserialization
- **Async/Await**: Non-blocking I/O patterns

## Prerequisites

- Post #4: Structs, Enums, and Error Handling
- Basic understanding of HTTP (GET/POST)

## Test Commands

```bash
# Run the server
cargo run

# Test health endpoint
curl http://localhost:3000/health

# Test search endpoint
curl -X POST http://localhost:3000/search \
  -H "Content-Type: application/json" \
  -d '{"data": [0.1, 0.2, 0.3], "metadata": {}}'
```
