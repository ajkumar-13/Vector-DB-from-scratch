// post-12-brute-force/code/brute-force-search.rs
// Complete k-NN search implementation using heap-based selection
//
// Run with: rustc brute-force-search.rs && ./brute-force-search
// Or: rustc --test brute-force-search.rs && ./brute-force-search

use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};

// ============================================================================
// Vector Math (duplicated for standalone demo)
// ============================================================================

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        0.0
    } else {
        dot / (mag_a * mag_b)
    }
}

// ============================================================================
// Candidate Struct
// ============================================================================

/// A search result candidate with ID and similarity score
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub id: String,
    pub score: f32,
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Eq for Candidate {}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // Handle NaN safely (treat as -infinity)
        self.partial_cmp(other).unwrap_or(Ordering::Less)
    }
}

// ============================================================================
// Heap Management
// ============================================================================

/// Push a candidate into the min-heap if it is better than the worst
fn push_candidate(heap: &mut BinaryHeap<Reverse<Candidate>>, candidate: Candidate, k: usize) {
    if heap.len() < k {
        // Heap not full, just add
        heap.push(Reverse(candidate));
    } else {
        // Heap full, check if new candidate beats the worst
        if let Some(Reverse(worst)) = heap.peek() {
            if candidate.score > worst.score {
                heap.pop(); // Remove worst
                heap.push(Reverse(candidate)); // Add better
            }
        }
    }
}

// ============================================================================
// Simple VectorStore (Mock)
// ============================================================================

/// Simplified vector database for demonstration
pub struct VectorStore {
    /// In-memory vectors (recent inserts)
    memtable: HashMap<String, Vec<f32>>,

    /// Simulated disk segments (in reality, these would be mmap'd)
    segments: Vec<HashMap<String, Vec<f32>>>,

    /// Deleted IDs (tombstones)
    tombstones: HashSet<String>,
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            memtable: HashMap::new(),
            segments: Vec::new(),
            tombstones: HashSet::new(),
        }
    }

    pub fn insert(&mut self, id: String, vector: Vec<f32>) {
        self.memtable.insert(id, vector);
    }

    pub fn delete(&mut self, id: &str) {
        // Remove from memtable
        self.memtable.remove(id);

        // Add to tombstones (for segments)
        self.tombstones.insert(id.to_string());
    }

    /// Simulate flushing memtable to segment
    pub fn flush(&mut self) {
        if !self.memtable.is_empty() {
            let segment = self.memtable.clone();
            self.segments.push(segment);
            self.memtable.clear();
        }
    }

    /// Brute force k-NN search
    pub fn search(&self, query: &[f32], k: usize) -> Vec<Candidate> {
        let mut heap: BinaryHeap<Reverse<Candidate>> = BinaryHeap::new();

        // 1. Scan MemTable
        for (id, vector) in &self.memtable {
            if self.tombstones.contains(id) {
                continue; // Skip deleted
            }

            let score = cosine_similarity(query, vector);
            push_candidate(
                &mut heap,
                Candidate {
                    id: id.clone(),
                    score,
                },
                k,
            );
        }

        // 2. Scan Segments
        for segment in &self.segments {
            for (id, vector) in segment {
                if self.tombstones.contains(id) {
                    continue; // Skip deleted
                }

                let score = cosine_similarity(query, vector);
                push_candidate(
                    &mut heap,
                    Candidate {
                        id: id.clone(),
                        score,
                    },
                    k,
                );
            }
        }

        // 3. Convert heap to sorted Vec (descending order)
        let mut results: Vec<Candidate> = heap.into_iter().map(|Reverse(c)| c).collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    pub fn count(&self) -> usize {
        let mem_count = self.memtable.len();
        let seg_count: usize = self.segments.iter().map(|s| s.len()).sum();
        mem_count + seg_count - self.tombstones.len()
    }
}

// ============================================================================
// Demo
// ============================================================================

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           Brute Force k-NN Search Demo                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Demo 1: Basic search
    println!("═══ Demo 1: Basic Search ═══\n");
    {
        let mut db = VectorStore::new();

        // Insert some vectors
        db.insert("doc1".to_string(), vec![1.0, 0.0, 0.0]);
        db.insert("doc2".to_string(), vec![0.9, 0.1, 0.0]);
        db.insert("doc3".to_string(), vec![0.0, 1.0, 0.0]);
        db.insert("doc4".to_string(), vec![0.0, 0.0, 1.0]);
        db.insert("doc5".to_string(), vec![0.8, 0.2, 0.0]);

        let query = vec![1.0, 0.0, 0.0];
        let results = db.search(&query, 3);

        println!("Query: {:?}", query);
        println!("Top 3 results:");
        for (i, candidate) in results.iter().enumerate() {
            println!(
                "  {}. {} (score: {:.4})",
                i + 1,
                candidate.id,
                candidate.score
            );
        }
    }

    // Demo 2: Search with segments
    println!("\n═══ Demo 2: MemTable + Segments ═══\n");
    {
        let mut db = VectorStore::new();

        // Insert batch 1
        db.insert("seg1_v1".to_string(), vec![1.0, 0.0]);
        db.insert("seg1_v2".to_string(), vec![0.9, 0.1]);
        db.flush(); // Move to segment

        // Insert batch 2
        db.insert("seg2_v1".to_string(), vec![0.8, 0.2]);
        db.insert("seg2_v2".to_string(), vec![0.7, 0.3]);
        db.flush(); // Move to segment

        // Insert batch 3 (stays in memtable)
        db.insert("mem_v1".to_string(), vec![0.95, 0.05]);
        db.insert("mem_v2".to_string(), vec![0.85, 0.15]);

        println!("Database state:");
        println!("  Segments: {}", db.segments.len());
        println!("  MemTable: {}", db.memtable.len());
        println!("  Total: {}", db.count());

        let query = vec![1.0, 0.0];
        let results = db.search(&query, 4);

        println!("\nQuery: {:?}", query);
        println!("Top 4 results:");
        for (i, candidate) in results.iter().enumerate() {
            println!(
                "  {}. {} (score: {:.4})",
                i + 1,
                candidate.id,
                candidate.score
            );
        }
    }

    // Demo 3: Tombstone filtering
    println!("\n═══ Demo 3: Deletion with Tombstones ═══\n");
    {
        let mut db = VectorStore::new();

        db.insert("vec1".to_string(), vec![1.0, 0.0]);
        db.insert("vec2".to_string(), vec![0.9, 0.1]);
        db.insert("vec3".to_string(), vec![0.8, 0.2]);
        db.flush();

        println!("Initial count: {}", db.count());

        // Delete vec2
        db.delete("vec2");
        println!("After deleting vec2: {}", db.count());

        let query = vec![1.0, 0.0];
        let results = db.search(&query, 10);

        println!("\nSearch results (vec2 should be absent):");
        for candidate in &results {
            println!("  {} (score: {:.4})", candidate.id, candidate.score);
        }
    }

    // Demo 4: Heap behavior visualization
    println!("\n═══ Demo 4: Heap Behavior (k=3) ═══\n");
    {
        let mut heap: BinaryHeap<Reverse<Candidate>> = BinaryHeap::new();
        let k = 3;

        let candidates = vec![
            Candidate {
                id: "A".to_string(),
                score: 0.5,
            },
            Candidate {
                id: "B".to_string(),
                score: 0.9,
            },
            Candidate {
                id: "C".to_string(),
                score: 0.3,
            },
            Candidate {
                id: "D".to_string(),
                score: 0.7,
            },
            Candidate {
                id: "E".to_string(),
                score: 0.6,
            },
        ];

        for candidate in candidates {
            println!(
                "Processing {} (score: {:.1})",
                candidate.id, candidate.score
            );

            let before_len = heap.len();
            push_candidate(&mut heap, candidate, k);
            let after_len = heap.len();

            if after_len > before_len {
                println!("  Added to heap");
            } else {
                println!("  Evicted worst, added this");
            }

            if let Some(Reverse(worst)) = heap.peek() {
                println!("  Current worst in heap: {} ({:.1})", worst.id, worst.score);
            }
            println!();
        }

        let results: Vec<_> = heap.into_iter().map(|Reverse(c)| c).collect();
        println!("Final top-3:");
        for candidate in &results {
            println!("  {} (score: {:.1})", candidate.id, candidate.score);
        }
    }

    // Demo 5: Performance characteristics
    println!("\n═══ Demo 5: Scalability Test ═══\n");
    {
        use std::time::Instant;

        let sizes = [100, 1_000, 10_000];
        let dimensions = 128;

        for &size in &sizes {
            let mut db = VectorStore::new();

            // Insert vectors
            for i in 0..size {
                let vector: Vec<f32> = (0..dimensions)
                    .map(|j| ((i * j) as f32 * 0.001).sin())
                    .collect();
                db.insert(format!("vec_{}", i), vector);
            }

            // Flush to segments
            db.flush();

            // Search
            let query: Vec<f32> = (0..dimensions).map(|_| 0.5).collect();

            let start = Instant::now();
            let results = db.search(&query, 10);
            let elapsed = start.elapsed();

            println!(
                "Vectors: {:>6} | Search time: {:>8.2?} | Top result: {:.4}",
                size, elapsed, results[0].score
            );
        }
    }

    println!("\nAll demos complete!");
    println!("\nKey observations:");
    println!("  Heap maintains top-k efficiently (O(log k) per insert)");
    println!("  Tombstones filter deleted vectors during search");
    println!("  Search time scales linearly with dataset size");
    println!("  For large datasets, we need approximate search (HNSW)");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_ordering() {
        let a = Candidate {
            id: "a".to_string(),
            score: 0.9,
        };
        let b = Candidate {
            id: "b".to_string(),
            score: 0.5,
        };

        assert!(a > b);
    }

    #[test]
    fn test_heap_maintains_top_k() {
        let mut heap = BinaryHeap::new();
        let k = 3;

        push_candidate(
            &mut heap,
            Candidate {
                id: "a".to_string(),
                score: 0.5,
            },
            k,
        );
        push_candidate(
            &mut heap,
            Candidate {
                id: "b".to_string(),
                score: 0.9,
            },
            k,
        );
        push_candidate(
            &mut heap,
            Candidate {
                id: "c".to_string(),
                score: 0.3,
            },
            k,
        );
        push_candidate(
            &mut heap,
            Candidate {
                id: "d".to_string(),
                score: 0.7,
            },
            k,
        );

        assert_eq!(heap.len(), 3);

        let results: Vec<_> = heap.into_iter().map(|Reverse(c)| c).collect();
        let scores: Vec<f32> = results.iter().map(|c| c.score).collect();

        // Should contain 0.9, 0.7, 0.5 (not 0.3)
        assert!(scores.contains(&0.9));
        assert!(scores.contains(&0.7));
        assert!(scores.contains(&0.5));
        assert!(!scores.contains(&0.3));
    }

    #[test]
    fn test_basic_search() {
        let mut db = VectorStore::new();

        db.insert("v1".to_string(), vec![1.0, 0.0]);
        db.insert("v2".to_string(), vec![0.0, 1.0]);
        db.insert("v3".to_string(), vec![0.9, 0.1]);

        let results = db.search(&vec![1.0, 0.0], 2);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "v1"); // Exact match should be first
    }

    #[test]
    fn test_tombstone_filtering() {
        let mut db = VectorStore::new();

        db.insert("v1".to_string(), vec![1.0, 0.0]);
        db.insert("v2".to_string(), vec![0.9, 0.1]);
        db.flush();

        db.delete("v1");

        let results = db.search(&vec![1.0, 0.0], 10);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "v2");
    }

    #[test]
    fn test_search_across_memtable_and_segments() {
        let mut db = VectorStore::new();

        // Segment 1
        db.insert("seg1".to_string(), vec![1.0, 0.0]);
        db.flush();

        // Segment 2
        db.insert("seg2".to_string(), vec![0.9, 0.1]);
        db.flush();

        // MemTable
        db.insert("mem".to_string(), vec![0.95, 0.05]);

        let results = db.search(&vec![1.0, 0.0], 3);

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].id, "seg1"); // Best match
    }

    #[test]
    fn test_empty_search() {
        let db = VectorStore::new();
        let results = db.search(&vec![1.0, 0.0], 10);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_k_larger_than_dataset() {
        let mut db = VectorStore::new();

        db.insert("v1".to_string(), vec![1.0, 0.0]);
        db.insert("v2".to_string(), vec![0.0, 1.0]);

        let results = db.search(&vec![1.0, 0.0], 100);

        // Should return all available results (2), not 100
        assert_eq!(results.len(), 2);
    }
}
