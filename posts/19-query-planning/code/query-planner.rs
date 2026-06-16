// query-planner.rs
// Cost-Based Optimizer for Hybrid Search
//
// This module implements a query planner that analyzes selectivity
// and chooses the optimal execution strategy: BruteForce, FilterFirst, or VectorFirst.

use std::error::Error;

/// Represents a chosen execution strategy for a hybrid search query
#[derive(Debug, Clone)]
pub enum ExecutionPlan {
    /// Scan only filtered documents (no HNSW)
    /// Best for: Selectivity < 1%
    BruteForce {
        /// DocIDs that match the filter
        matches: Vec<usize>,
        /// Estimated selectivity
        selectivity: f64,
    },

    /// Search HNSW first, then filter results
    /// Best for: Selectivity > 50%
    VectorFirst {
        /// How many results to fetch from HNSW (oversampled)
        k_expansion: usize,
        /// Estimated selectivity
        selectivity: f64,
    },

    /// Filter first, then search constrained HNSW
    /// Best for: Selectivity between 1% and 50%
    FilterFirst {
        /// Bitmask of allowed DocIDs
        bitmask: Vec<bool>,
        /// Estimated selectivity
        selectivity: f64,
    },
}

impl ExecutionPlan {
    /// Get the selectivity for this plan
    pub fn selectivity(&self) -> f64 {
        match self {
            ExecutionPlan::BruteForce { selectivity, .. } => *selectivity,
            ExecutionPlan::VectorFirst { selectivity, .. } => *selectivity,
            ExecutionPlan::FilterFirst { selectivity, .. } => *selectivity,
        }
    }

    /// Get a human-readable name for the strategy
    pub fn strategy_name(&self) -> &'static str {
        match self {
            ExecutionPlan::BruteForce { .. } => "BruteForce",
            ExecutionPlan::VectorFirst { .. } => "VectorFirst",
            ExecutionPlan::FilterFirst { .. } => "FilterFirst",
        }
    }
}

/// Cost-Based Query Planner
///
/// Analyzes a query's selectivity and chooses the optimal execution strategy.
///
/// # Strategy Selection
///
/// - **Selectivity < threshold_brute** => BruteForce (scan filtered docs)
/// - **Selectivity > threshold_pre** => VectorFirst (post-filtering)
/// - **Otherwise** => FilterFirst (pre-filtering with bitmask)
#[derive(Debug)]
pub struct QueryPlanner {
    pub threshold_brute: f64,

    /// Selectivity threshold for pre-filtering (e.g., 0.5 = 50%)
    pub threshold_pre: f64,

    /// Safety factor for k-expansion in VectorFirst (typically 1.5-2.0)
    pub safety_factor: f64,

    /// Maximum k' to prevent excessive HNSW traversal
    pub max_expansion: usize,
}

impl QueryPlanner {
    /// Create a new query planner with custom thresholds
    pub fn new(threshold_brute: f64, threshold_pre: f64, safety_factor: f64) -> Self {
        assert!(
            threshold_brute < threshold_pre,
            "threshold_brute must be < threshold_pre"
        );
        assert!(safety_factor >= 1.0, "safety_factor must be >= 1.0");

        Self {
            threshold_brute,
            threshold_pre,
            safety_factor,
            max_expansion: 10000,
        }
    }

    /// Create a planner with default thresholds based on empirical benchmarks
    ///
    /// - BruteForce: s < 1%
    /// - FilterFirst: 1% <= s <= 50%
    /// - VectorFirst: s > 50%
    pub fn default() -> Self {
        Self::new(0.01, 0.5, 1.5)
    }

    /// Analyze a query and choose the best execution plan
    ///
    /// # Arguments
    ///
    /// - `n_total`: Total number of documents in the index
    /// - `n_matches`: Number of documents matching the filter
    /// - `k`: Number of results requested by the user
    ///
    /// # Returns
    ///
    /// An `ExecutionPlan` with the optimal strategy
    pub fn plan_from_counts(&self, n_total: usize, n_matches: usize, k: usize) -> ExecutionPlan {
        // Calculate selectivity
        let s = if n_total == 0 {
            0.0
        } else {
            n_matches as f64 / n_total as f64
        };

        // Decision tree based on selectivity
        if s < self.threshold_brute {
            // Case 1: Tiny filter => Scan only valid IDs
            println!(
                "[Optimizer] Selectivity {:.2}% ({}/{}) => BruteForce",
                s * 100.0,
                n_matches,
                n_total
            );

            // We will need the actual DocIDs from the metadata index
            // For now, return a placeholder - the execution engine will fill this
            ExecutionPlan::BruteForce {
                matches: Vec::new(), // Filled by execution engine
                selectivity: s,
            }
        } else if s > self.threshold_pre {
            // Case 2: Broad filter => Search HNSW, then check
            let k_expansion = self.calculate_expansion(k, s);

            println!(
                "[Optimizer] Selectivity {:.2}% ({}/{}) => VectorFirst (k'={})",
                s * 100.0,
                n_matches,
                n_total,
                k_expansion
            );

            ExecutionPlan::VectorFirst {
                k_expansion,
                selectivity: s,
            }
        } else {
            // Case 3: Medium filter => Pre-filter HNSW
            println!(
                "[Optimizer] Selectivity {:.2}% ({}/{}) => FilterFirst",
                s * 100.0,
                n_matches,
                n_total
            );

            ExecutionPlan::FilterFirst {
                bitmask: Vec::new(), // Filled by execution engine
                selectivity: s,
            }
        }
    }

    /// Calculate how many results to fetch for Vector-First strategy
    ///
    /// # Formula
    ///
    /// k_prime = (k / s) * safety_factor
    ///
    /// This accounts for statistical variance in the filter match rate.
    ///
    /// # Example
    ///
    /// - k = 10
    /// - s = 0.2 (20% match rate)
    /// - safety_factor = 1.5
    /// - k_prime = (10 / 0.2) * 1.5 = 75
    ///
    /// Meaning: Fetch 75 results from HNSW, expect approximately 15 matches, return top 10.
    pub fn calculate_expansion(&self, k: usize, s: f64) -> usize {
        // Avoid division by zero
        if s < 0.001 {
            return self.max_expansion;
        }

        // Expected matches: k_prime * s
        // We need: k_prime * s >= k
        // Therefore: k_prime >= k / s
        // Add safety factor for variance
        let k_prime = (k as f64 / s * self.safety_factor).ceil() as usize;

        // Cap at max_expansion to avoid excessive HNSW traversal
        let capped = k_prime.min(self.max_expansion);

        if capped < k_prime {
            eprintln!(
                "[Warning] k' would be {} (too large), capping at {}. Consider FilterFirst instead.",
                k_prime, capped
            );
        }

        capped
    }

    /// Estimate the cost of brute force strategy
    ///
    /// Cost = s * N * C_dist
    pub fn estimate_cost_brute_force(&self, n_total: usize, selectivity: f64) -> f64 {
        const C_DIST: f64 = 1.0; // Relative cost of one distance calculation
        selectivity * n_total as f64 * C_DIST
    }

    /// Estimate the cost of filter-first strategy
    ///
    /// Cost = C_tantivy + C_hnsw * f(s) + N * C_check
    pub fn estimate_cost_filter_first(&self, n_total: usize, selectivity: f64) -> f64 {
        const C_TANTIVY: f64 = 1.0; // Cost of Tantivy query
        const C_HNSW: f64 = 2.0; // Base cost of HNSW traversal
        const C_CHECK: f64 = 0.01; // Cost of one bitmask check

        // Connectivity penalty: HNSW gets harder as selectivity drops
        let connectivity_penalty = if selectivity < 0.05 {
            3.0 // Graph is highly disconnected
        } else if selectivity < 0.2 {
            1.5 // Graph is somewhat connected
        } else {
            1.0 // Graph is well-connected
        };

        C_TANTIVY + C_HNSW * connectivity_penalty + n_total as f64 * C_CHECK
    }

    /// Estimate the cost of vector-first strategy
    ///
    /// Cost = C_hnsw + k_prime * C_check
    pub fn estimate_cost_vector_first(&self, k: usize, selectivity: f64) -> f64 {
        const C_HNSW: f64 = 2.0; // Cost of HNSW traversal
        const C_CHECK: f64 = 0.01; // Cost of one metadata check

        let k_expansion = self.calculate_expansion(k, selectivity);

        C_HNSW + k_expansion as f64 * C_CHECK
    }
}

/// Query statistics for adaptive tuning
#[derive(Debug, Clone)]
pub struct QueryStats {
    pub selectivity: f64,
    pub strategy: String,
    pub latency_ms: f64,
    pub recall: f64,
    pub n_total: usize,
    pub n_matches: usize,
    pub k: usize,
}

/// Collects query statistics and recommends optimal thresholds
pub struct QueryAnalyzer {
    history: Vec<QueryStats>,
}

impl QueryAnalyzer {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    /// Record a query execution
    pub fn record(&mut self, stats: QueryStats) {
        self.history.push(stats);
    }

    /// Analyze last N queries to find optimal thresholds
    ///
    /// Returns (threshold_brute, threshold_pre)
    pub fn recommend_thresholds(&self, n: usize) -> (f64, f64) {
        let recent: Vec<_> = self.history.iter().rev().take(n).collect();

        if recent.is_empty() {
            return (0.01, 0.5); // Default
        }

        // Find selectivity where BruteForce = FilterFirst
        let threshold_brute = self
            .find_crossover(&recent, "BruteForce", "FilterFirst")
            .unwrap_or(0.01);

        // Find selectivity where FilterFirst = VectorFirst
        let threshold_pre = self
            .find_crossover(&recent, "FilterFirst", "VectorFirst")
            .unwrap_or(0.5);

        (threshold_brute, threshold_pre)
    }

    /// Find the selectivity where two strategies have equal latency
    fn find_crossover(&self, stats: &[&QueryStats], strat_a: &str, strat_b: &str) -> Option<f64> {
        // Collect queries for each strategy
        let mut a_queries: Vec<_> = stats
            .iter()
            .filter(|s| s.strategy == strat_a)
            .map(|s| (s.selectivity, s.latency_ms))
            .collect();

        let mut b_queries: Vec<_> = stats
            .iter()
            .filter(|s| s.strategy == strat_b)
            .map(|s| (s.selectivity, s.latency_ms))
            .collect();

        if a_queries.is_empty() || b_queries.is_empty() {
            return None;
        }

        // Sort by selectivity
        a_queries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        b_queries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Find crossover point (simplified: just average the medians)
        // Production: Use linear regression or ML
        let a_median = a_queries[a_queries.len() / 2].0;
        let b_median = b_queries[b_queries.len() / 2].0;

        Some((a_median + b_median) / 2.0)
    }

    /// Get summary statistics
    pub fn summary(&self) -> String {
        if self.history.is_empty() {
            return "No queries recorded".to_string();
        }

        let n = self.history.len();
        let avg_latency: f64 = self.history.iter().map(|s| s.latency_ms).sum::<f64>() / n as f64;
        let avg_recall: f64 = self.history.iter().map(|s| s.recall).sum::<f64>() / n as f64;

        // Count by strategy
        let mut counts = std::collections::HashMap::new();
        for stats in &self.history {
            *counts.entry(stats.strategy.clone()).or_insert(0) += 1;
        }

        format!(
            "Queries: {}\nAvg Latency: {:.2}ms\nAvg Recall: {:.1}%\nStrategies: {:?}",
            n,
            avg_latency,
            avg_recall * 100.0,
            counts
        )
    }
}

/// Adaptive query planner that learns from query history
pub struct AdaptivePlanner {
    planner: QueryPlanner,
    analyzer: QueryAnalyzer,
    update_interval: usize, // Update thresholds every N queries
    queries_since_update: usize,
}

impl AdaptivePlanner {
    pub fn new(planner: QueryPlanner, update_interval: usize) -> Self {
        Self {
            planner,
            analyzer: QueryAnalyzer::new(),
            update_interval,
            queries_since_update: 0,
        }
    }

    /// Create an adaptive planner with default settings
    pub fn default() -> Self {
        Self::new(QueryPlanner::default(), 1000)
    }

    /// Plan a query and record statistics
    pub fn plan_and_record(&mut self, n_total: usize, n_matches: usize, k: usize) -> ExecutionPlan {
        let plan = self.planner.plan_from_counts(n_total, n_matches, k);

        self.queries_since_update += 1;

        // Check if we should update thresholds
        if self.queries_since_update >= self.update_interval {
            self.update_thresholds();
            self.queries_since_update = 0;
        }

        plan
    }

    /// Record the execution of a query
    pub fn record(&mut self, stats: QueryStats) {
        self.analyzer.record(stats);
    }

    /// Update thresholds based on query history
    fn update_thresholds(&mut self) {
        let (t_brute, t_pre) = self.analyzer.recommend_thresholds(1000);

        println!(
            "[Adaptive] Updating thresholds: brute={:.3} (was {:.3}), pre={:.3} (was {:.3})",
            t_brute, self.planner.threshold_brute, t_pre, self.planner.threshold_pre
        );

        self.planner.threshold_brute = t_brute;
        self.planner.threshold_pre = t_pre;
    }

    /// Get summary of query history
    pub fn summary(&self) -> String {
        self.analyzer.summary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_brute_force() {
        let planner = QueryPlanner::default();
        let plan = planner.plan_from_counts(100_000, 50, 10);

        match plan {
            ExecutionPlan::BruteForce { selectivity, .. } => {
                assert!((selectivity - 0.0005).abs() < 0.0001);
            }
            _ => panic!("Expected BruteForce strategy"),
        }
    }

    #[test]
    fn test_plan_filter_first() {
        let planner = QueryPlanner::default();
        let plan = planner.plan_from_counts(100_000, 5_000, 10);

        match plan {
            ExecutionPlan::FilterFirst { selectivity, .. } => {
                assert!((selectivity - 0.05).abs() < 0.001);
            }
            _ => panic!("Expected FilterFirst strategy"),
        }
    }

    #[test]
    fn test_plan_vector_first() {
        let planner = QueryPlanner::default();
        let plan = planner.plan_from_counts(100_000, 80_000, 10);

        match plan {
            ExecutionPlan::VectorFirst {
                k_expansion,
                selectivity,
                ..
            } => {
                assert!((selectivity - 0.8).abs() < 0.01);
                // k_prime = 10 / 0.8 * 1.5 = 18.75, approximately 19
                assert!(k_expansion >= 18 && k_expansion <= 20);
            }
            _ => panic!("Expected VectorFirst strategy"),
        }
    }

    #[test]
    fn test_calculate_expansion() {
        let planner = QueryPlanner::default();

        // s = 0.2, k = 10
        // k_prime = 10 / 0.2 * 1.5 = 75
        let k_prime = planner.calculate_expansion(10, 0.2);
        assert_eq!(k_prime, 75);

        // s = 0.5, k = 10
        // k_prime = 10 / 0.5 * 1.5 = 30
        let k_prime = planner.calculate_expansion(10, 0.5);
        assert_eq!(k_prime, 30);

        // s = 0.9, k = 10
        // k_prime = 10 / 0.9 * 1.5 = 16.67, approximately 17
        let k_prime = planner.calculate_expansion(10, 0.9);
        assert!(k_prime >= 16 && k_prime <= 17);
    }

    #[test]
    fn test_expansion_capping() {
        let mut planner = QueryPlanner::default();
        planner.max_expansion = 100;

        // s = 0.001, k = 10
        // k_prime = 10 / 0.001 * 1.5 = 15000
        // Should be capped at 100
        let k_prime = planner.calculate_expansion(10, 0.001);
        assert_eq!(k_prime, 100);
    }

    #[test]
    fn test_cost_estimates() {
        let planner = QueryPlanner::default();
        let n_total = 100_000;

        // BruteForce: s * N * C_dist
        let cost_brute = planner.estimate_cost_brute_force(n_total, 0.01);
        assert!((cost_brute - 1000.0).abs() < 0.1);

        // VectorFirst: C_hnsw + k_prime * C_check
        let cost_vector = planner.estimate_cost_vector_first(10, 0.8);
        // k_prime = 10 / 0.8 * 1.5 = 19
        // Cost = 2.0 + 19 * 0.01 = 2.19
        assert!((cost_vector - 2.19).abs() < 0.1);
    }

    #[test]
    fn test_analyzer() {
        let mut analyzer = QueryAnalyzer::new();

        analyzer.record(QueryStats {
            selectivity: 0.005,
            strategy: "BruteForce".to_string(),
            latency_ms: 0.5,
            recall: 1.0,
            n_total: 100_000,
            n_matches: 500,
            k: 10,
        });

        analyzer.record(QueryStats {
            selectivity: 0.05,
            strategy: "FilterFirst".to_string(),
            latency_ms: 3.0,
            recall: 1.0,
            n_total: 100_000,
            n_matches: 5_000,
            k: 10,
        });

        let summary = analyzer.summary();
        assert!(summary.contains("Queries: 2"));
    }
}

fn main() {
    let planner = QueryPlanner::default();
    println!("QueryPlanner initialized: {:?}", planner);
    println!("Run `cargo test` to execute all test cases.");
}
