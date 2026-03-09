// src/models.rs
//
// Core type definitions for our vector database.
// Created in Post #4: Structs, Enums, and Error Handling
//
// These types form the foundation that every later phase builds upon:
// - Phase 2 (Storage) serializes Vector to binary
// - Phase 3 (Search) uses DistanceMetric and SearchResult
// - Phase 4 (Hybrid) extends metadata filtering

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// CORE DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// A vector embedding with metadata.
///
/// This is the fundamental unit stored in our database.
/// Each vector has embedding data and optional key-value metadata.
///
/// # Example
/// ```
/// use vectordb::models::Vector;
/// let v = Vector::new(vec![0.1, 0.2, 0.3]);
/// assert_eq!(v.dimension(), 3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    /// The raw embedding data (e.g., 768 floats for BERT)
    pub data: Vec<f32>,

    /// Key-value metadata: {"title": "Document Name", "category": "tech"}
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Vector {
    /// Create a new vector with just data (no metadata)
    pub fn new(data: Vec<f32>) -> Self {
        Self {
            data,
            metadata: HashMap::new(),
        }
    }

    /// Create a vector with metadata
    pub fn with_metadata(data: Vec<f32>, metadata: HashMap<String, String>) -> Self {
        Self { data, metadata }
    }

    /// Get the dimensionality of this vector
    pub fn dimension(&self) -> usize {
        self.data.len()
    }

    /// Calculate the L2 norm (magnitude)
    pub fn magnitude(&self) -> f32 {
        self.data.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// Normalize the vector in-place (make magnitude = 1.0)
    pub fn normalize(&mut self) {
        let mag = self.magnitude();
        if mag > 0.0 {
            for x in &mut self.data {
                *x /= mag;
            }
        }
    }

    /// Get a normalized copy (original unchanged)
    pub fn normalized(&self) -> Self {
        let mut copy = self.clone();
        copy.normalize();
        copy
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DISTANCE METRICS
// ═══════════════════════════════════════════════════════════════════════════

/// Supported distance/similarity metrics.
///
/// Different use cases require different metrics:
/// - Cosine: Good for text embeddings (direction matters, not magnitude)
/// - Euclidean: Good for spatial data
/// - Dot: Fast, works well with normalized vectors
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DistanceMetric {
    /// Cosine similarity: 1 = identical, 0 = orthogonal, -1 = opposite
    Cosine,

    /// Euclidean distance: 0 = identical, larger = more different
    Euclidean,

    /// Dot product: higher = more similar (assumes normalized vectors)
    Dot,
}

impl DistanceMetric {
    /// Calculate distance/similarity between two vectors.
    ///
    /// # Panics
    /// Silently truncates if vectors have different lengths (zip behavior).
    /// Use `validate_dimensions` before calling this in production.
    pub fn calculate(&self, a: &[f32], b: &[f32]) -> f32 {
        match self {
            DistanceMetric::Cosine => {
                let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
                let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm_a == 0.0 || norm_b == 0.0 {
                    0.0
                } else {
                    dot / (norm_a * norm_b)
                }
            }
            DistanceMetric::Euclidean => a
                .iter()
                .zip(b)
                .map(|(x, y)| (x - y).powi(2))
                .sum::<f32>()
                .sqrt(),
            DistanceMetric::Dot => a.iter().zip(b).map(|(x, y)| x * y).sum(),
        }
    }
}

impl Default for DistanceMetric {
    fn default() -> Self {
        DistanceMetric::Cosine
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SEARCH TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// A single search result with ID and similarity score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The ID of the matching vector
    pub id: String,

    /// Similarity/distance score
    pub score: f32,
}

/// Parameters for a search query (received from clients).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    /// The query vector
    pub vector: Vec<f32>,

    /// Number of results to return (default: 10)
    #[serde(default = "default_top_k")]
    pub top_k: usize,

    /// Distance metric to use (default: cosine)
    #[serde(default)]
    pub metric: DistanceMetric,
}

fn default_top_k() -> usize {
    10
}

impl SearchRequest {
    /// Create a simple search request
    pub fn new(vector: Vec<f32>, top_k: usize) -> Self {
        Self {
            vector,
            top_k,
            metric: DistanceMetric::Cosine,
        }
    }
}

/// Wrapper for upsert payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertRequest {
    pub points: Vec<PointInput>,
}

/// A single point to insert/update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointInput {
    pub id: String,
    pub vector: Vec<f32>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// COLLECTION TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Request to create a new collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub dimension: usize,
    #[serde(default)]
    pub distance: DistanceMetric,
}

/// Information about a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    pub name: String,
    pub dimension: usize,
    pub distance: DistanceMetric,
    pub count: usize,
}

// ═══════════════════════════════════════════════════════════════════════════
// ERROR TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Errors that can occur in our vector database.
#[derive(Debug)]
pub enum VectorDbError {
    /// Query vector has no dimensions
    EmptyVector,

    /// Vector dimensions don't match collection's configured dimension
    DimensionMismatch { expected: usize, got: usize },

    /// Requested vector/collection not found
    NotFound(String),

    /// Collection already exists
    AlreadyExists(String),

    /// Invalid parameter value
    InvalidParameter(String),

    /// I/O error (file operations)
    IoError(std::io::Error),

    /// JSON serialization error
    SerializationError(String),
}

// Implement Display for user-friendly error messages
impl fmt::Display for VectorDbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VectorDbError::EmptyVector => {
                write!(f, "Vector cannot be empty")
            }
            VectorDbError::DimensionMismatch { expected, got } => {
                write!(f, "Dimension mismatch: expected {}, got {}", expected, got)
            }
            VectorDbError::NotFound(id) => {
                write!(f, "Not found: {}", id)
            }
            VectorDbError::AlreadyExists(name) => {
                write!(f, "Already exists: {}", name)
            }
            VectorDbError::InvalidParameter(msg) => {
                write!(f, "Invalid parameter: {}", msg)
            }
            VectorDbError::IoError(e) => {
                write!(f, "I/O error: {}", e)
            }
            VectorDbError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
        }
    }
}

// Implement std::error::Error for compatibility with ? and error chains
impl std::error::Error for VectorDbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            VectorDbError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

// Allow automatic conversion from io::Error → VectorDbError
// This lets the ? operator work: File::open("x")?
impl From<std::io::Error> for VectorDbError {
    fn from(err: std::io::Error) -> Self {
        VectorDbError::IoError(err)
    }
}

// Allow automatic conversion from serde_json::Error
impl From<serde_json::Error> for VectorDbError {
    fn from(err: serde_json::Error) -> Self {
        VectorDbError::SerializationError(err.to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TYPE ALIASES
// ═══════════════════════════════════════════════════════════════════════════

/// Standard Result type for our database operations.
/// Saves typing `Result<T, VectorDbError>` everywhere.
pub type Result<T> = std::result::Result<T, VectorDbError>;

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_creation() {
        let v = Vector::new(vec![0.1, 0.2, 0.3]);
        assert_eq!(v.dimension(), 3);
        assert!(v.metadata.is_empty());
    }

    #[test]
    fn test_vector_with_metadata() {
        let mut meta = HashMap::new();
        meta.insert("title".to_string(), "Test Doc".to_string());
        let v = Vector::with_metadata(vec![1.0, 2.0], meta);
        assert_eq!(v.metadata["title"], "Test Doc");
    }

    #[test]
    fn test_vector_magnitude() {
        let v = Vector::new(vec![3.0, 4.0]);
        assert!((v.magnitude() - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_vector_normalize() {
        let mut v = Vector::new(vec![3.0, 4.0]);
        v.normalize();
        assert!((v.magnitude() - 1.0).abs() < 0.0001);
        assert!((v.data[0] - 0.6).abs() < 0.0001);
        assert!((v.data[1] - 0.8).abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        // Orthogonal vectors → cosine = 0
        let score = DistanceMetric::Cosine.calculate(&a, &b);
        assert!((score - 0.0).abs() < 0.0001);

        // Identical vectors → cosine = 1
        let score = DistanceMetric::Cosine.calculate(&a, &a);
        assert!((score - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let dist = DistanceMetric::Euclidean.calculate(&a, &b);
        assert!((dist - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let dot = DistanceMetric::Dot.calculate(&a, &b);
        assert!((dot - 32.0).abs() < 0.0001); // 1*4 + 2*5 + 3*6 = 32
    }

    #[test]
    fn test_error_display() {
        let err = VectorDbError::DimensionMismatch {
            expected: 768,
            got: 384,
        };
        let msg = err.to_string();
        assert!(msg.contains("768"));
        assert!(msg.contains("384"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let db_err: VectorDbError = io_err.into();
        assert!(matches!(db_err, VectorDbError::IoError(_)));
    }
}
