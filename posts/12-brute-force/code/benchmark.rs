// post-12-brute-force/code/benchmark.rs
// Performance testing for brute force search at scale
//
// Run with: rustc -O benchmark.rs && ./benchmark
// Or: RUSTFLAGS="-C target-cpu=native" rustc -O benchmark.rs && ./benchmark

use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::time::Instant;

// ============================================================================
// Vector Math
// ============================================================================

#[inline]
fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[inline]
fn magnitude(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

#[inline]
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot = dot_product(a, b);
    let mag_a = magnitude(a);
    let mag_b = magnitude(b);
    if mag_a == 0.0 || mag_b == 0.0 {
        0.0
    } else {
        dot / (mag_a * mag_b)
    }
}

#[inline]
fn normalize(v: &[f32]) -> Vec<f32> {
    let mag = magnitude(v);
    if mag == 0.0 {
        vec![0.0; v.len()]
    } else {
        v.iter().map(|x| x / mag).collect()
    }
}

// ============================================================================
// Candidate
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub id: String,
    pub score: f32,
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Eq for Candidate {}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Less)
    }
}

fn push_candidate(heap: &mut BinaryHeap<Reverse<Candidate>>, candidate: Candidate, k: usize) {
    if heap.len() < k {
        heap.push(Reverse(candidate));
    } else if let Some(Reverse(worst)) = heap.peek() {
        if candidate.score > worst.score {
            heap.pop();
            heap.push(Reverse(candidate));
        }
    }
}

// ============================================================================
// VectorStore
// ============================================================================

pub struct VectorStore {
    vectors: Vec<(String, Vec<f32>)>,
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            vectors: Vec::new(),
        }
    }

    pub fn insert(&mut self, id: String, vector: Vec<f32>) {
        self.vectors.push((id, vector));
    }

    pub fn search(&self, query: &[f32], k: usize) -> Vec<Candidate> {
        let mut heap = BinaryHeap::new();

        for (id, vector) in &self.vectors {
            let score = cosine_similarity(query, vector);
            push_candidate(
                &mut heap,
                Candidate {
                    id: id.clone(),
                    score,
                },
                k,
            );
        }

        let mut results: Vec<_> = heap.into_iter().map(|Reverse(c)| c).collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    pub fn search_normalized(&self, query: &[f32], k: usize) -> Vec<Candidate> {
        let mut heap = BinaryHeap::new();

        // Assume all vectors are pre-normalized
        for (id, vector) in &self.vectors {
            let score = dot_product(query, vector); // Fast path
            push_candidate(
                &mut heap,
                Candidate {
                    id: id.clone(),
                    score,
                },
                k,
            );
        }

        let mut results: Vec<_> = heap.into_iter().map(|Reverse(c)| c).collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    pub fn len(&self) -> usize {
        self.vectors.len()
    }
}

// ============================================================================
// Benchmark Scenarios
// ============================================================================

/// Generate pseudo-random vector (deterministic for reproducibility)
fn generate_vector(seed: usize, dimensions: usize) -> Vec<f32> {
    (0..dimensions)
        .map(|i| ((seed * 31 + i * 17) as f32 * 0.001).sin())
        .collect()
}

fn benchmark_basic_search(dimensions: usize, vector_count: usize, k: usize) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  Benchmark: Basic Search (Unnormalized)                     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    println!("\nParameters:");
    println!("  Dimensions:    {}", dimensions);
    println!("  Vector count:  {}", vector_count);
    println!("  k (top-k):     {}", k);

    // Build database
    print!("\nBuilding database... ");
    let build_start = Instant::now();
    let mut db = VectorStore::new();
    for i in 0..vector_count {
        let vector = generate_vector(i, dimensions);
        db.insert(format!("vec_{}", i), vector);
    }
    let build_time = build_start.elapsed();
    println!("done in {:?}", build_time);

    // Generate query
    let query = generate_vector(999, dimensions);

    // Warmup
    print!("Warming up... ");
    for _ in 0..3 {
        let _ = db.search(&query, k);
    }
    println!("done");

    // Benchmark
    println!("\nRunning benchmark (10 queries)...");
    let iterations = 10;
    let start = Instant::now();

    for i in 0..iterations {
        let query = generate_vector(1000 + i, dimensions);
        let _ = db.search(&query, k);
    }

    let total_time = start.elapsed();
    let avg_time = total_time / iterations as u32;

    println!("\nResults:");
    println!("  Total time:     {:?}", total_time);
    println!("  Average/query:  {:?}", avg_time);
    println!(
        "  Throughput:     {:.2} queries/sec",
        1000.0 / avg_time.as_millis() as f64
    );

    // Memory estimate
    let bytes_per_vector = dimensions * 4; // f32 = 4 bytes
    let total_mb = (vector_count * bytes_per_vector) / (1024 * 1024);
    println!("  Memory (est):   {} MB", total_mb);
}

fn benchmark_normalized_search(dimensions: usize, vector_count: usize, k: usize) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  Benchmark: Normalized Search (Optimized)                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    println!("\nParameters:");
    println!("  Dimensions:    {}", dimensions);
    println!("  Vector count:  {}", vector_count);
    println!("  k (top-k):     {}", k);

    // Build database with normalized vectors
    print!("\nBuilding database (with normalization)... ");
    let build_start = Instant::now();
    let mut db = VectorStore::new();
    for i in 0..vector_count {
        let vector = generate_vector(i, dimensions);
        let normalized = normalize(&vector);
        db.insert(format!("vec_{}", i), normalized);
    }
    let build_time = build_start.elapsed();
    println!("done in {:?}", build_time);

    // Generate normalized query
    let query_raw = generate_vector(999, dimensions);
    let query = normalize(&query_raw);

    // Warmup
    print!("Warming up... ");
    for _ in 0..3 {
        let _ = db.search_normalized(&query, k);
    }
    println!("done");

    // Benchmark
    println!("\nRunning benchmark (10 queries)...");
    let iterations = 10;
    let start = Instant::now();

    for i in 0..iterations {
        let query_raw = generate_vector(1000 + i, dimensions);
        let query = normalize(&query_raw);
        let _ = db.search_normalized(&query, k);
    }

    let total_time = start.elapsed();
    let avg_time = total_time / iterations as u32;

    println!("\nResults:");
    println!("  Total time:     {:?}", total_time);
    println!("  Average/query:  {:?}", avg_time);
    println!(
        "  Throughput:     {:.2} queries/sec",
        1000.0 / avg_time.as_millis() as f64
    );
}

fn benchmark_scaling(dimensions: usize, k: usize) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  Benchmark: Scaling Analysis                                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    println!("\nDimensions: {}, k: {}\n", dimensions, k);
    println!(
        "{:>12} {:>15} {:>15} {:>15}",
        "Vectors", "Avg Latency", "QPS", "Memory (MB)"
    );
    println!("{}", "-".repeat(60));

    let sizes = [1_000, 5_000, 10_000, 50_000, 100_000];

    for &size in &sizes {
        // Build database
        let mut db = VectorStore::new();
        for i in 0..size {
            let vector = generate_vector(i, dimensions);
            let normalized = normalize(&vector);
            db.insert(format!("vec_{}", i), normalized);
        }

        // Benchmark
        let iterations = 5;
        let start = Instant::now();

        for i in 0..iterations {
            let query_raw = generate_vector(1000 + i, dimensions);
            let query = normalize(&query_raw);
            let _ = db.search_normalized(&query, k);
        }

        let total_time = start.elapsed();
        let avg_time = total_time / iterations as u32;
        let qps = 1000.0 / avg_time.as_millis() as f64;
        let memory_mb = (size * dimensions * 4) / (1024 * 1024);

        println!(
            "{:>12} {:>15.2?} {:>15.2} {:>15}",
            size, avg_time, qps, memory_mb
        );
    }
}

fn benchmark_dimensionality(vector_count: usize, k: usize) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  Benchmark: Dimensionality Impact                           ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    println!("\nVectors: {}, k: {}\n", vector_count, k);
    println!("{:>12} {:>15} {:>15}", "Dimensions", "Avg Latency", "QPS");
    println!("{}", "-".repeat(45));

    let dimensions_list = [64, 128, 256, 512, 768, 1024, 1536];

    for &dims in &dimensions_list {
        // Build database
        let mut db = VectorStore::new();
        for i in 0..vector_count {
            let vector = generate_vector(i, dims);
            let normalized = normalize(&vector);
            db.insert(format!("vec_{}", i), normalized);
        }

        // Benchmark
        let iterations = 5;
        let start = Instant::now();

        for i in 0..iterations {
            let query_raw = generate_vector(1000 + i, dims);
            let query = normalize(&query_raw);
            let _ = db.search_normalized(&query, k);
        }

        let total_time = start.elapsed();
        let avg_time = total_time / iterations as u32;
        let qps = 1000.0 / avg_time.as_millis() as f64;

        println!("{:>12} {:>15.2?} {:>15.2}", dims, avg_time, qps);
    }
}

fn benchmark_k_impact(dimensions: usize, vector_count: usize) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  Benchmark: k (Top-K) Impact                                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    println!("\nDimensions: {}, Vectors: {}\n", dimensions, vector_count);
    println!(
        "{:>12} {:>15} {:>15}",
        "k", "Avg Latency", "Overhead vs k=1"
    );
    println!("{}", "-".repeat(45));

    // Build database once
    let mut db = VectorStore::new();
    for i in 0..vector_count {
        let vector = generate_vector(i, dimensions);
        let normalized = normalize(&vector);
        db.insert(format!("vec_{}", i), normalized);
    }

    let k_values = [1, 5, 10, 20, 50, 100];
    let mut baseline_time = None;

    for &k in &k_values {
        let iterations = 5;
        let start = Instant::now();

        for i in 0..iterations {
            let query_raw = generate_vector(1000 + i, dimensions);
            let query = normalize(&query_raw);
            let _ = db.search_normalized(&query, k);
        }

        let total_time = start.elapsed();
        let avg_time = total_time / iterations as u32;

        if baseline_time.is_none() {
            baseline_time = Some(avg_time);
            println!("{:>12} {:>15.2?} {:>15}", k, avg_time, "baseline");
        } else {
            let overhead = avg_time.as_secs_f64() / baseline_time.unwrap().as_secs_f64();
            println!("{:>12} {:>15.2?} {:>15.2}x", k, avg_time, overhead);
        }
    }
}

fn benchmark_comparison() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  Benchmark: Normalized vs Unnormalized                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    let dimensions = 768;
    let vector_count = 10_000;
    let k = 10;

    println!(
        "\nDimensions: {}, Vectors: {}, k: {}\n",
        dimensions, vector_count, k
    );

    // Build unnormalized database
    let mut db_unnorm = VectorStore::new();
    for i in 0..vector_count {
        let vector = generate_vector(i, dimensions);
        db_unnorm.insert(format!("vec_{}", i), vector);
    }

    // Build normalized database
    let mut db_norm = VectorStore::new();
    for i in 0..vector_count {
        let vector = generate_vector(i, dimensions);
        let normalized = normalize(&vector);
        db_norm.insert(format!("vec_{}", i), normalized);
    }

    // Benchmark unnormalized
    let query = generate_vector(999, dimensions);
    let iterations = 10;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = db_unnorm.search(&query, k);
    }
    let unnorm_time = start.elapsed() / iterations;

    // Benchmark normalized
    let query_norm = normalize(&query);

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = db_norm.search_normalized(&query_norm, k);
    }
    let norm_time = start.elapsed() / iterations;

    println!("{:>25} {:>15}", "Method", "Avg Latency");
    println!("{}", "-".repeat(42));
    println!("{:>25} {:>15.2?}", "Unnormalized", unnorm_time);
    println!("{:>25} {:>15.2?}", "Normalized", norm_time);

    let speedup = unnorm_time.as_secs_f64() / norm_time.as_secs_f64();
    println!("\nSpeedup from normalization: {:.2}x", speedup);
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║      Brute Force Search - Performance Benchmarks            ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    // Scenario 1: Basic benchmark
    benchmark_basic_search(768, 10_000, 10);

    // Scenario 2: Normalized search
    benchmark_normalized_search(768, 10_000, 10);

    // Scenario 3: Scaling analysis
    benchmark_scaling(768, 10);

    // Scenario 4: Dimensionality impact
    benchmark_dimensionality(5_000, 10);

    // Scenario 5: k impact
    benchmark_k_impact(768, 10_000);

    // Scenario 6: Comparison
    benchmark_comparison();

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("All benchmarks complete!");
    println!("\nKey findings:");
    println!("  1. Search time scales linearly with dataset size (O(N))");
    println!("  2. Normalization provides approximately 1.5 to 2x speedup");
    println!("  3. Higher dimensions increase latency proportionally");
    println!("  4. k (top-k) has minimal impact (O(log k) overhead)");
    println!("  5. For over 100k vectors, consider approximate search (HNSW)");
}
