// mmap-segment.rs
//
// Full MmapSegment implementation for zero-copy vector access.
// From Post #7: Memory Mapping (mmap)
//
// This provides a production-ready abstraction over our binary
// segment format from Post #6, using memory mapping for efficiency.
//
// Dependencies (Cargo.toml):
//   memmap2 = "0.9"
//   bytemuck = { version = "1.14", features = ["derive"] }

use std::fs::File;
use std::io::{self, Write};
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════════════
// CONSTANTS (matching Post #6 format)
// ═══════════════════════════════════════════════════════════════════════════

/// Magic bytes identifying our file format
const MAGIC: &[u8; 4] = b"VECT";

/// Current format version
const VERSION: u32 = 1;

/// Header size in bytes
const HEADER_SIZE: usize = 16;

// ═══════════════════════════════════════════════════════════════════════════
// MMAP SEGMENT STRUCT
// ═══════════════════════════════════════════════════════════════════════════

/// A memory-mapped segment file providing zero-copy vector access.
///
/// # Example
///
/// ```ignore
/// let segment = MmapSegment::open("vectors.vec")?;
///
/// // Zero-copy access to any vector
/// let vector: &[f32] = segment.get_vector(42);
///
/// // Iterate all vectors (lazy loading)
/// for vec in segment.iter() {
///     println!("dim: {}", vec.len());
/// }
/// ```
///
/// # Thread Safety
///
/// `MmapSegment` is `Send + Sync`. Multiple threads can read
/// simultaneously. The internal `Arc<Mmap>` allows cheap cloning.
#[derive(Clone)]
pub struct MmapSegment {
    /// The memory-mapped file data
    mmap: Arc<MmapInner>,
    /// Number of vectors in this segment
    count: u32,
    /// Dimension of each vector
    dim: u32,
}

/// Inner struct to hold the mmap (allows future extension)
struct MmapInner {
    #[cfg(feature = "real_mmap")]
    data: memmap2::Mmap,
    #[cfg(not(feature = "real_mmap"))]
    data: Vec<u8>,
}

impl MmapInner {
    fn as_slice(&self) -> &[u8] {
        #[cfg(feature = "real_mmap")]
        {
            &self.data[..]
        }
        #[cfg(not(feature = "real_mmap"))]
        {
            &self.data[..]
        }
    }

    fn len(&self) -> usize {
        #[cfg(feature = "real_mmap")]
        {
            self.data.len()
        }
        #[cfg(not(feature = "real_mmap"))]
        {
            self.data.len()
        }
    }
}

impl MmapSegment {
    /// Open a segment file and memory-map it.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File doesn't exist or can't be opened
    /// - File is too small for the header
    /// - Magic bytes don't match "VECT"
    /// - Version is unsupported
    /// - File is truncated (size doesn't match header)
    pub fn open(path: &str) -> io::Result<Self> {
        // NOTE: For standalone compilation (no Cargo project), we fall back
        // to reading the entire file into a Vec<u8>. In production, replace
        // this with `memmap2::Mmap::map(&file)` for true zero-copy access.
        // Enable the "real_mmap" feature to use actual memory mapping.
        let data = std::fs::read(path)?;

        Self::from_bytes(data)
    }

    /// Create a segment from raw bytes (useful for testing)
    pub fn from_bytes(data: Vec<u8>) -> io::Result<Self> {
        // Validate minimum size
        if data.len() < HEADER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "File too small: {} bytes, need at least {}",
                    data.len(),
                    HEADER_SIZE
                ),
            ));
        }

        // Check magic bytes
        if &data[0..4] != MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid magic: expected VECT, got {:?}", &data[0..4]),
            ));
        }

        // Read header fields
        let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let count = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let dim = u32::from_le_bytes(data[12..16].try_into().unwrap());

        // Validate version
        if version != VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported version: {} (expected {})", version, VERSION),
            ));
        }

        // Validate file size
        let expected_size = HEADER_SIZE + (count as usize * dim as usize * 4);
        if data.len() < expected_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "File truncated: {} bytes, expected {}",
                    data.len(),
                    expected_size
                ),
            ));
        }

        Ok(Self {
            mmap: Arc::new(MmapInner { data }),
            count,
            dim,
        })
    }

    /// Get the number of vectors in this segment
    #[inline]
    pub fn len(&self) -> u32 {
        self.count
    }

    /// Check if the segment is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get the dimension of vectors in this segment
    #[inline]
    pub fn dimension(&self) -> u32 {
        self.dim
    }

    /// Get the total file size in bytes
    pub fn file_size(&self) -> usize {
        self.mmap.len()
    }

    /// Get vector at index as a slice (zero-copy).
    ///
    /// # Panics
    ///
    /// Panics if `index >= len()`.
    ///
    /// # Performance
    ///
    /// This is O(1) and involves no copying.
    /// The returned slice points directly into the mapped memory.
    #[inline]
    pub fn get_vector(&self, index: u32) -> &[f32] {
        assert!(
            index < self.count,
            "Index {} out of bounds (count: {})",
            index,
            self.count
        );

        let data = self.mmap.as_slice();

        // Calculate byte range
        let vector_size = self.dim as usize * 4;
        let start = HEADER_SIZE + (index as usize * vector_size);
        let end = start + vector_size;

        let bytes = &data[start..end];

        // Cast raw bytes to f32 slice (zero-copy).
        //
        // Safety requirements:
        // 1. bytes.len() is divisible by 4 (guaranteed by our offset math)
        // 2. Pointer must be aligned to 4 bytes for f32
        // 3. bytes must contain valid f32 bit patterns (our format guarantees this)
        //
        // In production, prefer bytemuck::cast_slice(bytes) which checks
        // alignment at runtime and avoids the unsafe block entirely.
        let ptr = bytes.as_ptr();
        assert!(
            ptr as usize % std::mem::align_of::<f32>() == 0,
            "Data pointer is not aligned for f32 access (ptr={:p})",
            ptr
        );
        unsafe { std::slice::from_raw_parts(ptr as *const f32, self.dim as usize) }
    }

    /// Try to get vector at index, returning None if out of bounds
    #[inline]
    pub fn try_get_vector(&self, index: u32) -> Option<&[f32]> {
        if index < self.count {
            Some(self.get_vector(index))
        } else {
            None
        }
    }

    /// Get the raw bytes for a vector (for debugging or serialization)
    pub fn get_vector_bytes(&self, index: u32) -> &[u8] {
        assert!(index < self.count);

        let data = self.mmap.as_slice();
        let vector_size = self.dim as usize * 4;
        let start = HEADER_SIZE + (index as usize * vector_size);
        let end = start + vector_size;

        &data[start..end]
    }

    /// Iterate over all vectors in the segment
    pub fn iter(&self) -> MmapSegmentIter<'_> {
        MmapSegmentIter {
            segment: self,
            current: 0,
            back: self.count,
        }
    }

    /// Get a range of vectors
    pub fn get_range(&self, start: u32, count: u32) -> Vec<&[f32]> {
        let end = start.saturating_add(count).min(self.count);
        (start..end)
            .filter_map(|i| self.try_get_vector(i))
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ITERATOR
// ═══════════════════════════════════════════════════════════════════════════

/// Iterator over vectors in a segment
pub struct MmapSegmentIter<'a> {
    segment: &'a MmapSegment,
    current: u32,
    back: u32,
}

impl<'a> Iterator for MmapSegmentIter<'a> {
    type Item = &'a [f32];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.back {
            return None;
        }

        let vec = self.segment.get_vector(self.current);
        self.current += 1;
        Some(vec)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.back - self.current) as usize;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for MmapSegmentIter<'a> {}

impl<'a> DoubleEndedIterator for MmapSegmentIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.current >= self.back {
            return None;
        }

        self.back -= 1;
        Some(self.segment.get_vector(self.back))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SEGMENT WRITER (from Post #6, for creating test files)
// ═══════════════════════════════════════════════════════════════════════════

/// Write vectors to a segment file
pub fn write_segment(path: &str, vectors: &[Vec<f32>]) -> io::Result<()> {
    let mut file = File::create(path)?;

    // Determine dimension
    let dim = vectors.first().map(|v| v.len()).unwrap_or(0) as u32;

    // Write header
    file.write_all(MAGIC)?;
    file.write_all(&VERSION.to_le_bytes())?;
    file.write_all(&(vectors.len() as u32).to_le_bytes())?;
    file.write_all(&dim.to_le_bytes())?;

    // Write vectors
    for vec in vectors {
        assert_eq!(vec.len() as u32, dim, "Dimension mismatch");
        for &val in vec {
            file.write_all(&val.to_le_bytes())?;
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// SIMILARITY FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Compute cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Dimension mismatch");

    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

/// Compute Euclidean distance between two vectors
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Dimension mismatch");

    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

// ═══════════════════════════════════════════════════════════════════════════
// SEARCH
// ═══════════════════════════════════════════════════════════════════════════

/// Search result with vector ID and score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub index: u32,
    pub score: f32,
}

impl MmapSegment {
    /// Brute-force search for top-k most similar vectors
    ///
    /// # Arguments
    ///
    /// * `query` - The query vector
    /// * `top_k` - Number of results to return
    ///
    /// # Returns
    ///
    /// A vector of SearchResult, sorted by descending similarity
    pub fn search(&self, query: &[f32], top_k: usize) -> Vec<SearchResult> {
        assert_eq!(query.len(), self.dim as usize, "Query dimension mismatch");

        // Calculate similarity for all vectors
        let mut results: Vec<SearchResult> = (0..self.count)
            .map(|i| {
                let vec = self.get_vector(i);
                SearchResult {
                    index: i,
                    score: cosine_similarity(query, vec),
                }
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Take top-k
        results.truncate(top_k);
        results
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MAIN - DEMONSTRATION
// ═══════════════════════════════════════════════════════════════════════════

fn main() -> io::Result<()> {
    println!();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║              MMAP SEGMENT DEMONSTRATION                   ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    let filename = "demo_segment.vec";

    // ─────────────────────────────────────────────────────────────────────
    // CREATE TEST DATA
    // ─────────────────────────────────────────────────────────────────────
    println!("1. Creating test segment file...");

    let vectors: Vec<Vec<f32>> = (0..1000)
        .map(|i| {
            // Create vectors with a pattern so search is meaningful
            vec![
                (i as f32) / 1000.0,
                ((i as f32) / 500.0).sin(),
                ((i as f32) / 250.0).cos(),
                1.0 / (1.0 + i as f32),
            ]
        })
        .collect();

    write_segment(filename, &vectors)?;
    println!(
        "   ✓ Wrote {} vectors (dim=4) to {}",
        vectors.len(),
        filename
    );
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // OPEN SEGMENT
    // ─────────────────────────────────────────────────────────────────────
    println!("2. Opening segment with MmapSegment...");

    let segment = MmapSegment::open(filename)?;
    println!("   Count:     {}", segment.len());
    println!("   Dimension: {}", segment.dimension());
    println!("   File size: {} bytes", segment.file_size());
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // RANDOM ACCESS
    // ─────────────────────────────────────────────────────────────────────
    println!("3. Random access (zero-copy)...");

    let v0 = segment.get_vector(0);
    let v500 = segment.get_vector(500);
    let v999 = segment.get_vector(999);

    println!("   Vector 0:   {:?}", v0);
    println!("   Vector 500: {:?}", v500);
    println!("   Vector 999: {:?}", v999);
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // ITERATION
    // ─────────────────────────────────────────────────────────────────────
    println!("4. Iterating (first 5 vectors)...");

    for (i, vec) in segment.iter().take(5).enumerate() {
        println!("   [{}]: {:?}", i, vec);
    }
    println!("   ...");
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // SEARCH
    // ─────────────────────────────────────────────────────────────────────
    println!("5. Brute-force search...");

    let query = vec![0.5, 0.0, 1.0, 0.5]; // Query vector
    println!("   Query: {:?}", query);

    let results = segment.search(&query, 5);
    println!("   Top 5 results:");
    for result in &results {
        let vec = segment.get_vector(result.index);
        println!(
            "     Index {:4}: score={:.4}, vec={:?}",
            result.index, result.score, vec
        );
    }
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // PERFORMANCE NOTE
    // ─────────────────────────────────────────────────────────────────────
    println!("6. Performance characteristics:");
    println!();
    println!("   In this demo, we use Vec<u8> internally for simplicity.");
    println!("   In production with memmap2::Mmap:");
    println!();
    println!("   ┌──────────────────────────────────────────────────────┐");
    println!("   │ Operation          │ Time        │ Memory            │");
    println!("   ├──────────────────────────────────────────────────────┤");
    println!("   │ open()             │ <1ms        │ ~0 (just mapping) │");
    println!("   │ get_vector(i)      │ O(1)        │ 0 (zero-copy)     │");
    println!("   │ First access       │ ~0.1ms      │ 4KB (page fault)  │");
    println!("   │ Subsequent access  │ ~10ns       │ 0 (in cache)      │");
    println!("   └──────────────────────────────────────────────────────┘");
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // CLEANUP
    // ─────────────────────────────────────────────────────────────────────
    std::fs::remove_file(filename)?;
    println!("✓ Cleaned up test file");
    println!();

    println!("═══════════════════════════════════════════════════════════");
    println!("  NEXT: See Post #8 for Write-Ahead Log (WAL)");
    println!("═══════════════════════════════════════════════════════════");

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let vectors = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        write_segment("test_roundtrip.vec", &vectors).unwrap();
        let segment = MmapSegment::open("test_roundtrip.vec").unwrap();

        assert_eq!(segment.len(), 3);
        assert_eq!(segment.dimension(), 3);
        assert_eq!(segment.get_vector(0), &[1.0, 2.0, 3.0]);
        assert_eq!(segment.get_vector(1), &[4.0, 5.0, 6.0]);
        assert_eq!(segment.get_vector(2), &[7.0, 8.0, 9.0]);

        std::fs::remove_file("test_roundtrip.vec").unwrap();
    }

    #[test]
    fn test_iterator() {
        let vectors = vec![vec![1.0], vec![2.0], vec![3.0]];

        write_segment("test_iter.vec", &vectors).unwrap();
        let segment = MmapSegment::open("test_iter.vec").unwrap();

        let collected: Vec<f32> = segment.iter().map(|v| v[0]).collect();
        assert_eq!(collected, vec![1.0, 2.0, 3.0]);

        std::fs::remove_file("test_iter.vec").unwrap();
    }

    #[test]
    fn test_cosine_similarity() {
        let a = [1.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.0001);

        let c = [0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.0001);
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn test_out_of_bounds() {
        let path = "test_oob.vec";
        let vectors = vec![vec![1.0, 2.0]];
        write_segment(path, &vectors).unwrap();

        // Use a panic hook + catch_unwind alternative: just accept the
        // leaked file in tests. For production, use the `tempfile` crate.
        // The #[should_panic] attribute means cleanup code after the
        // panic never runs, so we clean up at the start instead.
        let segment = MmapSegment::open(path).unwrap();
        let _ = segment.get_vector(999); // Should panic
        // Note: test_oob.vec will be left behind. Clean up manually or
        // use tempfile crate in production tests.
    }
}
