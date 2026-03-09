# Post #8: Write-Ahead Log (WAL)

**Topic:** Append-Only Logging for Crash-Safe Writes

## Contents

```
post-08-wal/
├── README.md                 ← You are here
├── blog.md                   ← Main blog post (~15 min read)
├── code/
│   ├── wal-entry.rs          ← WAL entry types and serialization
│   └── wal.rs                ← Full WAL implementation
└── diagrams/
    └── mermaid-diagrams.md   ← Visual diagrams for the post
```

## Key Concepts

| Concept | Description |
|---------|-------------|
| Write-Ahead Log | Append-only log of all write operations |
| Sequential Writes | Maximum disk throughput (no random seeks) |
| CRC Checksum | Detect corruption from partial writes |
| fsync | Force data from OS cache to physical disk |
| Replay | Rebuild state by reading the entire log |

## The Golden Rule

> **Never modify data in memory until you have written a record of the intention to disk.**

## WAL Entry Format

```
┌──────────────┬──────────────┬──────────────┬──────────────────────┐
│ CRC32 (4B)   │ Length (4B)  │ Type (1B)    │ Payload (N bytes)    │
└──────────────┴──────────────┴──────────────┴──────────────────────┘

CRC32:   Checksum of (Length + Type + Payload)
Length:  Size of (Type + Payload) in bytes
Type:    0x01 = Insert, 0x02 = Delete
Payload: Serialized operation data
```

## Dependencies

```toml
[dependencies]
crc32fast = "1.3"           # Fast CRC32 checksums
bincode = "1.3"             # Compact binary serialization
serde = { version = "1.0", features = ["derive"] }
```

## Running the Examples

```powershell
# Create a Cargo project with dependencies
cargo new wal-demo
cd wal-demo

# Add dependencies to Cargo.toml, then copy wal.rs to src/main.rs
cargo run
```

## The fsync Trade-off

| Strategy | Durability | Performance |
|----------|------------|-------------|
| Sync every write |  Maximum |  ~200 writes/sec |
| Sync every N writes |  May lose N |  ~50,000 writes/sec |
| Sync every T ms |  May lose T ms |  ~50,000 writes/sec |

Most databases use "group commit" - batch multiple writes, then sync once.

## Write Path

```
Client POST /upsert
       │
       ▼
┌─────────────────┐
│ Append to WAL   │ ← Disk (durable)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Update HashMap  │ ← RAM (fast search)
└────────┬────────┘
         │
         ▼
    Return 200 OK
```

## Crash Recovery

```
1. Server crashes (RAM lost)
2. Server restarts
3. Open WAL file
4. Read entries until EOF or corruption
5. Replay each Insert/Delete into HashMap
6. Ready to serve!
```

## Next Post

→ Post #9: Crash Recovery - Replaying the WAL and Restoring State
