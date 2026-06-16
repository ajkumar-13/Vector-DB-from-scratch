// Skip List Demo: Understanding Hierarchical Navigation
// This demonstrates the core concept behind HNSW's layered structure

use rand::Rng;
use std::fmt;

/// A node in the skip list
#[derive(Clone)]
struct Node {
    value: i32,
    // Each node has multiple "next" pointers for different layers
    forward: Vec<Option<usize>>, // Vec of node indices
}

impl Node {
    fn new(value: i32, level: usize) -> Self {
        Self {
            value,
            forward: vec![None; level + 1],
        }
    }
}

/// Skip List: A probabilistic data structure with O(log n) search
pub struct SkipList {
    nodes: Vec<Node>,
    head_idx: usize,
    max_level: usize,
    current_level: usize,
    p: f32, // Probability for level selection (typically 0.5)
}

impl SkipList {
    pub fn new(max_level: usize) -> Self {
        let head = Node::new(i32::MIN, max_level);
        Self {
            nodes: vec![head],
            head_idx: 0,
            max_level,
            current_level: 0,
            p: 0.5,
        }
    }

    /// Random level selection with exponential decay
    fn random_level(&self) -> usize {
        let mut rng = rand::thread_rng();
        let mut level = 0;

        while rng.gen::<f32>() < self.p && level < self.max_level {
            level += 1;
        }

        level
    }

    /// Insert a value into the skip list
    pub fn insert(&mut self, value: i32) {
        let level = self.random_level();

        // Update current level if needed
        if level > self.current_level {
            self.current_level = level;
        }

        // Find the insertion point at each level
        let mut update = vec![self.head_idx; level + 1];
        let mut current_idx = self.head_idx;

        // Traverse from top level to bottom
        for l in (0..=level).rev() {
            // Move forward at this level while possible
            while let Some(next_idx) = self.nodes[current_idx].forward[l] {
                if self.nodes[next_idx].value < value {
                    current_idx = next_idx;
                } else {
                    break;
                }
            }
            update[l] = current_idx;
        }

        // Create new node
        let new_idx = self.nodes.len();
        let new_node = Node::new(value, level);
        self.nodes.push(new_node);

        // Update pointers at each level
        for l in 0..=level {
            let predecessor = update[l];
            self.nodes[new_idx].forward[l] = self.nodes[predecessor].forward[l];
            self.nodes[predecessor].forward[l] = Some(new_idx);
        }
    }

    /// Search for a value with instrumentation
    pub fn search(&self, target: i32) -> SearchResult {
        let mut hops = 0;
        let mut path = Vec::new();
        let mut current_idx = self.head_idx;

        // Start from the highest level
        for level in (0..=self.current_level).rev() {
            // Record which level we're searching
            let start_value = self.nodes[current_idx].value;

            // Move forward at this level
            while let Some(next_idx) = self.nodes[current_idx].forward[level] {
                let next_value = self.nodes[next_idx].value;

                if next_value <= target {
                    current_idx = next_idx;
                    hops += 1;
                } else {
                    break;
                }
            }

            path.push(LayerHop {
                level,
                from: if start_value == i32::MIN {
                    0
                } else {
                    start_value
                },
                to: self.nodes[current_idx].value,
                hops_at_layer: hops,
            });
        }

        let found = self.nodes[current_idx].value == target;

        SearchResult {
            found,
            value: if found { Some(target) } else { None },
            total_hops: hops,
            path,
        }
    }

    /// Visualize the skip list structure
    pub fn visualize(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "\nSkip List Structure (max_level={})\n",
            self.max_level
        ));
        output.push_str(&"=".repeat(60));
        output.push_str("\n\n");

        // Print from top level down
        for level in (0..=self.current_level).rev() {
            output.push_str(&format!("Layer {}: ", level));

            let mut current_idx = self.head_idx;
            let mut values = Vec::new();

            // Follow pointers at this level
            while let Some(next_idx) = self.nodes[current_idx].forward[level] {
                values.push(self.nodes[next_idx].value);
                current_idx = next_idx;
            }

            if values.is_empty() {
                output.push_str("(empty)\n");
            } else {
                for v in values {
                    output.push_str(&format!("{} -> ", v));
                }
                output.push_str("nil\n");
            }
        }

        output
    }

    /// Show statistics about the skip list
    pub fn stats(&self) -> SkipListStats {
        let mut layer_counts = vec![0; self.max_level + 1];

        for node in &self.nodes[1..] {
            // Skip head node
            for level in 0..node.forward.len() {
                layer_counts[level] += 1;
            }
        }

        SkipListStats {
            total_nodes: self.nodes.len() - 1, // Exclude head
            max_level: self.max_level,
            current_level: self.current_level,
            layer_counts,
        }
    }
}

/// Result of a search operation
#[derive(Debug)]
pub struct SearchResult {
    pub found: bool,
    pub value: Option<i32>,
    pub total_hops: usize,
    pub path: Vec<LayerHop>,
}

/// A hop at a specific layer
#[derive(Debug)]
pub struct LayerHop {
    pub level: usize,
    pub from: i32,
    pub to: i32,
    pub hops_at_layer: usize,
}

/// Statistics about the skip list
pub struct SkipListStats {
    pub total_nodes: usize,
    pub max_level: usize,
    pub current_level: usize,
    pub layer_counts: Vec<usize>,
}

impl fmt::Display for SearchResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\nSearch Result:")?;
        writeln!(f, "  Found: {}", self.found)?;
        writeln!(f, "  Total hops: {}", self.total_hops)?;
        writeln!(f, "\nSearch path (top to bottom):")?;

        for hop in &self.path {
            writeln!(
                f,
                "  Layer {}: {} to {} (cumulative hops: {})",
                hop.level, hop.from, hop.to, hop.hops_at_layer
            )?;
        }

        Ok(())
    }
}

impl fmt::Display for SkipListStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\nSkip List Statistics:")?;
        writeln!(f, "  Total nodes: {}", self.total_nodes)?;
        writeln!(f, "  Max level: {}", self.max_level)?;
        writeln!(f, "  Current level: {}", self.current_level)?;
        writeln!(f, "\nNodes per layer:")?;

        for (level, count) in self.layer_counts.iter().enumerate() {
            let percentage = if self.total_nodes > 0 {
                (*count as f32 / self.total_nodes as f32) * 100.0
            } else {
                0.0
            };
            writeln!(f, "  Layer {}: {} nodes ({:.1}%)", level, count, percentage)?;
        }

        Ok(())
    }
}

// ============================================================================
// Comparison: Skip List vs Linear Search
// ============================================================================

/// Linear search for comparison
fn linear_search(values: &[i32], target: i32) -> usize {
    let mut hops = 0;
    for &value in values {
        hops += 1;
        if value == target {
            break;
        }
    }
    hops
}

/// Compare skip list vs linear search
pub fn compare_search_performance(n: usize, num_searches: usize) {
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║     Skip List vs Linear Search Performance Test      ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    // Build skip list
    println!("\nBuilding skip list with {} elements...", n);
    let mut skip_list = SkipList::new(16); // max 16 levels
    let mut linear_list = Vec::new();

    for i in 1..=n as i32 {
        skip_list.insert(i * 10); // Insert 10, 20, 30, ...
        linear_list.push(i * 10);
    }

    // Print structure for small lists
    if n <= 20 {
        println!("{}", skip_list.visualize());
    }

    println!("{}", skip_list.stats());

    // Perform searches
    println!("\n\nPerforming {} random searches...\n", num_searches);

    let mut skip_hops = 0;
    let mut linear_hops = 0;

    let mut rng = rand::thread_rng();

    for _ in 0..num_searches {
        let target = rng.gen_range(1..=n as i32) * 10;

        // Skip list search
        let result = skip_list.search(target);
        skip_hops += result.total_hops;

        // Linear search
        linear_hops += linear_search(&linear_list, target);
    }

    let avg_skip = skip_hops as f64 / num_searches as f64;
    let avg_linear = linear_hops as f64 / num_searches as f64;

    println!("Results:");
    println!("  Skip List:    {:.2} hops per search (average)", avg_skip);
    println!(
        "  Linear List:  {:.2} hops per search (average)",
        avg_linear
    );
    println!("  Speedup:      {:.2}x faster", avg_linear / avg_skip);

    // Complexity comparison
    let theoretical_skip = (n as f64).log2();
    let theoretical_linear = n as f64 / 2.0; // Average case

    println!("\nTheoretical expectations:");
    println!("  Skip List:    O(log n) ≈ {:.2}", theoretical_skip);
    println!("  Linear List:  O(n)     ≈ {:.2}", theoretical_linear);
}

// ============================================================================
// Demonstration: How HNSW Uses This Concept
// ============================================================================

pub fn explain_hnsw_connection() {
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║        How HNSW Uses Skip List Principles            ║");
    println!("╚═══════════════════════════════════════════════════════╝");
    println!("\n1. Skip List (1D, ordered):");
    println!("   - Nodes are numbers on a line");
    println!("   - Higher layers skip over more nodes");
    println!("   - Edges point to 'next' in sorted order");
    println!("\n2. HNSW (N-dimensional, proximity graph):");
    println!("   - Nodes are vectors in high-dimensional space");
    println!("   - Higher layers connect distant regions");
    println!("   - Edges point to 'nearest neighbors' in vector space");
    println!("\n3. Key Similarities:");
    println!("   [check] Hierarchical layers");
    println!("   [check] Exponential probability for layer selection");
    println!("   [check] Start search at top, zoom in to bottom");
    println!("   [check] O(log n) search complexity");
    println!("\n4. Key Differences:");
    println!("   Skip List: 1 dimension (before/after)");
    println!("   HNSW: N dimensions (M nearest neighbors)");
    println!("   Skip List: Total ordering");
    println!("   HNSW: Proximity graph (no global order)");
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    // Demo 1: Small visualization
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║              Demo 1: Small Skip List                  ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    let mut skip_list = SkipList::new(4);

    println!("\nInserting: 10, 20, 30, 40, 50, 60, 70, 80, 90, 100");
    for i in 1..=10 {
        skip_list.insert(i * 10);
    }

    println!("{}", skip_list.visualize());
    println!("{}", skip_list.stats());

    // Search demo
    println!("\n\nSearching for 70:");
    let result = skip_list.search(70);
    println!("{}", result);

    println!("\n\nSearching for 35 (not present):");
    let result = skip_list.search(35);
    println!("{}", result);

    // Demo 2: Performance comparison
    println!("\n\n╔═══════════════════════════════════════════════════════╗");
    println!("║           Demo 2: Performance Comparison              ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    compare_search_performance(1000, 100);

    // Demo 3: Large scale
    println!("\n\n╔═══════════════════════════════════════════════════════╗");
    println!("║          Demo 3: Large Scale (10,000 nodes)          ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    compare_search_performance(10_000, 1000);

    // Explanation
    explain_hnsw_connection();

    println!("\n\n╔═══════════════════════════════════════════════════════╗");
    println!("║                   Key Takeaways                       ║");
    println!("╚═══════════════════════════════════════════════════════╝");
    println!("\nSkip lists achieve O(log n) search in sorted data");
    println!("HNSW applies the same hierarchical principle to graphs");
    println!("Layer selection uses exponential decay (p is approximately 0.5)");
    println!("Start search at top layer, descend to ground layer");
    println!("This is the H (Hierarchical) in HNSW.");
    println!();
}
