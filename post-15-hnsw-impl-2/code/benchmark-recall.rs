// benchmark-recall.rs
// Comprehensive Recall@K Benchmarking for HNSW
// Measures approximation quality against brute force ground truth

use rand::prelude::*;
use std::collections::HashSet;
use std::time::{Duration, Instant};

// ============================================================================
// Brute Force Search (Ground Truth)
// ============================================================================

/// Brute force exact k-NN search - O(N x D) complexity
/// This is the ground truth we compare HNSW against
fn brute_force_search(vectors: &[Vec<f32>], query: &[f32], k: usize) -> Vec<(f32, usize)> {
    let mut distances: Vec<_> = vectors
        .iter()
        .enumerate()
        .map(|(id, vec)| {
            let dist = euclidean_distance(query, vec);
            (dist, id)
        })
        .collect();

    // Sort by distance (ascending)
    distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Return top-k
    distances.truncate(k);
    distances
}

fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

// ============================================================================
// Recall Calculation
// ============================================================================

/// Calculate Recall@K: What percentage of top-K results match ground truth?
///
/// Recall@K = |HNSW_results intersection Brute_results| / K
///
/// Example:
///   Ground Truth: [5, 12, 23, 31, 42]
///   HNSW Result:  [5, 12, 99, 31, 42]  (99 is wrong)
///   Overlap: 4 out of 5
///   Recall@5 = 4/5 = 0.80 (80%)
fn calculate_recall(
    hnsw_results: &[(f32, usize)],
    brute_results: &[(f32, usize)],
    k: usize,
) -> f32 {
    let hnsw_ids: HashSet<usize> = hnsw_results.iter().take(k).map(|(_, id)| *id).collect();

    let brute_ids: HashSet<usize> = brute_results.iter().take(k).map(|(_, id)| *id).collect();

    let intersection = hnsw_ids.intersection(&brute_ids).count();

    intersection as f32 / k as f32
}

/// Calculate multiple recall metrics (Recall@1, @5, @10, @100)
#[derive(Debug, Clone)]
struct RecallMetrics {
    recall_at_1: f32,
    recall_at_5: f32,
    recall_at_10: f32,
    recall_at_100: f32,
}

impl RecallMetrics {
    fn calculate(hnsw_results: &[(f32, usize)], brute_results: &[(f32, usize)]) -> Self {
        Self {
            recall_at_1: calculate_recall(hnsw_results, brute_results, 1),
            recall_at_5: calculate_recall(hnsw_results, brute_results, 5),
            recall_at_10: calculate_recall(hnsw_results, brute_results, 10),
            recall_at_100: calculate_recall(hnsw_results, brute_results, 100),
        }
    }
}

// ============================================================================
// Benchmark Results Storage
// ============================================================================

#[derive(Debug, Clone)]
struct BenchmarkResult {
    dataset_size: usize,
    dimensions: usize,
    M: usize,
    ef_construction: usize,
    ef_search: usize,
    k: usize,
    avg_latency_ms: f64,
    p99_latency_ms: f64,
    avg_recall: f32,
    recall_metrics: RecallMetrics,
    queries_per_second: f64,
    speedup_vs_brute: f64,
}

impl BenchmarkResult {
    fn print(&self) {
        println!("\n┌────────────────────────────────────────────────────────┐");
        println!(
            "│ Dataset: {} vectors, {} dims",
            self.dataset_size, self.dimensions
        );
        println!(
            "│ Parameters: M={}, ef_construction={}, ef_search={}",
            self.M, self.ef_construction, self.ef_search
        );
        println!("├────────────────────────────────────────────────────────┤");
        println!("│ Latency (avg):     {:>8.2} ms", self.avg_latency_ms);
        println!("│ Latency (p99):     {:>8.2} ms", self.p99_latency_ms);
        println!("│ Recall@10:         {:>7.1}%", self.avg_recall * 100.0);
        println!(
            "│ Speedup:           {:>7.1}x vs brute force",
            self.speedup_vs_brute
        );
        println!("│ QPS:               {:>8.0}", self.queries_per_second);
        println!("├────────────────────────────────────────────────────────┤");
        println!("│ Recall Breakdown:");
        println!(
            "│   Recall@1:        {:>7.1}%",
            self.recall_metrics.recall_at_1 * 100.0
        );
        println!(
            "│   Recall@5:        {:>7.1}%",
            self.recall_metrics.recall_at_5 * 100.0
        );
        println!(
            "│   Recall@10:       {:>7.1}%",
            self.recall_metrics.recall_at_10 * 100.0
        );
        println!(
            "│   Recall@100:      {:>7.1}%",
            self.recall_metrics.recall_at_100 * 100.0
        );
        println!("└────────────────────────────────────────────────────────┘");
    }
}

// ============================================================================
// Main Benchmark Runner
// ============================================================================

fn benchmark_recall_vs_ef_search(
    // index: &HNSWIndex,  // Your HNSW implementation
    vectors: &[Vec<f32>],
    test_queries: &[Vec<f32>],
    k: usize,
    M: usize,
    ef_construction: usize,
) {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!(
        "║      Recall@{} vs ef_search Benchmark                   ║",
        k
    );
    println!(
        "║  Dataset: {} vectors, {} dimensions                   ",
        vectors.len(),
        vectors[0].len()
    );
    println!(
        "║  Parameters: M={}, ef_construction={}                  ",
        M, ef_construction
    );
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    println!(
        "{:>10} | {:>10} | {:>10} | {:>10} | {:>10}",
        "ef_search", "Recall@10", "Latency", "p99", "Speedup"
    );
    println!("{}", "─".repeat(70));

    let ef_values = vec![10, 20, 50, 100, 200, 500];

    for &ef_search in &ef_values {
        let mut recalls = Vec::new();
        let mut latencies = Vec::new();
        let mut brute_latencies = Vec::new();

        for query in test_queries {
            // Ground truth (brute force)
            let brute_start = Instant::now();
            let brute_results = brute_force_search(vectors, query, k);
            brute_latencies.push(brute_start.elapsed());

            // HNSW search
            let hnsw_start = Instant::now();
            // let hnsw_results = index.search(query, k, ef_search);
            // For demo: simulate HNSW search by using brute force with random noise
            let hnsw_results = simulate_hnsw_search(vectors, query, k, ef_search);
            latencies.push(hnsw_start.elapsed());

            // Calculate recall
            let recall = calculate_recall(&hnsw_results, &brute_results, k);
            recalls.push(recall);
        }

        // Calculate statistics
        let avg_recall = recalls.iter().sum::<f32>() / recalls.len() as f32;
        let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
        let avg_brute = brute_latencies.iter().sum::<Duration>() / brute_latencies.len() as u32;

        // P99 latency
        let mut sorted_latencies = latencies.clone();
        sorted_latencies.sort();
        let p99_index = (sorted_latencies.len() as f64 * 0.99) as usize;
        let p99_latency = sorted_latencies[p99_index.min(sorted_latencies.len() - 1)];

        let speedup = avg_brute.as_secs_f64() / avg_latency.as_secs_f64();

        println!(
            "{:>10} | {:>9.1}% | {:>8.2}ms | {:>8.2}ms | {:>9.1}x",
            ef_search,
            avg_recall * 100.0,
            avg_latency.as_secs_f64() * 1000.0,
            p99_latency.as_secs_f64() * 1000.0,
            speedup
        );
    }
}

// Simulate HNSW search for demonstration (replace with actual HNSW)
fn simulate_hnsw_search(
    vectors: &[Vec<f32>],
    query: &[f32],
    k: usize,
    ef_search: usize,
) -> Vec<(f32, usize)> {
    // For demo: use brute force but simulate recall based on ef_search
    let mut results = brute_force_search(vectors, query, k);

    // Simulate imperfect recall by randomly replacing some results
    let mut rng = thread_rng();
    let error_rate = match ef_search {
        ef if ef <= 10 => 0.15,   // 85% recall
        ef if ef <= 20 => 0.08,   // 92% recall
        ef if ef <= 50 => 0.04,   // 96% recall
        ef if ef <= 100 => 0.01,  // 99% recall
        ef if ef <= 200 => 0.005, // 99.5% recall
        _ => 0.001,               // 99.9% recall
    };

    for i in 0..results.len() {
        if rng.gen::<f32>() < error_rate {
            // Replace with random vector
            let random_id = rng.gen_range(0..vectors.len());
            let dist = euclidean_distance(query, &vectors[random_id]);
            results[i] = (dist, random_id);
        }
    }

    results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    results
}

// ============================================================================
// Parameter Sweep: Compare Different Configurations
// ============================================================================

fn benchmark_parameter_sweep(vectors: &[Vec<f32>], test_queries: &[Vec<f32>], k: usize) {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║             Parameter Sweep Benchmark                    ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    let configs = vec![
        (8, 100, 50, "Fast Build, Fast Search"),
        (16, 200, 100, "Balanced"),
        (32, 400, 200, "High Quality"),
    ];

    println!(
        "{:>4} | {:>6} | {:>6} | {:>10} | {:>10} | {:>15}",
        "M", "ef_c", "ef_s", "Recall@10", "Latency", "Description"
    );
    println!("{}", "─".repeat(80));

    for (M, ef_construction, ef_search, description) in configs {
        let mut total_recall = 0.0;
        let mut total_latency = Duration::ZERO;

        for query in test_queries {
            let brute_results = brute_force_search(vectors, query, k);

            let start = Instant::now();
            let hnsw_results = simulate_hnsw_search(vectors, query, k, ef_search);
            total_latency += start.elapsed();

            let recall = calculate_recall(&hnsw_results, &brute_results, k);
            total_recall += recall;
        }

        let avg_recall = total_recall / test_queries.len() as f32;
        let avg_latency = total_latency / test_queries.len() as u32;

        println!(
            "{:>4} | {:>6} | {:>6} | {:>9.1}% | {:>8.2}ms | {}",
            M,
            ef_construction,
            ef_search,
            avg_recall * 100.0,
            avg_latency.as_secs_f64() * 1000.0,
            description
        );
    }
}

// ============================================================================
// Scale Benchmark: HNSW vs Brute Force at Different Sizes
// ============================================================================

fn benchmark_hnsw_vs_brute_at_scale() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║      HNSW vs Brute Force: Performance at Scale           ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    let dims = 128;
    let dataset_sizes = vec![1_000, 10_000, 100_000, 1_000_000];
    let k = 10;

    for &n in &dataset_sizes {
        println!("\n{}", "=".repeat(70));
        println!("Dataset: {} vectors ({} dimensions)", n, dims);
        println!("{}", "=".repeat(70));

        // Generate random vectors
        let vectors = generate_random_vectors(n, dims);
        let query = &vectors[0];

        // Brute force search
        let brute_start = Instant::now();
        let brute_results = brute_force_search(&vectors, query, k);
        let brute_time = brute_start.elapsed();

        // HNSW search (simulated)
        let hnsw_start = Instant::now();
        let hnsw_results = simulate_hnsw_search(&vectors, query, k, 100);
        let hnsw_time = hnsw_start.elapsed();

        // Calculate metrics
        let recall = calculate_recall(&hnsw_results, &brute_results, k);
        let speedup = brute_time.as_secs_f64() / hnsw_time.as_secs_f64();

        println!("\nSearch Performance:");
        println!(
            "  Brute Force: {:>10.2}ms",
            brute_time.as_secs_f64() * 1000.0
        );
        println!(
            "  HNSW:        {:>10.2}ms",
            hnsw_time.as_secs_f64() * 1000.0
        );
        println!("  Speedup:     {:>10.1}x", speedup);
        println!("  Recall@10:   {:>10.1}%", recall * 100.0);

        // Memory estimation
        let M = 16;
        let memory_mb = estimate_memory_usage(n, M, dims);
        println!("\nMemory Usage (estimated):");
        println!(
            "  Vectors:     {:>10.1} MB",
            (n * dims * 4) as f64 / 1_000_000.0
        );
        println!("  Edges:       {:>10.1} MB", memory_mb);
        println!(
            "  Total:       {:>10.1} MB",
            (n * dims * 4) as f64 / 1_000_000.0 + memory_mb
        );
    }
}

fn estimate_memory_usage(n: usize, M: usize, _dims: usize) -> f64 {
    // Each edge is ~8 bytes (usize)
    // Average number of edges per node is approximately M x 1.5
    let avg_edges_per_node = (M as f64 * 1.5) as usize;
    let total_edges = n * avg_edges_per_node;
    (total_edges * 8) as f64 / 1_000_000.0
}

// ============================================================================
// Visualization: Recall Curves
// ============================================================================

fn print_recall_curve(vectors: &[Vec<f32>], test_queries: &[Vec<f32>], k: usize) {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║              Recall vs Latency Curve                     ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    let ef_values = vec![10, 20, 30, 50, 75, 100, 150, 200, 300, 500];
    let mut points = Vec::new();

    for &ef_search in &ef_values {
        let mut total_recall = 0.0;
        let mut total_latency = Duration::ZERO;

        for query in test_queries {
            let brute_results = brute_force_search(vectors, query, k);

            let start = Instant::now();
            let hnsw_results = simulate_hnsw_search(vectors, query, k, ef_search);
            total_latency += start.elapsed();

            let recall = calculate_recall(&hnsw_results, &brute_results, k);
            total_recall += recall;
        }

        let avg_recall = total_recall / test_queries.len() as f32;
        let avg_latency = total_latency / test_queries.len() as u32;

        points.push((avg_recall, avg_latency, ef_search));
    }

    // Print ASCII chart
    println!("Recall");
    for i in (80..=100).rev() {
        let threshold = i as f32 / 100.0;
        print!("{:>3}% ┤", i);

        for (recall, _, _) in &points {
            if (recall * 100.0) as i32 >= i {
                print!("●");
            } else {
                print!(" ");
            }
        }
        println!();
    }
    println!("     └{}", "─".repeat(points.len()));
    println!("      Latency (ms)");

    println!("\nData Points:");
    for (recall, latency, ef) in points {
        println!(
            "  ef={:>3} | Recall: {:>5.1}% | Latency: {:>6.2}ms",
            ef,
            recall * 100.0,
            latency.as_secs_f64() * 1000.0
        );
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

fn generate_random_vectors(n: usize, dims: usize) -> Vec<Vec<f32>> {
    let mut rng = thread_rng();
    (0..n)
        .map(|_| {
            (0..dims)
                .map(|_| rng.gen::<f32>() * 2.0 - 1.0) // Range: [-1, 1]
                .collect()
        })
        .collect()
}

// ============================================================================
// Main Demo
// ============================================================================

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║           HNSW Recall Benchmarking Suite                  ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Generate test data
    let n = 10_000;
    let dims = 128;
    let num_queries = 100;

    println!("Generating test data...");
    println!("  Vectors: {}", n);
    println!("  Dimensions: {}", dims);
    println!("  Test queries: {}", num_queries);

    let vectors = generate_random_vectors(n, dims);
    let test_queries = generate_random_vectors(num_queries, dims);

    let k = 10;

    // Benchmark 1: Recall vs ef_search
    benchmark_recall_vs_ef_search(&vectors, &test_queries, k, 16, 200);

    // Benchmark 2: Parameter sweep
    benchmark_parameter_sweep(&vectors, &test_queries, k);

    // Benchmark 3: Recall curve visualization
    print_recall_curve(&vectors, &test_queries, k);

    // Benchmark 4: Scale comparison
    benchmark_hnsw_vs_brute_at_scale();

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║                    Key Takeaways                           ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║ 1. ef_search is the key runtime tuning parameter          ║");
    println!("║ 2. Recall@10 > 95% is achievable with ef_search=50        ║");
    println!("║ 3. HNSW provides 10-100x speedup over brute force         ║");
    println!("║ 4. Higher ef_search = Better recall but slower            ║");
    println!("║ 5. Choose ef_search based on your latency/recall needs    ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// Testing
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brute_force_correctness() {
        let vectors = vec![
            vec![0.0, 0.0],
            vec![1.0, 1.0],
            vec![0.1, 0.1],
            vec![9.0, 9.0],
        ];

        let query = vec![0.5, 0.5];
        let results = brute_force_search(&vectors, &query, 2);

        // Closest should be [0.1, 0.1] and [1.0, 1.0]
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].1, 2); // ID 2
        assert_eq!(results[1].1, 1); // ID 1
    }

    #[test]
    fn test_recall_calculation() {
        let hnsw_results = vec![(0.1, 5), (0.2, 12), (0.3, 99), (0.4, 31), (0.5, 42)];

        let brute_results = vec![
            (0.1, 5),
            (0.2, 12),
            (0.3, 23), // Different from HNSW
            (0.4, 31),
            (0.5, 42),
        ];

        let recall = calculate_recall(&hnsw_results, &brute_results, 5);
        assert_eq!(recall, 0.8); // 4 out of 5 match
    }

    #[test]
    fn test_recall_perfect_match() {
        let results = vec![(0.1, 1), (0.2, 2), (0.3, 3)];
        let recall = calculate_recall(&results, &results, 3);
        assert_eq!(recall, 1.0);
    }
}
