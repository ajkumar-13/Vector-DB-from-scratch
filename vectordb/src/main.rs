// src/main.rs
//
// VectorDB HTTP Server — Post #5
//
// A real Axum server with:
// - Shared state (Arc<RwLock<AppState>>)
// - CRUD endpoints (insert, get, search)
// - JSON error handling (ApiError → IntoResponse)
// - Request logging middleware (TraceLayer)
// - Graceful shutdown (Ctrl+C)
//
// Run with: cargo run
// Test with: curl http://localhost:3000/health

mod models;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use models::{SearchResult, Vector};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;

// ═══════════════════════════════════════════════════════════════════════════
// APPLICATION STATE
// ═══════════════════════════════════════════════════════════════════════════

/// Shared state across all handlers.
/// Arc provides shared ownership, RwLock provides safe concurrent access.
#[derive(Default)]
struct AppState {
    /// In-memory vector storage: id → vector
    vectors: HashMap<String, Vector>,
    /// Total requests served (for stats)
    request_count: u64,
}

/// Type alias — saves typing Arc<RwLock<AppState>> everywhere.
type SharedState = Arc<RwLock<AppState>>;

// ═══════════════════════════════════════════════════════════════════════════
// REQUEST TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Payload for POST /vectors
#[derive(Debug, Deserialize)]
struct InsertRequest {
    id: String,
    vector: Vector,
}

// ═══════════════════════════════════════════════════════════════════════════
// ERROR HANDLING
// ═══════════════════════════════════════════════════════════════════════════

/// API error that always returns a JSON body with the correct HTTP status.
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
        }
    }

    fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: msg.into(),
        }
    }
}

/// Convert ApiError into an HTTP response with JSON body.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": true,
            "message": self.message,
        });
        (self.status, Json(body)).into_response()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MAIN ENTRY POINT
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    // 1. Initialize structured logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    tracing::info!("Starting VectorDB server...");

    // 2. Create shared state
    let state: SharedState = Arc::new(RwLock::new(AppState::default()));

    // 3. Build router with all routes + middleware
    let app = Router::new()
        // Public endpoints
        .route("/", get(handler_home))
        .route("/health", get(handler_health))
        // CRUD endpoints
        .route("/vectors", post(handler_insert))
        .route("/vectors/{id}", get(handler_get_vector))
        .route("/search", post(handler_search))
        .route("/stats", get(handler_stats))
        // Attach shared state
        .with_state(state)
        // Middleware: automatic request logging
        .layer(TraceLayer::new_for_http());

    // 4. Bind and serve with graceful shutdown
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("🚀 Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    tracing::info!("Server shut down gracefully");
}

/// Wait for Ctrl+C to initiate graceful shutdown.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
    tracing::info!("Shutdown signal received, finishing in-flight requests...");
}

// ═══════════════════════════════════════════════════════════════════════════
// HANDLERS
// ═══════════════════════════════════════════════════════════════════════════

/// Home page — returns a simple HTML overview of available endpoints.
async fn handler_home() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head><title>VectorDB</title></head>
        <body>
            <h1>🦀 VectorDB</h1>
            <p>A vector database built from scratch in Rust.</p>
            <h2>Endpoints:</h2>
            <ul>
                <li>GET /health — Health check</li>
                <li>POST /vectors — Insert a vector</li>
                <li>GET /vectors/:id — Get a vector by ID</li>
                <li>POST /search — Search for similar vectors</li>
                <li>GET /stats — Server statistics</li>
            </ul>
        </body>
        </html>
    "#,
    )
}

/// Health check — for load balancers and k8s probes.
async fn handler_health() -> &'static str {
    "OK"
}

/// Insert a new vector into the database.
///
/// POST /vectors
/// Body: { "id": "doc_001", "vector": { "data": [0.1, 0.2], "metadata": {} } }
async fn handler_insert(
    State(state): State<SharedState>,
    Json(req): Json<InsertRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Validate input
    if req.id.is_empty() {
        return Err(ApiError::bad_request("Vector ID cannot be empty"));
    }
    if req.vector.data.is_empty() {
        return Err(ApiError::bad_request("Vector data cannot be empty"));
    }

    let dimension = req.vector.dimension();

    // Write to shared state — lock scoped to this block
    {
        let mut state = state.write().await;
        state.vectors.insert(req.id.clone(), req.vector);
        state.request_count += 1;
    } // Lock released here

    tracing::info!("Inserted vector '{}' ({} dims)", req.id, dimension);

    Ok(Json(serde_json::json!({
        "status": "inserted",
        "id": req.id,
        "dimension": dimension
    })))
}

/// Get a vector by its ID.
///
/// GET /vectors/:id
async fn handler_get_vector(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<Vector>, ApiError> {
    let state = state.read().await;

    match state.vectors.get(&id) {
        Some(vector) => Ok(Json(vector.clone())),
        None => Err(ApiError::not_found(format!("Vector '{}' not found", id))),
    }
}

/// Search for similar vectors.
///
/// POST /search
/// Body: { "data": [0.1, 0.2, 0.3] }
async fn handler_search(
    State(state): State<SharedState>,
    Json(query): Json<Vector>,
) -> Result<Json<Vec<SearchResult>>, ApiError> {
    // Validate input
    if query.data.is_empty() {
        return Err(ApiError::bad_request("Vector data cannot be empty"));
    }

    let state = state.read().await;

    tracing::info!(
        "Search: {} dims across {} stored vectors",
        query.dimension(),
        state.vectors.len()
    );

    // TODO: Real similarity search (Post #12).
    // For now, return dummy results.
    let results = vec![
        SearchResult {
            id: "doc_001".into(),
            score: 0.95,
        },
        SearchResult {
            id: "doc_002".into(),
            score: 0.87,
        },
        SearchResult {
            id: "doc_003".into(),
            score: 0.72,
        },
    ];

    Ok(Json(results))
}

/// Get server statistics.
///
/// GET /stats
async fn handler_stats(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let state = state.read().await;

    Json(serde_json::json!({
        "vector_count": state.vectors.len(),
        "request_count": state.request_count,
        "status": "running"
    }))
}
