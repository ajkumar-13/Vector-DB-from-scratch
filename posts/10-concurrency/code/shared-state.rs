// post-10-concurrency/code/shared-state.rs
// Demonstrating Arc + RwLock patterns for concurrent access
//
// Run with: cargo run --example shared-state
// Or: rustc shared-state.rs && ./shared-state

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// We'll use std::sync::RwLock for this standalone example
// In async code, you'd use tokio::sync::RwLock instead
use std::sync::RwLock;

// ============================================================================
// Simulated VectorStore
// ============================================================================

pub struct VectorStore {
    vectors: HashMap<String, Vec<f32>>,
    write_count: usize,
    read_count: usize,
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            vectors: HashMap::new(),
            write_count: 0,
            read_count: 0,
        }
    }

    pub fn insert(&mut self, id: String, vector: Vec<f32>) {
        self.vectors.insert(id, vector);
        self.write_count += 1;
    }

    pub fn get(&self, id: &str) -> Option<&Vec<f32>> {
        self.vectors.get(id)
    }

    pub fn search(&self, _query: &[f32], _top_k: usize) -> Vec<String> {
        // Simulate search work
        thread::sleep(Duration::from_micros(100));
        self.vectors.keys().take(5).cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.read_count, self.write_count)
    }

    fn record_read(&mut self) {
        self.read_count += 1;
    }
}

// Type alias for shared state
type SharedVectorStore = Arc<RwLock<VectorStore>>;

// ============================================================================
// Demo 1: Basic Arc Sharing
// ============================================================================

fn demo_arc_basics() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║            Demo 1: Arc Basics (Reference Counting)           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let data = Arc::new(42);
    println!("Initial ref count: {}", Arc::strong_count(&data));

    let clone1 = Arc::clone(&data);
    println!("After clone1: {}", Arc::strong_count(&data));

    let clone2 = Arc::clone(&data);
    println!("After clone2: {}", Arc::strong_count(&data));

    // All point to the same data
    println!("\nAll clones see: {} {} {}", *data, *clone1, *clone2);

    drop(clone1);
    println!("After dropping clone1: {}", Arc::strong_count(&data));

    drop(clone2);
    println!("After dropping clone2: {}", Arc::strong_count(&data));

    println!();
}

// ============================================================================
// Demo 2: Mutex vs RwLock
// ============================================================================

fn demo_mutex_vs_rwlock() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          Demo 2: Mutex vs RwLock Performance                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let iterations = 10_000;
    let num_readers = 4;
    let num_writers = 1;

    // Test with Mutex
    {
        use std::sync::Mutex;
        let store = Arc::new(Mutex::new(VectorStore::new()));

        // Pre-populate
        {
            let mut db = store.lock().unwrap();
            for i in 0..100 {
                db.insert(format!("vec_{}", i), vec![i as f32; 128]);
            }
        }

        let start = Instant::now();
        let mut handles = vec![];

        // Spawn readers
        for _ in 0..num_readers {
            let s = Arc::clone(&store);
            handles.push(thread::spawn(move || {
                for _ in 0..iterations {
                    let db = s.lock().unwrap();
                    let _ = db.len();
                }
            }));
        }

        // Spawn writer
        for i in 0..num_writers {
            let s = Arc::clone(&store);
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    let mut db = s.lock().unwrap();
                    db.insert(format!("mutex_{}_{}", i, j), vec![1.0]);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let mutex_time = start.elapsed();
        println!("Mutex ({} readers, {} writer):", num_readers, num_writers);
        println!("  Time: {:?}", mutex_time);
    }

    // Test with RwLock
    {
        let store = Arc::new(RwLock::new(VectorStore::new()));

        // Pre-populate
        {
            let mut db = store.write().unwrap();
            for i in 0..100 {
                db.insert(format!("vec_{}", i), vec![i as f32; 128]);
            }
        }

        let start = Instant::now();
        let mut handles = vec![];

        // Spawn readers
        for _ in 0..num_readers {
            let s = Arc::clone(&store);
            handles.push(thread::spawn(move || {
                for _ in 0..iterations {
                    let db = s.read().unwrap();
                    let _ = db.len();
                }
            }));
        }

        // Spawn writer
        for i in 0..num_writers {
            let s = Arc::clone(&store);
            handles.push(thread::spawn(move || {
                for j in 0..100 {
                    let mut db = s.write().unwrap();
                    db.insert(format!("rwlock_{}_{}", i, j), vec![1.0]);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let rwlock_time = start.elapsed();
        println!(
            "\nRwLock ({} readers, {} writer):",
            num_readers, num_writers
        );
        println!("  Time: {:?}", rwlock_time);
    }

    println!();
}

// ============================================================================
// Demo 3: Concurrent Readers
// ============================================================================

fn demo_concurrent_readers() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           Demo 3: Concurrent Readers with RwLock             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let store: SharedVectorStore = Arc::new(RwLock::new(VectorStore::new()));

    // Pre-populate
    {
        let mut db = store.write().unwrap();
        for i in 0..1000 {
            db.insert(format!("vec_{}", i), vec![i as f32; 128]);
        }
    }

    let query = vec![0.5; 128];
    let num_threads = 8;
    let searches_per_thread = 100;

    println!(
        "Spawning {} reader threads, {} searches each...\n",
        num_threads, searches_per_thread
    );

    let start = Instant::now();
    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let s = Arc::clone(&store);
        let q = query.clone();

        handles.push(thread::spawn(move || {
            let mut results = 0;
            for _ in 0..searches_per_thread {
                let db = s.read().unwrap();
                results += db.search(&q, 10).len();
            }
            println!(
                "  Thread {} completed {} searches",
                thread_id, searches_per_thread
            );
            results
        }));
    }

    let total_results: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();

    let elapsed = start.elapsed();
    let total_searches = num_threads * searches_per_thread;

    println!("\nResults:");
    println!("  Total searches: {}", total_searches);
    println!("  Total time: {:?}", elapsed);
    println!(
        "  Searches/sec: {:.0}",
        total_searches as f64 / elapsed.as_secs_f64()
    );
    println!("  (All readers ran concurrently!)\n");
}

// ============================================================================
// Demo 4: Writer Blocks Readers
// ============================================================================

fn demo_writer_exclusion() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         Demo 4: Write Lock Excludes Readers                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let store: SharedVectorStore = Arc::new(RwLock::new(VectorStore::new()));

    // Spawn a writer that holds the lock for a while
    let writer_store = Arc::clone(&store);
    let writer = thread::spawn(move || {
        println!("[Writer] Acquiring write lock...");
        let mut db = writer_store.write().unwrap();
        println!("[Writer] Lock acquired! Holding for 500ms...");

        for i in 0..10 {
            db.insert(format!("item_{}", i), vec![i as f32]);
            thread::sleep(Duration::from_millis(50));
        }

        println!("[Writer] Releasing lock.");
    });

    // Give writer time to acquire lock
    thread::sleep(Duration::from_millis(50));

    // Spawn readers that will be blocked
    let mut readers = vec![];
    for i in 0..3 {
        let s = Arc::clone(&store);
        readers.push(thread::spawn(move || {
            let start = Instant::now();
            println!("[Reader {}] Waiting for read lock...", i);
            let db = s.read().unwrap();
            println!(
                "[Reader {}] Lock acquired after {:?}, found {} items",
                i,
                start.elapsed(),
                db.len()
            );
        }));
    }

    writer.join().unwrap();
    for r in readers {
        r.join().unwrap();
    }

    println!();
}

// ============================================================================
// Demo 5: Avoiding Deadlocks
// ============================================================================

fn demo_avoiding_deadlock() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          Demo 5: Avoiding the Upgrade Deadlock               ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let store: SharedVectorStore = Arc::new(RwLock::new(VectorStore::new()));

    // BAD PATTERN (commented out - would deadlock!)
    // {
    //     let db = store.read().unwrap();
    //     if db.get("new_item").is_none() {
    //         let mut w_db = store.write().unwrap();  // DEADLOCK!
    //         w_db.insert("new_item".into(), vec![1.0]);
    //     }
    // }

    // CORRECT PATTERN: Drop read lock before acquiring write lock
    fn safe_check_and_insert(store: &SharedVectorStore, id: &str, vector: Vec<f32>) -> bool {
        // Step 1: Check with read lock
        let exists = {
            let db = store.read().unwrap();
            db.get(id).is_some()
        }; // Read lock dropped here!

        // Step 2: Insert with write lock if needed
        if !exists {
            let mut db = store.write().unwrap();
            // Re-check (TOCTOU protection)
            if db.get(id).is_none() {
                db.insert(id.to_string(), vector);
                println!("  Inserted: {}", id);
                return true;
            }
        }

        println!("  Skipped (already exists): {}", id);
        false
    }

    println!("Using safe check-and-insert pattern:\n");

    safe_check_and_insert(&store, "item_1", vec![1.0]);
    safe_check_and_insert(&store, "item_2", vec![2.0]);
    safe_check_and_insert(&store, "item_1", vec![1.5]); // Should skip
    safe_check_and_insert(&store, "item_3", vec![3.0]);

    let db = store.read().unwrap();
    println!("\nFinal count: {} items", db.len());
    println!();
}

// ============================================================================
// Demo 6: Lock Contention Visualization
// ============================================================================

fn demo_lock_contention() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           Demo 6: Visualizing Lock Contention                ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let store: SharedVectorStore = Arc::new(RwLock::new(VectorStore::new()));
    let start_time = Instant::now();

    // Helper to print timeline
    fn log(start: Instant, msg: &str) {
        let elapsed = start.elapsed().as_millis();
        println!("[{:4}ms] {}", elapsed, msg);
    }

    let mut handles = vec![];

    // Reader 1: Quick reads
    {
        let s = Arc::clone(&store);
        let start = start_time;
        handles.push(thread::spawn(move || {
            for i in 0..3 {
                thread::sleep(Duration::from_millis(50));
                let _db = s.read().unwrap();
                log(start, &format!("Reader1: acquired read lock ({})", i));
                thread::sleep(Duration::from_millis(20));
            }
        }));
    }

    // Reader 2: Quick reads
    {
        let s = Arc::clone(&store);
        let start = start_time;
        handles.push(thread::spawn(move || {
            for i in 0..3 {
                thread::sleep(Duration::from_millis(60));
                let _db = s.read().unwrap();
                log(start, &format!("Reader2: acquired read lock ({})", i));
                thread::sleep(Duration::from_millis(20));
            }
        }));
    }

    // Writer: Slower writes
    {
        let s = Arc::clone(&store);
        let start = start_time;
        handles.push(thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            log(start, "Writer:  requesting write lock...");
            let mut db = s.write().unwrap();
            log(start, "Writer:  ACQUIRED write lock (readers blocked!)");
            db.insert("important".into(), vec![42.0]);
            thread::sleep(Duration::from_millis(150));
            log(start, "Writer:  releasing write lock");
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    println!();
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("\n");
    demo_arc_basics();
    demo_mutex_vs_rwlock();
    demo_concurrent_readers();
    demo_writer_exclusion();
    demo_avoiding_deadlock();
    demo_lock_contention();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              All Concurrency Demos Complete!                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concurrent_reads() {
        let store: SharedVectorStore = Arc::new(RwLock::new(VectorStore::new()));

        {
            let mut db = store.write().unwrap();
            db.insert("test".into(), vec![1.0]);
        }

        let mut handles = vec![];
        for _ in 0..10 {
            let s = Arc::clone(&store);
            handles.push(thread::spawn(move || {
                let db = s.read().unwrap();
                assert_eq!(db.len(), 1);
            }));
        }

        for h in handles {
            h.join().unwrap();
        }
    }

    #[test]
    fn test_write_exclusion() {
        let store: SharedVectorStore = Arc::new(RwLock::new(VectorStore::new()));
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let mut handles = vec![];
        for i in 0..10 {
            let s = Arc::clone(&store);
            let c = Arc::clone(&counter);
            handles.push(thread::spawn(move || {
                let mut db = s.write().unwrap();
                db.insert(format!("item_{}", i), vec![i as f32]);
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 10);
        assert_eq!(store.read().unwrap().len(), 10);
    }
}
