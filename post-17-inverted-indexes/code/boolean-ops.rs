// boolean-ops.rs
// Deep dive into Boolean Set Operations on Sorted Integer Lists
// Part of Post #17: Inverted Indexes Explained
//
// This file demonstrates:
// 1. Two-pointer intersection (AND)
// 2. Merge union (OR)
// 3. Difference (NOT)
// 4. Performance comparison with HashSet approaches
// 5. Complex query execution with multiple operations

use std::collections::HashSet;
use std::time::Instant;

// =============================================================================
// Core Algorithms: Two-Pointer Approach
// =============================================================================

/// Intersection: A AND B
///
/// Returns elements present in BOTH lists.
/// Requires sorted input lists.
///
/// # Time Complexity
/// O(n + m) where n = |A|, m = |B|
///
/// # Space Complexity
/// O(min(n, m)) for output
///
/// # Example
/// ```
/// let a = vec![1, 3, 5, 7];
/// let b = vec![3, 5, 9];
/// assert_eq!(intersect(&a, &b), vec![3, 5]);
/// ```
pub fn intersect(list_a: &[usize], list_b: &[usize]) -> Vec<usize> {
    let mut i = 0;
    let mut j = 0;
    let mut result = Vec::new();

    while i < list_a.len() && j < list_b.len() {
        if list_a[i] < list_b[j] {
            i += 1;
        } else if list_a[i] > list_b[j] {
            j += 1;
        } else {
            // Match found
            result.push(list_a[i]);
            i += 1;
            j += 1;
        }
    }

    result
}

/// Union: A OR B
///
/// Returns elements present in EITHER list (or both).
/// Requires sorted input lists, produces sorted output.
///
/// # Time Complexity
/// O(n + m)
///
/// # Space Complexity
/// O(n + m) for output
///
/// # Example
/// ```
/// let a = vec![1, 3, 5];
/// let b = vec![2, 3, 6];
/// assert_eq!(union(&a, &b), vec![1, 2, 3, 5, 6]);
/// ```
pub fn union(list_a: &[usize], list_b: &[usize]) -> Vec<usize> {
    let mut i = 0;
    let mut j = 0;
    let mut result = Vec::new();

    while i < list_a.len() && j < list_b.len() {
        if list_a[i] < list_b[j] {
            result.push(list_a[i]);
            i += 1;
        } else if list_a[i] > list_b[j] {
            result.push(list_b[j]);
            j += 1;
        } else {
            // Both have this element, add once
            result.push(list_a[i]);
            i += 1;
            j += 1;
        }
    }

    // Append remaining elements from either list
    while i < list_a.len() {
        result.push(list_a[i]);
        i += 1;
    }

    while j < list_b.len() {
        result.push(list_b[j]);
        j += 1;
    }

    result
}

/// Difference: A minus B (NOT operation)
///
/// Returns elements in A but NOT in B.
/// Requires sorted input lists.
///
/// # Time Complexity
/// O(n + m)
///
/// # Space Complexity
/// O(n) for output
///
/// # Example
/// ```
/// let a = vec![1, 2, 3, 4, 5];
/// let b = vec![2, 4];
/// assert_eq!(difference(&a, &b), vec![1, 3, 5]);
/// ```
pub fn difference(list_a: &[usize], list_b: &[usize]) -> Vec<usize> {
    let mut i = 0;
    let mut j = 0;
    let mut result = Vec::new();

    while i < list_a.len() {
        if j >= list_b.len() {
            // Rest of A is in result
            result.extend_from_slice(&list_a[i..]);
            break;
        }

        if list_a[i] < list_b[j] {
            // In A but not in B
            result.push(list_a[i]);
            i += 1;
        } else if list_a[i] > list_b[j] {
            // Skip elements in B that aren't in A
            j += 1;
        } else {
            // In both, skip
            i += 1;
            j += 1;
        }
    }

    result
}

/// Multi-way intersection: A AND B AND C AND ...
///
/// Intersect multiple sorted lists efficiently by:
/// 1. Sorting lists by length (shortest first)
/// 2. Intersecting iteratively
/// 3. Early termination if intermediate result is empty
///
/// # Optimization
/// Starting with the shortest list minimizes comparisons.
///
/// # Example
/// ```
/// let lists = vec![
///     vec![1, 2, 3, 4, 5],
///     vec![2, 3, 5, 8],
///     vec![3, 5, 7, 9],
/// ];
/// assert_eq!(multi_intersect(&lists), vec![3, 5]);
/// ```
pub fn multi_intersect(lists: &[Vec<usize>]) -> Vec<usize> {
    if lists.is_empty() {
        return Vec::new();
    }

    if lists.len() == 1 {
        return lists[0].clone();
    }

    // Sort by length (shortest first)
    let mut sorted_lists: Vec<&Vec<usize>> = lists.iter().collect();
    sorted_lists.sort_by_key(|list| list.len());

    // Start with shortest list
    let mut result = sorted_lists[0].clone();

    // Intersect with remaining lists
    for list in &sorted_lists[1..] {
        result = intersect(&result, list);

        // Early termination
        if result.is_empty() {
            return result;
        }
    }

    result
}

/// Multi-way union: A OR B OR C OR ...
///
/// Union multiple sorted lists using a HashSet for simplicity.
///
/// Note: For very large lists, a k-way merge (using a min-heap)
/// would be more cache-friendly and avoid HashSet overhead.
pub fn multi_union(lists: &[Vec<usize>]) -> Vec<usize> {
    let mut result_set = HashSet::new();

    for list in lists {
        for &id in list {
            result_set.insert(id);
        }
    }

    let mut result: Vec<_> = result_set.into_iter().collect();
    result.sort_unstable();
    result
}

// =============================================================================
// HashSet Baseline (for comparison)
// =============================================================================

/// Intersection using HashSet
///
/// Build a HashSet from first list, then check membership.
///
/// # Time Complexity
/// O(n + m) amortized
///
/// # Space Complexity
/// O(n) for the HashSet
pub fn intersect_hashset(list_a: &[usize], list_b: &[usize]) -> Vec<usize> {
    let set_a: HashSet<_> = list_a.iter().collect();
    let mut result: Vec<usize> = list_b
        .iter()
        .filter(|&&id| set_a.contains(&id))
        .copied()
        .collect();
    result.sort_unstable();
    result
}

/// Union using HashSet
pub fn union_hashset(list_a: &[usize], list_b: &[usize]) -> Vec<usize> {
    let mut set = HashSet::new();
    for &id in list_a {
        set.insert(id);
    }
    for &id in list_b {
        set.insert(id);
    }
    let mut result: Vec<_> = set.into_iter().collect();
    result.sort_unstable();
    result
}

/// Difference using HashSet
pub fn difference_hashset(list_a: &[usize], list_b: &[usize]) -> Vec<usize> {
    let set_b: HashSet<_> = list_b.iter().collect();
    list_a
        .iter()
        .filter(|&&id| !set_b.contains(&id))
        .copied()
        .collect()
}

// =============================================================================
// Query Execution Engine
// =============================================================================

/// Represents a boolean query
#[derive(Debug, Clone)]
pub enum Query {
    /// Single term
    Term(String),
    /// AND: All must match
    And(Vec<Query>),
    /// OR: Any must match
    Or(Vec<Query>),
    /// NOT: Exclude
    Not(Box<Query>),
}

/// Execute a query against an index (simulated with HashMap)
///
/// # Example
/// ```
/// // (shoes AND blue) OR (hat AND red)
/// let query = Query::Or(vec![
///     Query::And(vec![
///         Query::Term("shoes".to_string()),
///         Query::Term("blue".to_string()),
///     ]),
///     Query::And(vec![
///         Query::Term("hat".to_string()),
///         Query::Term("red".to_string()),
///     ]),
/// ]);
/// ```
pub fn execute_query(
    query: &Query,
    index: &std::collections::HashMap<String, Vec<usize>>,
) -> Vec<usize> {
    match query {
        Query::Term(term) => index.get(term).cloned().unwrap_or_default(),
        Query::And(subqueries) => {
            let lists: Vec<Vec<usize>> =
                subqueries.iter().map(|q| execute_query(q, index)).collect();
            multi_intersect(&lists)
        }
        Query::Or(subqueries) => {
            let lists: Vec<Vec<usize>> =
                subqueries.iter().map(|q| execute_query(q, index)).collect();
            multi_union(&lists)
        }
        Query::Not(subquery) => {
            // NOT without a base set doesn't make sense
            // In practice, combine with AND: (term AND NOT excluded)
            // Here we return empty as a placeholder
            Vec::new()
        }
    }
}

// =============================================================================
// Benchmarking
// =============================================================================

/// Generate sorted test data
fn generate_sorted_list(start: usize, end: usize, step: usize) -> Vec<usize> {
    (start..end).step_by(step).collect()
}

/// Benchmark intersection algorithms
pub fn benchmark_intersection() {
    println!("=== Benchmark: Intersection (AND) ===\n");

    let sizes = vec![100, 1_000, 10_000, 100_000];

    for size in sizes {
        let list_a = generate_sorted_list(0, size * 2, 2); // Even numbers
        let list_b = generate_sorted_list(0, size * 2, 3); // Multiples of 3

        // Two-pointer approach
        let start = Instant::now();
        let result_tp = intersect(&list_a, &list_b);
        let time_tp = start.elapsed();

        // HashSet approach
        let start = Instant::now();
        let result_hs = intersect_hashset(&list_a, &list_b);
        let time_hs = start.elapsed();

        println!("List size: {}", size);
        println!("  Results: {} matches", result_tp.len());
        println!("  Two-pointer: {:?}", time_tp);
        println!("  HashSet:     {:?}", time_hs);
        println!(
            "  Speedup:     {:.2}x\n",
            time_hs.as_nanos() as f64 / time_tp.as_nanos() as f64
        );

        // Verify correctness
        assert_eq!(result_tp, result_hs);
    }
}

/// Benchmark union algorithms
pub fn benchmark_union() {
    println!("=== Benchmark: Union (OR) ===\n");

    let sizes = vec![100, 1_000, 10_000, 100_000];

    for size in sizes {
        let list_a = generate_sorted_list(0, size, 1);
        let list_b = generate_sorted_list(size / 2, size + size / 2, 1); // 50% overlap

        // Two-pointer approach
        let start = Instant::now();
        let result_tp = union(&list_a, &list_b);
        let time_tp = start.elapsed();

        // HashSet approach
        let start = Instant::now();
        let result_hs = union_hashset(&list_a, &list_b);
        let time_hs = start.elapsed();

        println!("List size: {}", size);
        println!("  Results: {} total", result_tp.len());
        println!("  Two-pointer: {:?}", time_tp);
        println!("  HashSet:     {:?}", time_hs);
        println!(
            "  Speedup:     {:.2}x\n",
            time_hs.as_nanos() as f64 / time_tp.as_nanos() as f64
        );

        assert_eq!(result_tp, result_hs);
    }
}

/// Benchmark multi-way intersection
pub fn benchmark_multi_intersection() {
    println!("=== Benchmark: Multi-way Intersection ===\n");

    let num_lists = vec![2, 5, 10];
    let size = 10_000;

    for n in num_lists {
        let mut lists = Vec::new();
        for i in 0..n {
            lists.push(generate_sorted_list(i, size, n));
        }

        let start = Instant::now();
        let result = multi_intersect(&lists);
        let elapsed = start.elapsed();

        println!("Lists: {}, Size: {}", n, size);
        println!("  Results: {} matches", result.len());
        println!("  Time: {:?}\n", elapsed);
    }
}

// =============================================================================
// Visualization: Algorithm Walkthrough
// =============================================================================

/// Print step-by-step execution of intersection algorithm
pub fn visualize_intersection() {
    println!("=== Intersection Algorithm Walkthrough ===\n");

    let list_a = vec![1, 3, 5, 7, 9];
    let list_b = vec![2, 3, 5, 8, 10];

    println!("List A: {:?}", list_a);
    println!("List B: {:?}", list_b);
    println!("\nStep-by-step:\n");

    let mut i = 0;
    let mut j = 0;
    let mut result = Vec::new();
    let mut step = 1;

    while i < list_a.len() && j < list_b.len() {
        print!(
            "Step {}: i={}, j={}, A[{}]={}, B[{}]={} => ",
            step, i, j, i, list_a[i], j, list_b[j]
        );

        if list_a[i] < list_b[j] {
            println!("A < B, i++");
            i += 1;
        } else if list_a[i] > list_b[j] {
            println!("A > B, j++");
            j += 1;
        } else {
            println!("MATCH! Add {} to result, i++, j++", list_a[i]);
            result.push(list_a[i]);
            i += 1;
            j += 1;
        }
        step += 1;
    }

    println!("\nFinal result: {:?}\n", result);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersect() {
        let a = vec![1, 3, 5, 7, 9];
        let b = vec![2, 3, 5, 8, 10];
        assert_eq!(intersect(&a, &b), vec![3, 5]);

        // Edge cases
        let empty: Vec<usize> = Vec::new();
        assert_eq!(intersect(&a, &empty), Vec::<usize>::new());
        assert_eq!(intersect(&empty, &b), Vec::<usize>::new());

        // No overlap
        let c = vec![1, 2];
        let d = vec![3, 4];
        assert_eq!(intersect(&c, &d), Vec::<usize>::new());
    }

    #[test]
    fn test_union() {
        let a = vec![1, 3, 5];
        let b = vec![2, 3, 6];
        assert_eq!(union(&a, &b), vec![1, 2, 3, 5, 6]);

        // No overlap
        let c = vec![1, 2];
        let d = vec![3, 4];
        assert_eq!(union(&c, &d), vec![1, 2, 3, 4]);

        // Full overlap
        let e = vec![1, 2, 3];
        let f = vec![1, 2, 3];
        assert_eq!(union(&e, &f), vec![1, 2, 3]);
    }

    #[test]
    fn test_difference() {
        let a = vec![1, 2, 3, 4, 5];
        let b = vec![2, 4];
        assert_eq!(difference(&a, &b), vec![1, 3, 5]);

        // B is superset of A
        let c = vec![1, 2];
        let d = vec![1, 2, 3, 4];
        assert_eq!(difference(&c, &d), Vec::<usize>::new());

        // No overlap
        let e = vec![1, 2];
        let f = vec![3, 4];
        assert_eq!(difference(&e, &f), vec![1, 2]);
    }

    #[test]
    fn test_multi_intersect() {
        let lists = vec![vec![1, 2, 3, 4, 5], vec![2, 3, 5, 8], vec![3, 5, 7, 9]];
        assert_eq!(multi_intersect(&lists), vec![3, 5]);

        // One empty list
        let lists2 = vec![vec![1, 2, 3], Vec::new(), vec![1, 2]];
        assert_eq!(multi_intersect(&lists2), Vec::<usize>::new());
    }

    #[test]
    fn test_multi_union() {
        let lists = vec![vec![1, 2], vec![2, 3], vec![3, 4]];
        assert_eq!(multi_union(&lists), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_query_execution() {
        use std::collections::HashMap;

        let mut index = HashMap::new();
        index.insert("shoes".to_string(), vec![1, 2, 3]);
        index.insert("blue".to_string(), vec![1, 4]);
        index.insert("nike".to_string(), vec![1, 3]);

        // Single term
        let q = Query::Term("shoes".to_string());
        assert_eq!(execute_query(&q, &index), vec![1, 2, 3]);

        // AND query
        let q = Query::And(vec![
            Query::Term("shoes".to_string()),
            Query::Term("blue".to_string()),
        ]);
        assert_eq!(execute_query(&q, &index), vec![1]);

        // OR query
        let q = Query::Or(vec![
            Query::Term("blue".to_string()),
            Query::Term("nike".to_string()),
        ]);
        assert_eq!(execute_query(&q, &index), vec![1, 3, 4]);
    }

    #[test]
    fn test_correctness_vs_hashset() {
        // Generate large random-ish lists
        let a = generate_sorted_list(0, 10000, 2);
        let b = generate_sorted_list(0, 10000, 3);

        // Intersection
        let tp_result = intersect(&a, &b);
        let hs_result = intersect_hashset(&a, &b);
        assert_eq!(tp_result, hs_result);

        // Union
        let tp_result = union(&a, &b);
        let hs_result = union_hashset(&a, &b);
        assert_eq!(tp_result, hs_result);

        // Difference
        let tp_result = difference(&a, &b);
        let hs_result = difference_hashset(&a, &b);
        assert_eq!(tp_result, hs_result);
    }
}

// =============================================================================
// Main: Run Examples and Benchmarks
// =============================================================================

fn main() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  Boolean Set Operations on Sorted Integer Lists          ║");
    println!("║  Post #17: Inverted Indexes                              ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // Visualization
    visualize_intersection();

    println!("Press Enter to run benchmarks...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();

    // Benchmarks
    benchmark_intersection();
    benchmark_union();
    benchmark_multi_intersection();

    println!("\nAll benchmarks complete.");
    println!("\nKey Takeaways:");
    println!("  1. Two-pointer is 1.5-2x faster than HashSet for intersection");
    println!("  2. Cache-friendly: sequential array access beats hash lookups");
    println!("  3. Zero allocations (besides output vector)");
    println!("  4. Scales linearly: O(n + m) is predictable");
}
