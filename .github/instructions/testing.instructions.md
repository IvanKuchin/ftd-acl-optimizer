---
description: "Use when writing, adding, or updating tests — unit tests, integration tests, test modules, #[test] functions, #[cfg(test)], or tests/ directory. Covers where to place tests, naming conventions, and what to assert."
applyTo: "src/**/*.rs, tests/**/*.rs"
---

# Testing Guidelines

## Unit Tests

- Place unit tests **inside the same file** as the code under test, in a `#[cfg(test)]` module at the bottom of the file:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn test_<function_name>_<scenario>() {
          // arrange
          // act
          // assert
      }
  }
  ```
- Test function names follow the pattern `test_<unit>_<scenario>`, e.g. `test_next_rule_single_rule`, `test_next_rule_no_rules`.
- Import only what is needed with `use super::*;` or explicit paths.
- Each test covers exactly **one scenario**. Do not mix multiple behaviors in one test.
- Use `assert_eq!` for equality, `assert!` for booleans, `assert!(matches!(...))` for enum variants.
- Test both the happy path **and** edge cases (empty input, single item, boundary values).

## Integration Tests

- Place integration tests in `tests/` at the workspace root (sibling of `src/`).
- One file per high-level feature area, e.g. `tests/reader.rs`, `tests/cli.rs`.
- Integration tests exercise the public API only — no access to private internals.
- Use real-world-shaped fixtures. Store fixture files in `tests/fixtures/` and load them with `include_str!` or `std::fs::read_to_string`.
  ```rust
  // tests/fixtures/single_rule.txt  ← store fixture here
  const FIXTURE: &str = include_str!("fixtures/single_rule.txt");
  ```
- Name integration test functions the same way as unit tests: `test_<feature>_<scenario>`.

## General Rules

- Run the full test suite with `cargo test` before submitting.
- Do **not** use `unwrap()` in tests — use `expect("descriptive message")` so failures are readable.
- Keep tests deterministic: no random data, no network calls, no filesystem side effects outside `tests/fixtures/`.
- If a test requires setup/teardown shared across several tests, extract a helper function in the same module — not a global fixture framework.
- Mark tests that are known to be slow with `#[ignore]` and document why; run them explicitly with `cargo test -- --ignored`.
