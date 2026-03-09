// src/lib.rs
//
// Library root for VectorDB.
// Re-exports all public modules so they can be used as:
//   use vectordb::models::Vector;
//
// This file grows as we add modules in later phases:
//   Phase 2: pub mod storage;   (WAL, segments, mmap)
//   Phase 3: pub mod engine;    (search, HNSW index)
//   Phase 4: pub mod transport; (Axum HTTP handlers)

pub mod models;
