// structs-examples.rs
//
// Runnable examples demonstrating Rust structs.
// From Post #4: Structs, Enums, and Error Handling
//
// Run with: rustc structs-examples.rs && ./structs-examples

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  RUST STRUCTS - DATA MODELING");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 1: Named Struct
    // ─────────────────────────────────────────────────────────────────
    println!("1. NAMED STRUCT");
    println!("─────────────────────────────────────────────────────────────");

    // Basic struct instantiation
    let v = Vector {
        id: String::from("vec_001"),
        data: vec![0.1, 0.2, 0.3],
        dimension: 3,
    };

    println!("   Created: {:?}", v);
    println!("   ID: {}", v.id);
    println!("   Dimension: {}", v.dimension);
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 2: Struct with impl Block (Methods)
    // ─────────────────────────────────────────────────────────────────
    println!("2. STRUCT WITH METHODS (impl)");
    println!("─────────────────────────────────────────────────────────────");

    // Using the "constructor" pattern
    let v = Vector::new("vec_002".to_string(), vec![3.0, 4.0]);

    println!("   Created via ::new(): {:?}", v);
    println!("   Magnitude: {}", v.magnitude()); // 5.0 (3-4-5 triangle)
    println!();

    // Mutable method
    let mut v = Vector::new("vec_003".to_string(), vec![3.0, 4.0]);
    println!("   Before normalize: {:?}", v.data);
    v.normalize();
    println!("   After normalize:  {:?}", v.data);
    println!("   New magnitude: {}", v.magnitude()); // ~1.0
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 3: Tuple Structs (Newtype Pattern)
    // ─────────────────────────────────────────────────────────────────
    println!("3. TUPLE STRUCTS (NEWTYPE PATTERN)");
    println!("─────────────────────────────────────────────────────────────");

    let id = VectorId(42);
    let dim = Dimension(768);

    println!("   VectorId: {:?}", id);
    println!("   Dimension: {:?}", dim);
    println!("   Inner values: id.0 = {}, dim.0 = {}", id.0, dim.0);

    // Type safety! These are incompatible:
    // let x = id.0 + dim.0;  // Works (both are usize)
    // But you can't accidentally pass VectorId where Dimension is expected!
    println!("   → Different types prevent accidental misuse!");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 4: Unit Struct
    // ─────────────────────────────────────────────────────────────────
    println!("4. UNIT STRUCT");
    println!("─────────────────────────────────────────────────────────────");

    let _marker = Marker;
    println!("   Unit structs have no fields");
    println!("   Used for: type markers, trait implementations, zero-size types");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 5: Struct Update Syntax
    // ─────────────────────────────────────────────────────────────────
    println!("5. STRUCT UPDATE SYNTAX");
    println!("─────────────────────────────────────────────────────────────");

    let v1 = Vector::new("original".to_string(), vec![1.0, 2.0, 3.0]);

    // Create a new struct, copying some fields from v1
    let v2 = Vector {
        id: String::from("copy"),
        ..v1.clone() // Use remaining fields from v1
    };

    println!("   v1: {:?}", v1);
    println!("   v2: {:?}", v2);
    println!("   → ..v1 copies data and dimension from v1");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 6: Self Types
    // ─────────────────────────────────────────────────────────────────
    println!("6. SELF PARAMETER TYPES");
    println!("─────────────────────────────────────────────────────────────");
    println!("   &self      → immutable borrow (read-only)");
    println!("   &mut self  → mutable borrow (can modify)");
    println!("   self       → takes ownership (consumes)");
    println!();

    let v = Vector::new("consumable".to_string(), vec![1.0, 2.0]);
    println!("   Before into_data(): {:?}", v);

    let data = v.into_data(); // v is moved/consumed
    println!("   After into_data(): {:?}", data);
    // println!("{:?}", v);  // ERROR: v was moved!
    println!("   → v is no longer valid (ownership transferred)");
    println!();

    println!("═══════════════════════════════════════════════════════════");
    println!("  STRUCT SUMMARY:");
    println!("  • Named structs: struct Foo {{ field: Type }}");
    println!("  • Tuple structs: struct Foo(Type) — for newtype pattern");
    println!("  • Unit structs: struct Foo — zero-size markers");
    println!("  • impl blocks add methods and associated functions");
    println!("═══════════════════════════════════════════════════════════");
}

// ═══════════════════════════════════════════════════════════════════════════
// STRUCT DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════

/// A vector embedding - the core data type in our database
#[derive(Debug, Clone)]
struct Vector {
    id: String,
    data: Vec<f32>,
    dimension: usize,
}

impl Vector {
    /// Constructor pattern - creates a new Vector
    fn new(id: String, data: Vec<f32>) -> Self {
        let dimension = data.len();
        Self {
            id,
            data,
            dimension,
        }
    }

    /// Calculate the magnitude (L2 norm)
    /// Takes &self - immutable borrow
    fn magnitude(&self) -> f32 {
        self.data.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// Normalize the vector in-place
    /// Takes &mut self - mutable borrow
    fn normalize(&mut self) {
        let mag = self.magnitude();
        if mag > 0.0 {
            for x in &mut self.data {
                *x /= mag;
            }
        }
    }

    /// Consume self and return the data
    /// Takes self - ownership transfer
    fn into_data(self) -> Vec<f32> {
        self.data // self is consumed, caller gets the Vec
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TUPLE STRUCTS (Newtype Pattern)
// ═══════════════════════════════════════════════════════════════════════════

/// Wraps a usize to represent a vector ID
#[derive(Debug, Clone, Copy)]
struct VectorId(usize);

/// Wraps a usize to represent dimensionality
#[derive(Debug, Clone, Copy)]
struct Dimension(usize);

// ═══════════════════════════════════════════════════════════════════════════
// UNIT STRUCT
// ═══════════════════════════════════════════════════════════════════════════

/// A zero-size marker type
struct Marker;
