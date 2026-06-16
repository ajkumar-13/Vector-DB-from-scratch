// enums-examples.rs
//
// Runnable examples demonstrating Rust enums, match, and Option.
// From Post #4: Structs, Enums, and Error Handling
//
// Run with: rustc enums-examples.rs && ./enums-examples

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  RUST ENUMS - ALGEBRAIC DATA TYPES");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 1: Simple Enum (C-style)
    // ─────────────────────────────────────────────────────────────────
    println!("1. SIMPLE ENUM");
    println!("─────────────────────────────────────────────────────────────");

    let metric = DistanceMetric::Cosine;
    println!("   Selected metric: {:?}", metric);

    // Match on enum
    let name = match metric {
        DistanceMetric::Cosine => "Cosine Similarity",
        DistanceMetric::Euclidean => "Euclidean Distance",
        DistanceMetric::Dot => "Dot Product",
    };
    println!("   Metric name: {}", name);
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 2: Enum with Data (Algebraic Data Type)
    // ─────────────────────────────────────────────────────────────────
    println!("2. ENUM WITH DATA");
    println!("─────────────────────────────────────────────────────────────");

    let metrics = vec![
        AdvancedMetric::Cosine,
        AdvancedMetric::Minkowski(2.0), // L2 norm
        AdvancedMetric::Minkowski(1.0), // Manhattan distance
        AdvancedMetric::Weighted(vec![1.0, 2.0, 3.0]),
    ];

    for m in &metrics {
        println!("   {:?}", m);
    }
    println!("   → Variants can hold different types of data!");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 3: Exhaustive Match
    // ─────────────────────────────────────────────────────────────────
    println!("3. EXHAUSTIVE MATCH (Compiler Enforced)");
    println!("─────────────────────────────────────────────────────────────");

    let a: Vec<f32> = vec![1.0, 0.0];
    let b: Vec<f32> = vec![0.0, 1.0];

    for metric in &metrics {
        let result = calculate_distance(metric, &a, &b);
        println!("   {:?} => {:.4}", metric, result);
    }
    println!("   → If you add a new variant, compiler forces you to handle it!");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 4: Option<T> - The Null Killer
    // ─────────────────────────────────────────────────────────────────
    println!("4. OPTION<T> - SAFE NULL REPLACEMENT");
    println!("─────────────────────────────────────────────────────────────");

    let vectors = create_sample_vectors();

    // Try to find existing and non-existing vectors
    let ids = vec!["vec_001", "vec_002", "missing"];

    for id in ids {
        match find_vector(&vectors, id) {
            Some(v) => println!("   Found '{}': {} dimensions", id, v.dimension),
            None => println!("   '{}' not found!", id),
        }
    }
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 5: Option Methods
    // ─────────────────────────────────────────────────────────────────
    println!("5. OPTION CONVENIENCE METHODS");
    println!("─────────────────────────────────────────────────────────────");

    let found = find_vector(&vectors, "vec_001");
    let missing = find_vector(&vectors, "missing");

    // is_some() / is_none()
    println!("   found.is_some(): {}", found.is_some());
    println!("   missing.is_none(): {}", missing.is_none());

    // unwrap_or() - provide default
    let dimension = missing.map(|v| v.dimension).unwrap_or(0);
    println!("   missing dimension (default 0): {}", dimension);

    // if let - single arm match
    if let Some(v) = found {
        println!("   if let: found vector with {} dimensions", v.dimension);
    }
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 6: Enum for State Machines
    // ─────────────────────────────────────────────────────────────────
    println!("6. ENUMS FOR STATE MACHINES");
    println!("─────────────────────────────────────────────────────────────");

    let mut connection = ConnectionState::Disconnected;
    println!("   Initial: {:?}", connection);

    connection = ConnectionState::Connecting {
        host: "localhost".to_string(),
        port: 6333,
    };
    println!("   After connect(): {:?}", connection);

    connection = ConnectionState::Connected { session_id: 12345 };
    println!("   After handshake(): {:?}", connection);

    // Use match to extract data
    if let ConnectionState::Connected { session_id } = connection {
        println!("   Session ID: {}", session_id);
    }
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 7: Tagged Union Memory Layout
    // ─────────────────────────────────────────────────────────────────
    println!("7. TAGGED UNION MEMORY");
    println!("─────────────────────────────────────────────────────────────");

    println!(
        "   Size of DistanceMetric: {} bytes",
        std::mem::size_of::<DistanceMetric>()
    );
    println!(
        "   Size of AdvancedMetric: {} bytes",
        std::mem::size_of::<AdvancedMetric>()
    );
    println!(
        "   Size of ConnectionState: {} bytes",
        std::mem::size_of::<ConnectionState>()
    );
    println!(
        "   Size of Option<u8>: {} bytes",
        std::mem::size_of::<Option<u8>>()
    );
    println!(
        "   Size of Option<&str>: {} bytes",
        std::mem::size_of::<Option<&str>>()
    );
    println!("   → Rust optimizes Option<&T> to same size as &T!");
    println!();

    println!("═══════════════════════════════════════════════════════════");
    println!("  ENUM SUMMARY:");
    println!("  • Enums represent mutually exclusive states");
    println!("  • Variants can hold different data types");
    println!("  • match forces exhaustive handling");
    println!("  • Option<T> replaces null (Some/None)");
    println!("  • Stored as tagged unions (tag + largest variant)");
    println!("═══════════════════════════════════════════════════════════");
}

// ═══════════════════════════════════════════════════════════════════════════
// ENUM DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Simple distance metric enum
#[derive(Debug, Clone, Copy)]
enum DistanceMetric {
    Cosine,
    Euclidean,
    Dot,
}

/// Advanced metric with embedded data
#[derive(Debug, Clone)]
enum AdvancedMetric {
    Cosine,
    Euclidean,
    Minkowski(f32),     // p parameter
    Weighted(Vec<f32>), // weight per dimension
}

/// Connection state machine
#[derive(Debug)]
enum ConnectionState {
    Disconnected,
    Connecting { host: String, port: u16 },
    Connected { session_id: u64 },
    Error(String),
}

// ═══════════════════════════════════════════════════════════════════════════
// HELPER STRUCTS AND FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
struct Vector {
    id: String,
    data: Vec<f32>,
    dimension: usize,
}

fn create_sample_vectors() -> Vec<Vector> {
    vec![
        Vector {
            id: "vec_001".to_string(),
            data: vec![0.1, 0.2, 0.3],
            dimension: 3,
        },
        Vector {
            id: "vec_002".to_string(),
            data: vec![0.4, 0.5, 0.6],
            dimension: 3,
        },
    ]
}

fn find_vector<'a>(vectors: &'a [Vector], id: &str) -> Option<&'a Vector> {
    vectors.iter().find(|v| v.id == id)
}

fn calculate_distance(metric: &AdvancedMetric, a: &[f32], b: &[f32]) -> f32 {
    match metric {
        AdvancedMetric::Cosine => {
            let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
            let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
            dot / (norm_a * norm_b)
        }
        AdvancedMetric::Euclidean => a
            .iter()
            .zip(b)
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt(),
        AdvancedMetric::Minkowski(p) => a
            .iter()
            .zip(b)
            .map(|(x, y)| (x - y).abs().powf(*p))
            .sum::<f32>()
            .powf(1.0 / p),
        AdvancedMetric::Weighted(weights) => a
            .iter()
            .zip(b)
            .zip(weights.iter())
            .map(|((x, y), w)| w * (x - y).powi(2))
            .sum::<f32>()
            .sqrt(),
    }
}
