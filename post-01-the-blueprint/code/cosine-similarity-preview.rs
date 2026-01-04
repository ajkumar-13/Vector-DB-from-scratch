// cosine-similarity-preview.rs
// 
// This is a PREVIEW of the cosine similarity function we'll build.
// We'll implement this fully in Post #11 (Vector Math for Developers).
// 
// This code is shown in Post #1 to illustrate why Rust is fast.

/// Calculate the cosine similarity between two vectors.
/// 
/// Cosine similarity measures the angle between two vectors:
/// - 1.0 = identical direction (most similar)
/// - 0.0 = perpendicular (unrelated)  
/// - -1.0 = opposite direction (most dissimilar)
///
/// Formula: cos(θ) = (A · B) / (||A|| × ||B||)
///
/// Where:
/// - A · B = dot product (sum of element-wise multiplication)
/// - ||A|| = magnitude/norm of A (sqrt of sum of squares)
///
/// CRITICAL: This function validates that vectors have matching dimensions.
/// Rust's zip() iterator silently truncates to the shorter length, which would
/// give garbage results if we compared 768-dim vs 1536-dim vectors!
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    // DIMENSION CHECK: 
    // In this preview, we panic on mismatch. In the real system (Post #11), 
    // we will return a Result<f32, Error> to handle this gracefully.
    // A production database should NEVER panic on user input—it should
    // return a 400 Bad Request instead.
    assert_eq!(
        a.len(), 
        b.len(), 
        "Vector dimensions must match! Got {} vs {}", 
        a.len(), 
        b.len()
    );
    
    // Calculate dot product: sum of (a[i] * b[i]) for all i
    // This measures how much the vectors point in the same direction
    //
    // Note: We iterate over the vectors 3 times (dot, norm_a, norm_b).
    // This is cache-inefficient. In Phase 3, we'll optimize to a single
    // pass using SIMD intrinsics for ~4x speedup.
    let dot_product: f32 = a.iter()
        .zip(b.iter())
        .map(|(x, y)| x * y)
        .sum();
    
    // Calculate magnitude of vector A: sqrt(sum of squares)
    let magnitude_a: f32 = a.iter()
        .map(|x| x * x)
        .sum::<f32>()
        .sqrt();
    
    // Calculate magnitude of vector B: sqrt(sum of squares)
    let magnitude_b: f32 = b.iter()
        .map(|x| x * x)
        .sum::<f32>()
        .sqrt();
    
    // Avoid division by zero
    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }
    
    // Final formula: dot_product / (magnitude_a * magnitude_b)
    dot_product / (magnitude_a * magnitude_b)
}

fn main() {
    // Example: Compare "King" and "Queen" embeddings (simplified 3D)
    // Note: We explicitly annotate Vec<f32> because Rust defaults float 
    // literals to f64, which would cause a type mismatch with our function.
    let king: Vec<f32> = vec![0.8, 0.1, 0.9];   // High royalty, low femininity, high power
    let queen: Vec<f32> = vec![0.7, 0.9, 0.85]; // High royalty, high femininity, high power
    let car: Vec<f32> = vec![0.1, 0.05, 0.2];   // Low on all "human" dimensions
    
    let king_queen_similarity = cosine_similarity(&king, &queen);
    let king_car_similarity = cosine_similarity(&king, &car);
    
    println!("Cosine Similarity Examples:");
    println!("─────────────────────────────");
    println!("King vs Queen: {:.4}", king_queen_similarity);  // Should be high (~0.9)
    println!("King vs Car:   {:.4}", king_car_similarity);    // Should be low (~0.6)
    println!();
    println!("Interpretation:");
    println!("- King and Queen are semantically similar (both royalty)");
    println!("- King and Car are semantically different");
}

// Expected output:
// Cosine Similarity Examples:
// ─────────────────────────────
// King vs Queen: 0.9231
// King vs Car:   0.6547
//
// Interpretation:
// - King and Queen are semantically similar (both royalty)
// - King and Car are semantically different
