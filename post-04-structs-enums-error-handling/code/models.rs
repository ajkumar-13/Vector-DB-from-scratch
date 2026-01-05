// models.rs
//
// Production-ready type definitions for our vector database.
// From Post #4: Structs, Enums, and Error Handling
//
// This file is designed to be used as src/models.rs in the vectordb project.
// Add `mod models;` to your main.rs to use these types.

use std::collections::HashMap;
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// CORE DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// A vector embedding with metadata.
///
/// This is the fundamental unit stored in our database.
/// Each vector has a unique ID, the embedding data, and optional metadata.
#[derive(Debug, Clone)]
pub struct Vector {
    /// The raw embedding data (e.g., 768 floats for BERT)
    pub data: Vec<f32>,

    /// Key-value metadata: {"title": "Document Name", "category": "tech"}
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

    /// Normalize the vector in-place
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistanceMetric {
    /// Cosine similarity: 1 = identical, 0 = orthogonal, -1 = opposite
    Cosine,

    /// Euclidean distance: 0 = identical, larger = more different
    Euclidean,

    /// Dot product: higher = more similar (assumes normalized vectors)
    Dot,
}

impl DistanceMetric {
    /// Calculate distance/similarity between two vectors
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

// ═══════════════════════════════════════════════════════════════════════════
// SEARCH TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// A single search result.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The ID of the matching vector
    pub id: String,

    /// Similarity/distance score
    pub score: f32,
}

/// Parameters for a search query.
#[derive(Debug, Clone)]
pub struct SearchRequest {
    /// The query vector
    pub vector: Vec<f32>,

    /// Number of results to return
    pub top_k: usize,

    /// Distance metric to use
    pub metric: DistanceMetric,

    /// Optional metadata filter
    pub filter: Option<HashMap<String, String>>,
}

impl SearchRequest {
    /// Create a simple search request
    pub fn new(vector: Vec<f32>, top_k: usize) -> Self {
        Self {
            vector,
            top_k,
            metric: DistanceMetric::Cosine,
            filter: None,
        }
    }

    /// Builder pattern: set metric
    pub fn with_metric(mut self, metric: DistanceMetric) -> Self {
        self.metric = metric;
        self
    }

    /// Builder pattern: set filter
    pub fn with_filter(mut self, filter: HashMap<String, String>) -> Self {
        self.filter = Some(filter);
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ERROR TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Errors that can occur in our vector database.
#[derive(Debug)]
pub enum VectorDbError {
    /// Query vector has no dimensions
    EmptyVector,

    /// Vector dimensions don't match
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
            VectorDbError::AlreadyExists(id) => {
                write!(f, "Already exists: {}", id)
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

// Implement std::error::Error for compatibility
impl std::error::Error for VectorDbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            VectorDbError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

// Allow automatic conversion from io::Error
impl From<std::io::Error> for VectorDbError {
    fn from(err: std::io::Error) -> Self {
        VectorDbError::IoError(err)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TYPE ALIASES
// ═══════════════════════════════════════════════════════════════════════════

/// Standard Result type for our database operations
pub type Result<T> = std::result::Result<T, VectorDbError>;

// ═══════════════════════════════════════════════════════════════════════════
// EXAMPLE USAGE (can be run as a standalone file)
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_creation() {
        let v = Vector::new(vec![0.1, 0.2, 0.3]);
        assert_eq!(v.dimension(), 3);
    }

    #[test]
    fn test_distance_metrics() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];

        // Orthogonal vectors have cosine similarity of 0
        let cosine = DistanceMetric::Cosine.calculate(&a, &b);
        assert!((cosine - 0.0).abs() < 0.0001);

        // Euclidean distance is sqrt(2) for unit orthogonal vectors
        let euclidean = DistanceMetric::Euclidean.calculate(&a, &b);
        assert!((euclidean - 1.414).abs() < 0.01);
    }

    #[test]
    fn test_error_display() {
        let err = VectorDbError::DimensionMismatch {
            expected: 768,
            got: 384,
        };
        assert!(err.to_string().contains("768"));
        assert!(err.to_string().contains("384"));
    }
}

// Main function for standalone execution
fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  VECTORDB MODELS - PRODUCTION TYPES");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Create a vector
    let mut v = Vector::new(vec![3.0, 4.0]);
    println!("Vector: {:?}", v);
    println!("Dimension: {}", v.dimension());
    println!("Magnitude: {}", v.magnitude()); // 5.0

    v.normalize();
    println!("Normalized: {:?}", v.data); // [0.6, 0.8]
    println!();

    // Test distance metrics
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 1.0, 0.0];

    println!("Distance Metrics for orthogonal vectors:");
    println!("  Cosine: {:.4}", DistanceMetric::Cosine.calculate(&a, &b));
    println!(
        "  Euclidean: {:.4}",
        DistanceMetric::Euclidean.calculate(&a, &b)
    );
    println!("  Dot: {:.4}", DistanceMetric::Dot.calculate(&a, &b));
    println!();

    // Test error handling
    let result: Result<()> = Err(VectorDbError::DimensionMismatch {
        expected: 768,
        got: 384,
    });

    if let Err(e) = result {
        println!("Error: {}", e);
    }

    // Demonstrate map_err for file operations
    println!();
    println!("Demonstrating map_err with file I/O:");
    if let Err(e) = load_file_example("nonexistent.bin") {
        println!("  Load failed (expected): {:?}", e);
    }
}

/// Example showing how to use map_err with custom error types.
/// The ? operator doesn't automatically convert io::Error to VectorDbError,
/// so we must use map_err to do the conversion explicitly.
fn load_file_example(path: &str) -> Result<Vec<u8>> {
    use std::io::Read;

    // Without map_err, this would NOT compile:
    // let mut f = std::fs::File::open(path)?;  // ❌ ERROR!

    // With map_err, we explicitly convert the error:
    let mut f = std::fs::File::open(path).map_err(VectorDbError::IoError)?; // ✅ Works!

    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).map_err(VectorDbError::IoError)?;

    Ok(buffer)
}
