// Neighbor Selection Deep Dive: Understanding the Diversity Heuristic
// This file explores different neighbor selection strategies and their impact

use rand::Rng;

// ============================================================================
// Simplified Node and Graph for Demo
// ============================================================================

#[derive(Clone)]
struct Point {
    id: usize,
    coords: Vec<f32>,
}

impl Point {
    fn new(id: usize, coords: Vec<f32>) -> Self {
        Self { id, coords }
    }

    fn distance(&self, other: &Point) -> f32 {
        self.coords
            .iter()
            .zip(other.coords.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

// ============================================================================
// Strategy 1: Naive (Closest M)
// ============================================================================

fn select_naive(query: &Point, candidates: &[Point], m: usize) -> Vec<usize> {
    // Just take the M closest candidates
    let mut sorted: Vec<_> = candidates
        .iter()
        .map(|c| (query.distance(c), c.id))
        .collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    sorted.iter().take(m).map(|(_, id)| *id).collect()
}

// ============================================================================
// Strategy 2: Diversity Heuristic (HNSW Paper)
// ============================================================================

fn select_heuristic(query: &Point, candidates: &[Point], m: usize) -> Vec<usize> {
    if candidates.len() <= m {
        return candidates.iter().map(|c| c.id).collect();
    }

    // Sort by distance to query
    let mut sorted: Vec<_> = candidates.iter().map(|c| (query.distance(c), c)).collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut selected = Vec::new();
    let mut selected_points = Vec::new();

    for (dist_to_query, candidate) in &sorted {
        if selected.len() >= m {
            break;
        }

        // Check if candidate is diverse w.r.t. already-selected
        let mut is_diverse = true;

        for sel_point in &selected_points {
            let dist_to_selected = candidate.distance(sel_point);

            // If candidate is closer to an already-selected point
            // than it is to the query, it is redundant
            if dist_to_selected < *dist_to_query {
                is_diverse = false;
                break;
            }
        }

        if is_diverse {
            selected.push(candidate.id);
            selected_points.push((*candidate).clone());
        }
    }

    // If we did not get M diverse candidates, fill with closest remaining
    if selected.len() < m {
        for (_, candidate) in &sorted {
            if !selected.contains(&candidate.id) {
                selected.push(candidate.id);
                if selected.len() >= m {
                    break;
                }
            }
        }
    }

    selected
}

// ============================================================================
// Strategy 3: Angular Diversity (Alternative)
// ============================================================================

fn select_angular(query: &Point, candidates: &[Point], m: usize) -> Vec<usize> {
    if candidates.len() <= m {
        return candidates.iter().map(|c| c.id).collect();
    }

    // Sort by distance to query
    let mut sorted: Vec<_> = candidates.iter().map(|c| (query.distance(c), c)).collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut selected = Vec::new();
    let mut selected_points = Vec::new();

    for (_dist_to_query, candidate) in &sorted {
        if selected.len() >= m {
            break;
        }

        // Check angular diversity
        let mut is_diverse = true;

        for sel_point in &selected_points {
            // Compute cosine similarity between vectors from query
            let angle = cosine_similarity_from_query(query, candidate, sel_point);

            // If angle is > 0.9, vectors are too similar (< 25 degrees)
            if angle > 0.9 {
                is_diverse = false;
                break;
            }
        }

        if is_diverse {
            selected.push(candidate.id);
            selected_points.push((*candidate).clone());
        }
    }

    // Fill remaining
    if selected.len() < m {
        for (_, candidate) in &sorted {
            if !selected.contains(&candidate.id) {
                selected.push(candidate.id);
                if selected.len() >= m {
                    break;
                }
            }
        }
    }

    selected
}

fn cosine_similarity_from_query(query: &Point, a: &Point, b: &Point) -> f32 {
    // Vectors from query to a and b
    let vec_a: Vec<f32> = a
        .coords
        .iter()
        .zip(query.coords.iter())
        .map(|(ai, qi)| ai - qi)
        .collect();

    let vec_b: Vec<f32> = b
        .coords
        .iter()
        .zip(query.coords.iter())
        .map(|(bi, qi)| bi - qi)
        .collect();

    // Dot product
    let dot: f32 = vec_a.iter().zip(vec_b.iter()).map(|(a, b)| a * b).sum();

    // Magnitudes
    let mag_a: f32 = vec_a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = vec_b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }

    dot / (mag_a * mag_b)
}

// ============================================================================
// Visualization: Show Selection Differences
// ============================================================================

fn visualize_selection(
    query: &Point,
    candidates: &[Point],
    selected: &[usize],
    strategy_name: &str,
) {
    println!("\n{} Selection:", strategy_name);
    println!("{}", "=".repeat(60));

    println!("Query: {:?}", query.coords);
    println!("\nCandidates (sorted by distance):");

    let mut sorted: Vec<_> = candidates.iter().map(|c| (query.distance(c), c)).collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    for (i, (dist, cand)) in sorted.iter().enumerate() {
        let status = if selected.contains(&cand.id) {
            "SELECTED"
        } else {
            "Rejected"
        };

        println!(
            "  {}. ID={} coords={:?} dist={:.3} {}",
            i + 1,
            cand.id,
            cand.coords,
            dist,
            status
        );
    }

    // Show diversity metrics
    println!("\nDiversity Analysis:");
    let selected_points: Vec<_> = candidates
        .iter()
        .filter(|c| selected.contains(&c.id))
        .collect();

    for (i, p1) in selected_points.iter().enumerate() {
        for (j, p2) in selected_points.iter().enumerate() {
            if j <= i {
                continue;
            }
            let dist = p1.distance(p2);
            println!("  Distance(ID={}, ID={}): {:.3}", p1.id, p2.id, dist);
        }
    }
}

// ============================================================================
// Demo Scenarios
// ============================================================================

fn demo_clustered_candidates() {
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║        Scenario 1: Clustered Candidates               ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    println!("\nSetup: Query at origin, 6 candidates:");
    println!("  - 4 candidates in tight cluster at (1.0, 0.0)");
    println!("  - 2 candidates far away at (5.0, 0.0)");

    let query = Point::new(999, vec![0.0, 0.0]);

    let candidates = vec![
        Point::new(0, vec![1.0, 0.0]),
        Point::new(1, vec![1.05, 0.05]),
        Point::new(2, vec![1.1, 0.0]),
        Point::new(3, vec![0.95, -0.05]),
        Point::new(4, vec![5.0, 0.0]),
        Point::new(5, vec![5.1, 0.1]),
    ];

    let m = 3;

    // Strategy 1: Naive
    let naive = select_naive(&query, &candidates, m);
    visualize_selection(&query, &candidates, &naive, "Naive (Closest M)");

    // Strategy 2: Heuristic
    let heuristic = select_heuristic(&query, &candidates, m);
    visualize_selection(&query, &candidates, &heuristic, "Diversity Heuristic");

    println!("\n\nKey Insight:");
    println!("  Naive: Picks all 3 from the tight cluster");
    println!("  Heuristic: Picks 1 from cluster + 1 far candidate");
    println!("  Result: Heuristic maintains long-range connectivity.");
}

fn demo_angular_vs_distance() {
    println!("\n\n╔═══════════════════════════════════════════════════════╗");
    println!("║        Scenario 2: Angular vs Distance Diversity      ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    println!("\nSetup: Query at origin, candidates at different angles");
    println!("  - Some close but at different angles");
    println!("  - Some far but at similar angles");

    let query = Point::new(999, vec![0.0, 0.0]);

    let candidates = vec![
        Point::new(0, vec![1.0, 0.0]), // Close, 0°
        Point::new(1, vec![0.7, 0.7]), // Close, 45°
        Point::new(2, vec![0.0, 1.0]), // Close, 90°
        Point::new(3, vec![2.0, 0.1]), // Far, ~0° (similar to ID=0)
        Point::new(4, vec![1.5, 1.5]), // Far, 45° (similar to ID=1)
    ];

    let m = 3;

    // Distance-based heuristic
    let dist_heuristic = select_heuristic(&query, &candidates, m);
    visualize_selection(&query, &candidates, &dist_heuristic, "Distance Heuristic");

    // Angular diversity
    let angular = select_angular(&query, &candidates, m);
    visualize_selection(&query, &candidates, &angular, "Angular Diversity");

    println!("\n\nKey Insight:");
    println!("  Distance heuristic: Prefers closest diverse points");
    println!("  Angular diversity: Ensures coverage of different directions");
    println!("  HNSW uses distance heuristic (simpler, works well in practice)");
}

fn demo_extreme_m_values() {
    println!("\n\n╔═══════════════════════════════════════════════════════╗");
    println!("║        Scenario 3: Effect of M Parameter              ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    let query = Point::new(999, vec![0.0, 0.0]);

    let mut rng = rand::thread_rng();
    let candidates: Vec<_> = (0..20)
        .map(|i| {
            let angle = (i as f32 / 20.0) * 2.0 * std::f32::consts::PI;
            let radius = 1.0 + rng.gen::<f32>() * 2.0;
            Point::new(i, vec![radius * angle.cos(), radius * angle.sin()])
        })
        .collect();

    for m in [2, 5, 10, 15] {
        println!("\n\nM = {} (select {} from 20 candidates):", m, m);
        let selected = select_heuristic(&query, &candidates, m);
        println!("  Selected IDs: {:?}", selected);

        // Compute average pairwise distance
        let selected_points: Vec<_> = candidates
            .iter()
            .filter(|c| selected.contains(&c.id))
            .collect();

        let mut total_dist = 0.0;
        let mut count = 0;

        for i in 0..selected_points.len() {
            for j in (i + 1)..selected_points.len() {
                total_dist += selected_points[i].distance(selected_points[j]);
                count += 1;
            }
        }

        let avg_dist = if count > 0 {
            total_dist / count as f32
        } else {
            0.0
        };

        println!("  Avg pairwise distance: {:.3}", avg_dist);
    }

    println!("\n\nKey Insight:");
    println!("  Small M (2-4): Very selective, high diversity");
    println!("  Medium M (8-16): Balance of coverage and selectivity");
    println!("  Large M (32-64): More connections, less diversity pressure");
    println!("  HNSW typical: M=16 (good balance)");
}

fn demo_pruning_scenario() {
    println!("\n\n╔═══════════════════════════════════════════════════════╗");
    println!("║        Scenario 4: Edge Pruning (Node Exceeds M)      ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    println!("\nSetup: Node has 8 connections, but M=5");
    println!("  Need to prune 3 connections while maintaining diversity");

    let node = Point::new(0, vec![5.0, 5.0]);

    let connections = vec![
        Point::new(1, vec![5.1, 5.0]),   // Very close
        Point::new(2, vec![5.0, 5.1]),   // Very close
        Point::new(3, vec![5.05, 5.05]), // Very close (redundant!)
        Point::new(4, vec![6.0, 5.0]),   // Medium distance, East
        Point::new(5, vec![5.0, 6.0]),   // Medium distance, North
        Point::new(6, vec![4.0, 5.0]),   // Medium distance, West
        Point::new(7, vec![5.0, 4.0]),   // Medium distance, South
        Point::new(8, vec![7.0, 7.0]),   // Far, Northeast
    ];

    let m = 5;

    println!("\nCurrent connections:");
    for conn in &connections {
        let dist = node.distance(conn);
        println!("  ID={} coords={:?} dist={:.3}", conn.id, conn.coords, dist);
    }

    let kept = select_heuristic(&node, &connections, m);

    println!("\nAfter pruning (keep {}):", m);
    for id in &kept {
        let conn = connections.iter().find(|c| c.id == *id).unwrap();
        let dist = node.distance(conn);
        println!("  Kept: ID={} coords={:?} dist={:.3}", id, conn.coords, dist);
    }

    println!("\nPruned:");
    for conn in &connections {
        if !kept.contains(&conn.id) {
            let dist = node.distance(conn);
            println!(
                "  Pruned: ID={} coords={:?} dist={:.3}",
                conn.id, conn.coords, dist
            );
        }
    }

    println!("\n\nKey Insight:");
    println!("  Pruning removes redundant close connections (e.g., ID=3)");
    println!("  Keeps connections spanning different directions (N/S/E/W)");
    println!("  Maintains navigability despite reducing edge count");
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║    Neighbor Selection Heuristics: Deep Dive           ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    demo_clustered_candidates();
    demo_angular_vs_distance();
    demo_extreme_m_values();
    demo_pruning_scenario();

    println!("\n\n╔═══════════════════════════════════════════════════════╗");
    println!("║                 Summary & Best Practices              ║");
    println!("╚═══════════════════════════════════════════════════════╝");

    println!("\n1. Why Diversity Matters:");
    println!("   Prevents graph degeneration (clustering)");
    println!("   Maintains O(log N) navigability");
    println!("   Enables long-range jumps in hierarchical layers");

    println!("\n2. The HNSW Heuristic:");
    println!("   Distance-based (not angular)");
    println!("   Prunes redundant close connections");
    println!("   Keeps connections spanning different regions");

    println!("\n3. Parameter Choice:");
    println!("   M=16 is typical (good balance)");
    println!("   M0=32 for Layer 0 (2xM, denser connections)");
    println!("   Smaller M = more selective, higher diversity");
    println!("   Larger M = more coverage, less diversity pressure");

    println!("\n4. Complexity:");
    println!("   Heuristic: O(M squared x D) per node");
    println!("   Acceptable because M is small (typically 4-64)");
    println!("   Critical for graph quality (worth the cost)");

    println!();
}
