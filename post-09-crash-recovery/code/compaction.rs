// post-09-crash-recovery/code/compaction.rs
// Focused implementation of the compaction algorithm
//
// Run with: rustc compaction.rs && ./compaction

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ============================================================================
// Compaction Configuration
// ============================================================================

/// When to trigger automatic compaction
#[derive(Debug, Clone)]
pub enum CompactionTrigger {
    /// Compact when memtable reaches this many entries
    SizeBased { max_entries: usize },

    /// Compact after this duration since last compaction
    TimeBased { interval: Duration },

    /// Compact when WAL file exceeds this size in bytes
    WalSizeBased { max_bytes: u64 },

    /// Only compact manually
    Manual,
}

impl Default for CompactionTrigger {
    fn default() -> Self {
        CompactionTrigger::SizeBased {
            max_entries: 10_000,
        }
    }
}

// ============================================================================
// Compaction Statistics
// ============================================================================

#[derive(Debug, Default)]
pub struct CompactionStats {
    pub compactions_total: u64,
    pub vectors_compacted: u64,
    pub bytes_written: u64,
    pub last_compaction: Option<Instant>,
    pub total_compaction_time: Duration,
}

impl CompactionStats {
    pub fn record(&mut self, vectors: usize, bytes: u64, duration: Duration) {
        self.compactions_total += 1;
        self.vectors_compacted += vectors as u64;
        self.bytes_written += bytes;
        self.last_compaction = Some(Instant::now());
        self.total_compaction_time += duration;
    }

    pub fn print_summary(&self) {
        println!("Compaction Statistics:");
        println!("  Total compactions: {}", self.compactions_total);
        println!("  Vectors compacted: {}", self.vectors_compacted);
        println!("  Bytes written: {} KB", self.bytes_written / 1024);
        println!("  Total time: {:?}", self.total_compaction_time);
        if self.compactions_total > 0 {
            let avg = self.total_compaction_time / self.compactions_total as u32;
            println!("  Avg time per compaction: {:?}", avg);
        }
    }
}

// ============================================================================
// Simplified Segment Writer
// ============================================================================

/// Write vectors to a segment file (simplified from Post #6)
fn write_segment(path: &Path, vectors: &[Vec<f32>]) -> io::Result<u64> {
    let mut file = File::create(path)?;
    let mut bytes_written = 0u64;

    // Header: MAGIC (4) + VERSION (4) + COUNT (8) + DIMENSION (4) = 20 bytes
    file.write_all(b"VECT")?;
    file.write_all(&1u32.to_le_bytes())?;
    file.write_all(&(vectors.len() as u64).to_le_bytes())?;

    let dimension = vectors.first().map(|v| v.len()).unwrap_or(0) as u32;
    file.write_all(&dimension.to_le_bytes())?;
    bytes_written += 20;

    // Write vectors
    for vec in vectors {
        for &v in vec {
            file.write_all(&v.to_le_bytes())?;
            bytes_written += 4;
        }
    }

    // Critical: sync to disk before returning
    file.sync_all()?;

    Ok(bytes_written)
}

// ============================================================================
// The Compactor
// ============================================================================

pub struct Compactor {
    base_path: PathBuf,
    next_segment_id: u64,
    trigger: CompactionTrigger,
    stats: CompactionStats,
}

impl Compactor {
    pub fn new(base_path: &Path, trigger: CompactionTrigger) -> Self {
        let next_segment_id = Self::scan_for_next_id(base_path);

        Self {
            base_path: base_path.to_path_buf(),
            next_segment_id,
            trigger,
            stats: CompactionStats::default(),
        }
    }

    fn scan_for_next_id(base_path: &Path) -> u64 {
        if !base_path.exists() {
            return 1;
        }

        fs::read_dir(base_path)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .and_then(|n| n.strip_prefix("segment_"))
                    .and_then(|n| n.parse::<u64>().ok())
            })
            .max()
            .map(|id| id + 1)
            .unwrap_or(1)
    }

    /// Check if compaction should be triggered
    pub fn should_compact(&self, memtable_size: usize, wal_size: u64) -> bool {
        match &self.trigger {
            CompactionTrigger::SizeBased { max_entries } => memtable_size >= *max_entries,
            CompactionTrigger::TimeBased { interval } => {
                match self.stats.last_compaction {
                    Some(last) => last.elapsed() >= *interval,
                    None => memtable_size > 0, // First compaction if we have data
                }
            }
            CompactionTrigger::WalSizeBased { max_bytes } => wal_size >= *max_bytes,
            CompactionTrigger::Manual => false,
        }
    }

    /// Perform compaction: MemTable → Segment file
    ///
    /// Returns the path to the new segment file
    pub fn compact(&mut self, memtable: &HashMap<String, Vec<f32>>) -> io::Result<PathBuf> {
        if memtable.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot compact empty memtable",
            ));
        }

        let start_time = Instant::now();
        let vector_count = memtable.len();

        println!("┌─ Compaction Start ─────────────────────────────────────┐");
        println!(
            "│ Vectors to compact: {:>6}                            │",
            vector_count
        );

        // === Step 1: Prepare data ===
        let vectors: Vec<Vec<f32>> = memtable.values().cloned().collect();

        // === Step 2: Generate filenames ===
        let segment_id = self.next_segment_id;
        self.next_segment_id += 1;

        let segment_name = format!("segment_{:016}.vec", segment_id);
        let temp_name = format!("{}.tmp", segment_name);

        let segment_path = self.base_path.join(&segment_name);
        let temp_path = self.base_path.join(&temp_name);

        // === Step 3: Write to temporary file ===
        println!("│ Step 1: Writing to {}      │", temp_name);
        let bytes_written = write_segment(&temp_path, &vectors)?;

        // === Step 4: Atomic rename ===
        // This is the critical step - rename is atomic on POSIX/NTFS
        println!("│ Step 2: Atomic rename → {}│", segment_name);
        fs::rename(&temp_path, &segment_path)?;

        // Record stats
        let duration = start_time.elapsed();
        self.stats.record(vector_count, bytes_written, duration);

        println!("│ Step 3: Complete!                                      │");
        println!(
            "│ Time: {:>10?} | Bytes: {:>10} KB             │",
            duration,
            bytes_written / 1024
        );
        println!("└────────────────────────────────────────────────────────┘\n");

        Ok(segment_path)
    }

    pub fn stats(&self) -> &CompactionStats {
        &self.stats
    }
}

// ============================================================================
// Simulated MemTable for Testing
// ============================================================================

struct SimulatedMemTable {
    data: HashMap<String, Vec<f32>>,
    dimension: usize,
}

impl SimulatedMemTable {
    fn new(dimension: usize) -> Self {
        Self {
            data: HashMap::new(),
            dimension,
        }
    }

    fn insert(&mut self, id: String, vector: Vec<f32>) {
        assert_eq!(vector.len(), self.dimension);
        self.data.insert(id, vector);
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn as_map(&self) -> &HashMap<String, Vec<f32>> {
        &self.data
    }

    /// Generate random test data
    fn fill_random(&mut self, count: usize) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        for i in 0..count {
            let id = format!("vec_{:08}", i);

            // Deterministic "random" vector based on index
            let mut hasher = DefaultHasher::new();
            i.hash(&mut hasher);
            let seed = hasher.finish();

            let vector: Vec<f32> = (0..self.dimension)
                .map(|j| {
                    let mut h = DefaultHasher::new();
                    (seed, j).hash(&mut h);
                    (h.finish() as f32 / u64::MAX as f32) * 2.0 - 1.0
                })
                .collect();

            self.data.insert(id, vector);
        }
    }
}

// ============================================================================
// Main: Demonstrate Compaction
// ============================================================================

fn main() -> io::Result<()> {
    let db_path = Path::new("./test_compaction_db");

    // Clean up
    if db_path.exists() {
        fs::remove_dir_all(db_path)?;
    }
    fs::create_dir_all(db_path)?;

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              Compaction Algorithm Demo                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // === Demo 1: Basic Compaction ===
    println!("═══ Demo 1: Basic Compaction ═══\n");
    {
        let mut memtable = SimulatedMemTable::new(128);
        let mut compactor = Compactor::new(db_path, CompactionTrigger::Manual);

        // Insert some vectors
        memtable.insert("user_1".into(), vec![0.1; 128]);
        memtable.insert("user_2".into(), vec![0.2; 128]);
        memtable.insert("user_3".into(), vec![0.3; 128]);

        println!("MemTable has {} vectors\n", memtable.len());

        // Compact
        let segment_path = compactor.compact(memtable.as_map())?;
        println!("Created segment: {:?}\n", segment_path.file_name().unwrap());

        // In real code, we'd now:
        // 1. Truncate the WAL
        // 2. Clear the memtable
        memtable.clear();
    }

    // === Demo 2: Size-Based Auto Compaction ===
    println!("═══ Demo 2: Size-Based Trigger (threshold: 100 vectors) ═══\n");
    {
        let trigger = CompactionTrigger::SizeBased { max_entries: 100 };
        let mut compactor = Compactor::new(db_path, trigger);
        let mut memtable = SimulatedMemTable::new(64);

        // Simulate inserts with auto-compaction check
        for batch in 0..5 {
            // Add 50 vectors per batch
            for i in 0..50 {
                let id = format!("batch{}_{}", batch, i);
                memtable.insert(id, vec![batch as f32; 64]);
            }

            println!(
                "After batch {}: {} vectors in memtable",
                batch,
                memtable.len()
            );

            // Check if we should compact (simulating WAL size = 0)
            if compactor.should_compact(memtable.len(), 0) {
                println!("→ Threshold reached! Triggering compaction...\n");
                compactor.compact(memtable.as_map())?;
                memtable.clear();
            }
        }
    }

    // === Demo 3: Performance Test ===
    println!("\n═══ Demo 3: Performance Test ═══\n");
    {
        let mut compactor = Compactor::new(db_path, CompactionTrigger::Manual);
        let mut memtable = SimulatedMemTable::new(256);

        // Fill with 10,000 vectors
        println!("Generating 10,000 vectors (256 dimensions each)...");
        let gen_start = Instant::now();
        memtable.fill_random(10_000);
        println!("Generated in {:?}\n", gen_start.elapsed());

        // Compact
        compactor.compact(memtable.as_map())?;
    }

    // === Demo 4: Multiple Compactions ===
    println!("═══ Demo 4: Multiple Compaction Cycles ═══\n");
    {
        let mut compactor = Compactor::new(db_path, CompactionTrigger::Manual);
        let mut memtable = SimulatedMemTable::new(128);

        for cycle in 0..3 {
            memtable.fill_random(1_000);
            println!(
                "Cycle {}: Compacting {} vectors...",
                cycle + 1,
                memtable.len()
            );
            compactor.compact(memtable.as_map())?;
            memtable.clear();
        }

        compactor.stats().print_summary();
    }

    // === Demo 5: Crash Simulation ===
    println!("\n═══ Demo 5: Crash During Compaction (Simulated) ═══\n");
    {
        // Create a .tmp file to simulate crash during compaction
        let orphan_tmp = db_path.join("segment_9999999999999999.vec.tmp");
        fs::write(&orphan_tmp, b"This is a partial/corrupt file")?;
        println!(
            "Created orphan temp file: {:?}",
            orphan_tmp.file_name().unwrap()
        );

        // List files before cleanup
        println!("\nFiles before cleanup:");
        for entry in fs::read_dir(db_path)? {
            let entry = entry?;
            let name = entry.file_name();
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let marker = if ext == "tmp" { " ← ORPHAN" } else { "" };
            println!("  {:?}{}", name, marker);
        }

        // Cleanup (this is what VectorStore::open() does)
        println!("\nCleaning up .tmp files...");
        for entry in fs::read_dir(db_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("tmp") {
                println!("  Deleting: {:?}", path.file_name().unwrap());
                fs::remove_file(path)?;
            }
        }

        // List files after cleanup
        println!("\nFiles after cleanup:");
        for entry in fs::read_dir(db_path)? {
            let entry = entry?;
            println!("  {:?}", entry.file_name());
        }
    }

    // Clean up
    fs::remove_dir_all(db_path)?;

    println!("\nAll compaction demos completed!");

    Ok(())
}

// ============================================================================
// The Atomic Rename Guarantee
// ============================================================================

/// This module explains why atomic rename is safe
#[allow(dead_code)]
mod atomic_rename_explained {
    /// On POSIX (Linux, macOS):
    /// - rename() is guaranteed atomic by the POSIX standard
    /// - The kernel performs this as a single metadata operation
    /// - Even if power fails mid-rename, you get either old or new name
    ///
    /// On Windows (NTFS):
    /// - MoveFileEx with MOVEFILE_REPLACE_EXISTING is atomic
    /// - Rust's std::fs::rename uses this under the hood
    ///
    /// The .tmp extension pattern:
    /// 1. Write data to file.tmp
    /// 2. fsync(file.tmp) - data is on disk
    /// 3. rename(file.tmp, file) - atomic switch
    ///
    /// If crash at step 1 or 2: .tmp file is garbage, we delete it
    /// If crash at step 3: Either .tmp exists or final file exists, never both
    ///
    /// This is why databases use this pattern universally:
    /// - SQLite (WAL mode)
    /// - PostgreSQL (pg_xlog)
    /// - LevelDB (SST files)
    /// - RocksDB (SST files)

    pub const EXPLANATION: &str = "See module documentation above";
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let dir = PathBuf::from(format!("./test_compact_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_compaction_creates_segment() {
        let dir = temp_dir();
        let mut compactor = Compactor::new(&dir, CompactionTrigger::Manual);
        let mut data = HashMap::new();
        data.insert("a".into(), vec![1.0, 2.0, 3.0]);

        let path = compactor.compact(&data).unwrap();
        assert!(path.exists());
        assert!(path.extension().unwrap() == "vec");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_size_based_trigger() {
        let trigger = CompactionTrigger::SizeBased { max_entries: 10 };
        let compactor = Compactor::new(Path::new("."), trigger);

        assert!(!compactor.should_compact(5, 0));
        assert!(!compactor.should_compact(9, 0));
        assert!(compactor.should_compact(10, 0));
        assert!(compactor.should_compact(100, 0));
    }

    #[test]
    fn test_wal_size_trigger() {
        let trigger = CompactionTrigger::WalSizeBased { max_bytes: 1024 };
        let compactor = Compactor::new(Path::new("."), trigger);

        assert!(!compactor.should_compact(1000, 500));
        assert!(compactor.should_compact(1, 1024));
        assert!(compactor.should_compact(1, 2048));
    }

    #[test]
    fn test_empty_memtable_error() {
        let dir = temp_dir();
        let mut compactor = Compactor::new(&dir, CompactionTrigger::Manual);
        let empty: HashMap<String, Vec<f32>> = HashMap::new();

        let result = compactor.compact(&empty);
        assert!(result.is_err());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_segment_id_increment() {
        let dir = temp_dir();
        let mut compactor = Compactor::new(&dir, CompactionTrigger::Manual);

        let mut data = HashMap::new();
        data.insert("x".into(), vec![1.0]);

        let path1 = compactor.compact(&data).unwrap();
        let path2 = compactor.compact(&data).unwrap();

        assert_ne!(path1, path2);
        assert!(path1.to_str().unwrap().contains("segment_0000000000000001"));
        assert!(path2.to_str().unwrap().contains("segment_0000000000000002"));

        fs::remove_dir_all(&dir).unwrap();
    }
}
