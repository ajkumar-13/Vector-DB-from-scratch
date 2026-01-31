# Post #2: Setting Up the Forge - Code Examples

This directory contains runnable code examples from Post #2.

## Running the Code

To run the async hello world example:

```bash
cd post-02-setting-up-the-forge/code
cargo run --bin hello-async
```

## What's Included

- **hello-async.rs** - Verifies that the Tokio async runtime is working correctly
- **settings.json** - VS Code configuration for rust-analyzer
- **Cargo.toml** - Project manifest with Tokio dependency

## Expected Output

When you run `cargo run --bin hello-async`, you should see:

```text
The Forge is Hot! Initializing VectorDB...

  [Main] Verifying system configuration...
  [Background] Loading write-ahead log...
  [Main] Configuration OK.
  [Background] WAL loaded. 1,247 entries recovered.
  [Background] Building vector index...
  [Background] Index ready. 50,000 vectors loaded.

All systems operational.
   Listening on http://127.0.0.1:8080
```

Notice how `[Main]` and `[Background]` interleave - that's async concurrency!
