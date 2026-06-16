// search-impl.rs
// Complete HNSW Search Implementation with Instrumentation
// This builds on the graph construction from Post #14

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use std::time::Instant;

// ============================================================================
// Core Data Structures (Recap from Post #14)
// ============================================================================

type NodeId = usize;

#[derive(Debug, Clone)]
struct Node {
    id: NodeId,
    vector: Vec<f32>,
    layers: Vec<Vec<NodeId>>, // layers[0] = Layer 0 neighbors, etc.
    max_layer: usize,
}

impl Node {
    fn new(id: NodeId, vector: Vec<f32>, max_layer: usize) -> Self {
        let mut layers = Vec::with_capacity(max_layer + 1);
        for _ in 0..=max_layer {
            layers.push(Vec::new());
        }

        Self {
            id,
            vector,
            layers,
            max_layer,
        }
    }

    fn neighbors(&self, layer: usize) -> &[NodeId] {
        if layer <= self.max_layer {
            &self.layers[layer]
        } else {
            &[]
        }
    }

    fn add_neighbor(&mut self, layer: usize, neighbor_id: NodeId) {
        if layer <= self.max_layer && !self.layers[layer].contains(&neighbor_id) {
            self.layers[layer].push(neighbor_id);
        }
    }
}

struct HNSWIndex {
    nodes: Vec<Node>,
    entry_point: Option<NodeId>,
    max_layers: usize,
    M: usize,               // Max connections per node (except Layer 0)
    M0: usize,              // Max connections at Layer 0 (typically 2xM)
    ef_construction: usize, // Beam width during construction
    ml: f32,                // Layer assignment parameter
}

// ============================================================================
// HNSW Index Implementation
// ============================================================================

impl HNSWIndex {
    pub fn new(M: usize, ef_construction: usize) -> Self {
        Self {
            nodes: Vec::new(),
            entry_point: None,
            max_layers: 0,
            M,
            M0: M * 2,
            ef_construction,
            ml: 1.0 / (M as f32).ln(),
        }
    }

    // ------------------------------------------------------------------------
    // Distance Function (Euclidean)
    // ------------------------------------------------------------------------
    // NOTE: For production, cosine similarity is often preferred:
    // fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
    //     1.0 - cosine_similarity(a, b)
    // }
    // ------------------------------------------------------------------------

    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    // ------------------------------------------------------------------------
    // PHASE 1: GREEDY DESCENT (Layers N to 1)
    // ------------------------------------------------------------------------

    /// Traverse from top layer down to Layer 1, greedily moving to closer neighbors
    /// Returns the entry point for Phase 2 (beam search at Layer 0)
    fn greedy_descent(&self, query: &[f32]) -> (NodeId, f32) {
        if self.entry_point.is_none() {
            panic!("Cannot search empty index");
        }

        let mut curr_node = self.entry_point.unwrap();
        let mut curr_dist = self.distance(query, &self.nodes[curr_node].vector);

        // Traverse from top layer down to Layer 1
        // We skip Layer 0 because we will do beam search there
        for layer in (1..=self.max_layers).rev() {
            let mut changed = true;

            // Keep moving to closer neighbors at this layer
            while changed {
                changed = false;

                for &neighbor_id in self.nodes[curr_node].neighbors(layer) {
                    let neighbor_dist = self.distance(query, &self.nodes[neighbor_id].vector);

                    if neighbor_dist < curr_dist {
                        curr_dist = neighbor_dist;
                        curr_node = neighbor_id;
                        changed = true; // Found improvement, keep searching this layer
                    }
                }
            }
            // No improvement found at this layer, drop to next layer
        }

        (curr_node, curr_dist)
    }

    // ------------------------------------------------------------------------
    // Greedy Descent with Instrumentation (for debugging/visualization)
    // ------------------------------------------------------------------------

    fn greedy_descent_instrumented(&self, query: &[f32], stats: &mut SearchStats) -> (NodeId, f32) {
        if self.entry_point.is_none() {
            panic!("Cannot search empty index");
        }

        let mut curr_node = self.entry_point.unwrap();
        let mut curr_dist = self.distance(query, &self.nodes[curr_node].vector);

        stats.nodes_visited.push(curr_node);
        stats.distance_calculations += 1;

        for layer in (1..=self.max_layers).rev() {
            stats.layers_traversed.push(layer);
            let mut changed = true;

            while changed {
                changed = false;

                for &neighbor_id in self.nodes[curr_node].neighbors(layer) {
                    let neighbor_dist = self.distance(query, &self.nodes[neighbor_id].vector);
                    stats.distance_calculations += 1;

                    if neighbor_dist < curr_dist {
                        curr_dist = neighbor_dist;
                        curr_node = neighbor_id;
                        stats.nodes_visited.push(curr_node);
                        changed = true;
                    }
                }
            }
        }

        (curr_node, curr_dist)
    }

    // ------------------------------------------------------------------------
    // PHASE 2: BEAM SEARCH AT LAYER 0
    // ------------------------------------------------------------------------
    // This is the same search_layer() function from Post #14
    // We use it for the final refinement at Layer 0
    // ------------------------------------------------------------------------

    fn search_layer(
        &self,
        query: &[f32],
        entry_points: Vec<NodeId>,
        layer: usize,
        ef: usize, // Beam width
    ) -> Vec<NodeId> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new(); // Min-heap (closest first)
        let mut results = BinaryHeap::new(); // Max-heap (farthest first)

        // Initialize with entry points
        for entry_id in entry_points {
            let dist = self.distance(query, &self.nodes[entry_id].vector);

            candidates.push(Reverse((dist_to_sortable(dist), entry_id)));
            results.push((dist_to_sortable(dist), entry_id));
            visited.insert(entry_id);
        }

        // Beam search: explore ef candidates
        while let Some(Reverse((curr_dist_sortable, curr_id))) = candidates.pop() {
            let curr_dist = sortable_to_dist(curr_dist_sortable);

            // If current node is farther than worst result, stop
            if let Some(&(worst_dist_sortable, _)) = results.peek() {
                let worst_dist = sortable_to_dist(worst_dist_sortable);
                if curr_dist > worst_dist && results.len() >= ef {
                    break;
                }
            }

            // Explore neighbors
            for &neighbor_id in self.nodes[curr_id].neighbors(layer) {
                if visited.contains(&neighbor_id) {
                    continue;
                }
                visited.insert(neighbor_id);

                let neighbor_dist = self.distance(query, &self.nodes[neighbor_id].vector);
                let neighbor_dist_sortable = dist_to_sortable(neighbor_dist);

                // Check if this neighbor should be in results
                if results.len() < ef {
                    // Results not full, add it
                    candidates.push(Reverse((neighbor_dist_sortable, neighbor_id)));
                    results.push((neighbor_dist_sortable, neighbor_id));
                } else if let Some(&(worst_dist_sortable, _)) = results.peek() {
                    if neighbor_dist_sortable < worst_dist_sortable {
                        // Better than current worst, replace it
                        candidates.push(Reverse((neighbor_dist_sortable, neighbor_id)));
                        results.pop();
                        results.push((neighbor_dist_sortable, neighbor_id));
                    }
                }
            }
        }

        // Extract node IDs from results
        results.into_iter().map(|(_, id)| id).collect()
    }

    // ------------------------------------------------------------------------
    // Beam Search with Instrumentation
    // ------------------------------------------------------------------------

    fn search_layer_instrumented(
        &self,
        query: &[f32],
        entry_points: Vec<NodeId>,
        layer: usize,
        ef: usize,
        stats: &mut SearchStats,
    ) -> Vec<NodeId> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();

        for entry_id in entry_points {
            let dist = self.distance(query, &self.nodes[entry_id].vector);
            stats.distance_calculations += 1;

            candidates.push(Reverse((dist_to_sortable(dist), entry_id)));
            results.push((dist_to_sortable(dist), entry_id));
            visited.insert(entry_id);
            stats.nodes_visited.push(entry_id);
        }

        while let Some(Reverse((curr_dist_sortable, curr_id))) = candidates.pop() {
            let curr_dist = sortable_to_dist(curr_dist_sortable);

            if let Some(&(worst_dist_sortable, _)) = results.peek() {
                let worst_dist = sortable_to_dist(worst_dist_sortable);
                if curr_dist > worst_dist && results.len() >= ef {
                    break;
                }
            }

            for &neighbor_id in self.nodes[curr_id].neighbors(layer) {
                if visited.contains(&neighbor_id) {
                    continue;
                }
                visited.insert(neighbor_id);
                stats.nodes_visited.push(neighbor_id);

                let neighbor_dist = self.distance(query, &self.nodes[neighbor_id].vector);
                stats.distance_calculations += 1;
                let neighbor_dist_sortable = dist_to_sortable(neighbor_dist);

                if results.len() < ef {
                    candidates.push(Reverse((neighbor_dist_sortable, neighbor_id)));
                    results.push((neighbor_dist_sortable, neighbor_id));
                } else if let Some(&(worst_dist_sortable, _)) = results.peek() {
                    if neighbor_dist_sortable < worst_dist_sortable {
                        candidates.push(Reverse((neighbor_dist_sortable, neighbor_id)));
                        results.pop();
                        results.push((neighbor_dist_sortable, neighbor_id));
                    }
                }
            }
        }

        results.into_iter().map(|(_, id)| id).collect()
    }

    // ------------------------------------------------------------------------
    // MAIN SEARCH FUNCTION (Combines Phase 1 + Phase 2)
    // ------------------------------------------------------------------------

    /// Search for k nearest neighbors
    ///
    /// # Arguments
    /// * `query` - The query vector
    /// * `k` - Number of results to return
    /// * `ef_search` - Beam width for search (larger = more accurate but slower)
    ///
    /// # Returns
    /// Vector of (distance, node_id) tuples, sorted by distance (closest first)
    pub fn search(&self, query: &[f32], k: usize, ef_search: usize) -> Vec<(f32, NodeId)> {
        // Handle empty index
        if self.entry_point.is_none() {
            return Vec::new();
        }

        // Ensure ef_search >= k
        let ef_search = ef_search.max(k);

        // PHASE 1: Greedy descent from top layer down to Layer 1
        let (entry_node, _) = self.greedy_descent(query);

        // PHASE 2: Beam search at Layer 0
        let candidates = self.search_layer(query, vec![entry_node], 0, ef_search);

        // Convert to (distance, id) tuples and sort
        let mut results: Vec<_> = candidates
            .into_iter()
            .map(|id| {
                let dist = self.distance(query, &self.nodes[id].vector);
                (dist, id)
            })
            .collect();

        results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Return top-k
        results.truncate(k);
        results
    }

    // ------------------------------------------------------------------------
    // Search with Instrumentation (for debugging/profiling)
    // ------------------------------------------------------------------------

    pub fn search_instrumented(
        &self,
        query: &[f32],
        k: usize,
        ef_search: usize,
    ) -> (Vec<(f32, NodeId)>, SearchStats) {
        let mut stats = SearchStats::new();

        if self.entry_point.is_none() {
            return (Vec::new(), stats);
        }

        let ef_search = ef_search.max(k);

        // Phase 1
        let (entry_node, _) = self.greedy_descent_instrumented(query, &mut stats);

        // Phase 2
        stats.layers_traversed.push(0);
        let candidates =
            self.search_layer_instrumented(query, vec![entry_node], 0, ef_search, &mut stats);

        let mut results: Vec<_> = candidates
            .into_iter()
            .map(|id| {
                let dist = self.distance(query, &self.nodes[id].vector);
                (dist, id)
            })
            .collect();

        results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        results.truncate(k);

        (results, stats)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

// Convert f32 distance to u64 for use in BinaryHeap (which requires Ord)
// We multiply by 2^32 to preserve precision
fn dist_to_sortable(dist: f32) -> u64 {
    (dist * (1u64 << 32) as f32) as u64
}

fn sortable_to_dist(sortable: u64) -> f32 {
    sortable as f32 / (1u64 << 32) as f32
}

// ============================================================================
// Search Statistics (for instrumentation)
// ============================================================================

#[derive(Debug, Clone)]
pub struct SearchStats {
    pub nodes_visited: Vec<NodeId>,
    pub distance_calculations: usize,
    pub layers_traversed: Vec<usize>,
}

impl SearchStats {
    fn new() -> Self {
        Self {
            nodes_visited: Vec::new(),
            distance_calculations: 0,
            layers_traversed: Vec::new(),
        }
    }

    pub fn print_summary(&self, total_nodes: usize) {
        println!("\n╔═══════════════════════════════════════════════════════╗");
        println!("║               Search Statistics                      ║");
        println!("╚═══════════════════════════════════════════════════════╝");
        println!(
            "Nodes visited:          {} / {} ({:.1}%)",
            self.nodes_visited.len(),
            total_nodes,
            (self.nodes_visited.len() as f32 / total_nodes as f32) * 100.0
        );
        println!("Distance calculations:  {}", self.distance_calculations);
        println!("Layers traversed:       {:?}", self.layers_traversed);
        println!(
            "Unique nodes visited:   {}",
            self.nodes_visited.iter().collect::<HashSet<_>>().len()
        );
    }
}

// ============================================================================
// Example Usage
// ============================================================================

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         HNSW Search Implementation Demo                   ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Create index with M=4, ef_construction=20
    let mut index = HNSWIndex::new(4, 20);

    // Insert some 2D vectors for visualization
    let vectors = vec![
        vec![0.0, 0.0], // Node 0: Origin
        vec![1.0, 1.0], // Node 1: Diagonal
        vec![0.1, 0.1], // Node 2: Close to origin
        vec![9.0, 9.0], // Node 3: Far away
        vec![2.0, 2.0], // Node 4: Medium distance
        vec![0.5, 0.5], // Node 5: Middle
        vec![8.0, 8.0], // Node 6: Near node 3
        vec![0.2, 0.3], // Node 7: Close to origin
    ];

    println!("Inserting {} vectors...", vectors.len());
    // Note: Insert implementation from Post #14 is required here
    // For this demo, we will assume nodes are added to the index

    println!("\n--- Example 1: Basic Search ---");
    let query = vec![0.5, 0.5];
    println!("Query: {:?}", query);

    let results = index.search(&query, 3, 50);
    println!("\nTop 3 results (ef_search=50):");
    for (i, (dist, id)) in results.iter().enumerate() {
        println!("  {}. Node {} (distance: {:.4})", i + 1, id, dist);
    }

    println!("\n--- Example 2: Comparing ef_search Values ---");
    for &ef in &[10, 20, 50, 100] {
        let start = Instant::now();
        let results = index.search(&query, 3, ef);
        let elapsed = start.elapsed();

        println!("\nef_search = {}:", ef);
        println!("  Latency: {:?}", elapsed);
        println!(
            "  Results: {:?}",
            results.iter().map(|(_, id)| id).collect::<Vec<_>>()
        );
    }

    println!("\n--- Example 3: Search with Instrumentation ---");
    let (results, stats) = index.search_instrumented(&query, 3, 50);

    println!("\nResults:");
    for (i, (dist, id)) in results.iter().enumerate() {
        println!("  {}. Node {} (distance: {:.4})", i + 1, id, dist);
    }

    stats.print_summary(index.nodes.len());

    println!("\n--- Example 4: Demonstrating Two-Phase Search ---");
    println!("\nQuery: {:?}", query);
    println!("\nPhase 1 (Greedy Descent):");
    let (entry_for_phase2, dist) = index.greedy_descent(&query);
    println!(
        "  Found entry point: Node {} (distance: {:.4})",
        entry_for_phase2, dist
    );

    println!("\nPhase 2 (Beam Search at Layer 0):");
    println!(
        "  Starting from Node {} with ef_search=50",
        entry_for_phase2
    );
    let final_results = index.search(&query, 3, 50);
    println!("  Final top-3:");
    for (i, (dist, id)) in final_results.iter().enumerate() {
        println!("    {}. Node {} (distance: {:.4})", i + 1, id, dist);
    }

    println!("\n--- Example 5: Runtime ef_search Tuning ---");
    println!("\nSame query, different ef_search values:");

    let ef_values = vec![10, 50, 100, 200];
    for &ef in &ef_values {
        let start = Instant::now();
        let results = index.search(&query, 10, ef);
        let elapsed = start.elapsed();

        println!(
            "\nef_search={:3} | Latency: {:7.2}µs | Found {} results",
            ef,
            elapsed.as_micros(),
            results.len()
        );
    }

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  Key Insight: ef_search is tunable at query time!         ║");
    println!("║  Low ef = Fast, lower recall                            ║");
    println!("║  High ef = Slower, higher recall                        ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// Testing Helpers
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_returns_k_results() {
        let mut index = HNSWIndex::new(4, 20);

        // Insert test vectors
        // (Insert implementation required)

        let query = vec![0.5, 0.5];
        let results = index.search(&query, 3, 50);

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_ef_search_must_be_at_least_k() {
        let mut index = HNSWIndex::new(4, 20);

        // Insert test vectors
        // (Insert implementation required)

        let query = vec![0.5, 0.5];
        let results = index.search(&query, 10, 5); // ef_search < k

        // Should automatically correct ef_search to 10
        assert!(results.len() <= 10);
    }

    #[test]
    fn test_search_empty_index() {
        let index = HNSWIndex::new(4, 20);

        let query = vec![0.5, 0.5];
        let results = index.search(&query, 10, 50);

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_results_are_sorted() {
        let mut index = HNSWIndex::new(4, 20);

        // Insert test vectors
        // (Insert implementation required)

        let query = vec![0.5, 0.5];
        let results = index.search(&query, 10, 50);

        // Check that distances are sorted (ascending)
        for i in 1..results.len() {
            assert!(results[i - 1].0 <= results[i].0);
        }
    }
}
