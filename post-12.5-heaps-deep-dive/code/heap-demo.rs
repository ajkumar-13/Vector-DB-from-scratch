// post-12.5-heaps-deep-dive/code/heap-demo.rs
// Interactive demonstrations of binary heap operations
//
// Run with: rustc heap-demo.rs && ./heap-demo
// Or: rustc --test heap-demo.rs && ./heap-demo

use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

// ============================================================================
// Candidate Struct
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct Candidate {
    id: String,
    score: f32,
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Eq for Candidate {}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Less)
    }
}

// ============================================================================
// Top-K Implementations
// ============================================================================

/// Naive approach: sort everything
fn top_k_by_sorting(items: Vec<Candidate>, k: usize) -> Vec<Candidate> {
    let mut sorted = items;
    sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap()); // Descending
    sorted.truncate(k);
    sorted
}

/// Heap approach: maintain min-heap of size k
fn top_k_by_heap(items: Vec<Candidate>, k: usize) -> Vec<Candidate> {
    let mut heap: BinaryHeap<Reverse<Candidate>> = BinaryHeap::with_capacity(k);

    for item in items {
        if heap.len() < k {
            heap.push(Reverse(item));
        } else if let Some(Reverse(min)) = heap.peek() {
            if item.score > min.score {
                heap.pop();
                heap.push(Reverse(item));
            }
        }
    }

    let mut results: Vec<_> = heap.into_iter().map(|Reverse(c)| c).collect();
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    results
}

// ============================================================================
// Visualization Helpers
// ============================================================================

fn visualize_heap_state(heap: &BinaryHeap<Reverse<Candidate>>, step: usize, action: &str) {
    println!("  Step {}: {}", step, action);

    if heap.is_empty() {
        println!("    Heap: [empty]");
        return;
    }

    // Extract for visualization (destructive, so we clone)
    let items: Vec<_> = heap
        .iter()
        .map(|Reverse(c)| (c.id.clone(), c.score))
        .collect();

    // Show as array
    print!("    Heap: [");
    for (i, (id, score)) in items.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        print!("{}:{:.1}", id, score);
    }
    println!("]");

    // Show root (min)
    if let Some(Reverse(min)) = heap.peek() {
        println!("    Root (bouncer): {} ({:.1})", min.id, min.score);
    }
    println!();
}

// ============================================================================
// Demo 1: Basic Heap Operations
// ============================================================================

fn demo_basic_operations() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         Demo 1: Basic Heap Operations                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut heap: BinaryHeap<Reverse<i32>> = BinaryHeap::new();

    println!("Starting with empty min-heap\n");

    let values = [5, 3, 7, 1, 9];

    for val in values {
        println!("Push({})", val);
        heap.push(Reverse(val));

        if let Some(Reverse(min)) = heap.peek() {
            println!("  Root (min): {}", min);
        }

        let contents: Vec<i32> = heap.iter().map(|Reverse(x)| *x).collect();
        println!("  Contents: {:?}\n", contents);
    }

    println!("Popping all elements (should be in ascending order):");
    while let Some(Reverse(val)) = heap.pop() {
        println!("  Pop: {}", val);
    }
}

// ============================================================================
// Demo 2: The Bouncer Algorithm
// ============================================================================

fn demo_bouncer_algorithm() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         Demo 2: The Bouncer Algorithm (k=3)                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let k = 3;
    let scores = vec![
        ("A", 5.0),
        ("B", 9.0),
        ("C", 3.0),
        ("D", 7.0),
        ("E", 4.0),
        ("F", 8.0),
        ("G", 1.0),
    ];

    let mut heap: BinaryHeap<Reverse<Candidate>> = BinaryHeap::new();
    let mut step = 0;

    println!(
        "Finding top-{} from: {:?}\n",
        k,
        scores
            .iter()
            .map(|(id, s)| format!("{}:{}", id, s))
            .collect::<Vec<_>>()
    );

    for (id, score) in scores {
        step += 1;
        let candidate = Candidate {
            id: id.to_string(),
            score,
        };

        println!(
            "Processing {} (score: {:.1})",
            candidate.id, candidate.score
        );

        if heap.len() < k {
            println!("  Heap not full, adding it");
            heap.push(Reverse(candidate));
        } else if let Some(Reverse(min)) = heap.peek() {
            if candidate.score > min.score {
                println!(
                    "  {:.1} > {:.1} (bouncer): Evict {}, add {}",
                    candidate.score, min.score, min.id, candidate.id
                );
                heap.pop();
                heap.push(Reverse(candidate));
            } else {
                println!(
                    "  {:.1} <= {:.1} (bouncer): Reject",
                    candidate.score, min.score
                );
            }
        }

        visualize_heap_state(&heap, step, &format!("After processing {}", id));
    }

    let results: Vec<_> = heap.into_iter().map(|Reverse(c)| c).collect();
    println!(
        "Final top-3: {:?}",
        results
            .iter()
            .map(|c| format!("{}:{:.1}", c.id, c.score))
            .collect::<Vec<_>>()
    );
}

// ============================================================================
// Demo 3: Complexity Comparison
// ============================================================================

fn demo_complexity_comparison() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         Demo 3: Complexity Comparison                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    use std::time::Instant;

    let sizes = [1_000, 10_000, 100_000];
    let k = 10;

    println!("Finding top-{} using Sort vs Heap\n", k);
    println!(
        "{:>10} {:>15} {:>15} {:>10}",
        "Size", "Sort", "Heap", "Speedup"
    );
    println!("{}", "-".repeat(52));

    for &size in &sizes {
        // Generate data
        let data: Vec<Candidate> = (0..size)
            .map(|i| Candidate {
                id: format!("v{}", i),
                score: ((i * 31) % 1000) as f32 / 1000.0,
            })
            .collect();

        // Benchmark sorting
        let data_clone = data.clone();
        let start = Instant::now();
        let _ = top_k_by_sorting(data_clone, k);
        let sort_time = start.elapsed();

        // Benchmark heap
        let data_clone = data.clone();
        let start = Instant::now();
        let _ = top_k_by_heap(data_clone, k);
        let heap_time = start.elapsed();

        let speedup = sort_time.as_secs_f64() / heap_time.as_secs_f64();

        println!(
            "{:>10} {:>15.2?} {:>15.2?} {:>10.2}x",
            size, sort_time, heap_time, speedup
        );
    }
}

// ============================================================================
// Demo 4: Array Representation
// ============================================================================

fn demo_array_representation() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         Demo 4: Array Representation of Heap                ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let values = vec![3, 5, 7, 9, 8, 11, 10];

    println!("Heap as tree:");
    println!("       3");
    println!("      / \\");
    println!("     5   7");
    println!("    / \\ / \\");
    println!("   9  8 11 10");

    println!("\nArray representation: {:?}", values);
    println!("Indices:              [0, 1, 2, 3, 4,  5,  6]\n");

    println!("For each node, we can find children/parent using array index:\n");

    for i in 0..values.len() {
        println!("Index {}: Value = {}", i, values[i]);

        if i > 0 {
            let parent_idx = (i - 1) / 2;
            println!("  Parent: index {} = {}", parent_idx, values[parent_idx]);
        } else {
            println!("  Parent: (none, this is root)");
        }

        let left_idx = 2 * i + 1;
        if left_idx < values.len() {
            println!("  Left child: index {} = {}", left_idx, values[left_idx]);
        }

        let right_idx = 2 * i + 2;
        if right_idx < values.len() {
            println!("  Right child: index {} = {}", right_idx, values[right_idx]);
        }

        println!();
    }

    println!("Key insight: No pointers needed! Just integer arithmetic.");
    println!("This makes heaps cache-friendly and fast.");
}

// ============================================================================
// Demo 5: Why Min-Heap for Max-K?
// ============================================================================

fn demo_why_min_heap() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         Demo 5: Why Min-Heap for Max-K?                     ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("Goal: Find top-3 highest scores");
    println!("Strategy: Use min-heap to track worst of the best\n");

    let k = 3;
    let mut heap: BinaryHeap<Reverse<i32>> = BinaryHeap::new();

    // Fill with first 3
    heap.push(Reverse(5));
    heap.push(Reverse(9));
    heap.push(Reverse(3));

    println!("Initial top-3: [5, 9, 3]");
    println!(
        "Min-heap root (worst of top-3): {}\n",
        heap.peek().unwrap().0
    );

    println!("New candidate: 7");
    println!("  Question: Does 7 beat the worst top-3 member?");
    println!("  Answer: 7 > 3 (root). Yes.");
    println!("  Action: Evict 3, add 7");
    heap.pop();
    heap.push(Reverse(7));
    println!("  New root: {}\n", heap.peek().unwrap().0);

    println!("New candidate: 4");
    println!("  Question: Does 4 beat the worst top-3 member?");
    println!("  Answer: 4 < 5 (root). No.");
    println!("  Action: Reject\n");

    println!("New candidate: 8");
    println!("  Question: Does 8 beat the worst top-3 member?");
    println!("  Answer: 8 > 5 (root). Yes.");
    println!("  Action: Evict 5, add 8");
    heap.pop();
    heap.push(Reverse(8));

    let results: Vec<_> = heap.into_iter().map(|Reverse(x)| x).collect();
    println!("\nFinal top-3: {:?}", results);
    println!("\nKey insight: Min-heap root = weakest top-k member = eviction candidate");
}

// ============================================================================
// Demo 6: Streaming Top-K
// ============================================================================

fn demo_streaming_topk() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         Demo 6: Streaming Top-K                             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let k = 5;
    let mut heap: BinaryHeap<Reverse<Candidate>> = BinaryHeap::new();

    println!("Simulating real-time top-{} tracker\n", k);

    // Simulate streaming data
    let stream = vec![
        ("tweet1", 10.0),
        ("tweet2", 5.0),
        ("tweet3", 15.0),
        ("tweet4", 3.0),
        ("tweet5", 20.0),
        ("tweet6", 8.0),
        ("tweet7", 25.0), // New leader
        ("tweet8", 7.0),
        ("tweet9", 12.0),
        ("tweet10", 30.0), // Even better
    ];

    for (id, score) in stream {
        let candidate = Candidate {
            id: id.to_string(),
            score,
        };

        print!("New item: {} ({:.0} likes): ", id, score);

        if heap.len() < k {
            heap.push(Reverse(candidate));
            println!("Added (heap not full)");
        } else if let Some(Reverse(min)) = heap.peek() {
            if candidate.score > min.score {
                let evicted = heap.pop().unwrap().0;
                heap.push(Reverse(candidate));
                println!("Evicted {} ({:.0}), added this", evicted.id, evicted.score);
            } else {
                println!("Rejected (below top-{})", k);
            }
        }

        // Show current top-5
        let current: Vec<_> = heap
            .iter()
            .map(|Reverse(c)| format!("{}:{:.0}", c.id, c.score))
            .collect();
        println!("  Current top-{}: [{}]", k, current.join(", "));
        println!();
    }
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║      Binary Heap Demonstrations for Top-K Selection         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    demo_basic_operations();
    demo_bouncer_algorithm();
    demo_complexity_comparison();
    demo_array_representation();
    demo_why_min_heap();
    demo_streaming_topk();

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("All demonstrations complete!");
    println!("\nKey insights:");
    println!("  1. Min-heap root = weakest top-k member (eviction candidate)");
    println!("  2. O(N log k) << O(N log N) when k << N");
    println!("  3. Array representation = cache-friendly");
    println!("  4. Heap perfect for streaming/incremental top-k");
    println!("  5. Reverse wrapper turns max-heap into min-heap");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_k_consistency() {
        let items = vec![
            Candidate {
                id: "a".to_string(),
                score: 5.0,
            },
            Candidate {
                id: "b".to_string(),
                score: 9.0,
            },
            Candidate {
                id: "c".to_string(),
                score: 3.0,
            },
            Candidate {
                id: "d".to_string(),
                score: 7.0,
            },
            Candidate {
                id: "e".to_string(),
                score: 1.0,
            },
        ];

        let k = 3;
        let by_sort = top_k_by_sorting(items.clone(), k);
        let by_heap = top_k_by_heap(items.clone(), k);

        assert_eq!(by_sort.len(), k);
        assert_eq!(by_heap.len(), k);

        // Both should return same top-3 (order may differ)
        let sort_scores: Vec<_> = by_sort.iter().map(|c| c.score).collect();
        let heap_scores: Vec<_> = by_heap.iter().map(|c| c.score).collect();

        assert!(sort_scores.contains(&9.0));
        assert!(sort_scores.contains(&7.0));
        assert!(sort_scores.contains(&5.0));

        assert!(heap_scores.contains(&9.0));
        assert!(heap_scores.contains(&7.0));
        assert!(heap_scores.contains(&5.0));
    }

    #[test]
    fn test_min_heap_behavior() {
        let mut heap = BinaryHeap::new();

        heap.push(Reverse(5));
        heap.push(Reverse(3));
        heap.push(Reverse(7));

        assert_eq!(heap.pop(), Some(Reverse(3))); // Min
        assert_eq!(heap.pop(), Some(Reverse(5)));
        assert_eq!(heap.pop(), Some(Reverse(7))); // Max
    }

    #[test]
    fn test_array_indexing() {
        // For node at index i:
        assert_eq!(2 * 0 + 1, 1); // Left child of 0
        assert_eq!(2 * 0 + 2, 2); // Right child of 0
        assert_eq!((1 - 1) / 2, 0); // Parent of 1
        assert_eq!((2 - 1) / 2, 0); // Parent of 2

        assert_eq!(2 * 1 + 1, 3); // Left child of 1
        assert_eq!(2 * 1 + 2, 4); // Right child of 1
        assert_eq!((3 - 1) / 2, 1); // Parent of 3
        assert_eq!((4 - 1) / 2, 1); // Parent of 4
    }
}
