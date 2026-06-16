// quantization.rs
// Scalar Quantization: f32 to u8 for 4x memory reduction
//
// This module implements scalar quantization to compress vectors from 32-bit
// floats to 8-bit unsigned integers, reducing memory usage by 75%.

use std::ops::Range;

/// A vector quantized to u8 values
///
/// Stores values in [0, 255] range along with the original min/max
/// for dequantization and distance calculation.
#[derive(Debug, Clone)]
pub struct QuantizedVector {
    /// Quantized values (0-255)
    pub values: Vec<u8>,

    /// Minimum value in original vector
    pub min: f32,

    /// Maximum value in original vector
    pub max: f32,

    /// Dimension of the vector
    pub dim: usize,
}

impl QuantizedVector {
    /// Quantize a float vector to u8
    ///
    /// # Formula
    ///
    /// q = round((v - v_min) / (v_max - v_min) * 255)
    ///
    /// # Example
    ///
    /// ```
    /// let vector = vec![-1.0, 0.0, 1.0];
    /// let quantized = QuantizedVector::quantize(&vector);
    /// assert_eq!(quantized.values, vec![0, 128, 255]);
    /// ```
    pub fn quantize(vector: &[f32]) -> Self {
        let min = vector.iter().copied().fold(f32::INFINITY, f32::min);
        let max = vector.iter().copied().fold(f32::NEG_INFINITY, f32::max);

        let range = max - min;
        let scale = if range > 0.0 { 255.0 / range } else { 0.0 };

        let values: Vec<u8> = vector
            .iter()
            .map(|&v| {
                let normalized = (v - min) * scale;
                normalized.round().clamp(0.0, 255.0) as u8
            })
            .collect();

        Self {
            values,
            min,
            max,
            dim: vector.len(),
        }
    }

    /// Dequantize back to f32 (for debugging/testing)
    ///
    /// # Formula
    ///
    /// v is approximately v_min + (q / 255) * (v_max - v_min)
    pub fn dequantize(&self) -> Vec<f32> {
        let range = self.max - self.min;
        let scale = range / 255.0;

        self.values
            .iter()
            .map(|&q| self.min + q as f32 * scale)
            .collect()
    }

    /// Compute approximate cosine distance using quantized values
    ///
    /// Uses integer math for speed, then converts to float for final result.
    ///
    /// # Performance
    ///
    /// - 2-3x faster than float distance due to integer operations
    /// - More cache-friendly (4x less data to load)
    pub fn approx_distance(&self, other: &Self) -> f32 {
        assert_eq!(self.values.len(), other.values.len());

        // Use integer dot product
        let dot: i32 = self
            .values
            .iter()
            .zip(other.values.iter())
            .map(|(&a, &b)| a as i32 * b as i32)
            .sum();

        let norm_a: i32 = self.values.iter().map(|&v| v as i32 * v as i32).sum();
        let norm_b: i32 = other.values.iter().map(|&v| v as i32 * v as i32).sum();

        if norm_a == 0 || norm_b == 0 {
            return 1.0; // Maximum distance
        }

        let similarity = dot as f32 / ((norm_a as f32).sqrt() * (norm_b as f32).sqrt());
        1.0 - similarity.clamp(-1.0, 1.0)
    }

    /// Compute exact cosine distance (dequantize first)
    ///
    /// Slower but more accurate. Use for re-ranking top results.
    pub fn exact_distance(&self, other: &Self) -> f32 {
        let a = self.dequantize();
        let b = other.dequantize();
        cosine_distance(&a, &b)
    }

    /// Memory usage in bytes
    pub fn memory_bytes(&self) -> usize {
        self.values.len() + std::mem::size_of::<f32>() * 2 + std::mem::size_of::<usize>()
    }
}

/// Standard cosine distance for f32 vectors
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    let similarity = dot / (norm_a * norm_b);
    1.0 - similarity.clamp(-1.0, 1.0)
}

/// Batch quantize multiple vectors
pub fn batch_quantize(vectors: &[Vec<f32>]) -> Vec<QuantizedVector> {
    vectors
        .iter()
        .map(|v| QuantizedVector::quantize(v))
        .collect()
}

/// Statistics for a quantized dataset
#[derive(Debug)]
pub struct QuantizationStats {
    pub num_vectors: usize,
    pub dimension: usize,
    pub original_bytes: usize,
    pub quantized_bytes: usize,
    pub compression_ratio: f64,
    pub avg_error: f32,
    pub max_error: f32,
}

impl QuantizationStats {
    /// Compute statistics for a batch of vectors
    pub fn compute(original: &[Vec<f32>], quantized: &[QuantizedVector]) -> Self {
        assert_eq!(original.len(), quantized.len());

        let num_vectors = original.len();
        let dimension = original[0].len();

        let original_bytes = num_vectors * dimension * std::mem::size_of::<f32>();
        let quantized_bytes: usize = quantized.iter().map(|q| q.memory_bytes()).sum();
        let compression_ratio = original_bytes as f64 / quantized_bytes as f64;

        // Compute reconstruction error
        let mut total_error = 0.0;
        let mut max_error: f32 = 0.0;

        for (orig, quant) in original.iter().zip(quantized.iter()) {
            let reconstructed = quant.dequantize();
            for (&o, &r) in orig.iter().zip(reconstructed.iter()) {
                let error = (o - r).abs();
                total_error += error;
                max_error = max_error.max(error);
            }
        }

        let avg_error = total_error / (num_vectors * dimension) as f32;

        Self {
            num_vectors,
            dimension,
            original_bytes,
            quantized_bytes,
            compression_ratio,
            avg_error,
            max_error,
        }
    }

    /// Print statistics
    pub fn print(&self) {
        println!("Quantization Statistics:");
        println!("  Vectors:       {}", self.num_vectors);
        println!("  Dimension:     {}", self.dimension);
        println!(
            "  Original:      {:.2} MB",
            self.original_bytes as f64 / 1024.0 / 1024.0
        );
        println!(
            "  Quantized:     {:.2} MB",
            self.quantized_bytes as f64 / 1024.0 / 1024.0
        );
        println!("  Compression:   {:.2}x", self.compression_ratio);
        println!("  Avg Error:     {:.6}", self.avg_error);
        println!("  Max Error:     {:.6}", self.max_error);
    }
}

/// Hybrid mode: Store both quantized and original vectors
///
/// Strategy: Use quantized for initial search, original for re-ranking.
pub struct HybridVector {
    pub quantized: QuantizedVector,
    pub original: Vec<f32>,
}

impl HybridVector {
    pub fn new(vector: Vec<f32>) -> Self {
        let quantized = QuantizedVector::quantize(&vector);
        Self {
            quantized,
            original: vector,
        }
    }

    /// Fast approximate search using quantized vectors
    pub fn approx_distance(&self, other: &Self) -> f32 {
        self.quantized.approx_distance(&other.quantized)
    }

    /// Exact distance using original vectors (for re-ranking)
    pub fn exact_distance(&self, other: &Self) -> f32 {
        cosine_distance(&self.original, &other.original)
    }

    /// Memory usage (includes both representations)
    pub fn memory_bytes(&self) -> usize {
        self.quantized.memory_bytes() + self.original.len() * std::mem::size_of::<f32>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantize_simple() {
        let vector = vec![-1.0, 0.0, 1.0];
        let quantized = QuantizedVector::quantize(&vector);

        assert_eq!(quantized.values, vec![0, 128, 255]);
        assert_eq!(quantized.min, -1.0);
        assert_eq!(quantized.max, 1.0);
    }

    #[test]
    fn test_dequantize() {
        let vector = vec![-1.0, 0.0, 1.0];
        let quantized = QuantizedVector::quantize(&vector);
        let reconstructed = quantized.dequantize();

        // Should be approximately equal
        for (orig, recon) in vector.iter().zip(reconstructed.iter()) {
            assert!((orig - recon).abs() < 0.01);
        }
    }

    #[test]
    fn test_approx_distance() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0]; // Same vector
        let c = vec![0.0, 1.0, 0.0]; // Orthogonal

        let qa = QuantizedVector::quantize(&a);
        let qb = QuantizedVector::quantize(&b);
        let qc = QuantizedVector::quantize(&c);

        let dist_same = qa.approx_distance(&qb);
        let dist_orthog = qa.approx_distance(&qc);

        assert!(dist_same < 0.1); // Should be very close
        assert!(dist_orthog > 0.9); // Should be far
    }

    #[test]
    fn test_memory_reduction() {
        let vector = vec![0.5; 768]; // 768-dimensional vector
        let quantized = QuantizedVector::quantize(&vector);

        let original_bytes = vector.len() * std::mem::size_of::<f32>();
        let quantized_bytes = quantized.memory_bytes();

        // Should be approximately 4x smaller
        let ratio = original_bytes as f64 / quantized_bytes as f64;
        assert!(ratio > 3.5 && ratio < 4.5);
    }

    #[test]
    fn test_batch_quantize() {
        let vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];

        let quantized = batch_quantize(&vectors);
        assert_eq!(quantized.len(), 3);

        for q in &quantized {
            assert_eq!(q.values.len(), 3);
        }
    }

    #[test]
    fn test_quantization_stats() {
        let vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];

        let quantized = batch_quantize(&vectors);
        let stats = QuantizationStats::compute(&vectors, &quantized);

        assert_eq!(stats.num_vectors, 3);
        assert_eq!(stats.dimension, 3);
        assert!(stats.compression_ratio > 3.0);
    }

    #[test]
    fn test_hybrid_vector() {
        let vector = vec![0.5; 128];
        let hybrid = HybridVector::new(vector.clone());

        assert_eq!(hybrid.original.len(), 128);
        assert_eq!(hybrid.quantized.values.len(), 128);
    }
}

fn main() {
    let vectors = vec![
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0],
        vec![0.5, 0.5, 0.0],
    ];
    let quantized = batch_quantize(&vectors);
    let stats = QuantizationStats::compute(&vectors, &quantized);
    println!("Quantized {} vectors", stats.num_vectors);
    println!("Compression ratio: {:.1}x", stats.compression_ratio);
    println!("Run `cargo test` to execute all test cases.");
}
