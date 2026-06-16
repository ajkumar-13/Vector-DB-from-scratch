// simd-distance.rs
// Optimized distance calculations using SIMD intrinsics
//
// This module implements hand-optimized distance functions using AVX2/AVX-512
// for significant performance improvements over scalar code.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Compute dot product using scalar operations (fallback)
pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Compute dot product with automatic SIMD selection
///
/// Uses AVX2 if available, falls back to scalar
#[cfg(target_arch = "x86_64")]
pub fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    if is_x86_feature_detected!("avx2") {
        unsafe { dot_product_avx2(a, b) }
    } else {
        dot_product_scalar(a, b)
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    dot_product_scalar(a, b)
}

/// AVX2-optimized dot product (8x f32 per instruction)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut sum = _mm256_setzero_ps(); // 8x f32 accumulator

    let chunks = len / 8;
    for i in 0..chunks {
        let offset = i * 8;

        // Load 8 floats from each array
        let va = _mm256_loadu_ps(a.as_ptr().add(offset));
        let vb = _mm256_loadu_ps(b.as_ptr().add(offset));

        // Multiply and accumulate: sum += va * vb
        sum = _mm256_fmadd_ps(va, vb, sum);
    }

    // Horizontal sum of 8 lanes
    let mut result = hsum_avx2(sum);

    // Handle remainder (scalar)
    for i in (chunks * 8)..len {
        result += a[i] * b[i];
    }

    result
}

/// Horizontal sum of __m256 (8x f32)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn hsum_avx2(v: __m256) -> f32 {
    // Add adjacent pairs: [a,b,c,d,e,f,g,h] -> [a+b, c+d, e+f, g+h, ...]
    let v = _mm256_hadd_ps(v, v);
    let v = _mm256_hadd_ps(v, v);

    // Extract low and high 128-bit lanes
    let lo = _mm256_castps256_ps128(v);
    let hi = _mm256_extractf128_ps(v, 1);

    // Add lanes and extract first element
    let sum = _mm_add_ps(lo, hi);
    _mm_cvtss_f32(sum)
}

/// Cosine distance with SIMD
pub fn cosine_distance_simd(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());

    let dot = dot_product_simd(a, b);
    let norm_a = dot_product_simd(a, a).sqrt();
    let norm_b = dot_product_simd(b, b).sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    let similarity = dot / (norm_a * norm_b);
    1.0 - similarity.clamp(-1.0, 1.0)
}

/// Euclidean distance with SIMD
#[cfg(target_arch = "x86_64")]
pub fn euclidean_distance_simd(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());

    if is_x86_feature_detected!("avx2") {
        unsafe { euclidean_distance_avx2(a, b) }
    } else {
        euclidean_distance_scalar(a, b)
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn euclidean_distance_simd(a: &[f32], b: &[f32]) -> f32 {
    euclidean_distance_scalar(a, b)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn euclidean_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut sum = _mm256_setzero_ps();

    let chunks = len / 8;
    for i in 0..chunks {
        let offset = i * 8;

        let va = _mm256_loadu_ps(a.as_ptr().add(offset));
        let vb = _mm256_loadu_ps(b.as_ptr().add(offset));

        // diff = va - vb
        let diff = _mm256_sub_ps(va, vb);

        // sum += diff * diff
        sum = _mm256_fmadd_ps(diff, diff, sum);
    }

    let mut result = hsum_avx2(sum);

    // Handle remainder
    for i in (chunks * 8)..len {
        let diff = a[i] - b[i];
        result += diff * diff;
    }

    result.sqrt()
}

fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let diff = x - y;
            diff * diff
        })
        .sum::<f32>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_product_scalar() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let result = dot_product_scalar(&a, &b);
        assert!((result - 32.0).abs() < 0.001); // 1*4 + 2*5 + 3*6 = 32
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_dot_product_simd() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = vec![8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];

        let scalar_result = dot_product_scalar(&a, &b);
        let simd_result = dot_product_simd(&a, &b);

        assert!((scalar_result - simd_result).abs() < 0.001);
    }

    #[test]
    fn test_cosine_distance() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0]; // Same vector
        let c = vec![0.0, 1.0, 0.0]; // Orthogonal

        let dist_same = cosine_distance_simd(&a, &b);
        let dist_orthog = cosine_distance_simd(&a, &c);

        assert!(dist_same < 0.001); // Very close
        assert!((dist_orthog - 1.0).abs() < 0.001); // Maximum distance
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];

        let dist = euclidean_distance_simd(&a, &b);
        assert!((dist - 5.0).abs() < 0.001); // 3-4-5 triangle
    }

    #[test]
    fn test_large_vectors() {
        let a: Vec<f32> = (0..768).map(|i| i as f32 * 0.01).collect();
        let b: Vec<f32> = (0..768).map(|i| (768 - i) as f32 * 0.01).collect();

        let scalar_dot = dot_product_scalar(&a, &b);
        let simd_dot = dot_product_simd(&a, &b);

        // Should be approximately equal
        assert!((scalar_dot - simd_dot).abs() < 0.1);
    }
}

#[cfg(all(test, not(target_env = "msvc")))]
mod benches {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_dot_product_scalar_768(b: &mut Bencher) {
        let a: Vec<f32> = (0..768).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..768).map(|i| (768 - i) as f32).collect();

        b.iter(|| dot_product_scalar(&a, &b));
    }

    #[bench]
    #[cfg(target_arch = "x86_64")]
    fn bench_dot_product_simd_768(b: &mut Bencher) {
        let a: Vec<f32> = (0..768).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..768).map(|i| (768 - i) as f32).collect();

        b.iter(|| dot_product_simd(&a, &b));
    }
}

fn main() {
    let a: Vec<f32> = (0..128).map(|i| (i as f32) / 128.0).collect();
    let b: Vec<f32> = (0..128).map(|i| ((128 - i) as f32) / 128.0).collect();

    let naive = dot_product_scalar(&a, &b);
    println!("Scalar dot product: {:.6}", naive);

    #[cfg(target_arch = "x86_64")]
    {
        let simd = dot_product_simd(&a, &b);
        println!("SIMD dot product:  {:.6}", simd);
        println!("Match: {}", (naive - simd).abs() < 1e-4);
    }
}
