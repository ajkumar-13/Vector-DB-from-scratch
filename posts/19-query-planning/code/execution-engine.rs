// execution-engine.rs
// Execution engine for hybrid search queries
//
// This module implements the actual execution logic for each strategy:
// - BruteForce: Scan filtered documents
// - FilterFirst: Pre-filtering with bitmask-constrained HNSW
// - VectorFirst: Post-filtering with oversampling

use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// A search result with metadata
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The point ID (document ID)
    pub point_id: usize,
    /// Similarity score (0.0 to 1.0, higher is better)
    pub score: f64,
    /// Optional metadata (can be fetched lazily)
    pub metadata: Option<String>,
}

/// Trait for ordering results by score (higher is better)
#[derive(Debug, Clone, Copy, PartialEq)]
struct OrderedFloat(f64);

impl Eq for OrderedFloat {}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// Distance function between two vectors
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f64 {
    assert_eq!(a.len(), b.len(), "Vectors must have same dimension");

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0; // Maximum distance
    }

    let similarity = dot / (norm_a * norm_b);
    1.0 - similarity as f64
}

/// Convert cosine distance to similarity score (0.0 to 1.0)
pub fn distance_to_score(distance: f64) -> f64 {
    1.0 - distance.clamp(0.0, 2.0) / 2.0
}

/// Hybrid search execution engine
///
/// This struct coordinates the execution of different search strategies.
/// It acts as a facade over the HNSW index and metadata index.
pub struct ExecutionEngine<'a> {
    /// Vectors storage (indexed by point_id)
    vectors: &'a [Vec<f32>],
}

impl<'a> ExecutionEngine<'a> {
    pub fn new(vectors: &'a [Vec<f32>]) -> Self {
        Self { vectors }
    }

    /// Get a vector by ID
    pub fn get_vector(&self, point_id: usize) -> &[f32] {
        &self.vectors[point_id]
    }

    /// Execute brute force scan
    ///
    /// Scans only the documents that match the filter and computes
    /// their distances to the query vector.
    ///
    /// # Performance
    ///
    /// - Best for: s < 1% (very few matches)
    /// - Time: O(s * N)
    /// - Memory: O(k)
    pub fn execute_brute_force(
        &self,
        query: &[f32],
        k: usize,
        matches: &[usize],
    ) -> Vec<SearchResult> {
        if matches.is_empty() {
            return Vec::new();
        }

        // Use a max-heap to keep top k results (min distance = max score)
        // We store Reverse to make it a min-heap (for distance)
        let mut heap: BinaryHeap<Reverse<(OrderedFloat, usize)>> = BinaryHeap::new();

        for &doc_id in matches {
            if doc_id >= self.vectors.len() {
                continue; // Skip invalid IDs
            }

            let vector = self.get_vector(doc_id);
            let dist = cosine_distance(query, vector);

            heap.push(Reverse((OrderedFloat(dist), doc_id)));

            // Keep only top k
            if heap.len() > k {
                heap.pop();
            }
        }

        // Convert heap to sorted results (best first)
        let mut results: Vec<_> = heap
            .into_iter()
            .map(|Reverse((dist, id))| SearchResult {
                point_id: id,
                score: distance_to_score(dist.0),
                metadata: None,
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    /// Execute vector-first strategy (post-filtering)
    ///
    /// Searches HNSW to get k' candidates, then filters them.
    ///
    /// # Performance
    ///
    /// - Best for: s > 50% (broad filters)
    /// - Time: O(log N) + O(k_prime * C_check)
    /// - Memory: O(k')
    pub fn execute_vector_first(
        &self,
        query: &[f32],
        k: usize,
        k_expansion: usize,
        bitmask: &[bool],
    ) -> Vec<SearchResult> {
        // Step 1: Get k' candidates from HNSW (using a simple greedy search)
        let hnsw_results = self.greedy_search(query, k_expansion);

        // Step 2: Filter results
        let mut filtered: Vec<_> = hnsw_results
            .into_iter()
            .filter(|(_, id)| *id < bitmask.len() && bitmask[*id])
            .take(k)
            .map(|(score, id)| SearchResult {
                point_id: id,
                score,
                metadata: None,
            })
            .collect();

        // Already sorted by score (greedy_search returns sorted)
        filtered
    }

    /// Execute filter-first strategy (pre-filtering)
    ///
    /// Searches HNSW but skips nodes not in the bitmask.
    ///
    /// # Performance
    ///
    /// - Best for: 1% <= s <= 50% (medium filters)
    /// - Time: O(log N * f(s)) where f(s) is connectivity penalty
    /// - Memory: O(N) for bitmask
    pub fn execute_filter_first(
        &self,
        query: &[f32],
        k: usize,
        bitmask: &[bool],
    ) -> Vec<SearchResult> {
        // Search HNSW with bitmask constraint
        let results = self.greedy_search_filtered(query, k, bitmask);

        results
            .into_iter()
            .map(|(score, id)| SearchResult {
                point_id: id,
                score,
                metadata: None,
            })
            .collect()
    }

    /// Simple greedy search (without filter)
    ///
    /// This is a simplified HNSW search for demonstration.
    /// In production, use a proper HNSW implementation.
    fn greedy_search(&self, query: &[f32], k: usize) -> Vec<(f64, usize)> {
        let mut results: Vec<(f64, usize)> = Vec::new();

        // For demonstration: just compute distances to all points
        for (id, vector) in self.vectors.iter().enumerate() {
            let dist = cosine_distance(query, vector);
            let score = distance_to_score(dist);
            results.push((score, id));
        }

        // Sort by score (descending)
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Return top k
        results.truncate(k);
        results
    }

    /// Greedy search with bitmask filter
    fn greedy_search_filtered(
        &self,
        query: &[f32],
        k: usize,
        bitmask: &[bool],
    ) -> Vec<(f64, usize)> {
        let mut results: Vec<(f64, usize)> = Vec::new();

        // Only compute distances for allowed points
        for (id, vector) in self.vectors.iter().enumerate() {
            if id >= bitmask.len() || !bitmask[id] {
                continue; // Skip filtered out points
            }

            let dist = cosine_distance(query, vector);
            let score = distance_to_score(dist);
            results.push((score, id));
        }

        // Sort by score (descending)
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Return top k
        results.truncate(k);
        results
    }
}

/// Fallback strategy when primary execution fails
pub struct FallbackStrategy {
    /// Threshold for disconnected graph detection
    pub min_results_fraction: f64,
}

impl FallbackStrategy {
    pub fn new() -> Self {
        Self {
            min_results_fraction: 0.5,
        }
    }

    /// Check if we should fallback to brute force
    ///
    /// This happens when:
    /// - FilterFirst returns too few results (disconnected graph)
    /// - VectorFirst filters out all results
    pub fn should_fallback(&self, results: &[SearchResult], k: usize) -> bool {
        let min_results = (k as f64 * self.min_results_fraction).ceil() as usize;
        results.len() < min_results
    }

    /// Execute fallback (brute force with available matches)
    pub fn execute_fallback(
        &self,
        engine: &ExecutionEngine,
        query: &[f32],
        k: usize,
        matches: &[usize],
    ) -> Vec<SearchResult> {
        println!("[Fallback] Insufficient results, falling back to brute force");
        engine.execute_brute_force(query, k, matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vectors() -> Vec<Vec<f32>> {
        vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
            vec![0.5, 0.5, 0.0],
            vec![0.0, 0.5, 0.5],
        ]
    }

    #[test]
    fn test_cosine_distance() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let dist = cosine_distance(&a, &b);
        assert!((dist - 0.0).abs() < 0.001); // Same vector

        let c = vec![0.0, 1.0, 0.0];
        let dist = cosine_distance(&a, &c);
        assert!((dist - 1.0).abs() < 0.001); // Orthogonal
    }

    #[test]
    fn test_brute_force() {
        let vectors = create_test_vectors();
        let engine = ExecutionEngine::new(&vectors);

        let query = vec![1.0, 0.0, 0.0];
        let matches = vec![0, 1, 2]; // First 3 vectors

        let results = engine.execute_brute_force(&query, 2, &matches);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].point_id, 0); // Exact match
        assert!(results[0].score > 0.99);
    }

    #[test]
    fn test_vector_first() {
        let vectors = create_test_vectors();
        let engine = ExecutionEngine::new(&vectors);

        let query = vec![1.0, 0.0, 0.0];
        let k_expansion = 5;

        // Bitmask: allow only points 0 and 3
        let bitmask = vec![true, false, false, true, false];

        let results = engine.execute_vector_first(&query, 2, k_expansion, &bitmask);

        // Should return points 0 and 3 (both allowed)
        assert!(results.len() <= 2);
        assert!(results.iter().all(|r| bitmask[r.point_id]));
    }

    #[test]
    fn test_filter_first() {
        let vectors = create_test_vectors();
        let engine = ExecutionEngine::new(&vectors);

        let query = vec![1.0, 0.0, 0.0];

        // Bitmask: allow only points 0, 1, 3
        let bitmask = vec![true, true, false, true, false];

        let results = engine.execute_filter_first(&query, 2, &bitmask);

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| bitmask[r.point_id]));
        assert_eq!(results[0].point_id, 0); // Best match
    }

    #[test]
    fn test_fallback_detection() {
        let fallback = FallbackStrategy::new();

        // Enough results
        let results = vec![
            SearchResult {
                point_id: 0,
                score: 0.9,
                metadata: None,
            },
            SearchResult {
                point_id: 1,
                score: 0.8,
                metadata: None,
            },
        ];
        assert!(!fallback.should_fallback(&results, 3));

        // Too few results
        let results = vec![SearchResult {
            point_id: 0,
            score: 0.9,
            metadata: None,
        }];
        assert!(fallback.should_fallback(&results, 10));
    }

    #[test]
    fn test_distance_to_score() {
        assert!((distance_to_score(0.0) - 1.0).abs() < 0.001); // Min distance
        assert!((distance_to_score(1.0) - 0.5).abs() < 0.001); // Mid distance
        assert!((distance_to_score(2.0) - 0.0).abs() < 0.001); // Max distance
    }
}

fn main() {
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 1.0, 0.0];
    let dist = cosine_distance(&a, &b);
    println!("Cosine distance between orthogonal vectors: {:.4}", dist);
    println!("Score: {:.4}", distance_to_score(dist));
    println!("Run `cargo test` to execute all test cases.");
}
