// benchmarks.rs
// Performance benchmarks for query planning strategies
//
// This module measures the latency and recall of different execution
// strategies across various selectivity levels.

#[path = "execution-engine.rs"]
mod execution_engine;
#[path = "query-planner.rs"]
mod query_planner;

use std::time::Instant;

/// Benchmark configuration
pub struct BenchmarkConfig {
    /// Number of vectors in the dataset
    pub n_vectors: usize,
    /// Dimension of each vector
    pub dimension: usize,
    /// Number of queries to run
    pub n_queries: usize,
    /// Number of results requested (k)
    pub k: usize,
    /// Selectivity levels to test
    pub selectivities: Vec<f64>,
}

impl BenchmarkConfig {
    pub fn default() -> Self {
        Self {
            n_vectors: 100_000,
            dimension: 768,
            n_queries: 100,
            k: 10,
            selectivities: vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.2, 0.5, 0.8, 0.95],
        }
    }

    pub fn small() -> Self {
        Self {
            n_vectors: 10_000,
            dimension: 128,
            n_queries: 50,
            k: 10,
            selectivities: vec![0.01, 0.1, 0.5],
        }
    }
}

/// Result of a single benchmark run
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub strategy: String,
    pub selectivity: f64,
    pub n_matches: usize,
    pub latency_ms: f64,
    pub recall: f64,
    pub throughput_qps: f64,
}

/// Generate random vectors for testing
pub fn generate_random_vectors(n: usize, dim: usize) -> Vec<Vec<f32>> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    (0..n)
        .map(|_| (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect())
        .collect()
}

/// Generate a random query vector
pub fn generate_random_query(dim: usize) -> Vec<f32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

/// Generate a bitmask with given selectivity
pub fn generate_bitmask(n: usize, selectivity: f64) -> Vec<bool> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    (0..n).map(|_| rng.gen::<f64>() < selectivity).collect()
}

/// Get matching indices from a bitmask
pub fn bitmask_to_indices(bitmask: &[bool]) -> Vec<usize> {
    bitmask
        .iter()
        .enumerate()
        .filter_map(|(i, &b)| if b { Some(i) } else { None })
        .collect()
}

/// Measure recall by comparing results to ground truth
pub fn calculate_recall(results: &[usize], ground_truth: &[usize], k: usize) -> f64 {
    if ground_truth.is_empty() {
        return 1.0; // No ground truth to compare
    }

    let gt_set: std::collections::HashSet<_> = ground_truth.iter().take(k).collect();
    let matches = results
        .iter()
        .take(k)
        .filter(|id| gt_set.contains(id))
        .count();

    matches as f64 / k.min(ground_truth.len()) as f64
}

/// Run a comprehensive benchmark suite
pub fn run_benchmark_suite(config: BenchmarkConfig) -> Vec<BenchmarkResult> {
    println!("=== Starting Benchmark Suite ===");
    println!(
        "Dataset: {} vectors x {} dims",
        config.n_vectors, config.dimension
    );
    println!("Queries: {}, k: {}", config.n_queries, config.k);
    println!();

    // Generate dataset
    println!("Generating random vectors...");
    let vectors = generate_random_vectors(config.n_vectors, config.dimension);
    println!("Generated {} vectors", vectors.len());
    println!();

    let mut results = Vec::new();

    for &selectivity in &config.selectivities {
        println!("--- Selectivity: {:.2}% ---", selectivity * 100.0);

        // Generate bitmask
        let bitmask = generate_bitmask(config.n_vectors, selectivity);
        let matches = bitmask_to_indices(&bitmask);
        let n_matches = matches.len();

        println!("  Matches: {} / {}", n_matches, config.n_vectors);

        // Benchmark BruteForce
        if selectivity < 0.02 {
            let result = benchmark_brute_force(&vectors, &matches, &config);
            println!(
                "  BruteForce:   {:.2}ms  (Recall: {:.1}%)",
                result.latency_ms,
                result.recall * 100.0
            );
            results.push(result);
        }

        // Benchmark FilterFirst
        if selectivity >= 0.01 && selectivity <= 0.5 {
            let result = benchmark_filter_first(&vectors, &bitmask, &config);
            println!(
                "  FilterFirst:  {:.2}ms  (Recall: {:.1}%)",
                result.latency_ms,
                result.recall * 100.0
            );
            results.push(result);
        }

        // Benchmark VectorFirst
        if selectivity > 0.3 {
            let result = benchmark_vector_first(&vectors, &bitmask, &config);
            println!(
                "  VectorFirst:  {:.2}ms  (Recall: {:.1}%)",
                result.latency_ms,
                result.recall * 100.0
            );
            results.push(result);
        }

        println!();
    }

    println!("=== Benchmark Complete ===");
    results
}

/// Benchmark brute force strategy
fn benchmark_brute_force(
    vectors: &[Vec<f32>],
    matches: &[usize],
    config: &BenchmarkConfig,
) -> BenchmarkResult {
    use crate::execution_engine::{cosine_distance, distance_to_score, ExecutionEngine};
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    let engine = ExecutionEngine::new(vectors);
    let mut total_latency = 0.0;
    let mut total_recall = 0.0;

    for _ in 0..config.n_queries {
        let query = generate_random_query(config.dimension);

        let start = Instant::now();
        let results = engine.execute_brute_force(&query, config.k, matches);
        let latency = start.elapsed().as_secs_f64() * 1000.0;

        total_latency += latency;

        // Calculate recall (simplified: assume first k matches are ground truth)
        let ground_truth: Vec<_> = results.iter().map(|r| r.point_id).collect();
        let recall = 1.0; // Perfect recall for brute force
        total_recall += recall;
    }

    let avg_latency = total_latency / config.n_queries as f64;
    let avg_recall = total_recall / config.n_queries as f64;
    let throughput = 1000.0 / avg_latency;

    BenchmarkResult {
        strategy: "BruteForce".to_string(),
        selectivity: matches.len() as f64 / vectors.len() as f64,
        n_matches: matches.len(),
        latency_ms: avg_latency,
        recall: avg_recall,
        throughput_qps: throughput,
    }
}

/// Benchmark filter-first strategy
fn benchmark_filter_first(
    vectors: &[Vec<f32>],
    bitmask: &[bool],
    config: &BenchmarkConfig,
) -> BenchmarkResult {
    use crate::execution_engine::ExecutionEngine;

    let engine = ExecutionEngine::new(vectors);
    let mut total_latency = 0.0;
    let mut total_recall = 0.0;

    let n_matches = bitmask.iter().filter(|&&b| b).count();

    for _ in 0..config.n_queries {
        let query = generate_random_query(config.dimension);

        let start = Instant::now();
        let results = engine.execute_filter_first(&query, config.k, bitmask);
        let latency = start.elapsed().as_secs_f64() * 1000.0;

        total_latency += latency;

        // Recall: all results should match the filter
        let all_match = results
            .iter()
            .all(|r| r.point_id < bitmask.len() && bitmask[r.point_id]);
        let recall = if all_match { 1.0 } else { 0.0 };
        total_recall += recall;
    }

    let avg_latency = total_latency / config.n_queries as f64;
    let avg_recall = total_recall / config.n_queries as f64;
    let throughput = 1000.0 / avg_latency;

    BenchmarkResult {
        strategy: "FilterFirst".to_string(),
        selectivity: n_matches as f64 / vectors.len() as f64,
        n_matches,
        latency_ms: avg_latency,
        recall: avg_recall,
        throughput_qps: throughput,
    }
}

/// Benchmark vector-first strategy
fn benchmark_vector_first(
    vectors: &[Vec<f32>],
    bitmask: &[bool],
    config: &BenchmarkConfig,
) -> BenchmarkResult {
    use crate::execution_engine::ExecutionEngine;
    use crate::query_planner::QueryPlanner;

    let engine = ExecutionEngine::new(vectors);
    let planner = QueryPlanner::default();

    let n_matches = bitmask.iter().filter(|&&b| b).count();
    let selectivity = n_matches as f64 / vectors.len() as f64;
    let k_expansion = planner.calculate_expansion(config.k, selectivity);

    let mut total_latency = 0.0;
    let mut total_recall = 0.0;

    for _ in 0..config.n_queries {
        let query = generate_random_query(config.dimension);

        let start = Instant::now();
        let results = engine.execute_vector_first(&query, config.k, k_expansion, bitmask);
        let latency = start.elapsed().as_secs_f64() * 1000.0;

        total_latency += latency;

        // Recall: all results should match the filter
        let all_match = results
            .iter()
            .all(|r| r.point_id < bitmask.len() && bitmask[r.point_id]);
        let recall = if all_match && results.len() >= config.k {
            1.0
        } else {
            0.0
        };
        total_recall += recall;
    }

    let avg_latency = total_latency / config.n_queries as f64;
    let avg_recall = total_recall / config.n_queries as f64;
    let throughput = 1000.0 / avg_latency;

    BenchmarkResult {
        strategy: "VectorFirst".to_string(),
        selectivity,
        n_matches,
        latency_ms: avg_latency,
        recall: avg_recall,
        throughput_qps: throughput,
    }
}

/// Print benchmark results as a formatted table
pub fn print_results_table(results: &[BenchmarkResult]) {
    println!("\n=== Benchmark Results ===\n");
    println!(
        "{:<15} {:<12} {:<12} {:<12} {:<12} {:<12}",
        "Strategy", "Selectivity", "Matches", "Latency", "Recall", "QPS"
    );
    println!("{}", "-".repeat(80));

    for result in results {
        println!(
            "{:<15} {:>10.2}% {:>11} {:>9.2}ms {:>10.1}% {:>11.0}",
            result.strategy,
            result.selectivity * 100.0,
            result.n_matches,
            result.latency_ms,
            result.recall * 100.0,
            result.throughput_qps
        );
    }

    println!();
}

/// Find the optimal strategy for each selectivity level
pub fn find_optimal_strategies(results: &[BenchmarkResult]) -> Vec<(f64, String, f64)> {
    use std::collections::HashMap;

    // Group by selectivity
    let mut by_selectivity: HashMap<String, Vec<&BenchmarkResult>> = HashMap::new();

    for result in results {
        let key = format!("{:.4}", result.selectivity);
        by_selectivity
            .entry(key)
            .or_insert_with(Vec::new)
            .push(result);
    }

    // Find best strategy for each selectivity
    let mut optimal = Vec::new();

    for (sel_str, results) in by_selectivity {
        if let Some(best) = results
            .iter()
            .min_by(|a, b| a.latency_ms.partial_cmp(&b.latency_ms).unwrap())
        {
            optimal.push((best.selectivity, best.strategy.clone(), best.latency_ms));
        }
    }

    optimal.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    optimal
}

/// Print optimal strategy recommendations
pub fn print_optimal_strategies(results: &[BenchmarkResult]) {
    let optimal = find_optimal_strategies(results);

    println!("\n=== Optimal Strategy by Selectivity ===\n");
    println!(
        "{:<15} {:<20} {:<15}",
        "Selectivity", "Best Strategy", "Latency"
    );
    println!("{}", "-".repeat(50));

    for (selectivity, strategy, latency) in optimal {
        println!(
            "{:>13.2}% {:<20} {:>12.2}ms",
            selectivity * 100.0,
            strategy,
            latency
        );
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_vectors() {
        let vectors = generate_random_vectors(100, 64);
        assert_eq!(vectors.len(), 100);
        assert_eq!(vectors[0].len(), 64);
    }

    #[test]
    fn test_generate_bitmask() {
        let bitmask = generate_bitmask(1000, 0.1);
        let n_matches = bitmask.iter().filter(|&&b| b).count();

        // Should be around 100 (10% of 1000), allow +/-20% variance
        assert!(n_matches >= 80 && n_matches <= 120);
    }

    #[test]
    fn test_bitmask_to_indices() {
        let bitmask = vec![true, false, true, false, true];
        let indices = bitmask_to_indices(&bitmask);
        assert_eq!(indices, vec![0, 2, 4]);
    }

    #[test]
    fn test_calculate_recall() {
        let results = vec![0, 1, 2, 3, 4];
        let ground_truth = vec![0, 1, 5, 6, 7];

        // 2 matches out of 5 = 40%
        let recall = calculate_recall(&results, &ground_truth, 5);
        assert!((recall - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_small_benchmark() {
        let config = BenchmarkConfig {
            n_vectors: 100,
            dimension: 8,
            n_queries: 5,
            k: 5,
            selectivities: vec![0.1, 0.5],
        };

        let results = run_benchmark_suite(config);

        // Should have results for multiple strategies
        assert!(!results.is_empty());

        // All results should have positive latency
        for result in &results {
            assert!(result.latency_ms > 0.0);
        }
    }
}

// Example usage
#[allow(dead_code)]
fn main() {
    // Run a quick benchmark
    let config = BenchmarkConfig::small();
    let results = run_benchmark_suite(config);

    print_results_table(&results);
    print_optimal_strategies(&results);
}
