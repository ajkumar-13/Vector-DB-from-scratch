// HNSW Basic Implementation: Core Graph Structure and Insertion
// This demonstrates the Arena Pattern and layer-based graph construction

use rand::Rng;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

// ============================================================================
// Core Data Structures
// ============================================================================

type NodeId = usize;

/// A node in the HNSW graph
#[derive(Clone)]
struct Node {
    vector: Vec<f32>,
    // connections[layer] = list of neighbor IDs at that layer
    // Layer 0 is at index 0, Layer N at index N
    connections: Vec<Vec<NodeId>>,
    layer_count: usize,
}

impl Node {
    fn new(vector: Vec<f32>, max_layer: usize) -> Self {
        // Create empty connection lists for each layer (0..=max_layer)
        let connections = vec![Vec::new(); max_layer + 1];

        Self {
            vector,
            connections,
            layer_count: max_layer + 1,
        }
    }

    /// Get neighbors at a specific layer
    fn neighbors(&self, layer: usize) -> &[NodeId] {
        if layer < self.layer_count {
            &self.connections[layer]
        } else {
            &[] // Node does not exist at this layer
        }
    }

    /// Add a connection at a specific layer (does not check for duplicates)
    fn connect(&mut self, neighbor: NodeId, layer: usize) {
        if layer < self.layer_count {
            if !self.connections[layer].contains(&neighbor) {
                self.connections[layer].push(neighbor);
            }
        }
    }

    /// Replace connections at a layer
    fn set_connections(&mut self, neighbors: Vec<NodeId>, layer: usize) {
        if layer < self.layer_count {
            self.connections[layer] = neighbors;
        }
    }
}

/// HNSW Index (in-memory only, no disk persistence)
pub struct HNSWIndex {
    nodes: Vec<Node>,            // The Arena: all nodes stored here
    entry_point: Option<NodeId>, // The highest node in the graph
    max_layers: usize,           // Current maximum height

    // Hyperparameters
    m: usize,               // Max neighbors per node (typical: 16)
    m0: usize,              // Max neighbors at Layer 0 (typical: 2×M = 32)
    ef_construction: usize, // Beam width during construction (typical: 200)
    level_lambda: f32,      // ml = 1/ln(M) for layer probability
}

impl HNSWIndex {
    pub fn new(m: usize, ef_construction: usize) -> Self {
        Self {
            nodes: Vec::new(),
            entry_point: None,
            max_layers: 0,
            m,
            m0: m * 2, // Layer 0 gets more connections
            ef_construction,
            level_lambda: 1.0 / (m as f32).ln(),
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

// ============================================================================
// Helper: OrderedFloat for heap comparisons
// ============================================================================

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
struct OrderedFloat(f32);

impl Eq for OrderedFloat {}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

// ============================================================================
// Distance Calculation
// ============================================================================

impl HNSWIndex {
    /// Euclidean distance between two vectors
    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

// ============================================================================
// Core Primitive: Search a Single Layer (Beam Search)
// ============================================================================

impl HNSWIndex {
    /// Search a single layer for the ef nearest neighbors to query
    ///
    /// entry_points: Starting nodes for the search
    /// layer: Which layer to search
    /// ef: Beam width (number of candidates to maintain)
    ///
    /// Returns: Up to ef nearest neighbors
    fn search_layer(
        &self,
        query: &[f32],
        entry_points: Vec<NodeId>,
        layer: usize,
        ef: usize,
    ) -> Vec<NodeId> {
        let mut visited = HashSet::new();

        // Candidates: min-heap (closest first) - nodes to explore
        let mut candidates: BinaryHeap<Reverse<(OrderedFloat, NodeId)>> = BinaryHeap::new();

        // Results: max-heap (farthest first) - best nodes found so far
        let mut results: BinaryHeap<(OrderedFloat, NodeId)> = BinaryHeap::new();

        // Initialize with entry points
        for &ep in &entry_points {
            let dist = self.distance(query, &self.nodes[ep].vector);
            let dist_ord = OrderedFloat(dist);

            candidates.push(Reverse((dist_ord, ep)));
            results.push((dist_ord, ep));
            visited.insert(ep);
        }

        // Beam search
        while let Some(Reverse((current_dist, current_id))) = candidates.pop() {
            // If current is worse than the worst result, no point continuing
            if let Some(&(worst_dist, _)) = results.peek() {
                if current_dist > worst_dist {
                    break;
                }
            }

            // Explore neighbors
            for &neighbor_id in self.nodes[current_id].neighbors(layer) {
                if visited.contains(&neighbor_id) {
                    continue;
                }
                visited.insert(neighbor_id);

                let neighbor_dist = self.distance(query, &self.nodes[neighbor_id].vector);
                let neighbor_dist_ord = OrderedFloat(neighbor_dist);

                // Is this neighbor better than our worst result?
                let should_add = if results.len() < ef {
                    true
                } else if let Some(&(worst_dist, _)) = results.peek() {
                    neighbor_dist_ord < worst_dist
                } else {
                    false
                };

                if should_add {
                    candidates.push(Reverse((neighbor_dist_ord, neighbor_id)));
                    results.push((neighbor_dist_ord, neighbor_id));

                    // Trim results to size ef
                    if results.len() > ef {
                        results.pop();
                    }
                }
            }
        }

        // Extract IDs from results
        results.into_iter().map(|(_, id)| id).collect()
    }
}

// ============================================================================
// Probabilistic Layer Generation
// ============================================================================

impl HNSWIndex {
    /// Generate a random layer using exponential decay
    ///
    /// Returns a layer number where:
    /// - ~50% of nodes: Layer 0 only
    /// - ~25% of nodes: Reach Layer 1
    /// - ~12.5% of nodes: Reach Layer 2
    /// - etc.
    fn random_level(&self) -> usize {
        let mut rng = rand::thread_rng();
        let r: f32 = rng.gen(); // Random [0, 1)

        // Exponential decay: level = floor(-ln(r) × ml)
        let level = (-r.ln() * self.level_lambda).floor() as usize;

        // Cap at some reasonable maximum (16 layers is enough for billions of vectors)
        level.min(16)
    }
}

// ============================================================================
// Neighbor Selection Heuristic (Diversity)
// ============================================================================

impl HNSWIndex {
    /// Select M neighbors using diversity heuristic
    ///
    /// Ensures connections span different directions in the vector space
    /// rather than all clustering in one region
    fn select_neighbors_heuristic(
        &self,
        query: &[f32],
        candidates: Vec<NodeId>,
        m: usize,
        _layer: usize,
    ) -> Vec<NodeId> {
        if candidates.len() <= m {
            return candidates;
        }

        // Sort candidates by distance to query (closest first)
        let mut sorted_candidates: Vec<_> = candidates
            .into_iter()
            .map(|id| {
                let dist = self.distance(query, &self.nodes[id].vector);
                (OrderedFloat(dist), id)
            })
            .collect();
        sorted_candidates.sort_by_key(|(dist, _)| *dist);

        let mut selected: Vec<usize> = Vec::new();

        // Diversity heuristic: prefer candidates that are not too close to already-selected
        for (cand_dist, cand_id) in sorted_candidates.iter() {
            if selected.len() >= m {
                break;
            }

            // Check if candidate is diverse compared to already-selected
            let mut is_diverse = true;

            for &sel_id in &selected {
                let dist_to_selected =
                    self.distance(&self.nodes[*cand_id].vector, &self.nodes[sel_id].vector);

                // If candidate is closer to an already-selected neighbor
                // than it is to the query, it is redundant
                if dist_to_selected < cand_dist.0 {
                    is_diverse = false;
                    break;
                }
            }

            if is_diverse {
                selected.push(*cand_id);
            }
        }

        // If we did not get M diverse candidates, fill with closest remaining
        if selected.len() < m {
            for (_, cand_id) in sorted_candidates.iter() {
                if !selected.contains(cand_id) {
                    selected.push(*cand_id);
                    if selected.len() >= m {
                        break;
                    }
                }
            }
        }

        selected
    }
}

// ============================================================================
// Edge Pruning
// ============================================================================

impl HNSWIndex {
    /// Prune connections if a node exceeds M limit
    ///
    /// Uses the same diversity heuristic to select best M connections
    fn prune_connections(&mut self, node_id: NodeId, m_max: usize, layer: usize) {
        let neighbors = self.nodes[node_id].neighbors(layer).to_vec();

        if neighbors.len() <= m_max {
            return; // No pruning needed
        }

        // Use the heuristic to select best M connections
        let node_vector = self.nodes[node_id].vector.clone();
        let best_neighbors = self.select_neighbors_heuristic(&node_vector, neighbors, m_max, layer);

        // Replace connections with pruned list
        self.nodes[node_id].set_connections(best_neighbors, layer);
    }
}

// ============================================================================
// Insertion Algorithm
// ============================================================================

impl HNSWIndex {
    /// Insert a new vector into the HNSW index
    ///
    /// Two-phase algorithm:
    /// 1. Zoom in: Descend from top layer to insertion layer (greedy search)
    /// 2. Insert: Link at each layer from insertion layer to 0 (beam search)
    pub fn insert(&mut self, vector: Vec<f32>) {
        let new_id = self.nodes.len();
        let new_level = self.random_level();

        // Create the new node
        let new_node = Node::new(vector.clone(), new_level);
        self.nodes.push(new_node);

        // Case 1: First node (becomes entry point)
        if self.entry_point.is_none() {
            self.entry_point = Some(new_id);
            self.max_layers = new_level;
            return;
        }

        let entry_point = self.entry_point.unwrap();

        // Phase 1: Zoom in from top to new_level + 1
        // Find the closest node to use as entry point for Phase 2
        let mut current_nearest = vec![entry_point];

        for layer in (new_level + 1..=self.max_layers).rev() {
            // Greedy search (ef=1) to find nearest in this layer
            current_nearest = self.search_layer(&vector, current_nearest, layer, 1);
        }

        // Phase 2: Insert and link from new_level down to 0
        for layer in (0..=new_level).rev() {
            // Find ef_construction nearest neighbors at this layer
            let candidates = self.search_layer(
                &vector,
                current_nearest.clone(),
                layer,
                self.ef_construction,
            );

            // Select M best neighbors (with diversity heuristic)
            let m_max = if layer == 0 { self.m0 } else { self.m };
            let neighbors = self.select_neighbors_heuristic(&vector, candidates, m_max, layer);

            // Bidirectional linking
            for &neighbor_id in &neighbors {
                // Add edge: new_node to neighbor
                self.nodes[new_id].connect(neighbor_id, layer);

                // Add edge: neighbor to new_node
                self.nodes[neighbor_id].connect(new_id, layer);

                // Prune neighbor if it exceeds M connections
                self.prune_connections(neighbor_id, m_max, layer);
            }

            // Update current_nearest for next layer
            current_nearest = neighbors;
        }

        // Update entry point if new node is taller
        if new_level > self.max_layers {
            self.entry_point = Some(new_id);
            self.max_layers = new_level;
        }
    }
}

// ============================================================================
// Visualization and Debugging
// ============================================================================

impl HNSWIndex {
    /// Print the graph structure (for debugging small graphs)
    pub fn print_structure(&self) {
        println!("\n╔═══════════════════════════════════════════════════════╗");
        println!("║            HNSW Graph Structure                       ║");
        println!("╚═══════════════════════════════════════════════════════╝");
        println!("\nTotal nodes: {}", self.len());
        println!("Max layers: {}", self.max_layers);
        println!("Entry point: {:?}", self.entry_point);
        println!("\nHyperparameters:");
        println!("  M (max connections): {}", self.m);
        println!("  M0 (Layer 0): {}", self.m0);
        println!("  ef_construction: {}", self.ef_construction);

        println!("\n{}", "=".repeat(60));

        for (i, node) in self.nodes.iter().enumerate() {
            println!("\nNode {} (layers: 0-{}):", i, node.layer_count - 1);
            println!(
                "  Vector: [{:.3}, {:.3}, ...]",
                node.vector.get(0).unwrap_or(&0.0),
                node.vector.get(1).unwrap_or(&0.0)
            );

            for layer in 0..node.layer_count {
                let neighbors = node.neighbors(layer);
                if neighbors.is_empty() {
                    println!("  Layer {}: (no connections)", layer);
                } else {
                    println!(
                        "  Layer {}: {:?} ({} connections)",
                        layer,
                        neighbors,
                        neighbors.len()
                    );
                }
            }
        }

        println!("\n{}", "=".repeat(60));
    }

    /// Get statistics about the graph
    pub fn statistics(&self) -> GraphStats {
        let mut layer_counts = vec![0; self.max_layers + 1];
        let mut total_connections = 0;
        let mut max_connections = 0;

        for node in &self.nodes {
            for layer in 0..node.layer_count {
                layer_counts[layer] += 1;
                let conn_count = node.neighbors(layer).len();
                total_connections += conn_count;
                max_connections = max_connections.max(conn_count);
            }
        }

        GraphStats {
            total_nodes: self.len(),
            max_layers: self.max_layers,
            layer_counts,
            total_connections,
            avg_connections: if self.len() > 0 {
                total_connections as f64 / self.len() as f64
            } else {
                0.0
            },
            max_connections,
        }
    }
}

#[derive(Debug)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub max_layers: usize,
    pub layer_counts: Vec<usize>,
    pub total_connections: usize,
    pub avg_connections: f64,
    pub max_connections: usize,
}

// ============================================================================
// Demo and Tests
// ============================================================================

fn main() {
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║         HNSW Basic Implementation Demo                ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    demo_small_graph();
    demo_statistics();
    demo_cluster_insertion();
}

fn demo_small_graph() {
    println!("\n\n╔═══════════════════════════════════════════════════════╗");
    println!("║              Demo 1: Small Graph (4 nodes)            ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    let mut index = HNSWIndex::new(2, 10); // M=2, ef_construction=10

    println!("\nInserting vectors:");
    println!("  Node 0: [0.0, 0.0]");
    index.insert(vec![0.0, 0.0]);

    println!("  Node 1: [1.0, 1.0]");
    index.insert(vec![1.0, 1.0]);

    println!("  Node 2: [0.1, 0.1]");
    index.insert(vec![0.1, 0.1]);

    println!("  Node 3: [9.0, 9.0]");
    index.insert(vec![9.0, 9.0]);

    index.print_structure();
}

fn demo_statistics() {
    println!("\n\n╔═══════════════════════════════════════════════════════╗");
    println!("║           Demo 2: Statistics (100 nodes)              ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    let mut index = HNSWIndex::new(16, 200);
    let mut rng = rand::thread_rng();

    println!("\nInserting 100 random 2D vectors...");
    for _ in 0..100 {
        let vector = vec![rng.gen::<f32>() * 10.0, rng.gen::<f32>() * 10.0];
        index.insert(vector);
    }

    let stats = index.statistics();

    println!("\nGraph Statistics:");
    println!("  Total nodes: {}", stats.total_nodes);
    println!("  Max layers: {}", stats.max_layers);
    println!("  Total connections: {}", stats.total_connections);
    println!("  Avg connections per node: {:.2}", stats.avg_connections);
    println!("  Max connections (any node): {}", stats.max_connections);

    println!("\nNodes per layer:");
    for (layer, count) in stats.layer_counts.iter().enumerate() {
        let percentage = (*count as f64 / stats.total_nodes as f64) * 100.0;
        println!("  Layer {}: {} nodes ({:.1}%)", layer, count, percentage);
    }
}

fn demo_cluster_insertion() {
    println!("\n\n╔═══════════════════════════════════════════════════════╗");
    println!("║        Demo 3: Clustered Data (Diversity Test)        ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    let mut index = HNSWIndex::new(4, 20);

    println!("\nInserting two clusters:");
    println!("  Cluster A (near origin): 5 points");
    for i in 0..5 {
        let offset = i as f32 * 0.1;
        index.insert(vec![offset, offset]);
    }

    println!("  Cluster B (far away): 5 points");
    for i in 0..5 {
        let offset = 10.0 + i as f32 * 0.1;
        index.insert(vec![offset, offset]);
    }

    index.print_structure();

    println!("\n\nObservations:");
    println!("  Look for cross-cluster connections (diversity heuristic at work)");
    println!("  Higher layers should bridge the gap between clusters");
    println!("  Layer 0 should have dense local connections");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insertion() {
        let mut index = HNSWIndex::new(2, 10);

        index.insert(vec![0.0, 0.0]);
        index.insert(vec![1.0, 1.0]);
        index.insert(vec![0.1, 0.1]);

        assert_eq!(index.len(), 3);
        assert!(index.entry_point.is_some());
    }

    #[test]
    fn test_layer_distribution() {
        let mut index = HNSWIndex::new(16, 200);
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let vector = vec![rng.gen::<f32>(), rng.gen::<f32>()];
            index.insert(vector);
        }

        let stats = index.statistics();

        // Layer 0 should have all nodes
        assert_eq!(stats.layer_counts[0], 1000);

        // Higher layers should have fewer nodes (exponential decay)
        for i in 1..stats.layer_counts.len() {
            assert!(stats.layer_counts[i] < stats.layer_counts[i - 1]);
        }
    }

    #[test]
    fn test_diversity_heuristic() {
        let mut index = HNSWIndex::new(2, 10);

        // Insert tight cluster
        for i in 0..5 {
            let offset = i as f32 * 0.01;
            index.insert(vec![offset, offset]);
        }

        // Insert far node
        index.insert(vec![10.0, 10.0]);

        // Far node should have connections (not isolated)
        let far_node = &index.nodes[5];
        assert!(
            !far_node.connections[0].is_empty(),
            "Far node should have connections due to diversity heuristic"
        );
    }
}
