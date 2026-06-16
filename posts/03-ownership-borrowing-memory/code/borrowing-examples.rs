// borrowing-examples.rs
//
// Runnable examples demonstrating Rust's borrowing system.
// From Post #3: Ownership, Borrowing, and Memory Management
//
// Run with: rustc borrowing-examples.rs && ./borrowing-examples

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  RUST BORROWING EXAMPLES");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 1: Immutable References (&T)
    // ─────────────────────────────────────────────────────────────────
    println!("1. IMMUTABLE REFERENCES (&T)");
    println!("─────────────────────────────────────────────────────────────");

    let s1 = String::from("hello");

    // Pass a reference - we're BORROWING, not taking ownership
    let len = calculate_length(&s1);

    println!("   The length of \"{}\" is {}.", s1, len);
    println!("   → s1 is still valid because we only borrowed it!");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 2: Multiple Immutable References
    // ─────────────────────────────────────────────────────────────────
    println!("2. MULTIPLE IMMUTABLE REFERENCES");
    println!("─────────────────────────────────────────────────────────────");

    let s = String::from("shared data");

    let r1 = &s; // First immutable borrow
    let r2 = &s; // Second immutable borrow - OK!
    let r3 = &s; // Third immutable borrow - still OK!

    println!("   r1: \"{}\", r2: \"{}\", r3: \"{}\"", r1, r2, r3);
    println!("   → Multiple readers are allowed simultaneously.");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 3: Mutable References (&mut T)
    // ─────────────────────────────────────────────────────────────────
    println!("3. MUTABLE REFERENCES (&mut T)");
    println!("─────────────────────────────────────────────────────────────");

    let mut s = String::from("hello");
    println!("   Before: \"{}\"", s);

    // Pass a mutable reference - allows modification
    append_world(&mut s);

    println!("   After:  \"{}\"", s);
    println!("   → The function modified our string through the &mut reference.");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 4: Only One Mutable Reference at a Time
    // ─────────────────────────────────────────────────────────────────
    println!("4. EXCLUSIVE MUTABLE ACCESS");
    println!("─────────────────────────────────────────────────────────────");

    let mut s = String::from("exclusive");

    {
        let r1 = &mut s;
        r1.push_str(" access");
        println!("   r1 modified: \"{}\"", r1);
    } // r1 goes out of scope here

    // Now we can create another mutable reference
    let r2 = &mut s;
    r2.push_str(" granted");
    println!("   r2 modified: \"{}\"", r2);
    println!("   → Only one &mut at a time, but they can be sequential.");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 5: Non-Lexical Lifetimes (NLL)
    // ─────────────────────────────────────────────────────────────────
    println!("5. NON-LEXICAL LIFETIMES (NLL)");
    println!("─────────────────────────────────────────────────────────────");

    let mut s = String::from("nll demo");

    let r1 = &s;
    let r2 = &s;
    println!("   Immutable borrows: r1=\"{}\", r2=\"{}\"", r1, r2);
    // r1 and r2 are no longer used after this line

    // Because of NLL, Rust knows r1 and r2 are "done"
    // So we can take a mutable reference now!
    let r3 = &mut s;
    r3.push_str(" - modified");
    println!("   Mutable borrow:    r3=\"{}\"", r3);
    println!("   → Rust tracks when references are LAST USED, not scope end.");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 6: The Borrowing Rules - Preventing Data Races
    // ─────────────────────────────────────────────────────────────────
    println!("6. DATA RACE PREVENTION");
    println!("─────────────────────────────────────────────────────────────");
    println!("   The following code would NOT compile:");
    println!();
    println!("   let mut s = String::from(\"hello\");");
    println!("   let r1 = &s;      // immutable borrow");
    println!("   let r2 = &mut s;  // ❌ ERROR: mutable borrow while immutable exists");
    println!("   println!(\"{{}}, {{}}\", r1, r2);");
    println!();
    println!("   Error: cannot borrow `s` as mutable because it is");
    println!("          also borrowed as immutable");
    println!();
    println!("   → This prevents data races at COMPILE TIME!");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 7: References in Functions
    // ─────────────────────────────────────────────────────────────────
    println!("7. PRACTICAL PATTERN: Borrow in Functions");
    println!("─────────────────────────────────────────────────────────────");

    let numbers: Vec<i32> = vec![1, 2, 3, 4, 5];

    // We borrow the vector - function can read but not take ownership
    let sum = sum_values(&numbers);
    let avg = average(&numbers);

    println!("   numbers = {:?}", numbers); // Still ours!
    println!("   sum = {}, average = {:.2}", sum, avg);
    println!("   → Multiple functions can borrow the same data for reading.");
    println!();

    println!("═══════════════════════════════════════════════════════════");
    println!("  BORROWING RULES SUMMARY:");
    println!("  • &T  = immutable borrow (read-only, many allowed)");
    println!("  • &mut T = mutable borrow (read-write, only ONE allowed)");
    println!("  • Cannot mix &T and &mut T on the same data");
    println!("  • References must always be valid (no dangling pointers)");
    println!("═══════════════════════════════════════════════════════════");
}

/// Borrows a String to calculate its length.
/// The caller retains ownership.
fn calculate_length(s: &String) -> usize {
    s.len()
} // s goes out of scope, but it doesn't own the data, so nothing happens

/// Mutably borrows a String to append text.
fn append_world(some_string: &mut String) {
    some_string.push_str(" world");
}

/// Borrows a slice to calculate sum.
fn sum_values(values: &[i32]) -> i32 {
    values.iter().sum()
}

/// Borrows a slice to calculate average.
fn average(values: &[i32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum: i32 = values.iter().sum();
    sum as f64 / values.len() as f64
}
