// main-server.rs
//
// Complete Axum server for VectorDB.
// From Post #5: The Async Runtime & HTTP Layer
//
// Run with: cargo run
// Test with: curl http://localhost:3000/health

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

// ═══════════════════════════════════════════════════════════════════════════
// DATA MODELS (normally in models.rs)
// ═══════════════════════════════════════════════════════════════════════════

/// A vector embedding with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    pub data: Vec<f32>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Vector {
    pub fn dimension(&self) -> usize {
        self.data.len()
    }
}

/// Search result returned to client
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
}

/// Request payload for inserting a vector
#[derive(Debug, Deserialize)]
pub struct InsertRequest {
    pub id: String,
    pub vector: Vector,
}

/// Generic API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// APPLICATION STATE
// ═══════════════════════════════════════════════════════════════════════════

/// Shared state across all handlers
/// Using Arc<RwLock<...>> for thread-safe shared access
#[derive(Default)]
pub struct AppState {
    // Simple in-memory storage for now
    vectors: HashMap<String, Vector>,
    request_count: u64,
}

type SharedState = Arc<RwLock<AppState>>;

// ═══════════════════════════════════════════════════════════════════════════
// MAIN ENTRY POINT
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    // 1. Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    tracing::info!("Starting VectorDB server...");

    // 2. Create shared state
    let state: SharedState = Arc::new(RwLock::new(AppState::default()));

    // 3. Build router with all routes
    let app = Router::new()
        // Public endpoints
        .route("/", get(handler_home))
        .route("/health", get(handler_health))
        // API endpoints
        .route("/api/search", post(handler_search))
        .route("/api/vectors", post(handler_insert))
        .route("/api/vectors/:id", get(handler_get_vector))
        .route("/api/stats", get(handler_stats))
        // Attach shared state
        .with_state(state);

    // 4. Bind and serve
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("🚀 Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════
// HANDLERS
// ═══════════════════════════════════════════════════════════════════════════

/// Home page - returns HTML
async fn handler_home() -> Html<&'static str> {
    Html(r#"
        <!DOCTYPE html>
        <html>
        <head><title>VectorDB</title></head>
        <body>
            <h1>🦀 VectorDB</h1>
            <p>A vector database built from scratch in Rust.</p>
            <h2>Endpoints:</h2>
            <ul>
                <li>GET /health - Health check</li>
                <li>POST /api/search - Search for similar vectors</li>
                <li>POST /api/vectors - Insert a vector</li>
                <li>GET /api/vectors/:id - Get a vector by ID</li>
                <li>GET /api/stats - Server statistics</li>
            </ul>
        </body>
        </html>
    "#)
}

/// Health check - for load balancers and k8s probes
async fn handler_health() -> &'static str {
    "OK"
}

/// Search for similar vectors
async fn handler_search(
    State(state): State<SharedState>,
    Json(query): Json<Vector>,
) -> Result<Json<ApiResponse<Vec<SearchResult>>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Validate input
    if query.data.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err("Vector data cannot be empty")),
        ));
    }

    // Increment request counter
    {
        let mut state = state.write().await;
        state.request_count += 1;
    }

    tracing::info!("Search request: {} dimensions", query.dimension());

    // TODO: Real similarity search
    // For now, return dummy results
    let results = vec![
        SearchResult { id: "doc_001".into(), score: 0.95 },
        SearchResult { id: "doc_002".into(), score: 0.87 },
        SearchResult { id: "doc_003".into(), score: 0.72 },
    ];

    Ok(Json(ApiResponse::ok(results)))
}

/// Insert a new vector
async fn handler_insert(
    State(state): State<SharedState>,
    Json(req): Json<InsertRequest>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Validate
    if req.id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err("Vector ID cannot be empty")),
        ));
    }

    if req.vector.data.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err("Vector data cannot be empty")),
        ));
    }

    // Insert into storage
    let dimension = req.vector.dimension();
    {
        let mut state = state.write().await;
        state.vectors.insert(req.id.clone(), req.vector);
        state.request_count += 1;
    }

    tracing::info!("Inserted vector '{}' with {} dimensions", req.id, dimension);

    Ok(Json(ApiResponse::ok(format!("Inserted vector '{}'", req.id))))
}

/// Get a vector by ID
async fn handler_get_vector(
    State(state): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<Vector>>, (StatusCode, Json<ApiResponse<()>>)> {
    let state = state.read().await;

    match state.vectors.get(&id) {
        Some(vector) => Ok(Json(ApiResponse::ok(vector.clone()))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::err(format!("Vector '{}' not found", id))),
        )),
    }
}

/// Get server statistics
async fn handler_stats(
    State(state): State<SharedState>,
) -> Json<serde_json::Value> {
    let state = state.read().await;

    Json(serde_json::json!({
        "vector_count": state.vectors.len(),
        "request_count": state.request_count,
        "status": "running"
    }))
}
