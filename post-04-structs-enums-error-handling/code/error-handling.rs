// error-handling.rs
//
// Runnable examples demonstrating Result<T, E> and the ? operator.
// From Post #4: Structs, Enums, and Error Handling
//
// Run with: rustc error-handling.rs && ./error-handling

use std::fs::File;
use std::io::{self, Read, Write};

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  RUST ERROR HANDLING - RESULT<T, E>");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 1: Basic Result Handling
    // ─────────────────────────────────────────────────────────────────
    println!("1. BASIC RESULT HANDLING");
    println!("─────────────────────────────────────────────────────────────");
    
    // File::open returns Result<File, io::Error>
    let result = File::open("nonexistent.txt");
    
    match result {
        Ok(file) => println!("   File opened: {:?}", file),
        Err(e) => println!("   Error (expected): {}", e),
    }
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 2: Custom Error Types
    // ─────────────────────────────────────────────────────────────────
    println!("2. CUSTOM ERROR TYPES");
    println!("─────────────────────────────────────────────────────────────");
    
    // Test with various inputs
    let test_cases = vec![
        (vec![], vec![1.0, 2.0]),           // Empty vector
        (vec![1.0, 2.0], vec![1.0, 2.0, 3.0]), // Dimension mismatch
        (vec![1.0, 2.0], vec![3.0, 4.0]),   // Valid
    ];
    
    for (a, b) in test_cases {
        match dot_product(&a, &b) {
            Ok(result) => println!("   {:?} · {:?} = {:.2}", a, b, result),
            Err(e) => println!("   Error: {:?}", e),
        }
    }
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 3: The ? Operator
    // ─────────────────────────────────────────────────────────────────
    println!("3. THE ? OPERATOR (ERROR PROPAGATION)");
    println!("─────────────────────────────────────────────────────────────");
    
    // This function uses ? internally
    match read_config_file() {
        Ok(contents) => println!("   Config: {}", contents),
        Err(e) => println!("   Failed to read config: {}", e),
    }
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 4: Chaining with ?
    // ─────────────────────────────────────────────────────────────────
    println!("4. CHAINING OPERATIONS WITH ?");
    println!("─────────────────────────────────────────────────────────────");
    
    // Create a test file first
    create_test_file().ok();
    
    match process_vector_file("test_vectors.txt") {
        Ok(sum) => println!("   Sum of vectors: {:.2}", sum),
        Err(e) => println!("   Processing failed: {:?}", e),
    }
    
    // Clean up
    std::fs::remove_file("test_vectors.txt").ok();
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 5: unwrap(), expect(), and When to Use Them
    // ─────────────────────────────────────────────────────────────────
    println!("5. UNWRAP VARIANTS");
    println!("─────────────────────────────────────────────────────────────");
    
    // unwrap_or - provide default on error
    let result: Result<i32, &str> = Err("failed");
    let value = result.unwrap_or(0);
    println!("   unwrap_or: Err(\"failed\").unwrap_or(0) = {}", value);
    
    // unwrap_or_else - compute default lazily
    let result: Result<i32, &str> = Err("failed");
    let value = result.unwrap_or_else(|e| {
        println!("   unwrap_or_else: Error was '{}', returning default", e);
        -1
    });
    println!("   Result: {}", value);
    
    // ok() - convert Result to Option (discards error)
    let result: Result<i32, &str> = Ok(42);
    let option = result.ok();
    println!("   ok(): Ok(42).ok() = {:?}", option);
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 6: Converting Between Error Types
    // ─────────────────────────────────────────────────────────────────
    println!("6. ERROR CONVERSION (map_err)");
    println!("─────────────────────────────────────────────────────────────");
    
    match read_vector_count("nonexistent.txt") {
        Ok(count) => println!("   Vector count: {}", count),
        Err(e) => println!("   VectorDbError: {:?}", e),
    }
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 7: Early Return Pattern
    // ─────────────────────────────────────────────────────────────────
    println!("7. EARLY RETURN PATTERN");
    println!("─────────────────────────────────────────────────────────────");
    
    let queries = vec![
        (vec![1.0, 2.0, 3.0], 5),
        (vec![], 5),  // Will fail - empty vector
    ];
    
    for (query, k) in queries {
        match search(&query, k) {
            Ok(results) => println!("   Found {} results for {:?}", results.len(), query),
            Err(e) => println!("   Search failed: {:?}", e),
        }
    }
    println!();

    println!("═══════════════════════════════════════════════════════════");
    println!("  ERROR HANDLING SUMMARY:");
    println!("  • Result<T, E> makes errors explicit in the type");
    println!("  • ? propagates errors up the call stack");
    println!("  • match for explicit handling");
    println!("  • unwrap_or / unwrap_or_else for defaults");
    println!("  • map_err to convert error types");
    println!("  • Use custom error enums for domain errors");
    println!("═══════════════════════════════════════════════════════════");
}

// ═══════════════════════════════════════════════════════════════════════════
// CUSTOM ERROR TYPE
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
enum VectorDbError {
    EmptyVector,
    DimensionMismatch { expected: usize, got: usize },
    NotFound(String),
    IoError(io::Error),
    ParseError(String),
}

// ═══════════════════════════════════════════════════════════════════════════
// FUNCTIONS DEMONSTRATING ERROR HANDLING
// ═══════════════════════════════════════════════════════════════════════════

/// Calculate dot product with proper error handling
fn dot_product(a: &[f32], b: &[f32]) -> Result<f32, VectorDbError> {
    if a.is_empty() || b.is_empty() {
        return Err(VectorDbError::EmptyVector);
    }
    
    if a.len() != b.len() {
        return Err(VectorDbError::DimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }
    
    let result = a.iter().zip(b).map(|(x, y)| x * y).sum();
    Ok(result)
}

/// Demonstrate ? operator with file reading
fn read_config_file() -> Result<String, io::Error> {
    // Each ? returns early if Err, otherwise unwraps Ok
    let mut file = File::open("config.toml")?;  // Returns Err if fails
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;        // Returns Err if fails
    Ok(contents)
}

/// Create a test file for demonstration
fn create_test_file() -> io::Result<()> {
    let mut file = File::create("test_vectors.txt")?;
    writeln!(file, "1.0 2.0 3.0")?;
    writeln!(file, "4.0 5.0 6.0")?;
    Ok(())
}

/// Process vectors from a file - demonstrates chained ? operators
fn process_vector_file(path: &str) -> Result<f32, VectorDbError> {
    // Convert io::Error to our error type with map_err
    let contents = std::fs::read_to_string(path)
        .map_err(VectorDbError::IoError)?;
    
    let mut sum: f32 = 0.0;
    
    for line in contents.lines() {
        for value in line.split_whitespace() {
            let num: f32 = value.parse()
                .map_err(|_| VectorDbError::ParseError(value.to_string()))?;
            sum += num;
        }
    }
    
    Ok(sum)
}

/// Demonstrate error conversion
fn read_vector_count(path: &str) -> Result<usize, VectorDbError> {
    let contents = std::fs::read_to_string(path)
        .map_err(VectorDbError::IoError)?;  // Convert io::Error → VectorDbError
    
    Ok(contents.lines().count())
}

/// Simulate a search with early return on error
fn search(query: &[f32], top_k: usize) -> Result<Vec<String>, VectorDbError> {
    // Early return if query is invalid
    if query.is_empty() {
        return Err(VectorDbError::EmptyVector);
    }
    
    if top_k == 0 {
        return Err(VectorDbError::ParseError("top_k must be > 0".to_string()));
    }
    
    // Simulate successful results
    Ok(vec!["result_1".to_string(), "result_2".to_string()])
}
