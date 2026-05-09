# Contributing

This document covers how to build, test, and release `ftd-acl-optimizer`.

## Overview

The project is a single-binary Rust crate. All development tasks go through `cargo`.

## Prerequisites

- Rust stable toolchain — install via [rustup](https://rustup.rs):
  ```bash
  rustup update stable
  ```
- No external system dependencies required.

## Build

```bash
# Debug build (faster compile, slower binary)
cargo build

# Release build (optimized)
cargo build --release
```

The debug binary is at `target/debug/ftd-acl-optimizer`.  
The release binary is at `target/release/ftd-acl-optimizer`.

## Run Tests

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test acp::reader

# Run a single test by name
cargo test test_next_rule_single_rule

# Run ignored (slow) tests
cargo test -- --ignored
```

## Test Layout

- **Unit tests** live in `#[cfg(test)] mod tests` blocks at the bottom of each source file.
- **Integration tests** belong in a `tests/` directory at the workspace root (one file per feature area).
- Fixture files for integration tests go in `tests/fixtures/`.

See [testing instructions](../../.github/instructions/testing.instructions.md) for naming conventions and assertion guidelines.

## Code Style

Use the standard Rust formatter and linter before committing:

```bash
cargo fmt
cargo clippy -- -D warnings
```

## Adding a New Command

1. Add the new subcommand variant to `src/cli/args.rs`.
2. Add the handler function to `src/cli/mod.rs`.
3. Wire it in `src/main.rs` inside the relevant `match` arm.
4. Update [CLI Reference](../cli/commands.md) with the new command, flags, and example output.

## Adding a New Optimization Type

1. Implement the detection logic in `src/acp/rule/network_object/utilities.rs`.
2. Expose it through `NetworkObject` and `Rule::get_optimized_networks`.
3. Add a new variant to `DescriptionType` in `src/acp/rule/protocol_object/description.rs` if needed.
4. Add unit tests covering the new case.
5. Update [Optimization Types](../concepts/optimization-types.md).

## Releasing

1. Bump the version in `Cargo.toml`.
2. Verify `cargo test` passes.
3. Build the release binary: `cargo build --release`.
4. Tag the commit: `git tag v<version>`.

## See Also

- [Architecture](../architecture/overview.md)
- [Getting Started](../guides/getting-started.md)
