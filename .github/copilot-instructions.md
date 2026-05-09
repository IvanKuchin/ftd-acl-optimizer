# GitHub Copilot Instructions

This file defines rules and conventions for AI-assisted code generation in the `ftd-acl-optimizer` project. Follow these guidelines whenever adding or modifying any code.

## Project Overview

`ftd-acl-optimizer` is a Rust CLI tool that parses Cisco FTD Access Control Policy (ACP) rule files and produces optimization reports — identifying shadowed, adjacent, and overlapping network/protocol objects.

- **Language:** Rust (edition 2021)
- **CLI framework:** `clap` (derive feature)
- **Error handling:** `thiserror`
- **Binary name:** `ftd-acl-optimizer`

## Module Structure

```
src/
├── main.rs                  — entry point; routes CLI args to cli:: functions
├── cli/
│   ├── mod.rs               — public API: analyze_rule, analyze_acp, analyze_topk_*
│   ├── args.rs              — clap argument structs (AppArgs, Verb, Entity, …)
│   └── utils.rs             — file I/O and formatted output helpers
└── acp/
    ├── mod.rs               — Acp struct (Vec<Rule>) with capacity/lookup methods
    ├── reader.rs            — line-by-line parser splitting raw text into rule blocks
    └── rule/
        ├── mod.rs           — Rule struct: name + src/dst networks + src/dst protocols
        ├── network_object/  — NetworkObject parsing and subnet math (adjacency, shadow, overlap)
        └── protocol_object/ — ProtocolObject parsing and port/protocol optimization
```

Data flows downward through typed structs and `TryFrom` conversions:
`file → cli::utils::read_acp_from_file → acp::Reader::next_rule → acp::Rule::try_from`.

## Code Style

- Follow standard Rust idioms and the conventions already present in the file being edited.
- Use `thiserror` for all custom error types. Define error variants in the module they belong to.
- Prefer `?` for error propagation over explicit `match` on `Result`/`Option` where idiomatic.
- Do not use `unwrap()` in production code — use `?`, `expect("reason")` only in tests.
- Keep functions small and single-purpose. Extract helpers when a function exceeds ~40 lines.
- Use `TryFrom`/`TryInto` for fallible conversions between domain types.

## Documentation Requirements

Every piece of code added or modified **must** be documented:

- **Modules** (`mod.rs` and top-level modules): Add a `//!` module-level doc comment explaining the module's responsibility, its main types, and how it fits into the data flow.
- **Public structs, enums, and traits**: Add a `///` doc comment describing the type's purpose and any important invariants.
- **Public and private functions/methods**: Add a `///` doc comment describing what the function does, its parameters, return value, and any errors it can return.
- **Non-obvious implementation blocks**: Add inline `//` comments explaining *why*, not just *what*.
- **Error variants**: Document each variant with `///` explaining when it is produced.

Example style:

```rust
/// Parses a single ACL rule block from its raw text lines.
///
/// # Errors
///
/// Returns [`RuleError::MissingName`] if the first line does not contain a rule name.
impl TryFrom<Vec<String>> for Rule {
    type Error = RuleError;

    fn try_from(lines: Vec<String>) -> Result<Self, Self::Error> {
        // Implementation
    }
}
```

## Testing Requirements

**Every feature addition and every bug fix must include both unit and integration tests.**

### Unit Tests

- Place unit tests in the **same file** as the code under test, inside a `#[cfg(test)]` module at the bottom of the file.
- Name test functions: `test_<unit>_<scenario>`, e.g. `test_next_rule_empty_input`.
- Each test covers exactly **one scenario**. Do not mix multiple behaviors in one test.
- Cover both the happy path and edge cases (empty input, single item, boundary values, error paths).
- Use `assert_eq!` for equality, `assert!` for booleans, `assert!(matches!(...))` for enum variants.
- Do not use `unwrap()` — use `expect("descriptive message")`.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_rule_single_rule() {
        // arrange
        // act
        // assert
    }
}
```

### Integration Tests

- Place integration tests in `tests/` at the workspace root (sibling of `src/`).
- One file per high-level feature area, e.g. `tests/reader.rs`, `tests/cli.rs`.
- Integration tests exercise the **public API only** — no access to private internals.
- Store fixture files in `tests/fixtures/` and load them with `include_str!` or `std::fs::read_to_string`.
- Name integration test functions the same way: `test_<feature>_<scenario>`.

### General Testing Rules

- Run `cargo test` before considering any change complete.
- Keep tests deterministic: no random data, no network calls, no filesystem writes outside `tests/fixtures/`.
- Slow tests must be marked `#[ignore]` with a comment explaining why.
- If setup is shared across several tests, extract a helper function in the same module.

## Documentation Files

When modifying or adding features, keep the docs in sync:

- New CLI subcommand → update `docs/cli/commands.md`.
- New optimization strategy → update or create a doc in `docs/concepts/`.
- Module structure changes → update `docs/architecture/overview.md`.
- New doc file → add a link in `docs/README.md`.

See `.github/instructions/documentation.instructions.md` for full documentation conventions.
See `.github/instructions/testing.instructions.md` for full testing conventions.
