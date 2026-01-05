// ownership-examples.rs
//
// Runnable examples demonstrating Rust's ownership system.
// From Post #3: Ownership, Borrowing, and Memory Management
//
// Run with: rustc ownership-examples.rs && ./ownership-examples

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  RUST OWNERSHIP EXAMPLES");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 1: Stack values are COPIED (not moved)
    // ─────────────────────────────────────────────────────────────────
    println!("1. COPY TYPES (Stack)");
    println!("─────────────────────────────────────────────────────────────");

    let x: i32 = 42;
    let y = x; // x is COPIED to y (i32 implements Copy trait)

    println!("   x = {}", x); // ✅ x is still valid!
    println!("   y = {}", y); // ✅ y is an independent copy
    println!("   → Simple types (i32, f32, bool) are copied, not moved.");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 2: Heap values are MOVED (not copied)
    // ─────────────────────────────────────────────────────────────────
    println!("2. MOVE SEMANTICS (Heap)");
    println!("─────────────────────────────────────────────────────────────");

    let s1 = String::from("hello");
    println!("   s1 owns: \"{}\"", s1);

    let s2 = s1; // Ownership MOVES from s1 to s2
    println!("   After `let s2 = s1;`:");
    println!("   s2 now owns: \"{}\"", s2);
    // println!("{}", s1); // ❌ Would error: "borrow of moved value"
    println!("   s1 is now INVALID (would cause compile error if used)");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 3: Clone creates a deep copy
    // ─────────────────────────────────────────────────────────────────
    println!("3. CLONING (Deep Copy)");
    println!("─────────────────────────────────────────────────────────────");

    let original = String::from("deep copy me");
    let cloned = original.clone(); // Allocates new memory on heap

    println!("   original: \"{}\"", original); // ✅ Still valid
    println!("   cloned:   \"{}\"", cloned); // ✅ Independent copy
    println!("   → Both are valid because clone() made a new heap allocation.");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 4: Ownership and Functions
    // ─────────────────────────────────────────────────────────────────
    println!("4. FUNCTIONS AND OWNERSHIP");
    println!("─────────────────────────────────────────────────────────────");

    let s = String::from("passed to function");
    println!("   Before: s = \"{}\"", s);

    takes_ownership(s); // s's ownership moves into the function
                        // println!("{}", s); // ❌ Would error: s has been moved
    println!("   After: s is no longer valid in main()");
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 5: Returning Ownership
    // ─────────────────────────────────────────────────────────────────
    println!("5. RETURNING OWNERSHIP");
    println!("─────────────────────────────────────────────────────────────");

    let s1 = gives_ownership(); // Function returns ownership
    println!("   Received from function: \"{}\"", s1);

    let s2 = String::from("take and give back");
    let s3 = takes_and_gives_back(s2); // s2 moved in, new owner is s3
    println!("   After take_and_give_back: \"{}\"", s3);
    println!();

    // ─────────────────────────────────────────────────────────────────
    // EXAMPLE 6: Scope and Drop
    // ─────────────────────────────────────────────────────────────────
    println!("6. SCOPE AND AUTOMATIC CLEANUP");
    println!("─────────────────────────────────────────────────────────────");

    {
        let scoped = String::from("I only exist in this block");
        println!("   Inside block: \"{}\"", scoped);
    } // scoped is dropped here - memory freed automatically

    println!("   Outside block: scoped no longer exists (memory freed)");
    println!();

    println!("═══════════════════════════════════════════════════════════");
    println!("  KEY TAKEAWAYS:");
    println!("  • Stack types (i32, f32, bool) are COPIED");
    println!("  • Heap types (String, Vec) are MOVED by default");
    println!("  • Use .clone() for explicit deep copies (expensive!)");
    println!("  • Ownership moves into functions unless returned");
    println!("  • Memory is freed when the owner goes out of scope");
    println!("═══════════════════════════════════════════════════════════");
}

/// This function takes ownership of the String passed to it.
/// After this call, the caller can no longer use that String.
fn takes_ownership(some_string: String) {
    println!("   Inside function: \"{}\"", some_string);
} // some_string goes out of scope, drop() is called, memory freed

/// This function creates a new String and transfers ownership to the caller.
fn gives_ownership() -> String {
    let some_string = String::from("created inside function");
    some_string // Ownership moves to caller
}

/// This function takes ownership AND gives it back.
/// The returned String may or may not be the same one that was passed in.
fn takes_and_gives_back(mut a_string: String) -> String {
    a_string.push_str(" (modified)");
    a_string // Ownership moves back to caller
}
