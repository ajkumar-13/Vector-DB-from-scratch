# Post #6: Binary File Formats

**Topic:** Designing a Custom Segment Layout for Vector Storage

## Contents

```
post-06-binary-file-formats/
├── README.md                 ← You are here
├── blog.md                   ← Main blog post (~15 min read)
├── code/
│   ├── binary-io.rs          ← Read/write helpers for binary data
│   └── segment-format.rs     ← Complete segment file implementation
└── diagrams/
    └── mermaid-diagrams.md   ← Visual diagrams for the post
```

## Key Concepts

| Concept | Description |
|---------|-------------|
| Endianness | Little Endian for x86/ARM compatibility |
| Magic Bytes | `VECT` signature to identify our files |
| Fixed-width records | Enables O(1) random access |
| Header + Body | Self-describing file format |

## File Format Specification

```
┌──────────────────────────┐
│ Magic Bytes (4 bytes)    │  "VECT" = 0x56454354
├──────────────────────────┤
│ Version (4 bytes)        │  1 (Little Endian)
├──────────────────────────┤
│ Vector Count (4 bytes)   │  N vectors
├──────────────────────────┤
│ Dimension (4 bytes)      │  D dimensions
├──────────────────────────┤
│ Vector 1 (D × 4 bytes)   │  f32[] Little Endian
├──────────────────────────┤
│ Vector 2 (D × 4 bytes)   │
├──────────────────────────┤
│ ...                      │
└──────────────────────────┘

Header Size: 16 bytes
Vector Size: D × 4 bytes
Total Size:  16 + (N × D × 4) bytes
```

## Running the Examples

```powershell
# Compile and run segment format example
rustc code/segment-format.rs -o segment-format.exe
./segment-format.exe

# This creates a test.vec file - inspect with hex editor
# VS Code: Install "Hex Editor" extension, right-click file → Open With → Hex Editor
```

## Hex Dump Reference

A file with 1 vector of dimension 2 containing `[1.0, 2.0]`:

```
Offset  00 01 02 03  04 05 06 07  08 09 0A 0B  0C 0D 0E 0F
------  -----------  -----------  -----------  -----------
0x0000  56 45 43 54  01 00 00 00  01 00 00 00  02 00 00 00
        ^^^^^^^^^^   ^^^^^^^^^^   ^^^^^^^^^^   ^^^^^^^^^^
        "VECT"       Version=1    Count=1      Dim=2

0x0010  00 00 80 3F  00 00 00 40
        ^^^^^^^^^^   ^^^^^^^^^^
        1.0 (f32)    2.0 (f32)
```

## Quick Reference: f32 → Hex

| Value | Hex (LE bytes) |
|-------|----------------|
| 0.0   | `00 00 00 00`  |
| 1.0   | `00 00 80 3F`  |
| 2.0   | `00 00 00 40`  |
| -1.0  | `00 00 80 BF`  |
| 0.5   | `00 00 00 3F`  |

## Next Post

→ Post #7: Memory Mapping (mmap) - Reading files at RAM speed
