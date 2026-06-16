// wal-entry.rs
//
// WAL entry types and serialization helpers.
// From Post #8: Write-Ahead Log (WAL)
//
// This file defines the types that can be logged to the WAL.
// We use bincode for serialization (fast, compact).
//
// Dependencies (Cargo.toml):
//   serde = { version = "1.0", features = ["derive"] }
//   bincode = "1.3"

use std::collections::HashMap;

// For the example, we'll use a simple serialization
// In production, use: use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// WAL ENTRY TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Operations that can be recorded in the Write-Ahead Log.
///
/// Each variant represents an atomic operation on the database.
/// The WAL stores a sequence of these entries.
#[derive(Debug, Clone, PartialEq)]
pub enum WalEntry {
    /// Insert or update a vector
    Insert {
        /// Unique identifier for the vector
        id: String,
        /// The embedding data
        vector: Vec<f32>,
        /// Optional key-value metadata
        metadata: HashMap<String, String>,
    },

    /// Delete a vector by ID
    Delete {
        /// ID of the vector to delete
        id: String,
    },

    /// Checkpoint marker (used during compaction)
    Checkpoint {
        /// Sequence number of this checkpoint
        sequence: u64,
        /// Number of entries compacted
        entries_compacted: u64,
    },
}

impl WalEntry {
    // ─────────────────────────────────────────────────────────────────────
    // CONSTRUCTORS
    // ─────────────────────────────────────────────────────────────────────

    /// Create an insert entry with just ID and vector
    pub fn insert(id: impl Into<String>, vector: Vec<f32>) -> Self {
        Self::Insert {
            id: id.into(),
            vector,
            metadata: HashMap::new(),
        }
    }

    /// Create an insert entry with metadata
    pub fn insert_with_metadata(
        id: impl Into<String>,
        vector: Vec<f32>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self::Insert {
            id: id.into(),
            vector,
            metadata,
        }
    }

    /// Create a delete entry
    pub fn delete(id: impl Into<String>) -> Self {
        Self::Delete { id: id.into() }
    }

    /// Create a checkpoint marker
    pub fn checkpoint(sequence: u64, entries_compacted: u64) -> Self {
        Self::Checkpoint {
            sequence,
            entries_compacted,
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // ACCESSORS
    // ─────────────────────────────────────────────────────────────────────

    /// Get the ID affected by this entry (if any)
    pub fn id(&self) -> Option<&str> {
        match self {
            WalEntry::Insert { id, .. } => Some(id),
            WalEntry::Delete { id } => Some(id),
            WalEntry::Checkpoint { .. } => None,
        }
    }

    /// Check if this is an insert operation
    pub fn is_insert(&self) -> bool {
        matches!(self, WalEntry::Insert { .. })
    }

    /// Check if this is a delete operation
    pub fn is_delete(&self) -> bool {
        matches!(self, WalEntry::Delete { .. })
    }

    /// Check if this is a checkpoint
    pub fn is_checkpoint(&self) -> bool {
        matches!(self, WalEntry::Checkpoint { .. })
    }

    /// Get the entry type as a string (for logging)
    pub fn entry_type(&self) -> &'static str {
        match self {
            WalEntry::Insert { .. } => "INSERT",
            WalEntry::Delete { .. } => "DELETE",
            WalEntry::Checkpoint { .. } => "CHECKPOINT",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SIMPLE SERIALIZATION (without serde/bincode)
// ═══════════════════════════════════════════════════════════════════════════

/// Entry type markers for binary format
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntryType {
    Insert = 0x01,
    Delete = 0x02,
    Checkpoint = 0x03,
}

impl WalEntry {
    /// Serialize to bytes (simple format for demonstration)
    ///
    /// Format:
    /// - Type (1 byte)
    /// - ID length (4 bytes) + ID (N bytes)  [for Insert/Delete]
    /// - Vector length (4 bytes) + Vector data (N * 4 bytes) [for Insert]
    /// - Metadata count (4 bytes) + pairs [for Insert]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            WalEntry::Insert {
                id,
                vector,
                metadata,
            } => {
                // Type
                buf.push(EntryType::Insert as u8);

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
                buf.push(EntryType::Delete as u8);
                buf.extend(&(id.len() as u32).to_le_bytes());
                buf.extend(id.as_bytes());
            }

            WalEntry::Checkpoint {
                sequence,
                entries_compacted,
            } => {
                buf.push(EntryType::Checkpoint as u8);
                buf.extend(&sequence.to_le_bytes());
                buf.extend(&entries_compacted.to_le_bytes());
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

        // Read type
        let entry_type = data[pos];
        pos += 1;

        match entry_type {
            0x01 => {
                // Insert
                // Read ID
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
                    let val = f32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
                    vector.push(val);
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
                    // Key
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

                    // Value
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
                // Delete
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

            0x03 => {
                // Checkpoint
                if pos + 16 > data.len() {
                    return Err("Truncated checkpoint data".to_string());
                }
                let sequence = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
                pos += 8;
                let entries_compacted = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());

                Ok(WalEntry::Checkpoint {
                    sequence,
                    entries_compacted,
                })
            }

            _ => Err(format!("Unknown entry type: 0x{:02X}", entry_type)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DISPLAY
// ═══════════════════════════════════════════════════════════════════════════

impl std::fmt::Display for WalEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalEntry::Insert {
                id,
                vector,
                metadata,
            } => {
                write!(
                    f,
                    "INSERT(id={}, dim={}, meta={})",
                    id,
                    vector.len(),
                    metadata.len()
                )
            }
            WalEntry::Delete { id } => {
                write!(f, "DELETE(id={})", id)
            }
            WalEntry::Checkpoint {
                sequence,
                entries_compacted,
            } => {
                write!(
                    f,
                    "CHECKPOINT(seq={}, compacted={})",
                    sequence, entries_compacted
                )
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_roundtrip() {
        let entry = WalEntry::insert("test-id", vec![1.0, 2.0, 3.0]);
        let bytes = entry.to_bytes();
        let recovered = WalEntry::from_bytes(&bytes).unwrap();
        assert_eq!(entry, recovered);
    }

    #[test]
    fn test_insert_with_metadata_roundtrip() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());

        let entry = WalEntry::insert_with_metadata("test", vec![1.0], metadata);
        let bytes = entry.to_bytes();
        let recovered = WalEntry::from_bytes(&bytes).unwrap();
        assert_eq!(entry, recovered);
    }

    #[test]
    fn test_delete_roundtrip() {
        let entry = WalEntry::delete("delete-me");
        let bytes = entry.to_bytes();
        let recovered = WalEntry::from_bytes(&bytes).unwrap();
        assert_eq!(entry, recovered);
    }

    #[test]
    fn test_checkpoint_roundtrip() {
        let entry = WalEntry::checkpoint(42, 1000);
        let bytes = entry.to_bytes();
        let recovered = WalEntry::from_bytes(&bytes).unwrap();
        assert_eq!(entry, recovered);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MAIN - DEMONSTRATION
// ═══════════════════════════════════════════════════════════════════════════

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  WAL ENTRY TYPES DEMONSTRATION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Create various entries
    let entries = vec![
        WalEntry::insert("vec-001", vec![0.1, 0.2, 0.3, 0.4]),
        WalEntry::insert("vec-002", vec![0.5, 0.6, 0.7, 0.8]),
        WalEntry::delete("vec-001"),
        WalEntry::checkpoint(1, 2),
    ];

    println!("Created entries:");
    for (i, entry) in entries.iter().enumerate() {
        println!("  [{}] {}", i, entry);
    }
    println!();

    // Serialize and deserialize
    println!("Serialization roundtrip:");
    for entry in &entries {
        let bytes = entry.to_bytes();
        let recovered = WalEntry::from_bytes(&bytes).unwrap();

        println!(
            "  {} → {} bytes → {}",
            entry.entry_type(),
            bytes.len(),
            recovered
        );
        assert_eq!(entry, &recovered);
    }
    println!();

    println!("✓ All roundtrips successful!");
}
