# References

Master bibliography for "Building a Vector Database from Scratch in Rust." Citations are grouped
by theme; where a source is used from several posts it is listed once. Full URLs appear in each
entry.

> Citation style: author(s) or maintainer, title, venue or publisher, year. Online sources include
> the URL. Where useful, a short note explains which point the source supports.

---

## Rust language and tooling

- **Klabnik, S., and Nichols, C.** *The Rust Programming Language* ("the Book"). No Starch Press / the Rust Project. https://doc.rust-lang.org/book/ , the canonical reference for ownership, borrowing, `Option`/`Result`, traits, and fearless concurrency used throughout the series.
- **The Rust Project.** *The Cargo Book.* https://doc.rust-lang.org/cargo/ , manifests, editions, and the build commands from Post 02.
- **The Rust Project.** *The Rustonomicon.* https://doc.rust-lang.org/nomicon/ , advanced ownership, aliasing, lifetimes, and the `unsafe` invariants behind the mmap layer.
- **The Rust Project.** *Rust by Example.* https://doc.rust-lang.org/rust-by-example/ , ownership, borrowing, and slices.
- **The Rust Project.** *Standard library documentation* , `Option`, `Result`, and `From` (https://doc.rust-lang.org/std/), `f32` byte-order helpers (https://doc.rust-lang.org/std/primitive.f32.html), `std::fs::File::sync_all` and `rename`, `std::sync::RwLock`, `std::collections::BinaryHeap` and `std::cmp::Reverse`, `f32::total_cmp`, and `std::arch` feature detection.
- **rust-analyzer.** *User manual.* https://rust-analyzer.github.io/manual.html , editor setup, inlay hints, and Clippy integration.
- **Tokio.** *Documentation and tutorial.* https://tokio.rs/tokio/tutorial , the async runtime, `#[tokio::main]`, `tokio::spawn`, and `tokio::sync::RwLock`.
- **Axum.** *Crate documentation.* https://docs.rs/axum , routers, extractors (`Json`, `State`, `Path`), `IntoResponse`, and `with_state`.
- **Tower.** *Crate documentation.* https://docs.rs/tower , the `Service` and `Layer` abstractions behind Axum's middleware.
- **Selected crates.** `thiserror` (https://docs.rs/thiserror), `byteorder` (https://docs.rs/byteorder), `bytemuck` (https://docs.rs/bytemuck), `memmap2` (https://docs.rs/memmap2), `bincode` (https://docs.rs/bincode), `crc32fast` (https://docs.rs/crc32fast), `rand` (https://docs.rs/rand), `rayon` (https://docs.rs/rayon), `loom` (https://docs.rs/loom), `roaring` (https://docs.rs/roaring), and `rust-stemmers` (https://docs.rs/rust-stemmers).

---

## Storage, persistence, and crash recovery

- **Mohan, C., Haderle, D., Lindsay, B., Pirahesh, H., and Schwarz, P.** "ARIES: A Transaction Recovery Method Supporting Fine-Granularity Locking and Partial Rollbacks Using Write-Ahead Logging." *ACM TODS*, 1992. , the canonical treatment of write-ahead logging and crash recovery.
- **O'Neil, P., Cheng, E., Gawlick, D., and O'Neil, E.** "The Log-Structured Merge-Tree (LSM-Tree)." *Acta Informatica*, 1996. , the foundation for the MemTable-plus-segments design.
- **The PostgreSQL Global Development Group.** *PostgreSQL Documentation* , "Reliability and the Write-Ahead Log" and "Query Planning / EXPLAIN." https://www.postgresql.org/docs/
- **Meta / RocksDB project.** *RocksDB Wiki* , "Compaction" and "WAL." https://github.com/facebook/rocksdb/wiki , a production reference for compaction and log truncation.
- **IEEE.** *IEEE 754-2019, Standard for Floating-Point Arithmetic.* https://standards.ieee.org/ieee/754/6210/ , the `f32`/`f64` bit layouts behind the binary segment format.
- **Linux man-pages project.** `mmap(2)`. https://man7.org/linux/man-pages/man2/mmap.2.html , page-fault semantics and the `madvise` hints surfaced as `Advice`.

---

## Vectors, similarity, and embeddings

- **Mikolov, T., Chen, K., Corrado, G., and Dean, J.** "Efficient Estimation of Word Representations in Vector Space" (Word2Vec). 2013. https://arxiv.org/abs/1301.3781 , the source of the king/queen embedding-geometry example.
- **Aggarwal, C. C., Hinneburg, A., and Keim, D. A.** "On the Surprising Behavior of Distance Metrics in High Dimensional Space." 2001. https://bib.dbvis.de/uploadedFiles/155.pdf , why distances concentrate as dimensionality grows.

---

## Approximate nearest neighbour and graph indexes

- **Malkov, Y. A., and Yashunin, D. A.** "Efficient and Robust Approximate Nearest Neighbor Search Using Hierarchical Navigable Small World Graphs." *IEEE TPAMI*, 2020. https://arxiv.org/abs/1603.09320 , the canonical HNSW paper: layer assignment, neighbour-selection heuristics, and the `M` / `ef_construction` / `ef_search` parameters.
- **Malkov, Y. A., Ponomarenko, A., Logvinov, A., and Krylov, V.** "Approximate Nearest Neighbor Algorithm Based on Navigable Small World Graphs." *Information Systems*, 2014. , the NSW precursor behind the proximity-graph and greedy-walk ideas.
- **Pugh, W.** "Skip Lists: A Probabilistic Alternative to Balanced Trees." *Communications of the ACM*, 1990. , the probabilistic layering HNSW generalises.
- **Jegou, H., Douze, M., and Schmid, C.** "Product Quantization for Nearest Neighbor Search." *IEEE TPAMI*, 2011. , the basis for the higher-compression quantization extension.
- **Aumuller, M., Bernhardsson, E., and Faithfull, A.** "ANN-Benchmarks: A Benchmarking Tool for Approximate Nearest Neighbor Algorithms." *Information Systems*, 2020. https://ann-benchmarks.com/ , the standard methodology for Recall@K versus queries-per-second.
- **Johnson, J., Douze, M., and Jegou, H.** "Billion-Scale Similarity Search with GPUs" (Faiss). 2017. https://github.com/facebookresearch/faiss , a production ANN library and a useful comparison point for the engine built here.
- **Guo, R., Sun, P., Lindgren, E., Geng, Q., Simcha, D., Chern, F., and Kumar, S.** "Accelerating Large-Scale Inference with Anisotropic Vector Quantization" (ScaNN). *ICML*, 2020. https://github.com/google-research/google-research/tree/master/scann
- **Ge, T., He, K., Ke, Q., and Sun, J.** "Optimized Product Quantization." *IEEE TPAMI*, 2014. , a refinement of product quantization for higher compression.
- **Malkov, Y. A., and contributors.** *hnswlib* , the reference C++/Python HNSW implementation. https://github.com/nmslib/hnswlib

---

## Text search, inverted indexes, and hybrid retrieval

- **Masurel, P., and the Quickwit team.** *Tantivy* , a full-text search engine library for Rust. https://docs.rs/tantivy , source at https://github.com/quickwit-oss/tantivy
- **The Apache Software Foundation.** *Apache Lucene.* https://lucene.apache.org/ , the inverted-index engine whose postings, segments, and BM25 design inspired the text layer.
- **Chambi, S., Lemire, D., Kaser, O., and Godin, R.** *Roaring Bitmaps.* https://roaringbitmap.org/ , compressed bitmaps for fast set operations on DocID lists.

---

## Query optimization

- **Selinger, P. G., Astrahan, M. M., Chamberlin, D. D., Lorie, R. A., and Price, T. G.** "Access Path Selection in a Relational Database Management System." *ACM SIGMOD*, 1979. , the founding paper on cost-based query optimization and selectivity-driven plan choice.
- **Silberschatz, A., Korth, H. F., and Sudarshan, S.** *Database System Concepts.* McGraw-Hill. , textbook treatment of query optimization and selectivity estimation.
- **Karpukhin, V., Oguz, B., Min, S., Lewis, P., Wu, L., Edunov, S., Chen, D., and Yih, W.** "Dense Passage Retrieval for Open-Domain Question Answering." *EMNLP*, 2020. https://arxiv.org/abs/2004.04906 , the dense-versus-sparse retrieval trade-offs behind hybrid search.

---

## Performance, SIMD, and benchmarking

- **Nethercote, N., and contributors.** *The Rust Performance Book.* https://nnethercote.github.io/perf-book/ , measuring and profiling Rust, including allocation in hot paths.
- **Heisler, B., and contributors.** *Criterion.rs documentation.* https://bheisler.github.io/criterion.rs/book/ , statistics-driven benchmarking with warmup and outlier detection.
- **Intel.** *Intel Intrinsics Guide.* https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html , the AVX2/AVX-512 instructions LLVM emits when auto-vectorising the distance kernels.
- **Gregg, B.** *Flame Graphs* (and the `flamegraph` Rust crate). https://www.brendangregg.com/flamegraphs.html , visualising where CPU time goes in hot paths.

---

## Production and operations

- **Beyer, B., Jones, C., Petoff, J., and Murphy, N. R. (eds.).** *Site Reliability Engineering.* Google / O'Reilly, 2016. https://sre.google/books/ , operating reliable services at scale.
- **Campbell, L., and Majors, C.** *Database Reliability Engineering.* O'Reilly, 2017. , applying SRE practices to data stores.

---

## Algorithms and data structures

- **Cormen, T. H., Leiserson, C. E., Rivest, R. L., and Stein, C.** *Introduction to Algorithms* (CLRS). MIT Press. , the binary heap and priority-queue chapter, including index math and heapify analysis.
- **Sedgewick, R., and Wayne, K.** *Algorithms.* Addison-Wesley. , priority queues and heap-based selection.
- **Hoare, T.** "Null References: The Billion Dollar Mistake." *QCon London*, 2009. , the talk behind why Rust chose `Option<T>` over null.
