# The Async Runtime: Understanding Tokio, Futures, and Building a Basic HTTP Server with Axum

**Series:** Building a Vector Database from Scratch in Rust  
**Post:** 5 of 20  
**Reading Time:** ~15 minutes

---

## 1. Introduction: The "Waiter" Problem

In the last post, we defined our data structures. We have a `Vector`, we have `DistanceMetric`, and we have error handling. But right now, our database is just a library that runs on your laptop.

To make it a **Server**, it needs to listen for requests over the network.

If you come from Python (Flask/Django) or Ruby (Rails), you might be used to a "Thread-per-Request" model. When a request comes in, the server spins up a thread (or uses one from a pool) and blocks that thread until the response is ready.

**For a database, this is fatal.**

Imagine your database is writing to disk (a slow operation). In a blocking model, if 100 users try to write at once, you need 100 threads waiting on the disk. Threads are heavy (they eat RAM and CPU for context switching).

Rust uses **Async I/O**.

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-1-sync-vs-async -->

**The Analogy:**

* **Blocking (Sync):** You order coffee. The cashier stands there, staring at the barista making your coffee, ignoring the line behind you. Only when you get your coffee does the cashier take the next order.
* **Non-Blocking (Async):** You order coffee. The cashier gives you a ticket (a **Future**) and immediately takes the next order. When your coffee is ready, they call your number.

In this post, we will build the **Transport Layer** of our database using **Tokio** (the cashier) and **Axum** (the menu).

---

## 2. The Engine: What is Tokio?

Rust's standard library does *not* include an async runtime. It provides the *concept* of a Future (a promise that a value will exist later), but it doesn't have the engine to execute them.

Enter **Tokio**.

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-2-tokio-architecture -->

Tokio is an asynchronous runtime that provides:

1. **A Multi-threaded Scheduler:** It runs thousands of lightweight "tasks" (green threads) on a small number of OS threads.
2. **Non-blocking I/O:** Drivers for network (TCP/UDP) and file systems.
3. **Timers:** `sleep`, `timeout`, etc.

When you annotate `main` with `#[tokio::main]`, you are essentially saying: *"Before my code runs, start the engine."*

```rust
#[tokio::main]  // Macro that sets up the runtime
async fn main() {
    // Your async code runs here
    // Tokio's scheduler is active
}
```

> **Systems Note:** Under the hood, `#[tokio::main]` expands to roughly:
> ```rust
> fn main() {
>     tokio::runtime::Runtime::new()
>         .unwrap()
>         .block_on(async { /* your code */ })
> }
> ```

---

## 3. The Framework: Why Axum?

We need a way to route HTTP requests (`POST /search`) to our Rust functions. We will use **Axum**.

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-3-axum-request-flow -->

Axum is built by the Tokio team and is currently the gold standard for Rust web services because:

| Feature | Benefit |
|---------|---------|
| **Macro-Free** | Unlike Rocket, it uses standard Rust traits. Easier debugging. |
| **Type-Safe Extraction** | If your function expects `Json<Vector>`, Axum validates automatically |
| **Built on Hyper** | Same HTTP engine used by Curl's Rust bindings. Battle-tested. |
| **Tower Middleware** | Composable layers for logging, auth, rate-limiting |

---

## 4. Hands-On: Building the Server

Let's turn our `vectordb` into a running web server.

<!-- See code/main-server.rs for complete implementation -->
<!-- See code/Cargo.toml for dependencies -->

### 4.1 Adding Dependencies

Open `Cargo.toml`. We need to add `axum` and `tracing` (for logging).

```toml
[dependencies]
# Serialization (from Post #4)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Web Framework
axum = "0.7"

# Async Runtime (ensure 'full' features)
tokio = { version = "1", features = ["full"] }

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 4.2 The Hello World Endpoint

Let's modify `src/main.rs`. We will initialize the standard "Boilerplate" for a production Axum app.

```rust
use axum::{
    routing::{get, post},
    Router,
    response::Html,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // 1. Initialize Logging
    tracing_subscriber::fmt::init();

    // 2. Define Routes
    let app = Router::new()
        .route("/", get(handler_home))
        .route("/health", get(handler_health));

    // 3. Bind to Address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Listening on http://{}", addr);

    // 4. Start Server (Axum 0.7 syntax)
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler: Basic HTML response
async fn handler_home() -> Html<&'static str> {
    Html("<h1>Welcome to VectorDB</h1>")
}

// Handler: Health check (for load balancers, k8s probes)
async fn handler_health() -> &'static str {
    "OK"
}
```

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-4-server-startup -->

Run it with `cargo run`.
Open `http://localhost:3000` in your browser. You should see the welcome message.

---

## 5. Connecting Data: JSON and Structs

Now for the real work. We want to accept a JSON payload, parse it into our `Vector` struct (from Post #4), and return a result.

### 5.1 The Setup

We need to make sure our structs in `src/models.rs` derive `Deserialize` (to read JSON) and `Serialize` (to write JSON).

<!-- See code/models-serde.rs for complete implementation -->

**Update `src/models.rs`:**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)] // Added Serde macros!
pub struct Vector {
    pub data: Vec<f32>,
    #[serde(default)]  // If missing in JSON, use Default::default()
    pub metadata: HashMap<String, String>,
}

impl Vector {
    pub fn new(data: Vec<f32>) -> Self {
        Self { data, metadata: HashMap::new() }
    }
    
    pub fn dimension(&self) -> usize {
        self.data.len()
    }
}

#[derive(Debug, Serialize)]  // Only Serialize - we return this, never receive it
pub struct SearchResult {
    pub id: String,
    pub score: f32,
}
```

> **Serde Tip:** Use `#[serde(default)]` for optional fields. This lets clients omit `metadata` entirely, and Serde will use an empty HashMap.

### 5.2 The Search Endpoint

Add this handler to `src/main.rs`. Notice how the argument is `Json<Vector>`. Axum does the heavy lifting.

```rust
use axum::Json;
mod models;
use models::{Vector, SearchResult};

// POST /search
// Axum automatically parses request body as JSON into 'Vector'.
// If JSON is invalid or missing fields, Axum returns 422 before your code runs.
async fn handler_search(Json(payload): Json<Vector>) -> Json<Vec<SearchResult>> {
    tracing::info!(
        "Search request: {} dimensions", 
        payload.dimension()
    );

    // TODO: Connect to real search engine later.
    // For now, return dummy data.
    let results = vec![
        SearchResult { id: "doc_1".into(), score: 0.99 },
        SearchResult { id: "doc_2".into(), score: 0.85 },
    ];

    Json(results)
}
```

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-5-json-extraction -->

**Register the route in `main`:**

```rust
let app = Router::new()
    .route("/", get(handler_home))
    .route("/health", get(handler_health))
    .route("/search", post(handler_search));  // New route!
```

### 5.3 Error Handling in Handlers

What if the search fails? We should return proper HTTP errors, not panic.

```rust
use axum::http::StatusCode;
use axum::response::IntoResponse;

// Return type that can be either success or error
async fn handler_search(
    Json(payload): Json<Vector>
) -> Result<Json<Vec<SearchResult>>, (StatusCode, String)> {
    
    // Validate input
    if payload.data.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Vector data cannot be empty".to_string()
        ));
    }

    // Perform search...
    let results = vec![
        SearchResult { id: "doc_1".into(), score: 0.99 },
    ];

    Ok(Json(results))
}
```

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-6-error-response-flow -->

---

## 6. Testing with Curl

Let's prove it works.

**Run the server:**

```bash
cargo run
```

**Test health endpoint:**

```bash
curl http://localhost:3000/health
# Output: OK
```

**Send a valid search request:**

```bash
curl -X POST http://localhost:3000/search \
  -H "Content-Type: application/json" \
  -d '{
    "data": [0.1, 0.2, 0.3],
    "metadata": {"title": "Test"}
  }'
```

*Expected Output:*
```json
[{"id":"doc_1","score":0.99},{"id":"doc_2","score":0.85}]
```

**Send an invalid request (missing data field):**

```bash
curl -X POST http://localhost:3000/search \
  -H "Content-Type: application/json" \
  -d '{ "metadata": {} }'
```

*Expected Output:* `422 Unprocessable Entity` — Axum caught the missing field for us!

**Send empty vector (our validation):**

```bash
curl -X POST http://localhost:3000/search \
  -H "Content-Type: application/json" \
  -d '{ "data": [] }'
```

*Expected Output:* `400 Bad Request` with message "Vector data cannot be empty"

---

## 7. Understanding Async/Await

Before we move on, let's demystify the `async` and `await` keywords.

### What is a Future?

A `Future` is a value that *might not exist yet*. When you call an async function, it doesn't run immediately—it returns a Future.

```rust
async fn fetch_data() -> String {
    // This doesn't run until someone .await's it
    "data".to_string()
}

let future = fetch_data();  // Nothing happens yet!
let data = future.await;    // NOW it runs
```

### Why `.await`?

The `.await` keyword tells Tokio: *"I'm going to wait for this. While waiting, feel free to run other tasks."*

```rust
async fn handle_request() {
    let db_result = query_database().await;  // Yields to other tasks
    let file = read_file().await;            // Yields again
    process(db_result, file);
}
```

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-7-await-yield -->

> **Systems Note:** Unlike threads, yielding a Future has near-zero cost. There's no context switch, no kernel involvement. Tokio just moves to the next ready task in its queue.

---

## 8. Summary

We now have the **Transport Layer** (Layer 1 from our architecture diagram) working.

| Component | Role |
|-----------|------|
| **Tokio** | Async runtime, manages thousands of concurrent connections |
| **Axum** | Routes requests, extracts JSON, validates input |
| **Serde** | Converts JSON ↔ Rust structs |
| **Tracing** | Structured logging for debugging |

<!-- DIAGRAM: diagrams/mermaid-diagrams.md#diagram-8-layer-complete -->

### What We Built

```
Client                     VectorDB Server
  │                              │
  │  POST /search {json}         │
  │ ───────────────────────────► │
  │                              │ Axum extracts Json<Vector>
  │                              │ Handler processes request
  │  [SearchResult, ...]         │
  │ ◄─────────────────────────── │
  │                              │
```

But our database has no memory. If you restart the server, everything vanishes. It stores nothing.

---

## 9. What's Next?

In the next post, we start building **Layer 3: The Storage Engine**. We will leave the high-level world of JSON and dive into the low-level world of **Binary File Formats** and **Byte Serialization**.

We'll learn:
- Why JSON is too slow for disk storage
- How to design a binary segment format
- Reading and writing raw bytes efficiently

**Next Post:** [Post #6: Binary File Formats: Designing a Custom Segment Layout →](../post-06-binary-formats/blog.md)

---

*Async isn't just about speed—it's about doing more with less. One thread, thousands of connections. That's the power of Tokio.*
