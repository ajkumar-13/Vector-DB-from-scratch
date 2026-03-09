// wal.rs
//
// Complete Write-Ahead Log implementation.
// From Post #8: Write-Ahead Log (WAL)
//
// This provides a production-ready WAL with:
// - CRC32 checksums for corruption detection
// - Configurable sync policies
// - Crash recovery via replay
//
// Dependencies (Cargo.toml):
//   crc32fast = "1.3"
//   bincode = "1.3"  (optional, we use custom serialization)
//   serde = { version = "1.0", features = ["derive"] }

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, Write};
use std::path::Path;
use std::time::{Duration, Instant};

// ═══════════════════════════════════════════════════════════════════════════
// WAL ENTRY (inline for standalone compilation)
// NOTE: The blog recommends using `bincode` + `serde` for serialization,
// which is the production approach. Here we use custom serialization so
// this file compiles standalone without Cargo dependencies.
// ═══════════════════════════════════════════════════════════════════════════

/// Operations that can be logged
#[derive(Debug, Clone, PartialEq)]
pub enum WalEntry {
    Insert {
        id: String,
        vector: Vec<f32>,
        metadata: HashMap<String, String>,
    },
    Delete {
        id: String,
    },
}

impl WalEntry {
    pub fn insert(id: impl Into<String>, vector: Vec<f32>) -> Self {
        Self::Insert {
            id: id.into(),
            vector,
            metadata: HashMap::new(),
        }
    }

    pub fn delete(id: impl Into<String>) -> Self {
        Self::Delete { id: id.into() }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            WalEntry::Insert {
                id,
                vector,
                metadata,
            } => {
                buf.push(0x01); // Type marker

                // ID
                buf.extend(&(id.len() as u32).to_le_bytes());
                buf.extend(id.as_bytes());

                // Vector
                buf.extend(&(vector.len() as u32).to_le_bytes());
                for &val in vector {
                    buf.extend(&val.to_le_bytes());
                }

                // Metadata
                buf.extend(&(metadata.len() as u32).to_le_bytes());
                for (key, value) in metadata {
                    buf.extend(&(key.len() as u32).to_le_bytes());
                    buf.extend(key.as_bytes());
                    buf.extend(&(value.len() as u32).to_le_bytes());
                    buf.extend(value.as_bytes());
                }
            }
            WalEntry::Delete { id } => {
                buf.push(0x02);
                buf.extend(&(id.len() as u32).to_le_bytes());
                buf.extend(id.as_bytes());
            }
        }

        buf
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.is_empty() {
            return Err("Empty data".to_string());
        }

        let mut pos = 0;
        let entry_type = data[pos];
        pos += 1;

        match entry_type {
            0x01 => {
                // Insert - read ID
                if pos + 4 > data.len() {
                    return Err("Truncated ID length".to_string());
                }
                let id_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                pos += 4;

                if pos + id_len > data.len() {
                    return Err("Truncated ID".to_string());
                }
                let id = String::from_utf8(data[pos..pos + id_len].to_vec())
                    .map_err(|e| e.to_string())?;
                pos += id_len;

                // Read vector
                if pos + 4 > data.len() {
                    return Err("Truncated vector length".to_string());
                }
                let vec_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                pos += 4;

                let mut vector = Vec::with_capacity(vec_len);
                for _ in 0..vec_len {
                    if pos + 4 > data.len() {
                        return Err("Truncated vector data".to_string());
                    }
                    vector.push(f32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()));
                    pos += 4;
                }

                // Read metadata
                if pos + 4 > data.len() {
                    return Err("Truncated metadata count".to_string());
                }
                let meta_count =
                    u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                pos += 4;
                let mut metadata = HashMap::new();
                for _ in 0..meta_count {
                    if pos + 4 > data.len() {
                        return Err("Truncated metadata key length".to_string());
                    }
                    let key_len =
                        u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                    pos += 4;

                    if pos + key_len > data.len() {
                        return Err("Truncated metadata key".to_string());
                    }
                    let key = String::from_utf8(data[pos..pos + key_len].to_vec())
                        .map_err(|e| e.to_string())?;
                    pos += key_len;

                    if pos + 4 > data.len() {
                        return Err("Truncated metadata value length".to_string());
                    }
                    let val_len =
                        u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                    pos += 4;

                    if pos + val_len > data.len() {
                        return Err("Truncated metadata value".to_string());
                    }
                    let value = String::from_utf8(data[pos..pos + val_len].to_vec())
                        .map_err(|e| e.to_string())?;
                    pos += val_len;

                    metadata.insert(key, value);
                }

                Ok(WalEntry::Insert {
                    id,
                    vector,
                    metadata,
                })
            }
            0x02 => {
                if pos + 4 > data.len() {
                    return Err("Truncated ID length".to_string());
                }
                let id_len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
                pos += 4;

                if pos + id_len > data.len() {
                    return Err("Truncated ID".to_string());
                }
                let id = String::from_utf8(data[pos..pos + id_len].to_vec())
                    .map_err(|e| e.to_string())?;
                Ok(WalEntry::Delete { id })
            }
            _ => Err(format!("Unknown entry type: 0x{:02X}", entry_type)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CRC32 (simple implementation without external crate for demo)
// ═══════════════════════════════════════════════════════════════════════════

/// Compute CRC32 checksum (IEEE polynomial)
fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

// ═══════════════════════════════════════════════════════════════════════════
// SYNC POLICY
// ═══════════════════════════════════════════════════════════════════════════

/// Configuration for when to sync data to disk
#[derive(Debug, Clone)]
pub enum SyncPolicy {
    /// Sync after every write (safest, slowest)
    Always,
    /// Sync after N writes
    EveryN(usize),
    /// Sync after duration since last sync
    Periodic(Duration),
    /// Never auto-sync (caller must call sync manually)
    Manual,
}

// ═══════════════════════════════════════════════════════════════════════════
// WRITE-AHEAD LOG
// ═══════════════════════════════════════════════════════════════════════════

/// A durable append-only log for recording database operations.
///
/// The WAL ensures that all writes are persisted to disk before being
/// applied to the in-memory state, allowing recovery after crashes.
pub struct WriteAheadLog {
    writer: BufWriter<File>,
    path: String,
    sync_policy: SyncPolicy,
    entries_since_sync: usize,
    last_sync: Instant,
    /// Number of entries written in this session (resets on restart)
    session_entries: u64,
    file_size: u64,
}

impl WriteAheadLog {
    /// Open or create a WAL file
    pub fn open(path: &str) -> io::Result<Self> {
        Self::open_with_policy(path, SyncPolicy::EveryN(100))
    }

    /// Open with a specific sync policy
    pub fn open_with_policy(path: &str, sync_policy: SyncPolicy) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true) // Append mode - never overwrite
            .open(path)?;

        let file_size = file.metadata()?.len();

        Ok(Self {
            writer: BufWriter::new(file),
            path: path.to_string(),
            sync_policy,
            entries_since_sync: 0,
            last_sync: Instant::now(),
            session_entries: 0,
            file_size,
        })
    }

    /// Append an entry to the log
    ///
    /// Entry format:
    /// ```text
    /// ┌──────────────┬──────────────┬──────────────────────┐
    /// │ CRC32 (4B)   │ Length (4B)  │ Payload (N bytes)    │
    /// └──────────────┴──────────────┴──────────────────────┘
    /// ```
    pub fn append(&mut self, entry: &WalEntry) -> io::Result<()> {
        // 1. Serialize the entry
        let payload = entry.to_bytes();

        // 2. Compute CRC32 of (length + payload)
        let len_bytes = (payload.len() as u32).to_le_bytes();
        let mut crc_input = Vec::with_capacity(4 + payload.len());
        crc_input.extend(&len_bytes);
        crc_input.extend(&payload);
        let crc = crc32(&crc_input);

        // 3. Write: CRC (4) + Length (4) + Payload
        self.writer.write_all(&crc.to_le_bytes())?;
        self.writer.write_all(&len_bytes)?;
        self.writer.write_all(&payload)?;

        self.entries_since_sync += 1;
        self.session_entries += 1;
        self.file_size += 8 + payload.len() as u64;

        // 4. Maybe sync based on policy
        self.maybe_sync()?;

        Ok(())
    }

    /// Force all buffered data to physical disk
    pub fn sync(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        self.writer.get_ref().sync_all()?;
        self.entries_since_sync = 0;
        self.last_sync = Instant::now();
        Ok(())
    }

    /// Sync based on the configured policy
    fn maybe_sync(&mut self) -> io::Result<()> {
        let should_sync = match &self.sync_policy {
            SyncPolicy::Always => true,
            SyncPolicy::EveryN(n) => self.entries_since_sync >= *n,
            SyncPolicy::Periodic(duration) => self.last_sync.elapsed() >= *duration,
            SyncPolicy::Manual => false,
        };

        if should_sync {
            self.sync()?;
        }

        Ok(())
    }

    /// Get the current file size
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    /// Get the number of entries written in this session
    pub fn entries_written(&self) -> u64 {
        self.session_entries
    }

    /// Get the file path
    pub fn path(&self) -> &str {
        &self.path
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WAL READER
// ═══════════════════════════════════════════════════════════════════════════

/// Statistics from reading a WAL file
#[derive(Debug, Default)]
pub struct WalReadStats {
    pub entries_read: u64,
    pub bytes_read: u64,
    pub corrupted_entries: u64,
    pub truncated: bool,
}

/// Read all valid entries from a WAL file
pub fn read_wal(path: &str) -> io::Result<(Vec<WalEntry>, WalReadStats)> {
    if !Path::new(path).exists() {
        return Ok((Vec::new(), WalReadStats::default()));
    }

    let file = File::open(path)?;
    let file_size = file.metadata()?.len();
    let mut reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut stats = WalReadStats::default();

    loop {
        // Record position for error reporting
        let pos = reader.stream_position()?;

        // 1. Read CRC (4 bytes)
        let mut crc_buf = [0u8; 4];
        match reader.read_exact(&mut crc_buf) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // Clean end of file
                break;
            }
            Err(e) => return Err(e),
        }
        let stored_crc = u32::from_le_bytes(crc_buf);

        // 2. Read length (4 bytes)
        let mut len_buf = [0u8; 4];
        if reader.read_exact(&mut len_buf).is_err() {
            eprintln!(
                "⚠️ WAL truncated at offset {}: incomplete length field",
                pos
            );
            stats.truncated = true;
            break;
        }
        let len = u32::from_le_bytes(len_buf) as usize;

        // Sanity check length
        if len > 100_000_000 {
            // 100MB max entry size
            eprintln!(
                "⚠️ WAL corrupted at offset {}: unreasonable length {}",
                pos, len
            );
            stats.corrupted_entries += 1;
            break;
        }

        // 3. Read payload
        let mut payload = vec![0u8; len];
        if reader.read_exact(&mut payload).is_err() {
            eprintln!("⚠️ WAL truncated at offset {}: incomplete payload", pos);
            stats.truncated = true;
            break;
        }

        // 4. Verify CRC
        let mut crc_input = Vec::with_capacity(4 + len);
        crc_input.extend(&len_buf);
        crc_input.extend(&payload);
        let computed_crc = crc32(&crc_input);

        if computed_crc != stored_crc {
            eprintln!(
                "⚠️ CRC mismatch at offset {}: stored={:08X}, computed={:08X}",
                pos, stored_crc, computed_crc
            );
            stats.corrupted_entries += 1;
            break;
        }

        // 5. Deserialize
        match WalEntry::from_bytes(&payload) {
            Ok(entry) => {
                entries.push(entry);
                stats.entries_read += 1;
                stats.bytes_read += 8 + len as u64;
            }
            Err(e) => {
                eprintln!("⚠️ Failed to deserialize entry at offset {}: {}", pos, e);
                stats.corrupted_entries += 1;
                break;
            }
        }
    }

    Ok((entries, stats))
}

// ═══════════════════════════════════════════════════════════════════════════
// VECTOR STORE (integration example)
// ═══════════════════════════════════════════════════════════════════════════

/// A simple vector store backed by WAL
pub struct VectorStore {
    vectors: HashMap<String, Vec<f32>>,
    wal: WriteAheadLog,
}

impl VectorStore {
    /// Create a new store, replaying any existing WAL
    pub fn new(wal_path: &str) -> io::Result<Self> {
        // 1. Read existing WAL
        let (entries, stats) = read_wal(wal_path)?;

        println!(
            "WAL recovery: {} entries, {} bytes",
            stats.entries_read, stats.bytes_read
        );
        if stats.corrupted_entries > 0 {
            println!(
                "  ⚠️ {} corrupted entries discarded",
                stats.corrupted_entries
            );
        }

        // 2. Replay into HashMap
        let mut vectors = HashMap::new();
        for entry in entries {
            match entry {
                WalEntry::Insert { id, vector, .. } => {
                    vectors.insert(id, vector);
                }
                WalEntry::Delete { id } => {
                    vectors.remove(&id);
                }
            }
        }

        println!("  Recovered {} vectors", vectors.len());

        // 3. Open WAL for appending
        let wal = WriteAheadLog::open(wal_path)?;

        Ok(Self { vectors, wal })
    }

    /// Insert a vector
    ///
    /// Note: Durability depends on the WAL's sync policy. With the default
    /// `SyncPolicy::EveryN(100)`, data may remain in the BufWriter buffer
    /// until the next sync. Call `sync()` for immediate durability.
    pub fn insert(&mut self, id: String, vector: Vec<f32>) -> io::Result<()> {
        // WAL first! (write intention to disk before updating memory)
        self.wal.append(&WalEntry::insert(&id, vector.clone()))?;

        // Then memory
        self.vectors.insert(id, vector);
        Ok(())
    }

    /// Delete a vector
    pub fn delete(&mut self, id: &str) -> io::Result<bool> {
        if !self.vectors.contains_key(id) {
            return Ok(false);
        }

        // WAL first!
        self.wal.append(&WalEntry::delete(id))?;

        // Then memory
        self.vectors.remove(id);
        Ok(true)
    }

    /// Get a vector
    pub fn get(&self, id: &str) -> Option<&Vec<f32>> {
        self.vectors.get(id)
    }

    /// Get the number of vectors
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Flush WAL to disk
    pub fn sync(&mut self) -> io::Result<()> {
        self.wal.sync()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MAIN - DEMONSTRATION
// ═══════════════════════════════════════════════════════════════════════════

fn main() -> io::Result<()> {
    println!();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           WRITE-AHEAD LOG DEMONSTRATION                   ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    let wal_path = "demo.wal";

    // Clean up from previous run
    if Path::new(wal_path).exists() {
        std::fs::remove_file(wal_path)?;
    }

    // ─────────────────────────────────────────────────────────────────────
    // PHASE 1: Write some data
    // ─────────────────────────────────────────────────────────────────────
    println!("═══════════════════════════════════════════════════════════");
    println!("  PHASE 1: Initial writes");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    {
        let mut store = VectorStore::new(wal_path)?;

        store.insert("vec-001".into(), vec![0.1, 0.2, 0.3])?;
        store.insert("vec-002".into(), vec![0.4, 0.5, 0.6])?;
        store.insert("vec-003".into(), vec![0.7, 0.8, 0.9])?;
        store.sync()?;

        println!("Inserted 3 vectors");
        println!("Store size: {}", store.len());
        println!("WAL size: {} bytes", store.wal.file_size());
    }
    // Store dropped here - simulates shutdown/crash
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // PHASE 2: Restart and verify recovery
    // ─────────────────────────────────────────────────────────────────────
    println!("═══════════════════════════════════════════════════════════");
    println!("  PHASE 2: Recovery after 'crash'");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    {
        let store = VectorStore::new(wal_path)?;

        println!("After recovery:");
        println!("  vec-001: {:?}", store.get("vec-001"));
        println!("  vec-002: {:?}", store.get("vec-002"));
        println!("  vec-003: {:?}", store.get("vec-003"));

        assert_eq!(store.get("vec-001"), Some(&vec![0.1, 0.2, 0.3]));
        assert_eq!(store.get("vec-002"), Some(&vec![0.4, 0.5, 0.6]));
        assert_eq!(store.get("vec-003"), Some(&vec![0.7, 0.8, 0.9]));
    }
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // PHASE 3: More operations including delete
    // ─────────────────────────────────────────────────────────────────────
    println!("═══════════════════════════════════════════════════════════");
    println!("  PHASE 3: Delete and update operations");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    {
        let mut store = VectorStore::new(wal_path)?;

        store.delete("vec-002")?;
        store.insert("vec-001".into(), vec![1.0, 1.0, 1.0])?; // Update
        store.insert("vec-004".into(), vec![4.0, 4.0, 4.0])?;
        store.sync()?;

        println!("Deleted vec-002, updated vec-001, inserted vec-004");
    }
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // PHASE 4: Final recovery verification
    // ─────────────────────────────────────────────────────────────────────
    println!("═══════════════════════════════════════════════════════════");
    println!("  PHASE 4: Final verification");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    {
        let store = VectorStore::new(wal_path)?;

        println!("Final state:");
        println!("  vec-001: {:?}", store.get("vec-001"));
        println!("  vec-002: {:?} (should be None)", store.get("vec-002"));
        println!("  vec-003: {:?}", store.get("vec-003"));
        println!("  vec-004: {:?}", store.get("vec-004"));

        assert_eq!(store.get("vec-001"), Some(&vec![1.0, 1.0, 1.0])); // Updated
        assert_eq!(store.get("vec-002"), None); // Deleted
        assert_eq!(store.get("vec-003"), Some(&vec![0.7, 0.8, 0.9]));
        assert_eq!(store.get("vec-004"), Some(&vec![4.0, 4.0, 4.0]));
    }
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // PERFORMANCE TEST
    // ─────────────────────────────────────────────────────────────────────
    println!("═══════════════════════════════════════════════════════════");
    println!("  PERFORMANCE TEST");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Clean slate
    std::fs::remove_file(wal_path)?;

    {
        let mut wal = WriteAheadLog::open_with_policy(wal_path, SyncPolicy::Manual)?;
        let count = 10000;

        let start = Instant::now();
        for i in 0..count {
            let entry = WalEntry::insert(format!("perf-{}", i), vec![i as f32; 128]);
            wal.append(&entry)?;
        }
        wal.sync()?;
        let elapsed = start.elapsed();

        println!("Wrote {} entries in {:?}", count, elapsed);
        println!(
            "Throughput: {:.0} entries/sec",
            count as f64 / elapsed.as_secs_f64()
        );
        println!("Final WAL size: {} bytes", wal.file_size());
    }
    println!();

    // Cleanup
    std::fs::remove_file(wal_path)?;
    println!("✓ Cleaned up test file");
    println!();

    println!("═══════════════════════════════════════════════════════════");
    println!("  NEXT: See Post #9 for Crash Recovery and Compaction");
    println!("═══════════════════════════════════════════════════════════");

    Ok(())
}
