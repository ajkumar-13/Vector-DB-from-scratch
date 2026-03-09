# Post #9: Crash Recovery — Writing the Startup Logic to Replay Logs and Restore State

> **Series:** Building a Vector Database from Scratch in Rust  
> **Reading Time:** ~15 minutes  
> **Difficulty:** Intermediate  

---

## 📁 Folder Contents

| File | Purpose |
|------|---------|
| [blog.md](blog.md) | Main post: WAL replay, compaction, segment loading |
| [code/recovery.rs](code/recovery.rs) | Complete VectorStore with startup recovery |
| [code/compaction.rs](code/compaction.rs) | Compaction algorithm with atomic rename |
| [diagrams/mermaid-diagrams.md](diagrams/mermaid-diagrams.md) | Visual guides for recovery flow |

---

## 🎯 What You'll Learn

1. **The "Morning After" Problem** — RAM is empty after restart
2. **WAL Replay** — Reconstructing HashMap from the log
3. **Compaction** — Freezing MemTable → writing Segment → truncating WAL
4. **Atomic Rename Trick** — Why `rename()` is your best friend
5. **Dirty State Handling** — What happens if we crash *during* compaction

---

## 🏗️ The LSM-Tree-Lite Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      VectorStore                            │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────────────┐│
│  │   MemTable  │   │     WAL     │   │      Segments       ││
│  │  (HashMap)  │   │ (append-only│   │  (MmapSegment[])    ││
│  │             │   │   log)      │   │                     ││
│  │  MUTABLE    │   │  DURABLE    │   │  IMMUTABLE + FAST   ││
│  └─────────────┘   └─────────────┘   └─────────────────────┘│
│        ↑                 ↑                    ↑             │
│     Write Path       Crash Recovery      Read Path          │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔄 The Startup Sequence

```
1. Load Segments    →  Open all .vec files as MmapSegment
2. Replay WAL       →  Read entries, apply Insert/Delete to HashMap
3. Clean Up         →  Delete any .tmp files (failed compaction)
4. Ready to Serve   →  Accept new reads and writes
```

---

## 🗜️ Compaction: The 5-Step Dance

| Step | Action | Crash Behavior |
|------|--------|----------------|
| 1. Freeze | Stop writes (or snapshot) | WAL still valid |
| 2. Dump | Write MemTable → `segment_N.vec.tmp` | tmp ignored on restart |
| 3. Sync | `file.sync_all()` | Data physically on disk |
| 4. Rename | `segment_N.vec.tmp` → `segment_N.vec` | Atomic operation |
| 5. Truncate | Clear WAL file | Safe: segment already visible |

**Golden Rule:** The `.tmp` extension protects us. We never see half-written data.

---

## ⚠️ Crash Scenarios

| Crashed At | On Restart | Data Status |
|------------|------------|-------------|
| During Step 2 (writing tmp) | Delete `.tmp`, replay WAL | ✅ Safe |
| After Step 4, before Step 5 | Load segment + replay WAL (idempotent) | ✅ Safe |
| After Step 5 | Load segment, WAL is empty | ✅ Safe |

---

## 📊 Why Compaction Matters

| Without Compaction | With Compaction |
|--------------------|-----------------|
| WAL grows forever | WAL stays small |
| Startup: O(all writes) | Startup: O(active vectors) |
| No mmap benefits | Zero-copy reads from segments |
| 10GB WAL for 100MB data | 100MB segment + tiny WAL |

---

## 🔗 Dependencies

- **Post #6:** Binary file format (`write_segment`)
- **Post #7:** Memory-mapped segments (`MmapSegment`)
- **Post #8:** Write-Ahead Log (`WriteAheadLog`, `WalEntry`)

---

## 🚀 Next Up

**Post #10:** Concurrency Control — `Arc<RwLock<...>>` for parallel reads during compaction
