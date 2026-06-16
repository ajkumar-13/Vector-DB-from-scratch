// hello-async.rs
//
// This is the "Systems Hello World" from Post #2.
// It verifies that the Tokio async runtime is working correctly.
//
// To run:
//   cd post-02-setting-up-the-forge/code
//   cargo run --bin hello-async

use tokio::time::{sleep, Duration};

/// The #[tokio::main] macro does the following:
/// 1. Creates a Tokio runtime (multi-threaded by default)
/// 2. Blocks on the async main function until completion
/// 3. Shuts down the runtime cleanly
///
/// It transforms:
///   #[tokio::main]
///   async fn main() { ... }
///
/// Into roughly:
///   fn main() {
///       tokio::runtime::Runtime::new().unwrap().block_on(async { ... })
///   }
#[tokio::main]
async fn main() {
    println!("The Forge is Hot! Initializing VectorDB...");
    println!();

    // ═══════════════════════════════════════════════════════════════════
    // CONCURRENT TASK: Simulating Background Startup Work
    // ═══════════════════════════════════════════════════════════════════
    // tokio::spawn() creates a new "task" - a lightweight, cooperative
    // green thread. Unlike OS threads, you can spawn millions of these.
    //
    // The task runs concurrently with our main function. When it hits
    // an .await point (like sleep()), it yields control, allowing other
    // tasks to make progress.
    let startup_task = tokio::spawn(async {
        // Simulate loading the Write-Ahead Log from disk
        println!("  [Background] Loading write-ahead log...");
        sleep(Duration::from_millis(500)).await; // Simulate disk I/O
        println!("  [Background] WAL loaded. 1,247 entries recovered.");
        
        // Simulate building the in-memory vector index
        println!("  [Background] Building vector index...");
        sleep(Duration::from_millis(300)).await;
        println!("  [Background] Index ready. 50,000 vectors loaded.");
        
        // The task implicitly returns () - we keep it simple for beginners.
        // In Post #4, we'll learn about Result<T, E> for proper error handling.
    });

    // ═══════════════════════════════════════════════════════════════════
    // MAIN THREAD: Doing Other Work Concurrently
    // ═══════════════════════════════════════════════════════════════════
    // While the background task is "sleeping" (simulating I/O), we can
    // do other work. This is the power of async - we're not blocked.
    println!("  [Main] Verifying system configuration...");
    sleep(Duration::from_millis(200)).await;
    println!("  [Main] Configuration OK.");
    
    // ═══════════════════════════════════════════════════════════════════
    // JOINING: Wait for Background Task to Complete
    // ═══════════════════════════════════════════════════════════════════
    // .await on a JoinHandle waits for the spawned task to finish.
    // It returns Result<T, JoinError> - Err only if the task panicked.
    match startup_task.await {
        Ok(_) => {
            println!();
            println!("All systems operational.");
            println!("   Listening on http://127.0.0.1:8080");
        }
        Err(e) => {
            // The task panicked (crashed). JoinError contains the panic info.
            println!("Startup task failed: {}", e);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXPECTED OUTPUT (notice how [Main] and [Background] interleave):
// ═══════════════════════════════════════════════════════════════════════════
//
//  The Forge is Hot! Initializing VectorDB...
//
//   [Main] Verifying system configuration...
//   [Background] Loading write-ahead log...
//   [Main] Configuration OK.
//   [Background] WAL loaded. 1,247 entries recovered.
//   [Background] Building vector index...
//   [Background] Index ready. 50,000 vectors loaded.
//
//  All systems operational.
//    Listening on http://127.0.0.1:8080
//
// ═══════════════════════════════════════════════════════════════════════════
