// models-serde.rs
//
// Updated models with Serde derives for JSON serialization.
// From Post #5: The Async Runtime & HTTP Layer
//
// This extends the models from Post #4 with Serde support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════
// CORE DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// A vector embedding with metadata.
///
/// Now with Serde derives for JSON serialization/deserialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    /// The raw embedding data
    pub data: Vec<f32>,

    /// Optional metadata - uses empty HashMap if not provided in JSON
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Vector {
    /// Create a new vector with just data
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

    /// Get dimensionality
    pub fn dimension(&self) -> usize {
        self.data.len()
    }

    /// Calculate magnitude (L2 norm)
    pub fn magnitude(&self) -> f32 {
        self.data.iter().map(|x| x * x).sum::<f32>().sqrt()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DISTANCE METRICS
// ═══════════════════════════════════════════════════════════════════════════

/// Supported distance metrics.
///
/// Derives Deserialize so clients can specify metric in requests.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] // Accept "cosine", "euclidean", "dot"
pub enum DistanceMetric {
    Cosine,
    Euclidean,
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

/// Search request from client
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    /// The query vector
    pub vector: Vec<f32>,

    /// Number of results to return
    #[serde(default = "default_top_k")]
    pub top_k: usize,

    /// Distance metric to use
    #[serde(default)]
    pub metric: Option<DistanceMetric>,
}

fn default_top_k() -> usize {
    10
}

/// Search result returned to client
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    /// Optionally include the vector data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<Vector>,
}

// ═══════════════════════════════════════════════════════════════════════════
// ERROR TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Custom error type for our database
#[derive(Debug)]
pub enum VectorDbError {
    EmptyVector,
    DimensionMismatch { expected: usize, got: usize },
    NotFound(String),
    InvalidParameter(String),
    IoError(std::io::Error),
}

impl fmt::Display for VectorDbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VectorDbError::EmptyVector => write!(f, "Vector cannot be empty"),
            VectorDbError::DimensionMismatch { expected, got } => {
                write!(f, "Dimension mismatch: expected {}, got {}", expected, got)
            }
            VectorDbError::NotFound(id) => write!(f, "Not found: {}", id),
            VectorDbError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            VectorDbError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for VectorDbError {}

/// Serialize errors for API responses
impl Serialize for VectorDbError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Type alias for cleaner function signatures
pub type Result<T> = std::result::Result<T, VectorDbError>;

// ═══════════════════════════════════════════════════════════════════════════
// EXAMPLE USAGE
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_json_roundtrip() {
        let v = Vector::new(vec![0.1, 0.2, 0.3]);
        let json = serde_json::to_string(&v).unwrap();
        let parsed: Vector = serde_json::from_str(&json).unwrap();
        assert_eq!(v.data, parsed.data);
    }

    #[test]
    fn test_vector_json_without_metadata() {
        // JSON without metadata field should work
        let json = r#"{"data": [1.0, 2.0, 3.0]}"#;
        let v: Vector = serde_json::from_str(json).unwrap();
        assert_eq!(v.dimension(), 3);
        assert!(v.metadata.is_empty());
    }

    #[test]
    fn test_distance_metric_json() {
        // Should accept lowercase
        let json = r#""cosine""#;
        let metric: DistanceMetric = serde_json::from_str(json).unwrap();
        assert_eq!(metric, DistanceMetric::Cosine);
    }
}

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  SERDE MODELS - JSON SERIALIZATION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Create a vector
    let v = Vector::new(vec![0.1, 0.2, 0.3]);

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&v).unwrap();
    println!("Vector as JSON:");
    println!("{}", json);
    println!();

    // Parse JSON back to Vector
    let json_input = r#"{
        "data": [1.0, 2.0, 3.0],
        "metadata": {"source": "test"}
    }"#;

    let parsed: Vector = serde_json::from_str(json_input).unwrap();
    println!("Parsed from JSON: {:?}", parsed);
    println!("Dimension: {}", parsed.dimension());
    println!();

    // Search request with defaults
    let search_json = r#"{"vector": [0.1, 0.2, 0.3]}"#;
    let search: SearchRequest = serde_json::from_str(search_json).unwrap();
    println!("Search request: {:?}", search);
    println!("Default top_k: {}", search.top_k);
}
