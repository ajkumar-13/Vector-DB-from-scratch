// slice-examples.rs
//
// Runnable examples demonstrating Rust slices - essential for our vector database.
// From Post #3: Ownership, Borrowing, and Memory Management
//
// Run with: rustc slice-examples.rs && ./slice-examples

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  RUST SLICES - THE FOUNDATION OF VECTOR DATABASE I/O");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 1: Basic Slice Syntax
    // ─────────────────────────────────────────────────────────────────
    println!("1. BASIC SLICE SYNTAX");
    println!("─────────────────────────────────────────────────────────────");

    let arr = [1, 2, 3, 4, 5];

    let full_slice = &arr[..]; // All elements
    let first_two = &arr[..2]; // Elements 0, 1
    let last_three = &arr[2..]; // Elements 2, 3, 4
    let middle = &arr[1..4]; // Elements 1, 2, 3

    println!("   Original array: {:?}", arr);
    println!("   &arr[..]  (full):        {:?}", full_slice);
    println!("   &arr[..2] (first two):   {:?}", first_two);
    println!("   &arr[2..] (last three):  {:?}", last_three);
    println!("   &arr[1..4] (middle):     {:?}", middle);
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 2: String Slices (&str)
    // ─────────────────────────────────────────────────────────────────
    println!("2. STRING SLICES (&str)");
    println!("─────────────────────────────────────────────────────────────");

    let s = String::from("Hello, Rust!");

    let hello = &s[0..5]; // "Hello"
    let rust = &s[7..11]; // "Rust"

    println!("   Full string: \"{}\"", s);
    println!("   &s[0..5]:    \"{}\"", hello);
    println!("   &s[7..11]:   \"{}\"", rust);
    println!("   → String literals (\"hello\") are actually &str slices!");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 3: Vector Slices - Critical for Our Database!
    // ─────────────────────────────────────────────────────────────────
    println!("3. f32 SLICES FOR VECTOR EMBEDDINGS");
    println!("─────────────────────────────────────────────────────────────");

    // This is exactly how we'll handle vector embeddings!
    let embedding: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];

    // Get a slice for processing - no allocation, just a view
    let slice: &[f32] = &embedding;

    println!("   Vector (owned):  {:?}", embedding);
    println!("   Slice (borrowed): {:?}", slice);
    println!("   Dimensions: {}", slice.len());

    // Calculate magnitude using a slice
    let magnitude = calculate_magnitude(slice);
    println!("   Magnitude: {:.4}", magnitude);
    println!("   → In our DB, we'll read f32 slices directly from memory-mapped files!");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 4: Slice from Memory-Mapped Bytes (Preview)
    // ─────────────────────────────────────────────────────────────────
    println!("4. BYTES TO f32 SLICE (MEMORY-MAPPING PREVIEW)");
    println!("─────────────────────────────────────────────────────────────");

    // Simulate raw bytes from a memory-mapped file
    let raw_bytes: [u8; 16] = [
        0x00, 0x00, 0x80, 0x3F, // 1.0f32 in little-endian
        0x00, 0x00, 0x00, 0x40, // 2.0f32
        0x00, 0x00, 0x40, 0x40, // 3.0f32
        0x00, 0x00, 0x80, 0x40, // 4.0f32
    ];

    println!("   Raw bytes: {:02X?}", raw_bytes);

    // Safely convert bytes to f32 slice
    // In production, we'd use `bytemuck` crate for this
    let floats = bytes_to_f32_slice(&raw_bytes);
    println!("   As f32 slice: {:?}", floats);
    println!("   → This is how we'll read vectors from disk with ZERO copies!");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 5: Slices in Functions - The Idiomatic Pattern
    // ─────────────────────────────────────────────────────────────────
    println!("5. IDIOMATIC FUNCTION PARAMETERS");
    println!("─────────────────────────────────────────────────────────────");

    let vec1: Vec<f32> = vec![1.0, 0.0, 0.0];
    let vec2: Vec<f32> = vec![0.0, 1.0, 0.0];
    let vec3: Vec<f32> = vec![1.0, 1.0, 0.0];

    // Functions take &[f32], not &Vec<f32> - more flexible!
    println!(
        "   vec1 · vec2 = {:.2} (orthogonal)",
        dot_product(&vec1, &vec2)
    );
    println!(
        "   vec1 · vec1 = {:.2} (same vector)",
        dot_product(&vec1, &vec1)
    );
    println!(
        "   vec1 · vec3 = {:.2} (partial overlap)",
        dot_product(&vec1, &vec3)
    );
    println!();
    println!("   → Use &[T] in function signatures, not &Vec<T>!");
    println!("     This accepts arrays, vecs, and other slices.");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 6: Mutable Slices
    // ─────────────────────────────────────────────────────────────────
    println!("6. MUTABLE SLICES (&mut [T])");
    println!("─────────────────────────────────────────────────────────────");

    let mut data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    println!("   Before normalization: {:?}", data);

    // Normalize in-place using a mutable slice
    normalize_in_place(&mut data);

    println!("   After normalization:  {:?}", data);
    println!("   → Mutable slices let us modify data in-place (no allocations).");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 7: Split Slices
    // ─────────────────────────────────────────────────────────────────
    println!("7. SPLITTING SLICES");
    println!("─────────────────────────────────────────────────────────────");

    let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];

    let (left, right) = data.split_at(3);
    println!("   Original: {:?}", data);
    println!("   Left half:  {:?}", left);
    println!("   Right half: {:?}", right);

    // Process in chunks (useful for batching)
    println!("   Chunks of 2:");
    for (i, chunk) in data.chunks(2).enumerate() {
        println!("      Chunk {}: {:?}", i, chunk);
    }
    println!("   → We'll use chunking for batch vector operations!");
    println!();

    println!("═══════════════════════════════════════════════════════════");
    println!("  SLICE SUMMARY FOR VECTOR DATABASES:");
    println!("  • &[f32] - View into embedding data (zero-copy)");
    println!("  • &mut [f32] - In-place normalization, modifications");
    println!("  • Memory-mapped files give us &[u8] → reinterpret as &[f32]");
    println!("  • Prefer &[T] over &Vec<T> in function signatures");
    println!("═══════════════════════════════════════════════════════════");
}

/// Calculate the magnitude (L2 norm) of a vector.
fn calculate_magnitude(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Calculate dot product of two vectors.
/// Takes slices, so works with Vec, arrays, or other slices.
fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have same dimension");
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Normalize a vector in-place (mutable slice).
fn normalize_in_place(v: &mut [f32]) {
    let magnitude: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for x in v.iter_mut() {
            *x /= magnitude;
        }
    }
}

/// Convert a byte slice to f32 slice.
/// In production, use `bytemuck::cast_slice` for safety.
fn bytes_to_f32_slice(bytes: &[u8]) -> Vec<f32> {
    // This is a simplified example - in production use bytemuck!
    bytes
        .chunks_exact(4)
        .map(|chunk| {
            let arr: [u8; 4] = chunk.try_into().unwrap();
            f32::from_le_bytes(arr)
        })
        .collect()
}
