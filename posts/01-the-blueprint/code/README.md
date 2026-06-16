# Post #1: The Blueprint - Code Examples

This directory contains runnable code examples from Post #1.

## Running the Code

To run the cosine similarity preview:

```bash
cd post-01-the-blueprint/code
cargo run --bin cosine-similarity-preview
```

## What's Included

- **cosine-similarity-preview.rs** - A preview of the cosine similarity function we'll build in Post #11
- **api-examples.md** - API design examples (markdown, not executable)

## Note About Edition 2021 vs 2024

**Why Edition 2021?**

When you run `cargo new`, the latest Rust versions create projects with `edition = "2024"` by default. However:

- **Edition 2024** is very new (released late 2024)
- **Edition 2021** is the stable, widely-supported edition
- Edition 2021 has better compatibility with existing crates and tooling

**What's the difference?**

Rust editions are **backwards compatible** - code written for Edition 2021 works in Edition 2024. Editions only change:
- How certain code is parsed (e.g., keyword additions)
- New syntax features
- Minor behavior changes in edge cases

For this tutorial series, we use **Edition 2021** because:
1. It's stable and battle-tested
2. All dependencies support it
3. It has no drawbacks for what we're building
4. You can easily upgrade to 2024 later if needed

You can read more about editions at: https://doc.rust-lang.org/edition-guide/
