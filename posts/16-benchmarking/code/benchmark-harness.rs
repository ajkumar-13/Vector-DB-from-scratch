// benchmark-harness.rs
// Reusable Benchmarking Framework for Vector Search Algorithms
// Handles warmup, percentile calculation, and result reporting

use rand::prelude::*;
use std::collections::HashSet;
use std::time::{Duration, Instant};

// ============================================================================
// Configuration & Results
// ============================================================================

#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub dataset_size: usize,
    pub dimensions: usize,
    pub num_queries: usize,
    pub warmup_queries: usize,
    pub k: usize, // Number of results to return
}

impl BenchmarkConfig {
    pub fn new(dataset_size: usize, dimensions: usize) -> Self {
        Self {
            dataset_size,
            dimensions,
            num_queries: 1000,
            warmup_queries: 100,
            k: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub algo_name: String,
    pub config: BenchmarkConfig,

    // Latency metrics
    pub avg_latency: Duration,
    pub p50_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
    pub min_latency: Duration,
    pub max_latency: Duration,

    // Throughput
    pub qps: f64,

    // Quality
    pub avg_recall: f32,

    // Resource usage
    pub build_time: Duration,
    pub memory_mb: f64,
}

impl BenchmarkResult {
    /// Print a nicely formatted results table
    pub fn print_table(&self) {
        println!("\n╔════════════════════════════════════════════════════════╗");
        println!("║  {} Benchmark Results", self.algo_name);
        println!("╠════════════════════════════════════════════════════════╣");
        println!(
            "║ Dataset:       {} vectors x {} dims",
            self.config.dataset_size, self.config.dimensions
        );
        println!("║ Build Time:    {:.2}s", self.build_time.as_secs_f64());
        println!("║ Memory Usage:  {:.1} MB", self.memory_mb);
        println!("╠════════════════════════════════════════════════════════╣");
        println!("║ Latency:");
        println!(
            "║   Average:     {:.2} ms",
            self.avg_latency.as_secs_f64() * 1000.0
        );
        println!(
            "║   P50:         {:.2} ms",
            self.p50_latency.as_secs_f64() * 1000.0
        );
        println!(
            "║   P95:         {:.2} ms",
            self.p95_latency.as_secs_f64() * 1000.0
        );
        println!(
            "║   P99:         {:.2} ms",
            self.p99_latency.as_secs_f64() * 1000.0
        );
        println!(
            "║   Min/Max:     {:.2} / {:.2} ms",
            self.min_latency.as_secs_f64() * 1000.0,
            self.max_latency.as_secs_f64() * 1000.0
        );
        println!("╠════════════════════════════════════════════════════════╣");
        println!("║ Throughput:    {:.0} QPS", self.qps);
        println!(
            "║ Recall@{}:      {:.1}%",
            self.config.k,
            self.avg_recall * 100.0
        );
        println!("╚════════════════════════════════════════════════════════╝");
    }
}

// ============================================================================
// Percentile Calculation
// ============================================================================

/// Calculate P50, P95, and P99 latencies from a list of measurements
pub fn calculate_percentiles(mut latencies: Vec<Duration>) -> (Duration, Duration, Duration) {
    if latencies.is_empty() {
        return (Duration::ZERO, Duration::ZERO, Duration::ZERO);
    }

    latencies.sort();

    let p50_idx = latencies.len() / 2;
    let p95_idx = (latencies.len() as f64 * 0.95) as usize;
    let p99_idx = (latencies.len() as f64 * 0.99) as usize;

    (
        latencies[p50_idx],
        latencies[p95_idx.min(latencies.len() - 1)],
        latencies[p99_idx.min(latencies.len() - 1)],
    )
}

// ============================================================================
// Recall Calculation
// ============================================================================

/// Calculate Recall@K: percentage of ground truth results found by algorithm
///
/// Recall@K = |algorithm_results intersection ground_truth| / K
///
/// Example:
///   Ground Truth: [5, 12, 23, 31, 42]
///   Algorithm:    [5, 12, 99, 31, 42]  (99 is wrong)
///   Overlap: 4 out of 5
///   Recall@5 = 4/5 = 0.80 (80%)
pub fn calculate_recall(
    algorithm_results: &[(f32, usize)],
    ground_truth: &[(f32, usize)],
    k: usize,
) -> f32 {
    let algo_ids: HashSet<usize> = algorithm_results
        .iter()
        .take(k)
        .map(|(_, id)| *id)
        .collect();

    let truth_ids: HashSet<usize> = ground_truth.iter().take(k).map(|(_, id)| *id).collect();

    let intersection = algo_ids.intersection(&truth_ids).count();

    intersection as f32 / k as f32
}

// ============================================================================
// Warmup Phase
// ============================================================================

/// Run warmup queries to stabilize caches and branch predictor
///
/// Why warmup is critical:
/// 1. CPU caches are cold on first access
/// 2. OS page cache needs to load data from disk
/// 3. Branch predictor needs to learn patterns
/// 4. Memory allocator needs to establish heap patterns
///
/// Without warmup, first query can be 10-100x slower than steady-state.
pub fn warmup_phase<F>(search_fn: &mut F, queries: &[Vec<f32>], num_warmup: usize, k: usize)
where
    F: FnMut(&[f32], usize) -> Vec<(f32, usize)>,
{
    println!("  Running {} warmup queries...", num_warmup);

    let warmup_start = Instant::now();

    for i in 0..num_warmup {
        let query = &queries[i % queries.len()];
        let _ = search_fn(query, k);
    }

    let warmup_duration = warmup_start.elapsed();

    println!(
        "  Warmup complete ({:.2}s)",
        warmup_duration.as_secs_f64()
    );
}

// ============================================================================
// Main Benchmark Runner
// ============================================================================

/// Run a complete benchmark with warmup, timing, and metrics calculation
///
/// # Arguments
/// * `algo_name` - Name of the algorithm for reporting
/// * `config` - Benchmark configuration
/// * `queries` - Query vectors to test
/// * `ground_truth` - Optional ground truth results for recall calculation
/// * `build_time` - Time taken to build the index
/// * `memory_mb` - Memory usage in MB
/// * `search_fn` - Function that performs search (query, k) -> results
pub fn run_benchmark<F>(
    algo_name: &str,
    config: BenchmarkConfig,
    queries: &[Vec<f32>],
    ground_truth: Option<&[Vec<(f32, usize)>]>,
    build_time: Duration,
    memory_mb: f64,
    mut search_fn: F,
) -> BenchmarkResult
where
    F: FnMut(&[f32], usize) -> Vec<(f32, usize)>,
{
    println!("\n┌────────────────────────────────────────────────────────┐");
    println!("│ Benchmarking: {}", algo_name);
    println!("└────────────────────────────────────────────────────────┘");

    // Warmup phase
    warmup_phase(&mut search_fn, queries, config.warmup_queries, config.k);

    // Measurement phase
    println!("  Running {} measurement queries...", config.num_queries);

    let mut latencies = Vec::with_capacity(config.num_queries);
    let mut total_recall = 0.0;
    let measurement_start = Instant::now();

    for (i, query) in queries.iter().enumerate().take(config.num_queries) {
        let query_start = Instant::now();
        let results = search_fn(query, config.k);
        let query_duration = query_start.elapsed();

        latencies.push(query_duration);

        // Calculate recall if ground truth available
        if let Some(truth) = ground_truth {
            let recall = calculate_recall(&results, &truth[i], config.k);
            total_recall += recall;
        } else {
            total_recall += 1.0; // Assume perfect recall if no ground truth
        }
    }

    let total_measurement_time = measurement_start.elapsed();

    // Calculate statistics
    let avg_latency = total_measurement_time / config.num_queries as u32;
    let (p50, p95, p99) = calculate_percentiles(latencies.clone());
    let min_latency = *latencies.iter().min().unwrap();
    let max_latency = *latencies.iter().max().unwrap();
    let qps = config.num_queries as f64 / total_measurement_time.as_secs_f64();
    let avg_recall = total_recall / config.num_queries as f32;

    println!("  Benchmark complete");
    println!(
        "    Average latency: {:.2} ms",
        avg_latency.as_secs_f64() * 1000.0
    );
    println!("    P99 latency:     {:.2} ms", p99.as_secs_f64() * 1000.0);
    println!("    Throughput:      {:.0} QPS", qps);
    println!("    Recall@{}:        {:.1}%", config.k, avg_recall * 100.0);

    BenchmarkResult {
        algo_name: algo_name.to_string(),
        config,
        avg_latency,
        p50_latency: p50,
        p95_latency: p95,
        p99_latency: p99,
        min_latency,
        max_latency,
        qps,
        avg_recall,
        build_time,
        memory_mb,
    }
}

// ============================================================================
// Comparison Utilities
// ============================================================================

/// Print a comparison table between two algorithms
pub fn print_comparison(baseline: &BenchmarkResult, candidate: &BenchmarkResult) {
    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║  {} vs {}", baseline.algo_name, candidate.algo_name);
    println!("╠════════════════════════════════════════════════════════╣");

    let speedup = baseline.avg_latency.as_secs_f64() / candidate.avg_latency.as_secs_f64();
    let build_diff = candidate.build_time.as_secs_f64() - baseline.build_time.as_secs_f64();
    let memory_diff = candidate.memory_mb - baseline.memory_mb;
    let memory_overhead = (memory_diff / baseline.memory_mb) * 100.0;
    let recall_diff = candidate.avg_recall - baseline.avg_recall;

    println!("║ Speedup:       {:.1}x faster", speedup);
    println!("║ Build Cost:    +{:.2}s", build_diff);
    println!(
        "║ Memory Cost:   +{:.1} MB ({:.0}% overhead)",
        memory_diff, memory_overhead
    );
    println!("║ Recall Loss:   {:.1}%", recall_diff * 100.0);
    println!("╠════════════════════════════════════════════════════════╣");

    // Verdict
    if speedup > 10.0 && recall_diff > -0.05 {
        println!(
            "║ Verdict:       {} is clearly superior",
            candidate.algo_name
        );
    } else if speedup > 2.0 && recall_diff > -0.02 {
        println!(
            "║ Verdict:       {} recommended for production",
            candidate.algo_name
        );
    } else if speedup < 1.5 {
        println!(
            "║ Verdict:       {} simpler, speedup marginal",
            baseline.algo_name
        );
    } else {
        println!("║ Verdict:       Trade-off depends on use case");
    }

    println!("╚════════════════════════════════════════════════════════╝");
}

// ============================================================================
// Test Data Generation
// ============================================================================

/// Generate random vectors for benchmarking
/// Uses uniform distribution in [-1, 1] range
pub fn generate_random_vectors(n: usize, dims: usize) -> Vec<Vec<f32>> {
    let mut rng = thread_rng();
    (0..n)
        .map(|_| {
            (0..dims)
                .map(|_| rng.gen::<f32>() * 2.0 - 1.0) // Range: [-1, 1]
                .collect()
        })
        .collect()
}

/// Estimate memory usage of raw vectors
pub fn estimate_vector_memory(n: usize, dims: usize) -> f64 {
    let bytes_per_vector = dims * std::mem::size_of::<f32>();
    let total_bytes = n * bytes_per_vector;
    total_bytes as f64 / 1_000_000.0 // Convert to MB
}

// ============================================================================
// Example Usage
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentile_calculation() {
        let latencies = vec![
            Duration::from_millis(1),
            Duration::from_millis(2),
            Duration::from_millis(3),
            Duration::from_millis(4),
            Duration::from_millis(5),
            Duration::from_millis(6),
            Duration::from_millis(7),
            Duration::from_millis(8),
            Duration::from_millis(9),
            Duration::from_millis(10),
        ];

        let (p50, p95, p99) = calculate_percentiles(latencies);

        assert_eq!(p50, Duration::from_millis(5));
        assert_eq!(p95, Duration::from_millis(10)); // 0.95 * 10 = 9.5, index 9
        assert_eq!(p99, Duration::from_millis(10)); // 0.99 * 10 = 9.9, index 9
    }

    #[test]
    fn test_recall_calculation() {
        let algo_results = vec![
            (0.1, 5),
            (0.2, 12),
            (0.3, 99), // Wrong
            (0.4, 31),
            (0.5, 42),
        ];

        let ground_truth = vec![
            (0.1, 5),
            (0.2, 12),
            (0.3, 23), // Correct answer
            (0.4, 31),
            (0.5, 42),
        ];

        let recall = calculate_recall(&algo_results, &ground_truth, 5);
        assert_eq!(recall, 0.8); // 4 out of 5 correct
    }

    #[test]
    fn test_recall_perfect() {
        let results = vec![(0.1, 1), (0.2, 2), (0.3, 3)];
        let recall = calculate_recall(&results, &results, 3);
        assert_eq!(recall, 1.0);
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         Benchmark Harness Demo                            ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Demo: Benchmark a simple linear search
    let config = BenchmarkConfig::new(1000, 128);
    let vectors = generate_random_vectors(config.dataset_size, config.dimensions);
    let queries = generate_random_vectors(config.num_queries, 128);

    // Simple euclidean distance function
    let euclidean_distance = |a: &[f32], b: &[f32]| -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    };

    // Brute force search closure
    let mut brute_search = |query: &[f32], k: usize| -> Vec<(f32, usize)> {
        let mut distances: Vec<_> = vectors
            .iter()
            .enumerate()
            .map(|(id, vec)| (euclidean_distance(query, vec), id))
            .collect();

        distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        distances.truncate(k);
        distances
    };

    let memory_mb = estimate_vector_memory(config.dataset_size, config.dimensions);

    let result = run_benchmark(
        "Brute Force Demo",
        config,
        &queries,
        None,           // No ground truth (it IS the ground truth)
        Duration::ZERO, // No build time
        memory_mb,
        brute_search,
    );

    result.print_table();

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("  ║  Key Insight: Proper benchmarking requires:                ║");
    println!("  ║  1. Warmup phase (100+ queries)                            ║");
    println!("  ║  2. Percentile metrics (P95, P99 not just average)         ║");
    println!("  ║  3. Recall measurement (quality vs speed)                  ║");
    println!("  ║  4. Diverse queries (avoid cache hits)                     ║");
    println!("  ╚════════════════════════════════════════════════════════════╝");
}
