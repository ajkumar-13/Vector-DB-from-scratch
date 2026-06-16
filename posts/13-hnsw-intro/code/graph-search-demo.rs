// Graph Search Demo: Understanding Greedy Search and Local Minima
// This demonstrates how HNSW navigates proximity graphs

use std::collections::{HashMap, HashSet};
use std::fmt;

/// A 2D point representing a vector
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub id: usize,
}

impl Point {
    fn new(id: usize, x: f32, y: f32) -> Self {
        Self { id, x, y }
    }

    /// Euclidean distance to another point
    fn distance(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// A graph where nodes are points and edges connect nearby points
pub struct ProximityGraph {
    nodes: Vec<Point>,
    edges: HashMap<usize, Vec<usize>>, // node_id -> list of neighbor ids
}

impl ProximityGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: HashMap::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, point: Point) {
        self.nodes.push(point);
        self.edges.insert(point.id, Vec::new());
    }

    /// Add an edge between two nodes (bidirectional)
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.entry(from).or_default().push(to);
        self.edges.entry(to).or_default().push(from);
    }

    /// Get neighbors of a node
    pub fn neighbors(&self, node_id: usize) -> &[usize] {
        self.edges
            .get(&node_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get a node by ID
    pub fn get_node(&self, id: usize) -> Option<&Point> {
        self.nodes.iter().find(|p| p.id == id)
    }

    /// Greedy search from an entry point to find nearest to query
    pub fn greedy_search(&self, entry_id: usize, query: &Point) -> SearchResult {
        let mut current_id = entry_id;
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        let mut steps = 0;

        visited.insert(current_id);

        let current = self.get_node(current_id).unwrap();
        path.push(SearchStep {
            node_id: current_id,
            distance_to_query: current.distance(query),
            reason: "Entry point".to_string(),
        });

        loop {
            steps += 1;
            let current = self.get_node(current_id).unwrap();
            let current_dist = current.distance(query);

            // Find best neighbor
            let mut best_id = current_id;
            let mut best_dist = current_dist;

            for &neighbor_id in self.neighbors(current_id) {
                if visited.contains(&neighbor_id) {
                    continue;
                }

                let neighbor = self.get_node(neighbor_id).unwrap();
                let neighbor_dist = neighbor.distance(query);

                if neighbor_dist < best_dist {
                    best_id = neighbor_id;
                    best_dist = neighbor_dist;
                }
            }

            // No improvement? We are at a local minimum
            if best_id == current_id {
                path.push(SearchStep {
                    node_id: current_id,
                    distance_to_query: current_dist,
                    reason: "Local minimum (no better neighbors)".to_string(),
                });
                break;
            }

            // Move to better neighbor
            visited.insert(best_id);
            current_id = best_id;

            path.push(SearchStep {
                node_id: best_id,
                distance_to_query: best_dist,
                reason: format!("Moved to closer neighbor"),
            });
        }

        SearchResult {
            found_id: current_id,
            steps,
            path,
        }
    }

    /// Find the actual closest node (brute force for comparison)
    pub fn find_closest(&self, query: &Point) -> (usize, f32) {
        self.nodes
            .iter()
            .map(|p| (p.id, p.distance(query)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }

    /// Visualize the graph (for small graphs)
    pub fn visualize(&self) -> String {
        let mut output = String::new();

        output.push_str("\nGraph Structure:\n");
        output.push_str(&"=".repeat(60));
        output.push_str("\n");

        for node in &self.nodes {
            output.push_str(&format!(
                "\nNode {} at ({:.2}, {:.2})\n",
                node.id, node.x, node.y
            ));

            let neighbors = self.neighbors(node.id);
            if neighbors.is_empty() {
                output.push_str("  No neighbors\n");
            } else {
                output.push_str("  Neighbors: ");
                for &neighbor_id in neighbors {
                    let neighbor = self.get_node(neighbor_id).unwrap();
                    output.push_str(&format!(
                        "{} (d={:.2}), ",
                        neighbor_id,
                        node.distance(neighbor)
                    ));
                }
                output.push('\n');
            }
        }

        output
    }
}

#[derive(Debug)]
pub struct SearchStep {
    pub node_id: usize,
    pub distance_to_query: f32,
    pub reason: String,
}

#[derive(Debug)]
pub struct SearchResult {
    pub found_id: usize,
    pub steps: usize,
    pub path: Vec<SearchStep>,
}

impl fmt::Display for SearchResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\nSearch Result:")?;
        writeln!(f, "  Found: Node {}", self.found_id)?;
        writeln!(f, "  Steps: {}", self.steps)?;
        writeln!(f, "\nSearch path:")?;

        for (i, step) in self.path.iter().enumerate() {
            writeln!(
                f,
                "  {}. Node {} (distance {:.3}) - {}",
                i + 1,
                step.node_id,
                step.distance_to_query,
                step.reason
            )?;
        }

        Ok(())
    }
}

// ============================================================================
// Demo Scenarios
// ============================================================================

/// Demo 1: Simple successful search
pub fn demo_simple_success() {
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║         Demo 1: Simple Successful Greedy Search       ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    let mut graph = ProximityGraph::new();

    // Create a simple cluster
    //     1 ---- 2 ---- 3
    //     |      |      |
    //     4 ---- 5 ---- 6
    //     |      |      |
    //     7 ---- 8 ---- 9

    let nodes = vec![
        Point::new(1, 0.0, 0.0),
        Point::new(2, 1.0, 0.0),
        Point::new(3, 2.0, 0.0),
        Point::new(4, 0.0, 1.0),
        Point::new(5, 1.0, 1.0),
        Point::new(6, 2.0, 1.0),
        Point::new(7, 0.0, 2.0),
        Point::new(8, 1.0, 2.0),
        Point::new(9, 2.0, 2.0),
    ];

    for node in nodes {
        graph.add_node(node);
    }

    // Add grid edges
    let edges = vec![
        (1, 2),
        (2, 3),
        (4, 5),
        (5, 6),
        (7, 8),
        (8, 9),
        (1, 4),
        (4, 7),
        (2, 5),
        (5, 8),
        (3, 6),
        (6, 9),
    ];

    for (from, to) in edges {
        graph.add_edge(from, to);
    }

    println!("{}", graph.visualize());

    // Query near node 9
    let query = Point::new(999, 1.9, 1.9);
    println!("\nQuery point: ({:.2}, {:.2})", query.x, query.y);

    // Search from node 1 (far corner)
    println!("\nStarting greedy search from Node 1...");
    let result = graph.greedy_search(1, &query);
    println!("{}", result);

    // Check if we found the actual closest
    let (actual_closest, actual_dist) = graph.find_closest(&query);
    println!(
        "\nActual closest node: {} (distance {:.3})",
        actual_closest, actual_dist
    );

    if result.found_id == actual_closest {
        println!("Success: Greedy search found the global optimum.");
    } else {
        println!("Missed: Greedy search found a local minimum.");
    }
}

/// Demo 2: Local minima problem
pub fn demo_local_minima() {
    println!("\n\n╔═══════════════════════════════════════════════════════╗");
    println!("║         Demo 2: Local Minima Problem                  ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    let mut graph = ProximityGraph::new();

    // Create two separate clusters with a bridge
    // Cluster A (left):  1 -- 2 -- 3
    // Bridge:                 4
    // Cluster B (right):      5 -- 6 -- 7

    let nodes = vec![
        Point::new(1, 0.0, 0.0),
        Point::new(2, 1.0, 0.0),
        Point::new(3, 2.0, 0.0),
        Point::new(4, 4.0, 0.0), // Bridge
        Point::new(5, 6.0, 0.0),
        Point::new(6, 7.0, 0.0),
        Point::new(7, 8.0, 0.0),
    ];

    for node in nodes {
        graph.add_node(node);
    }

    // Cluster A
    graph.add_edge(1, 2);
    graph.add_edge(2, 3);

    // Bridge
    graph.add_edge(3, 4);
    graph.add_edge(4, 5);

    // Cluster B
    graph.add_edge(5, 6);
    graph.add_edge(6, 7);

    println!("{}", graph.visualize());

    // Query near cluster B
    let query = Point::new(999, 7.5, 0.0);
    println!(
        "\nQuery point: ({:.2}, {:.2}) (very close to Node 7)",
        query.x, query.y
    );

    // Search from node 1 (cluster A)
    println!("\nStarting greedy search from Node 1 (Cluster A)...");
    let result = graph.greedy_search(1, &query);
    println!("{}", result);

    // Check actual closest
    let (actual_closest, actual_dist) = graph.find_closest(&query);
    println!(
        "\nActual closest node: {} (distance {:.3})",
        actual_closest, actual_dist
    );

    if result.found_id == actual_closest {
        println!("Success: Greedy search found the global optimum.");
    } else {
        println!("Missed: Greedy search got stuck in local minimum.");
        println!("\nExplanation:");
        println!("  Greedy search only looks at immediate neighbors");
        println!("  Node 3 is closer to query than Nodes 1 or 2");
        println!("  But Node 3's only neighbor (Node 4) is further away");
        println!("  Greedy search stops, even though Cluster B is much closer");
        println!("\n This is why HNSW needs multiple layers.");
    }
}

/// Demo 3: How layers help
pub fn demo_layers_solution() {
    println!("\n\n╔═══════════════════════════════════════════════════════╗");
    println!("║         Demo 3: How Multiple Layers Solve This        ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    let mut graph = ProximityGraph::new();

    // Same structure but with "highway" connections
    // Layer 0: 1 -- 2 -- 3 -- 4 -- 5 -- 6 -- 7
    // Layer 1: 1 ------------- 4 ----------- 7  (long-range)

    let nodes = vec![
        Point::new(1, 0.0, 0.0),
        Point::new(2, 1.0, 0.0),
        Point::new(3, 2.0, 0.0),
        Point::new(4, 4.0, 0.0),
        Point::new(5, 6.0, 0.0),
        Point::new(6, 7.0, 0.0),
        Point::new(7, 8.0, 0.0),
    ];

    for node in nodes {
        graph.add_node(node);
    }

    // Layer 0 edges (local)
    graph.add_edge(1, 2);
    graph.add_edge(2, 3);
    graph.add_edge(3, 4);
    graph.add_edge(4, 5);
    graph.add_edge(5, 6);
    graph.add_edge(6, 7);

    // Layer 1 edges (highways) - long-range connections
    graph.add_edge(1, 4);
    graph.add_edge(4, 7);

    println!("{}", graph.visualize());

    // Query near node 7
    let query = Point::new(999, 7.5, 0.0);
    println!("\nQuery point: ({:.2}, {:.2})", query.x, query.y);

    println!("\nStarting greedy search from Node 1...");
    let result = graph.greedy_search(1, &query);
    println!("{}", result);

    println!("\nExplanation:");
    println!("  Node 1 has a highway edge to Node 4");
    println!("  Node 4 has a highway edge to Node 7");
    println!("  Greedy search can jump across the graph");
    println!("  Then refine locally to find the exact answer");
    println!("\n This is the power of hierarchical navigation.");
}

/// Demo 4: Beam search vs greedy search
pub fn demo_beam_search() {
    println!("\n\n╔═══════════════════════════════════════════════════════╗");
    println!("║         Demo 4: Beam Search vs Greedy Search          ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    println!("\nGreedy Search:");
    println!("  Maintains 1 current node");
    println!("  Moves to the single best neighbor");
    println!("  Fast but can get stuck in local minima");
    println!("  Complexity: O(log N) hops");

    println!("\nBeam Search:");
    println!("  Maintains K candidate nodes (beam width)");
    println!("  Explores from all K simultaneously");
    println!("  Keeps the best K after each step");
    println!("  More robust, less likely to get stuck");
    println!("  Complexity: O(K x log N) hops");

    println!("\nHNSW uses:");
    println!("  Greedy search at higher layers (fast navigation)");
    println!("  Beam search at Layer 0 (accurate final answer)");
    println!("  Beam width = ef_search (configurable)");
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║    Graph-Based Search: Understanding HNSW Navigation  ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    demo_simple_success();
    demo_local_minima();
    demo_layers_solution();
    demo_beam_search();

    println!("\n\n╔═══════════════════════════════════════════════════════╗");
    println!("║                   Key Takeaways                       ║");
    println!("╚═══════════════════════════════════════════════════════╝");
    println!("\n1. Greedy search is fast but can get stuck in local minima");
    println!("2. Hierarchical layers provide 'highways' for long jumps");
    println!("3. Start at top layer (global view), descend to bottom (local)");
    println!("4. Beam search at ground layer prevents local minima");
    println!("5. This is how HNSW achieves O(log N) search complexity");
    println!();
}
