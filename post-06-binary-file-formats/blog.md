# Binary File Formats: Designing a Custom Segment Layout

**Series:** Building a Vector Database from Scratch in Rust  
**Post:** 6 of 20  
**Reading Time:** ~15 minutes

---

## 1. Introduction: Why Not JSON?

In the previous post, we built a web server that accepts JSON. It works great for talking to humans and web browsers.

But if you look inside the storage engine of **PostgreSQL**, **MongoDB**, or **Pinecone**, you won't find JSON files. You will find weird, unreadable binary files.

Why?

1. **Size:** A single floating point number like `0.123456789` takes **11 bytes** in JSON text. In binary (`f32`), it takes exactly **4 bytes**. That's a ~65% reduction instantly.

2. **Speed:** To read JSON, the CPU has to parse strings character-by-character, handle whitespace, and convert ASCII to numbers. Reading binary is just "copying memory." It is orders of magnitude faster.

3. **Random Access:** In a JSON file, if you want the 100th vector, you have to parse the first 99. In a binary file with fixed-width records, you just calculate the offset: `Position = Header + (100 * RecordSize)` and jump straight there.

<!-- Diagram: diagram-1-json-vs-binary -->

In this post, we will stop being "Web Developers" and start being "Systems Engineers." We will design the raw binary file format that our database uses to store vectors on disk.

---

## 2. Binary 101: The Byte

Computers don't know "Integers" or "Strings." They only know **Bytes** (`u8`).

A file on your disk is just a long array of `u8`. It is up to *us* to decide what those bytes mean.

### 2.1 Integers and Floats

| Type | Size | Range |
|------|------|-------|
| `u8` | 1 byte | 0 to 255 |
| `u16` | 2 bytes | 0 to 65,535 |
| `u32` | 4 bytes | 0 to 4,294,967,295 |
| `u64` | 8 bytes | 0 to 18 quintillion |
| `f32` | 4 bytes | ±3.4 × 10³⁸ (IEEE 754) |
| `f64` | 8 bytes | ±1.8 × 10³⁰⁸ (IEEE 754) |

If we have the number `500`, in hex it is `0x000001F4`. How do we store this sequence of 4 bytes?

### 2.2 The Endianness War

This is the classic "Gulliver's Travels" problem in computing.

<!-- Diagram: diagram-2-endianness -->

* **Big Endian:** Store the "Big" end first. `[00, 00, 01, F4]`. (Used by Networking protocols, Java).
* **Little Endian:** Store the "Little" end first. `[F4, 01, 00, 00]`. (Used by Intel/AMD/ARM CPUs).

```rust
let value: u32 = 500;

// Little Endian (what x86/ARM uses)
let le_bytes = value.to_le_bytes();  // [0xF4, 0x01, 0x00, 0x00]

// Big Endian (network byte order)
let be_bytes = value.to_be_bytes();  // [0x00, 0x00, 0x01, 0xF4]

// Native Endian (whatever your CPU uses)
let ne_bytes = value.to_ne_bytes();  // Usually same as LE on modern systems
```

> **SYSTEMS NOTE:** The name comes from Jonathan Swift's *Gulliver's Travels*, where two kingdoms went to war over which end of a boiled egg to crack first. Computer scientists have the same energy about byte order.

**Rule for Our DB:** We will use **Little Endian** everywhere. Why? Because x86 and ARM chips are Little Endian. If the file format matches the CPU format, we can read data by just "blasting" it into memory (Zero-Copy) without shuffling bytes around.

---

## 3. Designing the `.vec` File Format

We need a format that is simple, crash-safe, and easy to read.

We will define a file structure with two parts:

1. **The Header:** Metadata about the file (so we know how to read it).
2. **The Body:** The raw vector data, packed tightly.

### 3.1 The Layout

<!-- Diagram: diagram-3-file-layout -->

```text
File Start ──►  ┌──────────────────────────┐
                │ Magic Bytes (4 bytes)    │ "VECT"
                ├──────────────────────────┤
                │ Version (4 bytes)        │ 1
                ├──────────────────────────┤
                │ Vector Count (4 bytes)   │ N
                ├──────────────────────────┤
                │ Dimension (4 bytes)      │ D
 Header End ──► ├──────────────────────────┤
                │ Vector 1 (D × 4 bytes)   │
                ├──────────────────────────┤
                │ Vector 2 (D × 4 bytes)   │
                ├──────────────────────────┤
                │ ...                      │
                └──────────────────────────┘
```

**Quick Math:**
- Header size: 16 bytes (fixed)
- Each vector: `dimension × 4` bytes
- Total file size: `16 + (count × dimension × 4)` bytes

For 1 million 128-dimensional vectors:
```
16 + (1,000,000 × 128 × 4) = 512,000,016 bytes ≈ 488 MB
```

### 3.2 Why Magic Bytes?

Every good binary format starts with a unique signature.

| Format | Magic Bytes | Hex |
|--------|-------------|-----|
| Java class | `CAFEBABE` | `CA FE BA BE` |
| PDF | `%PDF` | `25 50 44 46` |
| PNG | `PNG` | `89 50 4E 47` |
| ZIP | `PK` | `50 4B` |
| **Ours** | `VECT` | `56 45 43 54` |

This prevents us from accidentally trying to load a JPEG or a text file as a database segment. When we open a file, the first thing we check is: "Does it start with `VECT`?" If not, reject it immediately.

### 3.3 Why a Version Number?

Someday we might need to change the format:
- Add a checksum field
- Support different data types (f64, i8 for quantized vectors)
- Add compression

By storing a version number, old code can say "I don't understand version 3" instead of silently corrupting data.

---

## 4. Hands-On: Encoding Data in Rust

Rust provides excellent tools for this in the `byteorder` crate, but the standard library is often enough.

### 4.1 The Helper Functions

First, let's write helpers to read and write primitive types:

```rust
use std::io::{self, Read, Write};

// ═══════════════════════════════════════════════════════════════════════════
// WRITING (Serialization)
// ═══════════════════════════════════════════════════════════════════════════

fn write_u32(w: &mut impl Write, value: u32) -> io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

fn write_f32(w: &mut impl Write, value: f32) -> io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

// ═══════════════════════════════════════════════════════════════════════════
// READING (Deserialization)
// ═══════════════════════════════════════════════════════════════════════════

fn read_u32(r: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_f32(r: &mut impl Read) -> io::Result<f32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}
```

> **SYSTEMS NOTE:** We use `read_exact` instead of `read`. The difference? `read` might return fewer bytes than requested (partial read). `read_exact` keeps reading until it gets all 4 bytes or errors. For structured binary data, you almost always want `read_exact`.

### 4.2 Writing a Segment File

<!-- Diagram: diagram-4-write-flow -->

```rust
use std::io::{self, Write};

/// Write vectors to our custom binary format
fn write_segment(file: &mut impl Write, vectors: &[Vector]) -> io::Result<()> {
    // ─────────────────────────────────────────────────────────────────────
    // HEADER
    // ─────────────────────────────────────────────────────────────────────
    
    // 1. Magic Bytes "VECT" (4 bytes)
    file.write_all(b"VECT")?;
    
    // 2. Version (4 bytes) - currently version 1
    write_u32(file, 1)?;
    
    // 3. Vector Count (4 bytes)
    let count = vectors.len() as u32;
    write_u32(file, count)?;
    
    // 4. Dimension (4 bytes) - get from first vector, or 0 if empty
    let dim = vectors.first().map(|v| v.data.len()).unwrap_or(0) as u32;
    write_u32(file, dim)?;
    
    // ─────────────────────────────────────────────────────────────────────
    // BODY
    // ─────────────────────────────────────────────────────────────────────
    
    for (i, vec) in vectors.iter().enumerate() {
        // Validate dimension consistency
        if vec.data.len() as u32 != dim {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Vector {} has dimension {}, expected {}", i, vec.data.len(), dim)
            ));
        }
        
        // Write each f32 component
        for val in &vec.data {
            write_f32(file, *val)?;
        }
    }
    
    Ok(())
}
```

### 4.3 Reading a Segment File

<!-- Diagram: diagram-5-read-flow -->

```rust
use std::io::{self, Read};

/// Read vectors from our custom binary format
fn read_segment(file: &mut impl Read) -> io::Result<Vec<Vector>> {
    // ─────────────────────────────────────────────────────────────────────
    // HEADER
    // ─────────────────────────────────────────────────────────────────────
    
    // 1. Validate Magic Bytes
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    
    if &magic != b"VECT" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid magic bytes: expected VECT, got {:?}", magic)
        ));
    }
    
    // 2. Read version (for future compatibility checks)
    let version = read_u32(file)?;
    if version != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsupported version: {}", version)
        ));
    }
    
    // 3. Read metadata
    let count = read_u32(file)?;
    let dim = read_u32(file)?;
    
    // ─────────────────────────────────────────────────────────────────────
    // BODY
    // ─────────────────────────────────────────────────────────────────────
    
    let mut vectors = Vec::with_capacity(count as usize);
    
    for _ in 0..count {
        let mut data = Vec::with_capacity(dim as usize);
        
        for _ in 0..dim {
            data.push(read_f32(file)?);
        }
        
        vectors.push(Vector::new(data));
    }
    
    Ok(vectors)
}
```

---

## 5. Random Access: The Real Power

Here's where binary shines. If you want to read vector #1000 without loading the other 999:

<!-- Diagram: diagram-6-random-access -->

```rust
use std::io::{Read, Seek, SeekFrom};

/// Read a single vector by index without loading the entire file
fn read_vector_at(
    file: &mut (impl Read + Seek),
    index: u32,
    dimension: u32,
) -> io::Result<Vector> {
    // Calculate offset:
    // - Skip 16-byte header
    // - Skip (index * dimension * 4) bytes to reach our vector
    let offset = 16 + (index as u64 * dimension as u64 * 4);
    
    // Jump directly to that position
    file.seek(SeekFrom::Start(offset))?;
    
    // Read just this one vector
    let mut data = Vec::with_capacity(dimension as usize);
    for _ in 0..dimension {
        data.push(read_f32(file)?);
    }
    
    Ok(Vector::new(data))
}
```

This is **O(1)** access. Whether you have 100 vectors or 100 million, reading vector #50 takes the same time.

---

## 6. Hex Editors: The X-Ray Vision

To verify our work, we need to look at the file. You can't open `.vec` files in a text editor—it will look like garbage characters.

You need a **Hex Editor**:
- **VS Code:** Install the "Hex Editor" extension
- **Windows:** HxD (free)
- **Cross-platform:** `xxd` command line tool

If we write a file containing one vector `[1.0, 2.0]` (dimension 2), it should look like this:

```text
Offset    00 01 02 03  04 05 06 07  08 09 0A 0B  0C 0D 0E 0F
────────  ───────────  ───────────  ───────────  ───────────
00000000  56 45 43 54  01 00 00 00  01 00 00 00  02 00 00 00
          ^^^^^^^^^^   ^^^^^^^^^^   ^^^^^^^^^^   ^^^^^^^^^^
          "VECT"       Version=1    Count=1      Dim=2

00000010  00 00 80 3F  00 00 00 40
          ^^^^^^^^^^   ^^^^^^^^^^
          1.0 (f32)    2.0 (f32)
```

> **TRAP:** "Wait, `1.0` is `00 00 80 3F`? That doesn't look like 1!"
>
> That's IEEE 754 floating point encoding. The bytes represent the sign, exponent, and mantissa in a specific format. You don't need to memorize this—just use online converters or Rust's `f32::to_le_bytes()` to check values.

---

## 7. Complete Example: Round-Trip Test

Let's put it all together with a test that writes and reads back:

```rust
fn main() -> io::Result<()> {
    // Create test vectors
    let vectors = vec![
        Vector::new(vec![1.0, 2.0, 3.0]),
        Vector::new(vec![4.0, 5.0, 6.0]),
        Vector::new(vec![7.0, 8.0, 9.0]),
    ];
    
    // Write to file
    let mut file = File::create("test.vec")?;
    write_segment(&mut file, &vectors)?;
    println!("Wrote {} vectors to test.vec", vectors.len());
    
    // Read back
    let mut file = File::open("test.vec")?;
    let loaded = read_segment(&mut file)?;
    println!("Read {} vectors from test.vec", loaded.len());
    
    // Verify
    for (i, (original, loaded)) in vectors.iter().zip(&loaded).enumerate() {
        assert_eq!(original.data, loaded.data, "Vector {} mismatch!", i);
    }
    println!("✓ All vectors match!");
    
    // Test random access
    let mut file = File::open("test.vec")?;
    let vector_1 = read_vector_at(&mut file, 1, 3)?;  // Read second vector
    println!("Random access vector[1]: {:?}", vector_1.data);
    
    Ok(())
}
```

Output:
```
Wrote 3 vectors to test.vec
Read 3 vectors from test.vec
✓ All vectors match!
Random access vector[1]: [4.0, 5.0, 6.0]
```

---

## 8. Design Decisions Recap

<!-- Diagram: diagram-7-design-decisions -->

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Byte Order | Little Endian | Matches x86/ARM, enables zero-copy |
| Header Size | 16 bytes fixed | Simple, predictable |
| Magic Bytes | "VECT" | File type identification |
| Version Field | u32 | Future format evolution |
| Vector Storage | Contiguous f32[] | Cache-friendly, random access |

---

## 9. What We Didn't Cover (Yet)

This is a minimal viable format. Production databases add:

1. **Checksums:** CRC32 or xxHash to detect corruption
2. **Compression:** LZ4 or Zstd to reduce file size
3. **Metadata:** Store vector IDs alongside data
4. **Alignment:** Pad to cache line boundaries (64 bytes)
5. **Memory Mapping:** Read without copying (next post!)

---

## 10. Summary

We have moved from abstract JSON objects to raw silicon.

<!-- Diagram: diagram-8-layer-progress -->

**What we learned:**
- **Endianness:** Little Endian matches modern CPUs
- **Magic Bytes:** Identify file format at a glance
- **Fixed-width records:** Enable O(1) random access
- **Seek + Read:** Jump directly to any vector

**But there's a problem.**

Our `read_segment` function reads the *entire file* into RAM. If the database is 100GB, we crash.

In the next post, we will solve this using the ultimate weapon of database engineers: **Memory Mapping (`mmap`)**. We will learn how to trick the OS into loading only the parts of the file we actually need.

---

**Next Post:** [Post #7: Memory Mapping (mmap): Reading Files at RAM Speed →](../post-07-mmap/blog.md)
