# Post #3: Ownership, Borrowing, and Memory Management - Code Examples

This directory contains runnable examples for Post #3.

## Running the Code

```bash
cd post-03-ownership-borrowing-memory/code

# Run ownership examples
cargo run --bin ownership-examples

# Run borrowing examples  
cargo run --bin borrowing-examples

# Run slice examples
cargo run --bin slice-examples
```

## What's Included

- **ownership-examples.rs** — Copy vs Move, automatic cleanup, RAII
- **borrowing-examples.rs** — Immutable and mutable references, the Golden Rule
- **slice-examples.rs** — Zero-copy views into memory, `&str` and `&[T]`

## Learning Path

1. **Start with ownership-examples.rs** — Understand the three rules
2. **Then borrowing-examples.rs** — Learn when to use references
3. **Finally slice-examples.rs** — See how to avoid unnecessary copies

## Try-It-Yourself Exercises

After running each example, try these:

### Ownership Exercise
```rust
let s1 = String::from("hello");
let s2 = s1;
println!("{}", s1); // What error do you get?
```

### Borrowing Exercise
```rust
let mut s = String::from("hello");
let r1 = &s;
let r2 = &s;
let r3 = &mut s;  // What error do you get?
```

### Slice Exercise
```rust
let v = vec![1, 2, 3];
modify_slice(&v);  // Try to modify through immutable slice

fn modify_slice(values: &[i32]) {
    values[0] = 99;  // What error do you get?
}
```

## Compiler Errors to Expect

These are **intentional** — they help you understand Rust's safety guarantees:

- **E0382**: Borrow of moved value
- **E0502**: Cannot borrow as mutable while borrowed as immutable
- **E0596**: Cannot borrow as mutable (immutable reference)

When you hit these errors, look at the compiler's suggestion — Rust is usually right!

## Key Concepts

| Concept | What It Means |
|---------|--------------|
| **Ownership** | One variable owns each heap value |
| **Move** | Transferring ownership (invalidates the original) |
| **Borrow** | Temporarily reading without ownership (`&T`) |
| **Mut Borrow** | Temporarily modifying with exclusive access (`&mut T`) |
| **Slice** | A zero-copy view into contiguous memory |

## Next Steps

Once you're comfortable with ownership:
- Read the compiler errors carefully — they're teaching you
- Try modifying the examples and see what breaks
- Move on to Post #4 (Structs, Enums, Error Handling)

---

*Ownership is Rust's superpower. Once it clicks, everything else becomes easier.*
