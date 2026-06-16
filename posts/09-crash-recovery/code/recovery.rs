// post-09-crash-recovery/code/recovery.rs
// Complete VectorStore with startup recovery and WAL replay
//
// Run with: rustc recovery.rs && ./recovery

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

// ============================================================================
// WAL Entry Types (from Post #8)
// ============================================================================

#[derive(Debug, Clone)]
pub enum WalEntry {
    Insert { id: String, vector: Vec<f32> },
    Delete { id: String },
}

impl WalEntry {
    const TAG_INSERT: u8 = 1;
    const TAG_DELETE: u8 = 2;

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            WalEntry::Insert { id, vector } => {
                buf.push(Self::TAG_INSERT);
                // ID length + ID bytes
                buf.extend(&(id.len() as u32).to_le_bytes());
                buf.extend(id.as_bytes());
                // Vector length + vector data
                buf.extend(&(vector.len() as u32).to_le_bytes());
                for &v in vector {
                    buf.extend(&v.to_le_bytes());
                }
            }
            WalEntry::Delete { id } => {
                buf.push(Self::TAG_DELETE);
                buf.extend(&(id.len() as u32).to_le_bytes());
                buf.extend(id.as_bytes());
            }
        }
        buf
    }

    pub fn from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Empty entry"));
        }

        let tag = data[0];
        let mut pos = 1;

        // Read ID
        if pos + 4 > data.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Truncated ID length"));
        }
        let id_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;

        if pos + id_len > data.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Truncated ID"));
        }
        let id = String::from_utf8(data[pos..pos + id_len].to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        pos += id_len;

        match tag {
            Self::TAG_INSERT => {
                if pos + 4 > data.len() {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Truncated vector length"));
                }
                let vec_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                pos += 4;
                let mut vector = Vec::with_capacity(vec_len);
                for _ in 0..vec_len {
                    if pos + 4 > data.len() {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Truncated vector data"));
                    }
                    let v = f32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
                    vector.push(v);
                    pos += 4;
                }
                Ok(WalEntry::Insert { id, vector })
            }
            Self::TAG_DELETE => Ok(WalEntry::Delete { id }),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown tag")),
        }
    }
}

// ============================================================================
// CRC32 (simple implementation for standalone compilation)
// ============================================================================

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB88320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

// ============================================================================
// Write-Ahead Log (from Post #8)
// ============================================================================

pub struct WriteAheadLog {
    file: BufWriter<File>,
    path: PathBuf,
}

impl WriteAheadLog {
    pub fn open(path: &str) -> io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self {
            file: BufWriter::new(file),
            path: PathBuf::from(path),
        })
    }

    pub fn append(&mut self, entry: &WalEntry) -> io::Result<()> {
        let payload = entry.to_bytes();
        let len_bytes = (payload.len() as u32).to_le_bytes();

        // Compute CRC over length + payload (matching blog Post #8 format)
        let mut crc_input = Vec::with_capacity(4 + payload.len());
        crc_input.extend(&len_bytes);
        crc_input.extend(&payload);
        let crc = crc32(&crc_input);

        // Write: CRC (4) + Length (4) + Payload (N)
        self.file.write_all(&crc.to_le_bytes())?;
        self.file.write_all(&len_bytes)?;
        self.file.write_all(&payload)?;
        self.file.flush()?;

        Ok(())
    }

    /// Read all valid entries from the WAL
    pub fn read_all(path: &str) -> io::Result<Vec<WalEntry>> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };

        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();

        loop {
            // Read CRC
            let mut crc_buf = [0u8; 4];
            match reader.read_exact(&mut crc_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
            let stored_crc = u32::from_le_bytes(crc_buf);

            // Read length
            let mut len_buf = [0u8; 4];
            reader.read_exact(&mut len_buf)?;
            let length = u32::from_le_bytes(len_buf) as usize;

            // Read payload
            let mut payload = vec![0u8; length];
            reader.read_exact(&mut payload)?;

            // Verify CRC (computed over length + payload)
            let mut crc_input = Vec::with_capacity(4 + length);
            crc_input.extend(&len_buf);
            crc_input.extend(&payload);
            let computed_crc = crc32(&crc_input);
            if stored_crc != computed_crc {
                eprintln!("CRC mismatch! Stopping replay at this point.");
                break;
            }

            // Deserialize
            let entry = WalEntry::from_bytes(&payload)?;
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Clear the WAL file, resetting it to empty
    pub fn truncate(&mut self) -> io::Result<()> {
        // Close and reopen with truncation
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)?;

        // Sync the truncation
        file.sync_all()?;

        // Reopen for appending
        self.file = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?,
        );

        Ok(())
    }
}

// ============================================================================
// Simplified Segment (mock for demonstration)
// ============================================================================

/// A simplified segment for demonstration purposes
/// In production, this would be MmapSegment from Post #7
pub struct Segment {
    pub path: PathBuf,
    pub vectors: Vec<Vec<f32>>, // In reality, this would be mmap'd
}

impl Segment {
    /// Create a new segment file with the given vectors
    pub fn create(path: &Path, vectors: &[Vec<f32>]) -> io::Result<Self> {
        let mut file = File::create(path)?;

        // Write header: MAGIC (4) + VERSION (4) + COUNT (8) + DIMENSION (4)
        file.write_all(b"VECT")?; // Magic
        file.write_all(&1u32.to_le_bytes())?; // Version
        file.write_all(&(vectors.len() as u64).to_le_bytes())?; // Count

        let dimension = vectors.first().map(|v| v.len()).unwrap_or(0) as u32;
        file.write_all(&dimension.to_le_bytes())?; // Dimension

        // Write vectors
        for vec in vectors {
            for &v in vec {
                file.write_all(&v.to_le_bytes())?;
            }
        }

        // Sync to disk
        file.sync_all()?;

        Ok(Self {
            path: path.to_path_buf(),
            vectors: vectors.to_vec(),
        })
    }

    /// Open an existing segment file
    pub fn open(path: &Path) -> io::Result<Self> {
        let mut file = File::open(path)?;

        // Read header
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != b"VECT" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Bad magic"));
        }

        let mut version_buf = [0u8; 4];
        file.read_exact(&mut version_buf)?;
        let _version = u32::from_le_bytes(version_buf);

        let mut count_buf = [0u8; 8];
        file.read_exact(&mut count_buf)?;
        let count = u64::from_le_bytes(count_buf) as usize;

        let mut dim_buf = [0u8; 4];
        file.read_exact(&mut dim_buf)?;
        let dimension = u32::from_le_bytes(dim_buf) as usize;

        // Read vectors
        let mut vectors = Vec::with_capacity(count);
        for _ in 0..count {
            let mut vec = Vec::with_capacity(dimension);
            for _ in 0..dimension {
                let mut v_buf = [0u8; 4];
                file.read_exact(&mut v_buf)?;
                vec.push(f32::from_le_bytes(v_buf));
            }
            vectors.push(vec);
        }

        Ok(Self {
            path: path.to_path_buf(),
            vectors,
        })
    }

    pub fn len(&self) -> usize {
        self.vectors.len()
    }
}

// ============================================================================
// VectorStore with Recovery
// ============================================================================

pub struct VectorStore {
    // === Hot Path (Recent Data) ===
    memtable: HashMap<String, Vec<f32>>,
    wal: WriteAheadLog,

    // === Cold Path (Historical Data) ===
    segments: Vec<Segment>,

    // === Configuration ===
    base_path: PathBuf,
    next_segment_id: u64,
}

impl VectorStore {
    /// Initialize the store by loading segments and replaying the WAL
    pub fn open(base_path: &Path) -> io::Result<Self> {
        println!("Opening VectorStore at {:?}", base_path);

        // Ensure the directory exists
        fs::create_dir_all(base_path)?;

        let wal_path = base_path.join("wal");

        // === Phase 1: Load Existing Segments ===
        let segments = Self::load_segments(base_path)?;
        println!("  Phase 1: Loaded {} existing segment(s)", segments.len());

        // === Phase 2: Replay WAL into MemTable ===
        let memtable = Self::replay_wal(&wal_path)?;
        println!(
            "  Phase 2: Replayed WAL → {} active vector(s)",
            memtable.len()
        );

        // === Phase 3: Clean up failed compactions ===
        let cleaned = Self::cleanup_temp_files(base_path)?;
        if cleaned > 0 {
            println!("  Phase 3: Deleted {} orphan temp file(s)", cleaned);
        }

        // === Phase 4: Compute next segment ID ===
        let next_segment_id = Self::compute_next_segment_id(&segments);
        println!("  Phase 4: Next segment ID = {}", next_segment_id);

        // === Phase 5: Open WAL for new writes ===
        let wal = WriteAheadLog::open(wal_path.to_str().unwrap())?;

        println!("VectorStore ready!\n");

        Ok(Self {
            memtable,
            wal,
            segments,
            base_path: base_path.to_path_buf(),
            next_segment_id,
        })
    }

    /// Replay WAL entries into a HashMap
    fn replay_wal(wal_path: &Path) -> io::Result<HashMap<String, Vec<f32>>> {
        let mut memtable = HashMap::new();

        let entries = WriteAheadLog::read_all(wal_path.to_str().unwrap_or(""))?;

        for entry in entries {
            match entry {
                WalEntry::Insert { id, vector } => {
                    memtable.insert(id, vector);
                }
                WalEntry::Delete { id } => {
                    memtable.remove(&id);
                }
            }
        }

        Ok(memtable)
    }

    /// Load all .vec segment files from the directory
    fn load_segments(base_path: &Path) -> io::Result<Vec<Segment>> {
        let mut segments = Vec::new();

        if !base_path.exists() {
            return Ok(segments);
        }

        for entry in fs::read_dir(base_path)? {
            let entry = entry?;
            let path = entry.path();

            // Only load .vec files (not .tmp!)
            if path.extension().and_then(|s| s.to_str()) == Some("vec") {
                match Segment::open(&path) {
                    Ok(segment) => {
                        println!(
                            "    Loaded: {:?} ({} vectors)",
                            path.file_name().unwrap(),
                            segment.len()
                        );
                        segments.push(segment);
                    }
                    Err(e) => {
                        eprintln!("    Warning: Failed to load {:?}: {}", path, e);
                    }
                }
            }
        }

        // Sort by filename to maintain order
        segments.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(segments)
    }

    /// Delete any .tmp files (failed compaction artifacts)
    fn cleanup_temp_files(base_path: &Path) -> io::Result<usize> {
        let mut count = 0;

        for entry in fs::read_dir(base_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("tmp") {
                println!("    Deleting orphan: {:?}", path.file_name().unwrap());
                fs::remove_file(path)?;
                count += 1;
            }
        }

        Ok(count)
    }

    /// Compute the next segment ID based on existing segments
    fn compute_next_segment_id(segments: &[Segment]) -> u64 {
        segments
            .iter()
            .filter_map(|s| {
                s.path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .and_then(|n| n.strip_prefix("segment_"))
                    .and_then(|n| n.parse::<u64>().ok())
            })
            .max()
            .map(|id| id + 1)
            .unwrap_or(1)
    }

    // === Write Operations ===

    pub fn insert(&mut self, id: String, vector: Vec<f32>) -> io::Result<()> {
        // 1. WAL first (durability)
        self.wal.append(&WalEntry::Insert {
            id: id.clone(),
            vector: vector.clone(),
        })?;

        // 2. MemTable second (availability)
        self.memtable.insert(id, vector);

        Ok(())
    }

    pub fn delete(&mut self, id: &str) -> io::Result<()> {
        // 1. WAL first
        self.wal.append(&WalEntry::Delete { id: id.to_string() })?;

        // 2. MemTable second
        self.memtable.remove(id);

        Ok(())
    }

    // === Read Operations ===

    pub fn get(&self, id: &str) -> Option<&Vec<f32>> {
        // Check MemTable first (hot data)
        self.memtable.get(id)

        // Note: A real implementation would also search segments
        // This requires an ID → index mapping which we'll cover in Post #11
    }

    pub fn memtable_len(&self) -> usize {
        self.memtable.len()
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    // === Compaction ===

    pub fn compact(&mut self) -> io::Result<()> {
        if self.memtable.is_empty() {
            println!("Nothing to compact (memtable is empty)");
            return Ok(());
        }

        let vector_count = self.memtable.len();
        println!("Compacting {} vectors to disk...", vector_count);

        // === Step 1: Prepare the data ===
        let vectors: Vec<Vec<f32>> = self.memtable.values().cloned().collect();

        // === Step 2: Generate filenames ===
        let segment_id = self.next_segment_id;
        self.next_segment_id += 1;

        let segment_name = format!("segment_{:016}.vec", segment_id);
        let temp_name = format!("{}.tmp", segment_name);

        let segment_path = self.base_path.join(&segment_name);
        let temp_path = self.base_path.join(&temp_name);

        // === Step 3: Write to temporary file ===
        println!("  Writing to {}", temp_name);
        Segment::create(&temp_path, &vectors)?;

        // === Step 4: Atomic Rename ===
        fs::rename(&temp_path, &segment_path)?;
        println!("  Renamed to {}", segment_name);

        // === Step 5: Open as Segment ===
        let segment = Segment::open(&segment_path)?;
        self.segments.push(segment);

        // === Step 6: Truncate WAL ===
        self.wal.truncate()?;
        println!("  WAL truncated");

        // === Step 7: Clear MemTable ===
        self.memtable.clear();

        println!("Compaction complete!\n");
        Ok(())
    }

    /// Graceful shutdown with final compaction
    pub fn shutdown(&mut self) -> io::Result<()> {
        println!("Shutting down VectorStore...");
        if !self.memtable.is_empty() {
            self.compact()?;
        }
        println!("Shutdown complete.");
        Ok(())
    }
}

// ============================================================================
// Main: Demonstrate Recovery
// ============================================================================

fn main() -> io::Result<()> {
    let db_path = Path::new("./test_recovery_db");

    // Clean up from previous runs
    if db_path.exists() {
        fs::remove_dir_all(db_path)?;
    }

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          Crash Recovery Demonstration                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // === Run 1: Initial writes (simulating a session that crashes) ===
    println!("═══ RUN 1: Initial Writes (will 'crash' without compacting) ═══\n");
    {
        let mut store = VectorStore::open(db_path)?;

        store.insert("vec_1".into(), vec![1.0, 0.0, 0.0])?;
        store.insert("vec_2".into(), vec![0.0, 1.0, 0.0])?;
        store.insert("vec_3".into(), vec![0.0, 0.0, 1.0])?;

        println!("Inserted 3 vectors");
        println!("MemTable size: {}", store.memtable_len());
        println!("Segment count: {}", store.segment_count());
        println!("\n[Simulating crash - dropping without compaction]\n");
        // Simulate crash: drop without calling shutdown()
    }

    // === Run 2: Recovery from WAL ===
    println!("═══ RUN 2: Recovery (replaying WAL) ═══\n");
    {
        let mut store = VectorStore::open(db_path)?;

        println!("After recovery:");
        println!(
            "  MemTable size: {} (recovered from WAL!)",
            store.memtable_len()
        );
        println!("  Segment count: {}", store.segment_count());

        // Verify data
        assert_eq!(store.get("vec_1"), Some(&vec![1.0, 0.0, 0.0]));
        assert_eq!(store.get("vec_2"), Some(&vec![0.0, 1.0, 0.0]));
        assert_eq!(store.get("vec_3"), Some(&vec![0.0, 0.0, 1.0]));
        println!("  All 3 vectors verified!\n");

        // Now do a proper compaction
        println!("Now compacting...\n");
        store.compact()?;

        println!("After compaction:");
        println!("  MemTable size: {}", store.memtable_len());
        println!("  Segment count: {}", store.segment_count());
        println!();
    }

    // === Run 3: Loading from segment ===
    println!("═══ RUN 3: Fresh Start (loading from segment) ═══\n");
    {
        let mut store = VectorStore::open(db_path)?;

        println!("Final state:");
        println!(
            "  MemTable size: {} (WAL was truncated)",
            store.memtable_len()
        );
        println!(
            "  Segment count: {} (data is in segment)",
            store.segment_count()
        );

        // Add more data
        store.insert("vec_4".into(), vec![1.0, 1.0, 0.0])?;
        store.insert("vec_5".into(), vec![0.0, 1.0, 1.0])?;

        println!("\nAdded 2 more vectors:");
        println!("  MemTable size: {}", store.memtable_len());

        // Proper shutdown
        store.shutdown()?;
    }

    // === Run 4: Final verification ===
    println!("\n═══ RUN 4: Final Verification ═══\n");
    {
        let store = VectorStore::open(db_path)?;

        println!("Final state:");
        println!("  MemTable size: {}", store.memtable_len());
        println!("  Segment count: {}", store.segment_count());
    }

    // Clean up
    fs::remove_dir_all(db_path)?;

    println!("\nAll recovery scenarios passed!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir = PathBuf::from(format!("./test_recovery_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn test_basic_recovery() {
        let dir = temp_dir();

        // Write some data
        {
            let mut store = VectorStore::open(&dir).unwrap();
            store.insert("a".into(), vec![1.0, 2.0]).unwrap();
            store.insert("b".into(), vec![3.0, 4.0]).unwrap();
        }

        // Recover
        {
            let store = VectorStore::open(&dir).unwrap();
            assert_eq!(store.memtable_len(), 2);
            assert_eq!(store.get("a"), Some(&vec![1.0, 2.0]));
            assert_eq!(store.get("b"), Some(&vec![3.0, 4.0]));
        }

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_recovery_with_deletes() {
        let dir = temp_dir();

        // Write and delete
        {
            let mut store = VectorStore::open(&dir).unwrap();
            store.insert("a".into(), vec![1.0]).unwrap();
            store.insert("b".into(), vec![2.0]).unwrap();
            store.delete("a").unwrap();
        }

        // Recover
        {
            let store = VectorStore::open(&dir).unwrap();
            assert_eq!(store.memtable_len(), 1);
            assert_eq!(store.get("a"), None);
            assert_eq!(store.get("b"), Some(&vec![2.0]));
        }

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_compaction() {
        let dir = temp_dir();

        {
            let mut store = VectorStore::open(&dir).unwrap();
            store.insert("x".into(), vec![1.0, 2.0, 3.0]).unwrap();
            store.insert("y".into(), vec![4.0, 5.0, 6.0]).unwrap();
            store.compact().unwrap();

            assert_eq!(store.memtable_len(), 0);
            assert_eq!(store.segment_count(), 1);
        }

        // After compaction, WAL should be empty
        {
            let store = VectorStore::open(&dir).unwrap();
            assert_eq!(store.memtable_len(), 0); // WAL was truncated
            assert_eq!(store.segment_count(), 1);
        }

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_temp_file_cleanup() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();

        // Create an orphan .tmp file
        let tmp_file = dir.join("segment_000.vec.tmp");
        fs::write(&tmp_file, b"garbage").unwrap();

        // Opening should clean it up
        {
            let _store = VectorStore::open(&dir).unwrap();
        }

        assert!(!tmp_file.exists(), "Temp file should be deleted");

        fs::remove_dir_all(&dir).unwrap();
    }
}
