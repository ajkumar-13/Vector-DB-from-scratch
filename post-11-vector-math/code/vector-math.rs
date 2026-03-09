// post-11-vector-math/code/vector-math.rs
// Core vector math implementations for similarity search
//
// Run with: rustc vector-math.rs && ./vector-math
// Or: rustc --test vector-math.rs && ./vector-math

use std::time::Instant;

// ============================================================================
// Core Math Functions
// ============================================================================

/// Calculate the magnitude (L2 norm / Euclidean norm) of a vector
///
/// Formula: ||v|| = sqrt(sum(v[i]^2))
#[inline]
pub fn magnitude(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Calculate the dot product of two vectors
///
/// Formula: a · b = sum(a[i] * b[i])
#[inline]
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Dimension mismatch");

    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Calculate cosine similarity between two vectors
///
/// Formula: cos(θ) = (a · b) / (||a|| * ||b||)
/// Returns a value in the range [-1, 1]
#[inline]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(
        a.len(),
        b.len(),
        "Dimension mismatch: {} vs {}",
        a.len(),
        b.len()
    );

    let dot = dot_product(a, b);
    let mag_a = magnitude(a);
    let mag_b = magnitude(b);

    // Handle zero vectors
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }

    dot / (mag_a * mag_b)
}

/// Cosine similarity for pre-normalized vectors (fast path)
///
/// When both vectors have magnitude = 1.0, cosine similarity = dot product
#[inline]
pub fn cosine_similarity_normalized(a: &[f32], b: &[f32]) -> f32 {
    dot_product(a, b)
}

/// Calculate Euclidean distance (L2 distance) between two vectors
///
/// Formula: d(a, b) = sqrt(sum((a[i] - b[i])^2))
#[inline]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Dimension mismatch");

    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Calculate squared Euclidean distance (avoids sqrt)
///
/// Useful for comparisons where we only need relative ordering
#[inline]
pub fn euclidean_distance_squared(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Dimension mismatch");

    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum()
}

/// Calculate Manhattan distance (L1 distance)
///
/// Formula: d(a, b) = sum(|a[i] - b[i]|)
#[inline]
pub fn manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Dimension mismatch");

    a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
}

// ============================================================================
// Normalization Functions
// ============================================================================

/// Normalize a vector to unit length (magnitude = 1.0)
///
/// Returns a new vector where ||v|| = 1.0
pub fn normalize(v: &[f32]) -> Vec<f32> {
    let mag = magnitude(v);

    if mag == 0.0 {
        return vec![0.0; v.len()];
    }

    v.iter().map(|x| x / mag).collect()
}

/// Normalize a vector in place
pub fn normalize_mut(v: &mut [f32]) {
    let mag = magnitude(v);

    if mag != 0.0 {
        for x in v.iter_mut() {
            *x /= mag;
        }
    }
}

/// Check if a vector is normalized (magnitude ≈ 1.0)
pub fn is_normalized(v: &[f32], epsilon: f32) -> bool {
    (magnitude(v) - 1.0).abs() < epsilon
}

// ============================================================================
// Vector Struct (Optional Wrapper)
// ============================================================================

/// A normalized vector that stores both raw and normalized forms
#[derive(Debug, Clone)]
pub struct NormalizedVector {
    /// The normalized vector (magnitude = 1.0)
    pub data: Vec<f32>,
    /// Original magnitude before normalization
    pub original_magnitude: f32,
}

impl NormalizedVector {
    pub fn new(v: Vec<f32>) -> Self {
        let original_magnitude = magnitude(&v);
        let data = normalize(&v);

        Self {
            data,
            original_magnitude,
        }
    }

    /// Fast cosine similarity with another normalized vector
    pub fn cosine_similarity(&self, other: &NormalizedVector) -> f32 {
        dot_product(&self.data, &other.data)
    }

    pub fn dimension(&self) -> usize {
        self.data.len()
    }
}

// ============================================================================
// Demo
// ============================================================================

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              Vector Math Demonstration                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Demo 1: Basic operations
    println!("═══ Demo 1: Basic Operations ═══\n");
    {
        let v = [3.0, 4.0];
        println!("Vector: {:?}", v);
        println!("Magnitude: {} (expected: 5.0)", magnitude(&v));

        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        println!("\nDot product of {:?} and {:?}:", a, b);
        println!("Result: {} (expected: 32.0)", dot_product(&a, &b));
    }

    // Demo 2: Similarity metrics
    println!("\n═══ Demo 2: Similarity Metrics ═══\n");
    {
        let king = [0.8, 0.6, 0.2];
        let queen = [0.75, 0.65, 0.15];
        let car = [-0.5, 0.3, 0.9];

        println!("Comparing semantic concepts:");
        println!("  King:  {:?}", king);
        println!("  Queen: {:?}", queen);
        println!("  Car:   {:?}", car);

        println!("\nCosine Similarity:");
        println!(
            "  King vs Queen: {:.4} (similar)",
            cosine_similarity(&king, &queen)
        );
        println!(
            "  King vs Car:   {:.4} (unrelated)",
            cosine_similarity(&king, &car)
        );

        println!("\nEuclidean Distance:");
        println!(
            "  King vs Queen: {:.4} (close)",
            euclidean_distance(&king, &queen)
        );
        println!(
            "  King vs Car:   {:.4} (far)",
            euclidean_distance(&king, &car)
        );
    }

    // Demo 3: Special cases
    println!("\n═══ Demo 3: Special Cases ═══\n");
    {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        let c = [-1.0, 0.0];

        println!("Perpendicular vectors:");
        println!("  a = {:?}, b = {:?}", a, b);
        println!(
            "  Cosine: {:.4} (orthogonal = 0)",
            cosine_similarity(&a, &b)
        );

        println!("\nOpposite vectors:");
        println!("  a = {:?}, c = {:?}", a, c);
        println!("  Cosine: {:.4} (opposite = -1)", cosine_similarity(&a, &c));

        println!("\nIdentical vectors:");
        println!("  a = {:?}", a);
        println!("  Cosine: {:.4} (same = 1)", cosine_similarity(&a, &a));
    }

    // Demo 4: Normalization
    println!("\n═══ Demo 4: Normalization ═══\n");
    {
        let v = [3.0, 4.0];
        let normalized = normalize(&v);

        println!("Original:   {:?} (magnitude: {})", v, magnitude(&v));
        println!(
            "Normalized: {:?} (magnitude: {:.6})",
            normalized,
            magnitude(&normalized)
        );
        println!("Is unit vector: {}", is_normalized(&normalized, 1e-6));
    }

    // Demo 5: Optimization comparison
    println!("\n═══ Demo 5: Normalization Speedup ═══\n");
    {
        let dim = 768;
        let iterations = 100_000;

        // Generate random-ish vectors
        let a: Vec<f32> = (0..dim).map(|i| (i as f32 * 0.001).sin()).collect();
        let b: Vec<f32> = (0..dim).map(|i| (i as f32 * 0.002).cos()).collect();

        // Pre-normalize
        let a_norm = normalize(&a);
        let b_norm = normalize(&b);

        // Benchmark regular cosine similarity
        let start = Instant::now();
        let mut sum = 0.0;
        for _ in 0..iterations {
            sum += cosine_similarity(&a, &b);
        }
        let regular_time = start.elapsed();

        // Benchmark normalized cosine similarity
        let start = Instant::now();
        let mut sum_norm = 0.0;
        for _ in 0..iterations {
            sum_norm += cosine_similarity_normalized(&a_norm, &b_norm);
        }
        let normalized_time = start.elapsed();

        println!("Dimension: {}, Iterations: {}", dim, iterations);
        println!(
            "Regular cosine:    {:?} (result: {:.6})",
            regular_time,
            sum / iterations as f32
        );
        println!(
            "Normalized cosine: {:?} (result: {:.6})",
            normalized_time,
            sum_norm / iterations as f32
        );
        println!(
            "Speedup: {:.2}x",
            regular_time.as_secs_f64() / normalized_time.as_secs_f64()
        );
    }

    println!("\nAll demos complete!");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-6;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    // Magnitude tests
    #[test]
    fn test_magnitude_3_4_5() {
        assert!(approx_eq(magnitude(&[3.0, 4.0]), 5.0));
    }

    #[test]
    fn test_magnitude_unit_vectors() {
        assert!(approx_eq(magnitude(&[1.0, 0.0, 0.0]), 1.0));
        assert!(approx_eq(magnitude(&[0.0, 1.0, 0.0]), 1.0));
        assert!(approx_eq(magnitude(&[0.0, 0.0, 1.0]), 1.0));
    }

    #[test]
    fn test_magnitude_zero_vector() {
        assert!(approx_eq(magnitude(&[0.0, 0.0, 0.0]), 0.0));
    }

    // Dot product tests
    #[test]
    fn test_dot_product_basic() {
        assert!(approx_eq(
            dot_product(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]),
            32.0
        ));
    }

    #[test]
    fn test_dot_product_perpendicular() {
        assert!(approx_eq(dot_product(&[1.0, 0.0], &[0.0, 1.0]), 0.0));
    }

    #[test]
    fn test_dot_product_opposite() {
        assert!(approx_eq(dot_product(&[1.0, 0.0], &[-1.0, 0.0]), -1.0));
    }

    // Cosine similarity tests
    #[test]
    fn test_cosine_identical() {
        let v = [1.0, 2.0, 3.0];
        assert!(approx_eq(cosine_similarity(&v, &v), 1.0));
    }

    #[test]
    fn test_cosine_opposite() {
        let a = [1.0, 0.0];
        let b = [-1.0, 0.0];
        assert!(approx_eq(cosine_similarity(&a, &b), -1.0));
    }

    #[test]
    fn test_cosine_perpendicular() {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        assert!(approx_eq(cosine_similarity(&a, &b), 0.0));
    }

    #[test]
    fn test_cosine_zero_vector() {
        let a = [1.0, 2.0];
        let zero = [0.0, 0.0];
        assert!(approx_eq(cosine_similarity(&a, &zero), 0.0));
    }

    // Euclidean distance tests
    #[test]
    fn test_euclidean_3_4_5() {
        let a = [0.0, 0.0];
        let b = [3.0, 4.0];
        assert!(approx_eq(euclidean_distance(&a, &b), 5.0));
    }

    #[test]
    fn test_euclidean_same_point() {
        let v = [1.0, 2.0, 3.0];
        assert!(approx_eq(euclidean_distance(&v, &v), 0.0));
    }

    // Normalization tests
    #[test]
    fn test_normalize() {
        let v = normalize(&[3.0, 4.0]);
        assert!(approx_eq(magnitude(&v), 1.0));
    }

    #[test]
    fn test_normalize_preserves_direction() {
        let v = [3.0, 4.0];
        let n = normalize(&v);
        // Normalized vector should point in same direction
        assert!(approx_eq(cosine_similarity(&v, &n), 1.0));
    }

    #[test]
    fn test_normalize_zero_vector() {
        let v = normalize(&[0.0, 0.0]);
        assert_eq!(v, vec![0.0, 0.0]);
    }

    #[test]
    fn test_normalized_cosine_equals_regular() {
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];

        let regular = cosine_similarity(&a, &b);

        let a_norm = normalize(&a);
        let b_norm = normalize(&b);
        let fast = cosine_similarity_normalized(&a_norm, &b_norm);

        assert!(approx_eq(regular, fast));
    }

    // NormalizedVector tests
    #[test]
    fn test_normalized_vector_struct() {
        let v1 = NormalizedVector::new(vec![3.0, 4.0]);
        let v2 = NormalizedVector::new(vec![3.0, 4.0]);

        assert!(approx_eq(v1.cosine_similarity(&v2), 1.0));
        assert!(approx_eq(v1.original_magnitude, 5.0));
    }
}
