// binary-io.rs
//
// Low-level binary I/O helpers for reading and writing primitive types.
// From Post #6: Binary File Formats
//
// These functions handle the conversion between Rust types and raw bytes
// using Little Endian byte order (matches x86/ARM CPUs).

use std::io::{self, Read, Write};

// ═══════════════════════════════════════════════════════════════════════════
// WRITING (Serialization)
// ═══════════════════════════════════════════════════════════════════════════

/// Write a u32 in Little Endian format
pub fn write_u32(w: &mut impl Write, value: u32) -> io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

/// Write a u64 in Little Endian format
pub fn write_u64(w: &mut impl Write, value: u64) -> io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

/// Write an f32 in Little Endian format
pub fn write_f32(w: &mut impl Write, value: f32) -> io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

/// Write an f64 in Little Endian format
pub fn write_f64(w: &mut impl Write, value: f64) -> io::Result<()> {
    w.write_all(&value.to_le_bytes())
}

/// Write a slice of f32 values
pub fn write_f32_slice(w: &mut impl Write, values: &[f32]) -> io::Result<()> {
    for val in values {
        write_f32(w, *val)?;
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// READING (Deserialization)
// ═══════════════════════════════════════════════════════════════════════════

/// Read a u32 in Little Endian format
pub fn read_u32(r: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

/// Read a u64 in Little Endian format
pub fn read_u64(r: &mut impl Read) -> io::Result<u64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

/// Read an f32 in Little Endian format
pub fn read_f32(r: &mut impl Read) -> io::Result<f32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

/// Read an f64 in Little Endian format
pub fn read_f64(r: &mut impl Read) -> io::Result<f64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(f64::from_le_bytes(buf))
}

/// Read a vector of f32 values
pub fn read_f32_vec(r: &mut impl Read, count: usize) -> io::Result<Vec<f32>> {
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        result.push(read_f32(r)?);
    }
    Ok(result)
}

// ═══════════════════════════════════════════════════════════════════════════
// ENDIANNESS DEMONSTRATION
// ═══════════════════════════════════════════════════════════════════════════

fn demonstrate_endianness() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  ENDIANNESS DEMONSTRATION");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    let value: u32 = 500; // 0x000001F4
    println!("Value: {} (0x{:08X})", value, value);
    println!();

    // Little Endian
    let le = value.to_le_bytes();
    println!(
        "Little Endian: {:02X} {:02X} {:02X} {:02X}",
        le[0], le[1], le[2], le[3]
    );
    println!("               (Least significant byte first)");
    println!();

    // Big Endian
    let be = value.to_be_bytes();
    println!(
        "Big Endian:    {:02X} {:02X} {:02X} {:02X}",
        be[0], be[1], be[2], be[3]
    );
    println!("               (Most significant byte first)");
    println!();

    // Round-trip verification
    let recovered = u32::from_le_bytes(le);
    println!("Round-trip: {} → LE bytes → {} ✓", value, recovered);
}

fn demonstrate_float_encoding() {
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("  IEEE 754 FLOAT ENCODING");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    let floats = [
        0.0f32,
        1.0,
        2.0,
        -1.0,
        0.5,
        3.14159,
        f32::INFINITY,
        f32::NAN,
    ];

    println!("{:<12} {:>20}", "Value", "LE Hex Bytes");
    println!("{}", "-".repeat(34));

    for f in floats {
        let bytes = f.to_le_bytes();
        println!(
            "{:<12} {:02X} {:02X} {:02X} {:02X}",
            format!("{}", f),
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3]
        );
    }
}

fn demonstrate_read_exact_vs_read() {
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("  read_exact vs read");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Simulate a data source
    let data: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let mut cursor = std::io::Cursor::new(data);

    // Using read_exact - always reads exactly N bytes or errors
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf).unwrap();
    println!("read_exact(&mut [u8; 4]): {:?}", buf);
    println!("  → Always reads exactly 4 bytes");
    println!("  → Errors if not enough data available");
    println!();

    // Reset cursor
    cursor.set_position(0);

    // Using read - may read fewer bytes
    let mut buf2 = [0u8; 4];
    let bytes_read = cursor.read(&mut buf2).unwrap();
    println!(
        "read(&mut [u8; 4]): {:?}, bytes_read = {}",
        buf2, bytes_read
    );
    println!("  → May read fewer than 4 bytes");
    println!("  → Returns how many bytes were actually read");
    println!();

    println!("For binary file formats, always use read_exact!");
}

fn main() {
    demonstrate_endianness();
    demonstrate_float_encoding();
    demonstrate_read_exact_vs_read();

    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("  IN-MEMORY ROUND TRIP TEST");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Write to an in-memory buffer
    let mut buffer = Vec::new();

    write_u32(&mut buffer, 42).unwrap();
    write_f32(&mut buffer, 3.14159).unwrap();
    write_u64(&mut buffer, 9999999999).unwrap();
    write_f32_slice(&mut buffer, &[1.0, 2.0, 3.0]).unwrap();

    println!("Wrote {} bytes to buffer", buffer.len());
    println!("Raw bytes: {:02X?}", &buffer);
    println!();

    // Read back from buffer
    let mut cursor = std::io::Cursor::new(&buffer);

    let val_u32 = read_u32(&mut cursor).unwrap();
    let val_f32 = read_f32(&mut cursor).unwrap();
    let val_u64 = read_u64(&mut cursor).unwrap();
    let val_vec = read_f32_vec(&mut cursor, 3).unwrap();

    println!("Read back:");
    println!("  u32: {}", val_u32);
    println!("  f32: {}", val_f32);
    println!("  u64: {}", val_u64);
    println!("  f32 vec: {:?}", val_vec);
    println!();
    println!("✓ Round-trip successful!");
}
