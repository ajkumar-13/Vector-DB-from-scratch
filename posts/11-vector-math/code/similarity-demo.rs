// post-11-vector-math/code/similarity-demo.rs
// Interactive demonstration comparing different similarity metrics
//
// Run with: rustc similarity-demo.rs && ./similarity-demo

use std::collections::HashMap;

// ============================================================================
// Core Math Functions (duplicated for standalone demo)
// ============================================================================

fn magnitude(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot = dot_product(a, b);
    let mag_a = magnitude(a);
    let mag_b = magnitude(b);
    if mag_a == 0.0 || mag_b == 0.0 {
        0.0
    } else {
        dot / (mag_a * mag_b)
    }
}

fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

fn normalize(v: &[f32]) -> Vec<f32> {
    let mag = magnitude(v);
    if mag == 0.0 {
        vec![0.0; v.len()]
    } else {
        v.iter().map(|x| x / mag).collect()
    }
}

fn manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
}

// ============================================================================
// Semantic Search Simulation
// ============================================================================

/// A simulated document with a fake embedding
struct Document {
    id: usize,
    title: String,
    embedding: Vec<f32>,
}

impl Document {
    fn new(id: usize, title: &str, embedding: Vec<f32>) -> Self {
        Self {
            id,
            title: title.to_string(),
            embedding,
        }
    }
}

/// Search results with scores from different metrics
struct SearchResult {
    doc_id: usize,
    title: String,
    cosine_score: f32,
    euclidean_dist: f32,
    dot_score: f32,
}

fn search_all_metrics(query: &[f32], docs: &[Document]) -> Vec<SearchResult> {
    docs.iter()
        .map(|doc| SearchResult {
            doc_id: doc.id,
            title: doc.title.clone(),
            cosine_score: cosine_similarity(query, &doc.embedding),
            euclidean_dist: euclidean_distance(query, &doc.embedding),
            dot_score: dot_product(query, &doc.embedding),
        })
        .collect()
}

// ============================================================================
// Demo: Word Embeddings Simulation
// ============================================================================

fn demo_word_embeddings() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         Demo 1: Simulated Word Embeddings                    ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Simulated word embeddings (3D for visualization)
    // In reality, embeddings are 768-1536 dimensions
    let words: HashMap<&str, Vec<f32>> = [
        // Royalty cluster
        ("king", vec![0.9, 0.1, 0.8]),
        ("queen", vec![0.85, 0.2, 0.75]),
        ("prince", vec![0.8, 0.1, 0.7]),
        ("royal", vec![0.75, 0.15, 0.65]),
        // Animals cluster
        ("dog", vec![0.1, 0.9, 0.3]),
        ("cat", vec![0.15, 0.85, 0.35]),
        ("puppy", vec![0.1, 0.88, 0.32]),
        // Vehicles cluster
        ("car", vec![-0.5, 0.1, 0.9]),
        ("truck", vec![-0.45, 0.15, 0.85]),
        ("vehicle", vec![-0.4, 0.12, 0.8]),
    ]
    .into_iter()
    .collect();

    let target = "king";
    let query = &words[target];

    println!("Query word: '{}' = {:?}\n", target, query);
    println!("Similarity to all words:\n");
    println!(
        "{:<10} {:>12} {:>12} {:>12}",
        "Word", "Cosine", "Euclidean", "Dot"
    );
    println!("{}", "-".repeat(50));

    let mut results: Vec<_> = words
        .iter()
        .map(|(word, emb)| {
            (
                *word,
                cosine_similarity(query, emb),
                euclidean_distance(query, emb),
                dot_product(query, emb),
            )
        })
        .collect();

    // Sort by cosine similarity (descending)
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (word, cos, euc, dot) in &results {
        let marker = if *word == target { " (query)" } else { "" };
        println!(
            "{:<10} {:>12.4} {:>12.4} {:>12.4}{}",
            word, cos, euc, dot, marker
        );
    }

    println!("\nNote: Similar concepts cluster together across all metrics");
}

// ============================================================================
// Demo: Metric Disagreement
// ============================================================================

fn demo_metric_disagreement() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         Demo 2: When Metrics Disagree                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Scenario: Magnitude differences
    let query = vec![1.0, 0.0]; // Unit vector pointing right
    let a = vec![10.0, 0.0]; // Same direction, large magnitude
    let b = vec![0.7, 0.7]; // Different direction, similar magnitude

    println!(
        "Query:    {:?} (magnitude: {:.2})",
        query,
        magnitude(&query)
    );
    println!("Vector A: {:?} (magnitude: {:.2})", a, magnitude(&a));
    println!("Vector B: {:?} (magnitude: {:.2})", b, magnitude(&b));

    println!("\nResults:");
    println!("─────────────────────────────────────────");
    println!("{:<20} {:>10} {:>10}", "Metric", "to A", "to B");
    println!("─────────────────────────────────────────");

    let cos_a = cosine_similarity(&query, &a);
    let cos_b = cosine_similarity(&query, &b);
    let winner_cos = if cos_a > cos_b { "A" } else { "B" };
    println!(
        "{:<20} {:>10.4} {:>10.4} → {} wins",
        "Cosine Similarity", cos_a, cos_b, winner_cos
    );

    let euc_a = euclidean_distance(&query, &a);
    let euc_b = euclidean_distance(&query, &b);
    let winner_euc = if euc_a < euc_b { "A" } else { "B" };
    println!(
        "{:<20} {:>10.4} {:>10.4} → {} wins",
        "Euclidean Distance", euc_a, euc_b, winner_euc
    );

    let dot_a = dot_product(&query, &a);
    let dot_b = dot_product(&query, &b);
    let winner_dot = if dot_a > dot_b { "A" } else { "B" };
    println!(
        "{:<20} {:>10.4} {:>10.4} → {} wins",
        "Dot Product", dot_a, dot_b, winner_dot
    );

    println!("\nKey insight:");
    println!("   Cosine: Ignores magnitude, focuses on direction. A wins (same direction)");
    println!("   Euclidean: Considers absolute position. B wins (closer in space)");
    println!("   Dot Product: Combines both. A wins (magnitude x alignment)");
}

// ============================================================================
// Demo: Semantic Document Search
// ============================================================================

fn demo_document_search() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         Demo 3: Semantic Document Search                     ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Simulated document embeddings (5D)
    let documents = vec![
        Document::new(
            1,
            "Introduction to Machine Learning",
            vec![0.8, 0.7, 0.2, 0.1, 0.3],
        ),
        Document::new(
            2,
            "Deep Learning Neural Networks",
            vec![0.85, 0.75, 0.3, 0.15, 0.35],
        ),
        Document::new(
            3,
            "Cooking Italian Pasta Recipes",
            vec![0.1, 0.2, 0.9, 0.8, 0.1],
        ),
        Document::new(
            4,
            "Mediterranean Diet Health",
            vec![0.15, 0.25, 0.85, 0.75, 0.15],
        ),
        Document::new(5, "Stock Market Analysis", vec![-0.3, 0.1, 0.1, 0.2, 0.9]),
        Document::new(6, "AI in Financial Trading", vec![0.5, 0.4, 0.1, 0.2, 0.7]),
    ];

    // Query: "neural network tutorial"
    let query = vec![0.82, 0.72, 0.25, 0.12, 0.32];

    println!("Query: \"neural network tutorial\"\n");

    let results = search_all_metrics(&query, &documents);

    // Rank by cosine similarity
    let mut by_cosine = results.clone();
    by_cosine.sort_by(|a, b| b.cosine_score.partial_cmp(&a.cosine_score).unwrap());

    println!("Ranked by Cosine Similarity:");
    println!("─────────────────────────────────────────────────────");
    for (rank, r) in by_cosine.iter().enumerate() {
        println!("{:>2}. [{:.4}] {}", rank + 1, r.cosine_score, r.title);
    }

    // Rank by Euclidean distance
    let mut by_euclidean = results.clone();
    by_euclidean.sort_by(|a, b| a.euclidean_dist.partial_cmp(&b.euclidean_dist).unwrap());

    println!("\nRanked by Euclidean Distance (lower = better):");
    println!("─────────────────────────────────────────────────────");
    for (rank, r) in by_euclidean.iter().enumerate() {
        println!("{:>2}. [{:.4}] {}", rank + 1, r.euclidean_dist, r.title);
    }

    println!("\nBoth metrics agree on top results for well-clustered data");
}

// ============================================================================
// Demo: Normalization Effect
// ============================================================================

fn demo_normalization() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         Demo 4: Normalization Effect                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let a = vec![3.0, 4.0, 0.0];
    let b = vec![6.0, 8.0, 0.0]; // Same direction as a, 2x magnitude
    let c = vec![4.0, 3.0, 0.0]; // Different direction, same plane

    println!("Original vectors:");
    println!("  a = {:?} (mag: {:.2})", a, magnitude(&a));
    println!("  b = {:?} (mag: {:.2})", b, magnitude(&b));
    println!("  c = {:?} (mag: {:.2})", c, magnitude(&c));

    println!("\nMetrics (unnormalized):");
    println!("  Euclidean(a, b): {:.4}", euclidean_distance(&a, &b));
    println!("  Euclidean(a, c): {:.4}", euclidean_distance(&a, &c));
    println!("  Cosine(a, b):    {:.4}", cosine_similarity(&a, &b));
    println!("  Cosine(a, c):    {:.4}", cosine_similarity(&a, &c));

    // Normalize all vectors
    let a_norm = normalize(&a);
    let b_norm = normalize(&b);
    let c_norm = normalize(&c);

    println!("\nNormalized vectors:");
    println!("  a = {:?}", a_norm);
    println!("  b = {:?}", b_norm);
    println!("  c = {:?}", c_norm);

    println!("\nMetrics (normalized):");
    println!(
        "  Euclidean(a, b): {:.4} (distance reflects angle only)",
        euclidean_distance(&a_norm, &b_norm)
    );
    println!(
        "  Euclidean(a, c): {:.4}",
        euclidean_distance(&a_norm, &c_norm)
    );
    println!(
        "  Dot product = Cosine: {:.4} (no sqrt needed)",
        dot_product(&a_norm, &b_norm)
    );

    println!("\nAfter normalization:");
    println!("   a and b become IDENTICAL (same direction)");
    println!("   Euclidean distance between unit vectors is proportional to angular difference");
    println!("   Cosine similarity = simple dot product");
}

// ============================================================================
// Demo: High Dimensional Effects
// ============================================================================

fn demo_high_dimensions() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         Demo 5: Curse of Dimensionality                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let dimensions = [2, 10, 100, 1000];

    println!(
        "{:>8} {:>15} {:>15} {:>15}",
        "Dims", "Avg Distance", "Std Dev", "Ratio"
    );
    println!("{}", "-".repeat(55));

    for &dim in &dimensions {
        // Generate "random" vectors (using deterministic pattern for reproducibility)
        let vectors: Vec<Vec<f32>> = (0..100)
            .map(|i| (0..dim).map(|j| ((i * j) as f32 * 0.001).sin()).collect())
            .collect();

        // Compute all pairwise distances
        let mut distances = Vec::new();
        for i in 0..vectors.len() {
            for j in (i + 1)..vectors.len() {
                distances.push(euclidean_distance(&vectors[i], &vectors[j]));
            }
        }

        let avg: f32 = distances.iter().sum::<f32>() / distances.len() as f32;
        let variance: f32 =
            distances.iter().map(|d| (d - avg).powi(2)).sum::<f32>() / distances.len() as f32;
        let std_dev = variance.sqrt();
        let ratio = std_dev / avg;

        println!("{:>8} {:>15.4} {:>15.4} {:>15.4}", dim, avg, std_dev, ratio);
    }

    println!("\nObservation:");
    println!("   As dimensions increase, distances concentrate around the mean");
    println!("   (Ratio decreases, meaning all points become roughly equidistant)");
    println!("   This is why cosine similarity often outperforms Euclidean in high dims");
}

// ============================================================================
// Demo: Practical Similarity Threshold
// ============================================================================

fn demo_thresholds() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║         Demo 6: Choosing Similarity Thresholds               ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Simulate embeddings with known semantic relationships
    let pairs = [
        ("exact duplicate", 1.0f32),
        ("paraphrase", 0.92),
        ("related topic", 0.75),
        ("loosely related", 0.55),
        ("different topics", 0.30),
        ("opposite meaning", -0.15),
    ];

    println!("Typical cosine similarity ranges:\n");
    println!(
        "{:<25} {:>10} {:>15}",
        "Relationship", "Score", "Interpretation"
    );
    println!("{}", "-".repeat(55));

    for (relationship, score) in pairs {
        let interpretation = match score {
            s if s >= 0.95 => "Duplicate",
            s if s >= 0.85 => "Very similar",
            s if s >= 0.70 => "Similar",
            s if s >= 0.50 => "Somewhat related",
            s if s >= 0.20 => "Different",
            _ => "Unrelated/Opposite",
        };
        println!(
            "{:<25} {:>10.2} {:>15}",
            relationship, score, interpretation
        );
    }

    println!("\nCommon thresholds:");
    println!("   Deduplication:    > 0.95");
    println!("   Semantic search:  > 0.70");
    println!("   Topic clustering: > 0.50");
}

// ============================================================================
// Main
// ============================================================================

impl Clone for SearchResult {
    fn clone(&self) -> Self {
        Self {
            doc_id: self.doc_id,
            title: self.title.clone(),
            cosine_score: self.cosine_score,
            euclidean_dist: self.euclidean_dist,
            dot_score: self.dot_score,
        }
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         Vector Similarity Metrics - Deep Dive                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    demo_word_embeddings();
    demo_metric_disagreement();
    demo_document_search();
    demo_normalization();
    demo_high_dimensions();
    demo_thresholds();

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("All demos complete!");
    println!("\nKey takeaways:");
    println!("  1. Cosine similarity ignores magnitude, making it best for semantic meaning");
    println!("  2. Euclidean distance works well for normalized vectors");
    println!("  3. Pre-normalization gives approximately 2x speedup (no sqrt in comparison)");
    println!("  4. High dimensions make Euclidean less discriminative");
    println!("  5. Choose thresholds based on your use case");
}
