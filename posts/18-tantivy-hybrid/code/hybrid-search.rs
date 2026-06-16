// hybrid-search.rs
// Complete Hybrid Search Engine combining HNSW + Tantivy
//
// This module provides a unified API for hybrid vector + metadata search,
// automatically handling the coordination between vector and text indexes.

use std::error::Error;
use std::sync::{Arc, RwLock};

/// A hybrid search query combining vector and metadata filter
#[derive(Debug, Clone)]
pub struct HybridQuery {
    /// Query vector for similarity search
    pub vector: Vec<f32>,
    /// Number of results to return
    pub k: usize,
    /// HNSW search parameter (larger = more accurate but slower)
    pub ef: usize,
    /// Optional metadata filter query (Tantivy query syntax)
    pub filter: Option<String>,
}

impl HybridQuery {
    pub fn new(vector: Vec<f32>, k: usize) -> Self {
        Self {
            vector,
            k,
            ef: 100, // Default ef
            filter: None,
        }
    }

    pub fn with_filter(mut self, filter: String) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn with_ef(mut self, ef: usize) -> Self {
        self.ef = ef;
        self
    }
}

/// A search result with vector similarity and metadata
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Document/point ID
    pub point_id: usize,
    /// Similarity score (0.0 to 1.0, higher is better)
    pub score: f64,
    /// Optional metadata (fetched from Tantivy)
    pub metadata: Option<String>,
}

/// Hybrid Search Engine coordinating HNSW and Tantivy
pub struct HybridSearchEngine {
    /// HNSW index for vector search
    hnsw: Arc<RwLock<HNSWIndex>>,
    /// Tantivy index for metadata filtering
    metadata: Arc<MetadataIndex>,
}

impl HybridSearchEngine {
    /// Create a new hybrid search engine
    pub fn new(hnsw: HNSWIndex, metadata: MetadataIndex) -> Self {
        Self {
            hnsw: Arc::new(RwLock::new(hnsw)),
            metadata: Arc::new(metadata),
        }
    }

    /// Index a document with both vector and metadata
    ///
    /// # Arguments
    ///
    /// * `point_id` - Unique identifier for the document
    /// * `vector` - Embedding vector
    /// * `metadata` - JSON metadata to index
    ///
    /// # Example
    ///
    /// ```
    /// let engine = HybridSearchEngine::new(hnsw, metadata_index);
    /// engine.index_document(
    ///     42,
    ///     vec![0.1, 0.2, 0.3],
    ///     r#"{"title": "Nike Shoes", "price": 89.99}"#
    /// )?;
    /// ```
    pub fn index_document(
        &self,
        point_id: usize,
        vector: Vec<f32>,
        metadata: &str,
    ) -> Result<(), Box<dyn Error>> {
        // Index in HNSW
        let mut hnsw = self.hnsw.write().unwrap();
        hnsw.insert(point_id, vector)?;
        drop(hnsw); // Release lock early

        // Index in Tantivy
        self.metadata.index_document(point_id, metadata)?;

        Ok(())
    }

    /// Execute a hybrid search query
    ///
    /// This method automatically chooses the best execution strategy:
    /// - If no filter: Pure HNSW search
    /// - If filter with low selectivity: Brute force on filtered docs
    /// - If filter with medium selectivity: Pre-filter HNSW with bitmask
    /// - If filter with high selectivity: Post-filter HNSW results
    ///
    /// See Post #19 (Query Planning) for the full optimizer logic.
    pub fn search(&self, query: &HybridQuery) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        // Case 1: No filter - pure vector search
        if query.filter.is_none() {
            return self.pure_vector_search(&query.vector, query.k, query.ef);
        }

        let filter_query = query.filter.as_ref().unwrap();

        // Estimate selectivity
        let n_total = self.hnsw.read().unwrap().num_points();
        let n_matches = self.metadata.count_matches(filter_query)?;
        let selectivity = n_matches as f64 / n_total as f64;

        println!(
            "[HybridSearch] Selectivity: {:.2}% ({}/{})",
            selectivity * 100.0,
            n_matches,
            n_total
        );

        // Choose strategy based on selectivity
        if selectivity < 0.01 {
            // Case 2: Very low selectivity (<1%) - brute force
            self.brute_force_search(&query.vector, query.k, filter_query)
        } else if selectivity < 0.5 {
            // Case 3: Medium selectivity (1-50%) - pre-filter
            self.prefiltered_search(&query.vector, query.k, query.ef, filter_query)
        } else {
            // Case 4: High selectivity (>50%) - post-filter
            self.postfiltered_search(&query.vector, query.k, query.ef, filter_query)
        }
    }

    /// Pure vector search (no filter)
    fn pure_vector_search(
        &self,
        query: &[f32],
        k: usize,
        ef: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        let hnsw = self.hnsw.read().unwrap();
        let results = hnsw.search(query, k, ef);

        Ok(results
            .into_iter()
            .map(|(score, id)| SearchResult {
                point_id: id,
                score,
                metadata: None, // Do not fetch metadata by default
            })
            .collect())
    }

    /// Brute force search on filtered documents
    fn brute_force_search(
        &self,
        query: &[f32],
        k: usize,
        filter: &str,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        println!("[HybridSearch] Using BruteForce strategy");

        // Get list of matching document IDs
        let matches = self.metadata.search_to_ids(filter)?;

        if matches.is_empty() {
            return Ok(Vec::new());
        }

        // Compute distance for each matching document
        let hnsw = self.hnsw.read().unwrap();
        let mut scored: Vec<_> = matches
            .into_iter()
            .map(|id| {
                let vector = hnsw.get_vector(id);
                let score = 1.0 - cosine_distance(query, vector);
                (score, id)
            })
            .collect();

        // Sort by score (descending)
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Take top k
        Ok(scored
            .into_iter()
            .take(k)
            .map(|(score, id)| SearchResult {
                point_id: id,
                score,
                metadata: None,
            })
            .collect())
    }

    /// Pre-filtered search (bitmask-constrained HNSW)
    fn prefiltered_search(
        &self,
        query: &[f32],
        k: usize,
        ef: usize,
        filter: &str,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        println!("[HybridSearch] Using FilterFirst strategy");

        // Build bitmask
        let hnsw = self.hnsw.read().unwrap();
        let n_total = hnsw.num_points();
        let bitmask = self.metadata.search_to_bitmask(filter, n_total)?;

        // Search HNSW with bitmask constraint
        let results = hnsw.search_with_filter(query, k, ef, Some(&bitmask));

        // Check if we got enough results
        if results.len() < (k / 2) {
            // Graph might be disconnected, fall back to brute force
            println!("[HybridSearch] Insufficient results, falling back to BruteForce");
            drop(hnsw);
            return self.brute_force_search(query, k, filter);
        }

        Ok(results
            .into_iter()
            .map(|(score, id)| SearchResult {
                point_id: id,
                score,
                metadata: None,
            })
            .collect())
    }

    /// Post-filtered search (oversample HNSW, then filter)
    fn postfiltered_search(
        &self,
        query: &[f32],
        k: usize,
        ef: usize,
        filter: &str,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        println!("[HybridSearch] Using VectorFirst strategy");

        // Calculate oversampling factor
        let hnsw = self.hnsw.read().unwrap();
        let n_total = hnsw.num_points();
        let n_matches = self.metadata.count_matches(filter)?;
        let selectivity = n_matches as f64 / n_total as f64;

        // k_prime = k / selectivity * safety_factor
        let k_prime = ((k as f64 / selectivity) * 1.5).ceil() as usize;
        let k_prime = k_prime.min(10000); // Cap at reasonable limit

        println!(
            "[HybridSearch] Oversampling: k'={} (selectivity={:.2}%)",
            k_prime,
            selectivity * 100.0
        );

        // Search HNSW for k' results
        let hnsw_results = hnsw.search(query, k_prime, ef);
        drop(hnsw);

        // Build bitmask for filtering
        let bitmask = self.metadata.search_to_bitmask(filter, n_total)?;

        // Filter results
        let filtered: Vec<_> = hnsw_results
            .into_iter()
            .filter(|(_, id)| *id < bitmask.len() && bitmask[*id])
            .take(k)
            .map(|(score, id)| SearchResult {
                point_id: id,
                score,
                metadata: None,
            })
            .collect();

        // Check if we got enough results
        if filtered.len() < (k / 2) {
            println!("[HybridSearch] Insufficient results after filtering, falling back");
            return self.brute_force_search(query, k, filter);
        }

        Ok(filtered)
    }

    /// Fetch metadata for search results (lazy loading)
    pub fn with_metadata(
        &self,
        results: Vec<SearchResult>,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        results
            .into_iter()
            .map(|mut result| {
                result.metadata = self.metadata.get_document(result.point_id).ok();
                Ok(result)
            })
            .collect()
    }
}

/// Cosine distance between two vectors
fn cosine_distance(a: &[f32], b: &[f32]) -> f64 {
    assert_eq!(a.len(), b.len());

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    let similarity = dot / (norm_a * norm_b);
    (1.0 - similarity as f64).clamp(0.0, 2.0)
}

// Placeholder types (would be imported from other modules)
struct HNSWIndex;
struct MetadataIndex;

impl HNSWIndex {
    fn insert(&mut self, _id: usize, _vector: Vec<f32>) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn search(&self, _query: &[f32], _k: usize, _ef: usize) -> Vec<(f64, usize)> {
        Vec::new()
    }

    fn search_with_filter(
        &self,
        _query: &[f32],
        _k: usize,
        _ef: usize,
        _filter: Option<&[bool]>,
    ) -> Vec<(f64, usize)> {
        Vec::new()
    }

    fn get_vector(&self, _id: usize) -> &[f32] {
        &[]
    }

    fn num_points(&self) -> usize {
        0
    }
}

impl MetadataIndex {
    fn index_document(&self, _point_id: usize, _metadata: &str) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn count_matches(&self, _query: &str) -> Result<usize, Box<dyn Error>> {
        Ok(0)
    }

    fn search_to_ids(&self, _query: &str) -> Result<Vec<usize>, Box<dyn Error>> {
        Ok(Vec::new())
    }

    fn search_to_bitmask(
        &self,
        _query: &str,
        _n_total: usize,
    ) -> Result<Vec<bool>, Box<dyn Error>> {
        Ok(Vec::new())
    }

    fn get_document(&self, _point_id: usize) -> Result<String, Box<dyn Error>> {
        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_engine() -> HybridSearchEngine {
        let hnsw = HNSWIndex;
        let metadata = MetadataIndex;
        HybridSearchEngine::new(hnsw, metadata)
    }

    #[test]
    fn test_hybrid_query_builder() {
        let query = HybridQuery::new(vec![1.0, 2.0, 3.0], 10)
            .with_filter("price:<100".to_string())
            .with_ef(200);

        assert_eq!(query.k, 10);
        assert_eq!(query.ef, 200);
        assert_eq!(query.filter, Some("price:<100".to_string()));
    }

    #[test]
    fn test_index_document() {
        let engine = create_test_engine();

        let result = engine.index_document(1, vec![0.1, 0.2, 0.3], r#"{"title": "Test"}"#);

        assert!(result.is_ok());
    }

    #[test]
    fn test_cosine_distance() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let dist = cosine_distance(&a, &b);
        assert!(dist < 0.001); // Same vector, distance is approximately 0

        let c = vec![0.0, 1.0, 0.0];
        let dist = cosine_distance(&a, &c);
        assert!((dist - 1.0).abs() < 0.001); // Orthogonal, distance is approximately 1
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Create indexes
    let hnsw = HNSWIndex;
    let metadata = MetadataIndex;

    // Create hybrid engine
    let engine = HybridSearchEngine::new(hnsw, metadata);

    // Index documents
    engine.index_document(
        1,
        vec![0.1, 0.2, 0.3],
        r#"{"title": "Nike Air Max", "price": 120.00, "brand": "Nike"}"#,
    )?;

    engine.index_document(
        2,
        vec![0.2, 0.3, 0.1],
        r#"{"title": "Adidas Ultraboost", "price": 180.00, "brand": "Adidas"}"#,
    )?;

    // Search with filter
    let query = HybridQuery::new(vec![0.15, 0.25, 0.2], 10)
        .with_filter("price:<150 AND brand:Nike".to_string());

    let results = engine.search(&query)?;

    // Lazy-load metadata
    let results_with_metadata = engine.with_metadata(results)?;

    for result in results_with_metadata {
        println!(
            "ID: {}, Score: {:.3}, Metadata: {:?}",
            result.point_id, result.score, result.metadata
        );
    }

    Ok(())
}
