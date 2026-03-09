# Post #7: Memory Mapping (mmap)

**Topic:** Zero-Copy File Access with memmap2

## Contents

```
post-07-mmap/
├── README.md                 ← You are here
├── blog.md                   ← Main blog post (~15 min read)
├── code/
│   ├── mmap-basics.rs        ← Basic mmap examples
│   └── mmap-segment.rs       ← Full MmapSegment implementation
└── diagrams/
    └── mermaid-diagrams.md   ← Visual diagrams for the post
```

## Key Concepts

| Concept | Description |
|---------|-------------|
| Memory Mapping | OS maps file directly into virtual address space |
| Zero-Copy | Data goes Disk → Page Cache → User Space (no intermediate copy) |
| Lazy Loading | Only pages actually accessed are loaded from disk |
| Page Fault | CPU trap when accessing unmapped page, triggers disk read |

## The Problem We're Solving

```
Standard read():
┌──────┐    ┌────────────┐    ┌─────────────┐
│ Disk │ →  │ Kernel Buf │ →  │ User Buffer │
└──────┘    └────────────┘    └─────────────┘
             (copy #1)          (copy #2)

Memory Mapping:
┌──────┐    ┌────────────────────────────────┐
│ Disk │ →  │ Page Cache (directly mapped)   │
└──────┘    └────────────────────────────────┘
             (zero copies to user space!)
```

## Dependencies

```toml
[dependencies]
memmap2 = "0.9"
bytemuck = { version = "1.14", features = ["derive"] }  # Safe casting
```

## Running the Examples

```powershell
# First, create a test segment file using Post #6's code
cd ../post-06-binary-file-formats/code
rustc segment-format.rs -o segment-format.exe
./segment-format.exe

# Copy the test file
copy test_segment.vec ../../post-07-mmap/code/

# Run mmap examples (requires dependencies, use cargo)
cd ../../post-07-mmap/code
cargo run --example mmap-segment
```

## Performance Comparison

| Metric | `read_to_end()` | `mmap` |
|--------|-----------------|--------|
| Open 1GB file | ~2 seconds | <1 ms |
| RAM usage | 1GB | ~0 (until accessed) |
| Random access | Already in RAM | Page fault on first access |
| Best for | Small files, sequential | Large files, random access |

## Quick Reference

```rust
use memmap2::Mmap;
use std::fs::File;

// Map a file (read-only)
let file = File::open("data.vec")?;
let mmap = unsafe { Mmap::map(&file)? };

// Access like a slice
let byte = mmap[100];
let slice = &mmap[0..16];

// Cast bytes to f32 (with bytemuck for safety)
let floats: &[f32] = bytemuck::cast_slice(&mmap[16..32]);
```

## Dangers

| Risk | Cause | Mitigation |
|------|-------|------------|
| SIGBUS crash | File truncated while mapped | Don't modify files externally |
| I/O stalls | Page fault on cold page | Use `madvise` hints, async-aware design |
| Data corruption | Wrong endianness | Always use Little Endian (Post #6) |

## Next Post

→ Post #8: Write-Ahead Log (WAL) - Safe writes before segment compaction
