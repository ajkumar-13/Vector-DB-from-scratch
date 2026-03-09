// post-10-concurrency/code/axum-server.rs
// Complete concurrent HTTP server using Axum + Arc<RwLock<VectorStore>>
//
// Add to Cargo.toml:
//   [dependencies]
//   axum = "0.7"
//   tokio = { version = "1", features = ["full"] }
//   serde = { version = "1", features = ["derive"] }
//   serde_json = "1"
//
// Run with: cargo run --bin server

use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// ============================================================================
// VectorStore (simplified for this demo)
// ============================================================================

pub struct VectorStore {
    vectors: HashMap<String, Vec<f32>>,
    stats: Stats,
}

#[derive(Default, Clone, Serialize)]
pub struct Stats {
    pub inserts: u64,
    pub deletes: u64,
    pub searches: u64,
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            vectors: HashMap::new(),
            stats: Stats::default(),
        }
    }

    pub fn insert(&mut self, id: String, vector: Vec<f32>) {
        self.vectors.insert(id, vector);
        self.stats.inserts += 1;
    }

    pub fn delete(&mut self, id: &str) -> bool {
        let existed = self.vectors.remove(id).is_some();
        if existed {
            self.stats.deletes += 1;
        }
        existed
    }

    pub fn get(&self, id: &str) -> Option<&Vec<f32>> {
        self.vectors.get(id)
    }

    pub fn search(&mut self, query: &[f32], top_k: usize) -> Vec<SearchResult> {
        self.stats.searches += 1;
        
        let mut results: Vec<_> = self.vectors
            .iter()
            .map(|(id, vec)| {
                let score = cosine_similarity(query, vec);
                SearchResult {
                    id: id.clone(),
                    score,
                }
            })
            .collect();
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(top_k);
        results
    }

    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    pub fn needs_compaction(&self) -> bool {
        // Placeholder: in real code, check memtable size
        self.vectors.len() > 10_000
    }

    pub fn compact(&mut self) -> Result<(), String> {
        // Placeholder for compaction
        println!("Compaction running... (simulated)");
        Ok(())
    }

    pub fn stats(&self) -> Stats {
        self.stats.clone()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot / (norm_a * norm_b)
}

// ============================================================================
// Shared State Type
// ============================================================================

/// The shared state type used throughout the application
/// Arc: Allows multiple owners across threads
/// RwLock: Allows concurrent reads, exclusive writes
pub type SharedVectorStore = Arc<RwLock<VectorStore>>;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Deserialize)]
pub struct InsertRequest {
    pub id: String,
    pub vector: Vec<f32>,
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    pub id: String,
}

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: Vec<f32>,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    10
}

#[derive(Serialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub vector_count: usize,
    pub inserts: u64,
    pub deletes: u64,
    pub searches: u64,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// Health check endpoint
async fn health_handler() -> &'static str {
    "OK"
}

/// Get statistics (read lock)
async fn stats_handler(
    State(store): State<SharedVectorStore>,
) -> Json<StatsResponse> {
    // Acquire read lock
    let db = store.read().await;
    
    let stats = db.stats();
    Json(StatsResponse {
        vector_count: db.len(),
        inserts: stats.inserts,
        deletes: stats.deletes,
        searches: stats.searches,
    })
}

/// Search for similar vectors (write lock because we update stats)
async fn search_handler(
    State(store): State<SharedVectorStore>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    // Validate input
    if request.query.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Acquire write lock (we update search stats)
    // In production, you might use a separate counter with AtomicU64
    let mut db = store.write().await;
    
    let results = db.search(&request.query, request.top_k);
    
    Ok(Json(results))
}

/// Insert a vector (write lock)
async fn insert_handler(
    State(store): State<SharedVectorStore>,
    Json(request): Json<InsertRequest>,
) -> Result<Json<MessageResponse>, StatusCode> {
    // Validate input
    if request.id.is_empty() || request.vector.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Acquire write lock
    let mut db = store.write().await;
    
    db.insert(request.id.clone(), request.vector);
    
    Ok(Json(MessageResponse {
        message: format!("Inserted vector '{}'", request.id),
    }))
}

/// Delete a vector (write lock)
async fn delete_handler(
    State(store): State<SharedVectorStore>,
    Json(request): Json<DeleteRequest>,
) -> Result<Json<MessageResponse>, StatusCode> {
    if request.id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let mut db = store.write().await;
    
    if db.delete(&request.id) {
        Ok(Json(MessageResponse {
            message: format!("Deleted vector '{}'", request.id),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Manual compaction trigger (write lock)
async fn compact_handler(
    State(store): State<SharedVectorStore>,
) -> Result<Json<MessageResponse>, StatusCode> {
    let mut db = store.write().await;
    
    match db.compact() {
        Ok(()) => Ok(Json(MessageResponse {
            message: "Compaction complete".into(),
        })),
        Err(e) => {
            eprintln!("Compaction failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Background Tasks
// ============================================================================

/// Spawn a background task that periodically checks for compaction
fn spawn_background_compaction(store: SharedVectorStore) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            // Step 1: Check if compaction needed (read lock)
            let needs_compact = {
                let db = store.read().await;
                db.needs_compaction()
            };  // Read lock dropped here!
            
            // Step 2: Compact if needed (write lock)
            if needs_compact {
                println!("[Background] Starting compaction...");
                let mut db = store.write().await;
                
                match db.compact() {
                    Ok(()) => println!("[Background] Compaction complete"),
                    Err(e) => eprintln!("[Background] Compaction failed: {}", e),
                }
            }
        }
    });
}

/// Spawn a task to handle graceful shutdown
/// 
/// WARNING: std::process::exit() does NOT run destructors!
/// We must explicitly flush before calling it.
fn spawn_shutdown_handler(store: SharedVectorStore) {
    tokio::spawn(async move {
        // Wait for Ctrl+C
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl-c");
        
        println!("\n[Shutdown] Received shutdown signal...");
        
        // Final compaction and flush
        {
            let mut db = store.write().await;
            
            // Compact any remaining memtable data
            println!("[Shutdown] Running final compaction...");
            if let Err(e) = db.compact() {
                eprintln!("[Shutdown] Compaction failed: {}", e);
            }
            
            // CRITICAL: std::process::exit() bypasses destructors!
            // BufWriter::drop() (which calls flush) will NOT run.
            // We must explicitly flush the WAL here.
            // In a real implementation: db.flush_wal().unwrap();
            println!("[Shutdown] Flushing WAL...");
        } // Lock released, but exit() still skips Drop
        
        println!("[Shutdown] Goodbye!");
        std::process::exit(0);
    });
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         Concurrent Vector Database Server (Axum)             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Initialize the store
    let store = VectorStore::new();
    let shared_store: SharedVectorStore = Arc::new(RwLock::new(store));

    // Start background tasks
    spawn_background_compaction(Arc::clone(&shared_store));
    spawn_shutdown_handler(Arc::clone(&shared_store));

    // Build the router
    let app = Router::new()
        // Read endpoints
        .route("/health", get(health_handler))
        .route("/stats", get(stats_handler))
        // Write endpoints
        .route("/search", post(search_handler))
        .route("/insert", post(insert_handler))
        .route("/delete", post(delete_handler))
        .route("/compact", post(compact_handler))
        // Inject shared state
        .with_state(shared_store);

    // Print API documentation
    println!("API Endpoints:");
    println!("  GET  /health        - Health check");
    println!("  GET  /stats         - Get statistics");
    println!("  POST /search        - Search for similar vectors");
    println!("  POST /insert        - Insert a vector");
    println!("  POST /delete        - Delete a vector");
    println!("  POST /compact       - Trigger manual compaction");
    println!();

    // Print example requests
    println!("Example requests:");
    println!("  curl http://localhost:3000/health");
    println!("  curl -X POST http://localhost:3000/insert \\");
    println!("       -H 'Content-Type: application/json' \\");
    println!("       -d '{{\"id\":\"vec_1\",\"vector\":[1.0,0.0,0.0]}}'");
    println!("  curl -X POST http://localhost:3000/search \\");
    println!("       -H 'Content-Type: application/json' \\");
    println!("       -d '{{\"query\":[1.0,0.0,0.0],\"top_k\":5}}'");
    println!();

    // Start the server
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Server running on http://{}", addr);
    println!("   Press Ctrl+C to shutdown gracefully\n");

    axum::serve(listener, app).await.unwrap();
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_inserts() {
        let store: SharedVectorStore = Arc::new(RwLock::new(VectorStore::new()));
        let mut handles = vec![];

        for i in 0..100 {
            let s = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                let mut db = s.write().await;
                db.insert(format!("vec_{}", i), vec![i as f32; 128]);
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        let db = store.read().await;
        assert_eq!(db.len(), 100);
    }

    #[tokio::test]
    async fn test_concurrent_reads() {
        let store: SharedVectorStore = Arc::new(RwLock::new(VectorStore::new()));
        
        // Insert test data
        {
            let mut db = store.write().await;
            for i in 0..10 {
                db.insert(format!("vec_{}", i), vec![i as f32; 128]);
            }
        }

        // Concurrent reads
        let mut handles = vec![];
        for _ in 0..100 {
            let s = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                let db = s.read().await;
                db.len()
            }));
        }

        for h in handles {
            let len = h.await.unwrap();
            assert_eq!(len, 10);
        }
    }

    #[tokio::test]
    async fn test_read_during_write() {
        let store: SharedVectorStore = Arc::new(RwLock::new(VectorStore::new()));
        
        // Insert initial data
        {
            let mut db = store.write().await;
            db.insert("initial".into(), vec![1.0]);
        }

        // Start a long write
        let writer_store = Arc::clone(&store);
        let writer = tokio::spawn(async move {
            let mut db = writer_store.write().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
            db.insert("new".into(), vec![2.0]);
        });

        // Give writer time to acquire lock
        tokio::time::sleep(Duration::from_millis(10)).await;

        // This read will wait for the writer
        let read_start = std::time::Instant::now();
        let db = store.read().await;
        let wait_time = read_start.elapsed();
        
        // Reader should have waited
        assert!(wait_time.as_millis() >= 50, "Reader should have waited");
        assert_eq!(db.len(), 2);

        writer.await.unwrap();
    }
}
