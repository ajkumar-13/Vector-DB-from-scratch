# Rust Crash Course Part 2: Structs, Enums, and Error Handling

**Series:** Building a Vector Database from Scratch in Rust  
**Post:** 4 of 20  
**Reading Time:** ~12 minutes

---

## 1. Introduction: Modeling the World

In the previous post, we mastered memory. We know how to own it, borrow it, and move it. But so far, we've only dealt with primitives like `f32` and `String`.

A database isn't built of raw floats. It's built of **Structures**.

We need to represent concepts like:

* A `Vector` (which has dimensions and data).
* A `DistanceMetric` (Cosine, Dot Product, or Euclidean).
* A `QueryResult` (which might succeed or fail).

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-1-from-primitives-to-structures -->

In many languages (like Java or Python), you'd use a **Class** for everything. In Rust, we split this into two powerful concepts: **Structs** (data layout) and **Enums** (state possibilities).

And most importantly, we will learn how to handle errors. In C, you check integer return codes (`-1`). In Java, you catch Exceptions. In Rust, you handle **Results**.

---

## 2. Structs: Shaping Your Data

A `struct` (structure) allows you to name and bundle related data together. If you know C, this looks familiar. If you know Python, think of it as a `class` with data but no methods attached (yet).

<!-- See code/structs-examples.rs for runnable examples -->

### 2.1 The Named Struct

Let's define the core atom of our database: the `Vector`.

```rust
// #[derive(Debug)] allows us to print the struct using {:?}
#[derive(Debug)]
struct Vector {
    id: String,
    data: Vec<f32>,
    dimension: usize,
}
```

We instantiate it like this:

```rust
let v = Vector {
    id: String::from("vec_001"),
    data: vec![0.1, 0.2, 0.3],
    dimension: 3,
};

println!("Vector ID: {}", v.id);
```

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-2-struct-memory-layout -->

### 2.2 Adding Behavior (`impl`)

Rust doesn't have classes, but it has **Methods**. We define behavior in an `impl` (implementation) block.

```rust
impl Vector {
    // This is a "Constructor" (convention, not syntax)
    // It is an associated function (no self)
    fn new(id: String, data: Vec<f32>) -> Self {
        let dim = data.len();
        Self {
            id,
            data,
            dimension: dim,
        }
    }

    // This is a Method (takes &self)
    fn magnitude(&self) -> f32 {
        self.data.iter().map(|x| x * x).sum::<f32>().sqrt()
    }
    
    // Mutable method (takes &mut self)
    fn normalize(&mut self) {
        let mag = self.magnitude();
        if mag > 0.0 {
            for x in &mut self.data {
                *x /= mag;
            }
        }
    }
}

// Usage:
let v = Vector::new("vec_001".into(), vec![0.1, 0.2]);
println!("Magnitude: {}", v.magnitude());
```

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-3-self-reference-types -->

**Systems Note:** The `self` parameter determines borrowing:
- `&self` — immutable borrow (read-only access)
- `&mut self` — mutable borrow (can modify fields)
- `self` — takes ownership (rare, used for transformations)

### 2.3 The Tuple Struct (The "Newtype" Pattern)

Sometimes you just want to wrap a primitive to give it a distinct type. This prevents logic errors (like accidentally adding `Temperature` to `Money`).

```rust
struct VectorId(usize); // Wrapper around a generic integer
struct Dimension(usize);

let id = VectorId(42);
let dim = Dimension(768);

// This would be a compile error - different types!
// let x = id + dim; // ERROR!

// Access the inner value with .0
println!("ID is {}", id.0);
```

---

## 3. Enums: Rust's Superpower

In C or Java, an `enum` is usually just a set of named integers (`RED = 0`, `BLUE = 1`).

In Rust, Enums are **Algebraic Data Types**. This means an Enum variant can **contain data**. 

<!-- See code/enums-examples.rs for runnable examples -->

### 3.1 Defining Distance Metrics

Our database needs to support different ways of calculating similarity.

```rust
enum DistanceMetric {
    Cosine,
    DotProduct,
    Euclidean,
    // Enums can hold data!
    // Minkowski requires a 'p' parameter
    Minkowski(f32), 
}
```

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-4-enum-variants -->

### 3.2 The `match` Control Flow

To use an Enum, we use `match`. It forces us to handle **every possible variant**. If you add a new metric later, the compiler will force you to update your code.

```rust
fn calculate(metric: DistanceMetric, a: &[f32], b: &[f32]) -> f32 {
    match metric {
        DistanceMetric::Cosine => cosine_similarity(a, b),
        DistanceMetric::DotProduct => dot_product(a, b),
        DistanceMetric::Euclidean => euclidean_distance(a, b),
        // Destructuring the data inside the variant
        DistanceMetric::Minkowski(p) => minkowski_distance(a, b, p),
    }
}
```

**Systems Note:** Under the hood, Rust implements this as a **tagged union**. It stores a small integer (the tag) + enough bytes to hold the largest variant. It is extremely memory efficient.

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-5-tagged-union-memory -->

---

## 4. The `Option` Enum: Killing the Null Pointer

Tony Hoare, the inventor of `null`, calls it his "billion-dollar mistake." Null causes crashes because you assume a value exists when it doesn't.

Rust **does not have null**.

Instead, it has `Option<T>`, which is just a standard enum!

```rust
enum Option<T> {
    Some(T), // Contains a value
    None,    // Contains nothing
}
```

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-6-option-vs-null -->

### Usage in Our DB

Let's say fetching a Vector by ID might fail if the ID doesn't exist.

```rust
fn get_vector(id: &str) -> Option<&Vector> {
    // logic to find vector...
    if found {
        Some(vector_ref)
    } else {
        None
    }
}
```

To use the value, you **must** unwrap the box. You cannot accidentally use `None` as a valid Vector.

```rust
match get_vector("missing_id") {
    Some(v) => println!("Found: {:?}", v),
    None => println!("Vector not found!"),
}

// Or use if-let for single-arm matching
if let Some(v) = get_vector("vec_001") {
    println!("Found vector with {} dimensions", v.dimension);
}
```

---

## 5. The `Result` Enum: Robust Error Handling

In systems programming, things fail. Disk is full. Network is down. File doesn't exist.

Rust encodes failure in the type system using `Result<T, E>`.

```rust
enum Result<T, E> {
    Ok(T),  // Success! Contains the value.
    Err(E), // Failure! Contains the error.
}
```

<!-- See code/error-handling.rs for runnable examples -->

### 5.1 Handling Errors

Let's look at `File::open`, which returns a `Result`.

```rust
use std::fs::File;

let f = File::open("database.wal");

match f {
    Ok(file) => println!("File opened successfully."),
    Err(error) => println!("Failed to open file: {:?}", error),
}
```

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-7-result-flow -->

### 5.2 The `?` Operator (Propagation)

Writing `match` for every error is tedious. Often, we just want to say: *"If this fails, return the error to my caller. If it succeeds, give me the value."*

That is what `?` does.

```rust
use std::fs::File;
use std::io::{self, Read};

// Function returns a Result
fn read_wal_header() -> Result<String, io::Error> {
    // If open fails, return Err immediately.
    // If success, unwrap file and continue.
    let mut f = File::open("database.wal")?; 
    
    let mut buffer = String::new();
    // Same here. Propagate error if read fails.
    f.read_to_string(&mut buffer)?; 
    
    Ok(buffer) // Return Success
}
```

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-8-question-mark-operator -->

**Rule of Thumb:**

| Context | Approach |
|---------|----------|
| Tests, prototypes, examples | `unwrap()` / `expect("message")` |
| Production code | `?` and `Result<T, E>` |
| Absolutely cannot fail | `unwrap()` with comment explaining why |

### 5.3 The `?` Trap: Custom Error Types

**Warning:** The `?` operator only works if Rust knows how to convert the error type. This code **will not compile**:

```rust
enum MyError {
    IoError(std::io::Error),
}

fn load() -> Result<(), MyError> {
    let f = File::open("data.bin")?; //  COMPILE ERROR!
    Ok(())
}
```

Why? `File::open` returns `io::Error`, but `?` doesn't know how to convert it to `MyError`.

**The Fix:** Use `map_err` to explicitly wrap the error:

```rust
fn load() -> Result<(), MyError> {
    let f = File::open("data.bin")
        .map_err(|e| MyError::IoError(e))?; //  Works!
    Ok(())
}
```

> **Systems Note:** For production code, you can implement the `From` trait to make `?` work automatically, or use the `thiserror` crate. For now, we'll use `map_err` to keep things explicit.

---

## 6. Application: Designing `vectordb` Types

Now, let's write the actual code for our database. We'll define the core types we'll use for the rest of the series.

<!-- See code/models.rs for complete implementation -->

Create a new file `src/models.rs` (and add `mod models;` to `main.rs`).

```rust
// src/models.rs
use std::collections::HashMap;

/// A vector embedding with metadata
#[derive(Debug, Clone)]
pub struct Vector {
    pub data: Vec<f32>,
    // Using HashMap for metadata: "title" -> "AI Paper"
    pub metadata: HashMap<String, String>, 
}

impl Vector {
    /// Constructor - enforces invariants
    pub fn new(data: Vec<f32>) -> Self {
        Self {
            data,
            metadata: HashMap::new(),
        }
    }
    
    /// Constructor with metadata
    pub fn with_metadata(data: Vec<f32>, metadata: HashMap<String, String>) -> Self {
        Self { data, metadata }
    }
    
    /// Get dimensionality
    pub fn dimension(&self) -> usize {
        self.data.len()
    }
}

/// Supported distance metrics for similarity search
#[derive(Debug, Clone, Copy)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    Dot,
}

/// A single search result with ID and similarity score
#[derive(Debug)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
}

/// Custom error type for our database
#[derive(Debug)]
pub enum VectorDbError {
    EmptyVector,
    DimensionMismatch { expected: usize, got: usize },
    NotFound(String),
    IoError(std::io::Error),
}

/// Type alias to avoid typing Result<T, VectorDbError> everywhere
pub type Result<T> = std::result::Result<T, VectorDbError>;
```

> **Pro Tip:** The type alias `pub type Result<T> = std::result::Result<T, VectorDbError>;` lets you write `Result<Vec<SearchResult>>` instead of `Result<Vec<SearchResult>, VectorDbError>`. This is a common pattern in Rust libraries (see `std::io::Result`).

And update `main.rs` to simulate a search that returns a `Result`:

> **Note:** In Post #2, we set up an async `main` with `#[tokio::main]`. For this example, we're using a simple synchronous `main` for clarity. In your real project, keep the Tokio attribute—it won't affect synchronous code.

```rust
mod models;
use models::{Vector, DistanceMetric, SearchResult, VectorDbError, Result};
use std::fs::File;

// Using our type alias - cleaner than Result<Vec<SearchResult>, VectorDbError>
fn search(query: &Vector, top_k: usize) -> Result<Vec<SearchResult>> {
    if query.data.is_empty() {
        return Err(VectorDbError::EmptyVector);
    }

    // (Simulated logic...)
    Ok(vec![
        SearchResult { id: "vec_1".to_string(), score: 0.95 }
    ])
}

// Example of using map_err with our custom error type
fn load_vectors(path: &str) -> Result<()> {
    let _f = File::open(path)
        .map_err(|e| VectorDbError::IoError(e))?; // Explicit conversion!
    Ok(())
}

fn main() {
    // Use the constructor we defined
    let q = Vector::new(vec![0.1, 0.2]);
    
    println!("Query dimension: {}", q.dimension());

    match search(&q, 10) {
        Ok(results) => println!("Found {} results", results.len()),
        Err(e) => eprintln!("Search failed: {:?}", e),
    }
    
    // Demonstrate error handling with map_err
    if let Err(e) = load_vectors("nonexistent.bin") {
        eprintln!("Load failed: {:?}", e);
    }
}
```

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-9-vectordb-type-hierarchy -->

---

## 7. Summary

You have just graduated from "Scripting" to "Systems Modeling".

| Concept | Purpose |
|---------|---------|
| **Struct** | Organize related data with named fields |
| **impl** | Add methods and associated functions |
| **Enum** | Model mutually exclusive states with optional data |
| **Option<T>** | Safe null replacement — forces you to check |
| **Result<T, E>** | Explicit error handling — no hidden exceptions |
| **?** | Propagate errors cleanly up the call stack |

### The Rust Type Design Flowchart

```
What are you modeling?
│
├─ A "thing" with properties? ──→ Use a Struct
│   └─ Does it need behavior? ──→ Add impl block
│
├─ One of several possibilities? ──→ Use an Enum
│   └─ Do variants carry data? ──→ Add fields to variants
│
└─ Something that might not exist?
    ├─ Absence is normal ──→ Option<T>
    └─ Absence is an error ──→ Result<T, E>
```

---

## 8. What's Next?

In the next post, we will stop simulating and start building the real thing. We will build the **Transport Layer**—an async HTTP server using `Axum` that accepts JSON and returns Vectors.

**Next Post:** [Post #5: The Async Runtime & HTTP Layer →](../post-05-async-axum/blog.md)

---

*Structs shape your data. Enums encode your possibilities. Results force you to face reality. This is systems programming done right.*
