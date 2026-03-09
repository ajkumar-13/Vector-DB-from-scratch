// mmap-basics.rs
//
// Basic memory mapping examples.
// From Post #7: Memory Mapping (mmap)
//
// This file demonstrates the fundamentals of memory mapping
// before we build the full MmapSegment abstraction.
//
// NOTE: This file requires the memmap2 crate.
// To run: create a Cargo project and add memmap2 = "0.9" to dependencies.

use std::fs::File;
use std::io::{self, Write};

// ═══════════════════════════════════════════════════════════════════════════
// EXAMPLE 1: Basic File Mapping
// ═══════════════════════════════════════════════════════════════════════════

/// Demonstrates the simplest possible mmap usage
fn example_basic_mmap() -> io::Result<()> {
    println!("═══════════════════════════════════════════════════════════");
    println!("  EXAMPLE 1: Basic Memory Mapping");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // First, create a test file
    let path = "test_basic.bin";
    {
        let mut file = File::create(path)?;
        file.write_all(b"Hello, Memory Mapping!")?;
    }

    // Now map it
    let file = File::open(path)?;

    // IMPORTANT: mmap is unsafe because:
    // 1. Another process could modify/truncate the file
    // 2. This would cause undefined behavior (SIGBUS crash)
    #[cfg(feature = "memmap2")]
    {
        use memmap2::Mmap;
        let mmap = unsafe { Mmap::map(&file)? };

        println!("File size: {} bytes", mmap.len());
        println!("Contents: {:?}", std::str::from_utf8(&mmap).unwrap());

        // Access individual bytes
        println!("Byte 0: '{}' (0x{:02X})", mmap[0] as char, mmap[0]);
        println!("Byte 7: '{}' (0x{:02X})", mmap[7] as char, mmap[7]);
    }

    #[cfg(not(feature = "memmap2"))]
    {
        println!("(memmap2 not available, showing conceptual code)");
        println!();
        println!("  let mmap = unsafe {{ Mmap::map(&file)? }};");
        println!("  let byte = mmap[0];  // Access like an array!");
        println!("  let slice = &mmap[0..10];  // Slice it!");
    }

    // Cleanup
    std::fs::remove_file(path)?;
    println!();
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// EXAMPLE 2: Understanding Page Faults
// ═══════════════════════════════════════════════════════════════════════════

/// Explains the lazy loading behavior of mmap
fn example_page_faults() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  EXAMPLE 2: Page Faults (Lazy Loading)");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    println!("When you mmap a 1GB file:");
    println!();
    println!("  1. OS creates virtual address range (1GB of addresses)");
    println!("  2. OS marks all pages as 'not present'");
    println!("  3. NO data is loaded from disk yet!");
    println!();
    println!("When you access mmap[1000]:");
    println!();
    println!("  1. CPU tries to read virtual address");
    println!("  2. Page table says 'not present' → PAGE FAULT");
    println!("  3. OS kernel handles the fault:");
    println!("     a. Finds a free physical page");
    println!("     b. Reads 4KB from disk into that page");
    println!("     c. Updates page table");
    println!("  4. CPU retries the read → SUCCESS!");
    println!();
    println!("This is called 'demand paging' or 'lazy loading'.");
    println!();

    // Visual representation
    println!("  Virtual Address Space         Physical RAM");
    println!("  ┌─────────────────────┐       ┌─────────────────┐");
    println!("  │ Page 0 (not loaded) │       │ Page A: code    │");
    println!("  │ Page 1 (not loaded) │       │ Page B: stack   │");
    println!("  │ Page 2 (ACCESSED!) ─┼──────►│ Page C: mmap[2] │");
    println!("  │ Page 3 (not loaded) │       │ Page D: free    │");
    println!("  │ ...                 │       │ ...             │");
    println!("  │ Page 999 (not load) │       │                 │");
    println!("  └─────────────────────┘       └─────────────────┘");
    println!();
}

// ═══════════════════════════════════════════════════════════════════════════
// EXAMPLE 3: Casting Bytes to Primitives
// ═══════════════════════════════════════════════════════════════════════════

/// Shows how to interpret raw bytes as structured data
fn example_byte_casting() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  EXAMPLE 3: Casting Bytes to Primitives");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Simulate what mmap returns: raw bytes
    let bytes: &[u8] = &[
        0x00, 0x00, 0x80, 0x3F, // 1.0 as f32 (Little Endian)
        0x00, 0x00, 0x00, 0x40, // 2.0 as f32 (Little Endian)
        0x00, 0x00, 0x40, 0x40, // 3.0 as f32 (Little Endian)
    ];

    println!("Raw bytes: {:02X?}", bytes);
    println!();

    // Method 1: Manual parsing (safe but verbose)
    println!("Method 1: from_le_bytes (safe)");
    let f1 = f32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let f2 = f32::from_le_bytes(bytes[4..8].try_into().unwrap());
    let f3 = f32::from_le_bytes(bytes[8..12].try_into().unwrap());
    println!("  Values: [{}, {}, {}]", f1, f2, f3);
    println!();

    // Method 2: Pointer casting (unsafe, zero-copy)
    println!("Method 2: Pointer cast (unsafe, zero-copy)");
    let floats: &[f32] = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const f32, 3) };
    println!("  Values: {:?}", floats);
    println!();

    // Method 3: bytemuck (safe wrapper, zero-copy)
    println!("Method 3: bytemuck::cast_slice (safe, zero-copy)");
    println!("  (requires bytemuck crate)");
    println!("  let floats: &[f32] = bytemuck::cast_slice(bytes);");
    println!();

    // Verify they're the same
    assert_eq!(f1, floats[0]);
    assert_eq!(f2, floats[1]);
    assert_eq!(f3, floats[2]);
    println!("✓ All methods produce identical results!");
    println!();
}

// ═══════════════════════════════════════════════════════════════════════════
// EXAMPLE 4: Why mmap is "unsafe"
// ═══════════════════════════════════════════════════════════════════════════

fn example_mmap_dangers() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  EXAMPLE 4: Why mmap Requires `unsafe`");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    println!("Scenario: Two processes, same file");
    println!();
    println!("  Process A                    Process B");
    println!("  ─────────                    ─────────");
    println!("  mmap('data.vec')");
    println!("  → gets 1GB virtual range");
    println!("                               open('data.vec')");
    println!("  read mmap[500MB]");
    println!("  → page fault, loads data");
    println!("  → returns value ✓");
    println!("                               file.set_len(100MB)");
    println!("                               → TRUNCATES FILE!");
    println!("  read mmap[500MB]");
    println!("  → page fault...");
    println!("  → OS: 'that part of file");
    println!("         doesn't exist!'");
    println!("  → SIGBUS CRASH! 💥");
    println!();
    println!("This violates Rust's safety guarantees:");
    println!("  - mmap returns &[u8] which implies valid memory");
    println!("  - But external mutation can invalidate it");
    println!();
    println!("Solutions:");
    println!("  1. Use file locking (flock/fcntl)");
    println!("  2. Don't modify mapped files externally");
    println!("  3. Use copy-on-write mappings");
    println!("  4. Accept the risk with `unsafe`");
    println!();
}

// ═══════════════════════════════════════════════════════════════════════════
// EXAMPLE 5: Performance Model
// ═══════════════════════════════════════════════════════════════════════════

fn example_performance() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  EXAMPLE 5: Performance Model");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    println!("Standard read() Performance:");
    println!();
    println!("  ┌────────┐     ┌──────────────┐     ┌─────────────┐");
    println!("  │  Disk  │ ──► │ Kernel Cache │ ──► │ User Buffer │");
    println!("  └────────┘     └──────────────┘     └─────────────┘");
    println!("                      Copy #1              Copy #2");
    println!();
    println!("  Time: O(file_size) - must read entire file");
    println!("  RAM:  2× file_size (kernel cache + your buffer)");
    println!();

    println!("mmap Performance:");
    println!();
    println!("  ┌────────┐     ┌──────────────────────────────────┐");
    println!("  │  Disk  │ ──► │ Page Cache (directly accessible) │");
    println!("  └────────┘     └──────────────────────────────────┘");
    println!("                              ↑");
    println!("                      Your pointer lands here");
    println!("                        (zero copies!)");
    println!();
    println!("  Time: O(1) to map, O(pages_accessed) to use");
    println!("  RAM:  Only pages you touch (4KB each)");
    println!();

    println!("When to use each:");
    println!();
    println!("  ┌─────────────────────────────────────────────────────┐");
    println!("  │ Use read() when:                                    │");
    println!("  │  • File is small (< 10MB)                           │");
    println!("  │  • You need all the data                            │");
    println!("  │  • Sequential processing, one time                  │");
    println!("  ├─────────────────────────────────────────────────────┤");
    println!("  │ Use mmap when:                                      │");
    println!("  │  • File is larger than RAM                          │");
    println!("  │  • Random access patterns                           │");
    println!("  │  • Multiple reads over time                         │");
    println!("  │  • Sharing between processes                        │");
    println!("  └─────────────────────────────────────────────────────┘");
    println!();
}

// ═══════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════

fn main() -> io::Result<()> {
    println!();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           MEMORY MAPPING (mmap) FUNDAMENTALS              ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    example_basic_mmap()?;
    example_page_faults();
    example_byte_casting();
    example_mmap_dangers();
    example_performance();

    println!("═══════════════════════════════════════════════════════════");
    println!("  NEXT STEPS");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("Now that you understand the basics, see mmap-segment.rs");
    println!("for the full MmapSegment implementation that wraps our");
    println!("binary file format from Post #6.");
    println!();

    Ok(())
}
