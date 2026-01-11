// segment-format.rs
//
// Complete implementation of our custom .vec binary file format.
// From Post #6: Binary File Formats
//
// File Layout:
// ┌──────────────────────────┐
// │ Magic "VECT" (4 bytes)   │
// │ Version (4 bytes)        │
// │ Count (4 bytes)          │
// │ Dimension (4 bytes)      │
// ├──────────────────────────┤
// │ Vector 1 (D × 4 bytes)   │
// │ Vector 2 (D × 4 bytes)   │
// │ ...                      │
// └──────────────────────────┘

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};

// ═══════════════════════════════════════════════════════════════════════════
// VECTOR STRUCT (from previous posts)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Vector {
    pub data: Vec<f32>,
    pub metadata: HashMap<String, String>,
}

impl Vector {
    pub fn new(data: Vec<f32>) -> Self {
        Self {
            data,
            metadata: HashMap::new(),
        }
    }

    pub fn dimension(&self) -> usize {
        self.data.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════

/// Magic bytes identifying our file format
const MAGIC: &[u8; 4] = b"VECT";

/// Current format version
const VERSION: u32 = 1;

/// Header size in bytes (magic + version + count + dimension)
const HEADER_SIZE: u64 = 16;

// ═══════════════════════════════════════════════════════════════════════════
// LOW-LEVEL I/O HELPERS
// ═══════════════════════════════════════════════════════════════════════════

fn write_u32(w: &mut impl Write, value: u32) -> io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

fn read_u32(r: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn write_f32(w: &mut impl Write, value: f32) -> io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

fn read_f32(r: &mut impl Read) -> io::Result<f32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

// ═══════════════════════════════════════════════════════════════════════════
// SEGMENT HEADER
// ═══════════════════════════════════════════════════════════════════════════

/// Header information for a segment file
#[derive(Debug, Clone)]
pub struct SegmentHeader {
    pub version: u32,
    pub count: u32,
    pub dimension: u32,
}

impl SegmentHeader {
    /// Calculate the byte offset where vector data starts
    pub fn data_offset(&self) -> u64 {
        HEADER_SIZE
    }

    /// Calculate the total file size
    pub fn file_size(&self) -> u64 {
        HEADER_SIZE + (self.count as u64 * self.dimension as u64 * 4)
    }

    /// Calculate byte offset for a specific vector index
    pub fn vector_offset(&self, index: u32) -> u64 {
        HEADER_SIZE + (index as u64 * self.dimension as u64 * 4)
    }

    /// Write header to a writer
    pub fn write(&self, w: &mut impl Write) -> io::Result<()> {
        w.write_all(MAGIC)?;
        write_u32(w, self.version)?;
        write_u32(w, self.count)?;
        write_u32(w, self.dimension)?;
        Ok(())
    }

    /// Read header from a reader
    pub fn read(r: &mut impl Read) -> io::Result<Self> {
        // Validate magic bytes
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;

        if &magic != MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid magic bytes: expected {:?}, got {:?}", MAGIC, magic),
            ));
        }

        let version = read_u32(r)?;
        if version != VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported version: expected {}, got {}", VERSION, version),
            ));
        }

        let count = read_u32(r)?;
        let dimension = read_u32(r)?;

        Ok(Self {
            version,
            count,
            dimension,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SEGMENT WRITER
// ═══════════════════════════════════════════════════════════════════════════

/// Write a collection of vectors to a segment file
pub fn write_segment(path: &str, vectors: &[Vector]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Determine dimension from first vector
    let dimension = vectors.first().map(|v| v.dimension()).unwrap_or(0) as u32;

    // Write header
    let header = SegmentHeader {
        version: VERSION,
        count: vectors.len() as u32,
        dimension,
    };
    header.write(&mut writer)?;

    // Write vector data
    for (i, vec) in vectors.iter().enumerate() {
        // Validate dimension consistency
        if vec.dimension() as u32 != dimension {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Vector {} has dimension {}, expected {}",
                    i,
                    vec.dimension(),
                    dimension
                ),
            ));
        }

        // Write each component
        for &val in &vec.data {
            write_f32(&mut writer, val)?;
        }
    }

    writer.flush()?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// SEGMENT READER
// ═══════════════════════════════════════════════════════════════════════════

/// Read all vectors from a segment file
pub fn read_segment(path: &str) -> io::Result<Vec<Vector>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Read and validate header
    let header = SegmentHeader::read(&mut reader)?;

    // Read all vectors
    let mut vectors = Vec::with_capacity(header.count as usize);

    for _ in 0..header.count {
        let mut data = Vec::with_capacity(header.dimension as usize);
        for _ in 0..header.dimension {
            data.push(read_f32(&mut reader)?);
        }
        vectors.push(Vector::new(data));
    }

    Ok(vectors)
}

/// Read only the header from a segment file
pub fn read_segment_header(path: &str) -> io::Result<SegmentHeader> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    SegmentHeader::read(&mut reader)
}

/// Read a single vector by index (random access)
pub fn read_vector_at(path: &str, index: u32) -> io::Result<Vector> {
    let mut file = File::open(path)?;

    // Read header first to get dimension
    let header = SegmentHeader::read(&mut file)?;

    // Validate index
    if index >= header.count {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Index {} out of bounds (count: {})", index, header.count),
        ));
    }

    // Seek to vector position
    let offset = header.vector_offset(index);
    file.seek(SeekFrom::Start(offset))?;

    // Read vector data
    let mut data = Vec::with_capacity(header.dimension as usize);
    for _ in 0..header.dimension {
        data.push(read_f32(&mut file)?);
    }

    Ok(Vector::new(data))
}

/// Read a range of vectors (more efficient than multiple read_vector_at calls)
pub fn read_vectors_range(path: &str, start: u32, count: u32) -> io::Result<Vec<Vector>> {
    let mut file = File::open(path)?;

    // Read header
    let header = SegmentHeader::read(&mut file)?;

    // Validate range
    if start + count > header.count {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Range {}..{} out of bounds (count: {})",
                start,
                start + count,
                header.count
            ),
        ));
    }

    // Seek to start position
    let offset = header.vector_offset(start);
    file.seek(SeekFrom::Start(offset))?;

    // Read vectors
    let mut vectors = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let mut data = Vec::with_capacity(header.dimension as usize);
        for _ in 0..header.dimension {
            data.push(read_f32(&mut file)?);
        }
        vectors.push(Vector::new(data));
    }

    Ok(vectors)
}

// ═══════════════════════════════════════════════════════════════════════════
// HEX DUMP UTILITY
// ═══════════════════════════════════════════════════════════════════════════

/// Print a hex dump of a file for debugging
pub fn hex_dump(path: &str, max_bytes: usize) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; max_bytes];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    println!("Hex dump of {} ({} bytes):", path, bytes_read);
    println!();
    println!("Offset    00 01 02 03  04 05 06 07  08 09 0A 0B  0C 0D 0E 0F   ASCII");
    println!("────────  ───────────  ───────────  ───────────  ───────────   ────────────────");

    for (i, chunk) in buffer.chunks(16).enumerate() {
        // Offset
        print!("{:08X}  ", i * 16);

        // Hex bytes in groups of 4
        for (j, byte) in chunk.iter().enumerate() {
            print!("{:02X} ", byte);
            if j % 4 == 3 {
                print!(" ");
            }
        }

        // Padding for incomplete lines
        for j in chunk.len()..16 {
            print!("   ");
            if j % 4 == 3 {
                print!(" ");
            }
        }

        // ASCII representation
        print!(" ");
        for byte in chunk {
            if *byte >= 0x20 && *byte < 0x7F {
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }
        println!();
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// MAIN - DEMONSTRATION
// ═══════════════════════════════════════════════════════════════════════════

fn main() -> io::Result<()> {
    println!("═══════════════════════════════════════════════════════════");
    println!("  SEGMENT FILE FORMAT DEMONSTRATION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Create test vectors
    let vectors = vec![
        Vector::new(vec![1.0, 2.0, 3.0]),
        Vector::new(vec![4.0, 5.0, 6.0]),
        Vector::new(vec![7.0, 8.0, 9.0]),
        Vector::new(vec![10.0, 11.0, 12.0]),
        Vector::new(vec![13.0, 14.0, 15.0]),
    ];

    let filename = "test_segment.vec";

    // ─────────────────────────────────────────────────────────────────────
    // WRITE
    // ─────────────────────────────────────────────────────────────────────
    println!("1. Writing {} vectors to '{}'...", vectors.len(), filename);
    write_segment(filename, &vectors)?;
    println!("   ✓ Done!");
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // HEX DUMP
    // ─────────────────────────────────────────────────────────────────────
    println!("2. Hex dump of file:");
    println!();
    hex_dump(filename, 128)?;
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // READ HEADER ONLY
    // ─────────────────────────────────────────────────────────────────────
    println!("3. Reading header only:");
    let header = read_segment_header(filename)?;
    println!("   Version:   {}", header.version);
    println!("   Count:     {}", header.count);
    println!("   Dimension: {}", header.dimension);
    println!("   File size: {} bytes", header.file_size());
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // READ ALL
    // ─────────────────────────────────────────────────────────────────────
    println!("4. Reading all vectors:");
    let loaded = read_segment(filename)?;
    for (i, vec) in loaded.iter().enumerate() {
        println!("   Vector {}: {:?}", i, vec.data);
    }
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // RANDOM ACCESS
    // ─────────────────────────────────────────────────────────────────────
    println!("5. Random access (read vector at index 2):");
    let single = read_vector_at(filename, 2)?;
    println!("   Vector 2: {:?}", single.data);
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // RANGE READ
    // ─────────────────────────────────────────────────────────────────────
    println!("6. Range read (vectors 1..3):");
    let range = read_vectors_range(filename, 1, 3)?;
    for (i, vec) in range.iter().enumerate() {
        println!("   Vector {}: {:?}", i + 1, vec.data);
    }
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // VERIFY ROUND-TRIP
    // ─────────────────────────────────────────────────────────────────────
    println!("7. Verifying round-trip integrity:");
    let loaded = read_segment(filename)?;
    let mut all_match = true;
    for (i, (original, loaded)) in vectors.iter().zip(&loaded).enumerate() {
        if original.data != loaded.data {
            println!("   ✗ Mismatch at vector {}", i);
            all_match = false;
        }
    }
    if all_match {
        println!("   ✓ All {} vectors match!", vectors.len());
    }
    println!();

    // ─────────────────────────────────────────────────────────────────────
    // SIZE COMPARISON
    // ─────────────────────────────────────────────────────────────────────
    println!("8. Size comparison (vs JSON):");
    let binary_size = std::fs::metadata(filename)?.len();

    // Approximate JSON size
    let json = serde_json::to_string(&vectors.iter().map(|v| &v.data).collect::<Vec<_>>())
        .unwrap_or_default();
    let json_size = json.len();

    println!("   Binary:  {} bytes", binary_size);
    println!("   JSON:    {} bytes", json_size);
    println!(
        "   Savings: {:.1}%",
        (1.0 - (binary_size as f64 / json_size as f64)) * 100.0
    );
    println!();

    // Cleanup
    std::fs::remove_file(filename)?;
    println!("✓ Cleaned up test file");

    Ok(())
}

// We need serde_json for the size comparison demo
// In a real project, this would be in Cargo.toml
#[cfg(not(feature = "no_serde"))]
mod serde_json {
    pub fn to_string<T: std::fmt::Debug>(_: &T) -> Result<String, ()> {
        // Fake implementation that approximates JSON size
        Ok(
            "[[1.0,2.0,3.0],[4.0,5.0,6.0],[7.0,8.0,9.0],[10.0,11.0,12.0],[13.0,14.0,15.0]]"
                .to_string(),
        )
    }
}
