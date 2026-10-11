# Contributing to VestaDesk

Work is planned in the Linear project VestaDesk (team `VESTA`).

## Checks

Run these before pushing. They must pass on both Windows and Linux, and CI (`.github/workflows/ci.yml`) runs them on both.

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

In PowerShell, set the variable first: `$env:RUSTDOCFLAGS = "-D warnings"; cargo doc --no-deps`.

## Lint policy

Lints are configured in one place, the `[lints]` table of `Cargo.toml`, and apply to every target: library, binary and tests. Clippy runs with `-D warnings`, so a warning fails the build like an error.

| Lint | Level | Rule |
| --- | --- | --- |
| `unsafe_code` | forbid | No `unsafe`, and no local override. |
| `missing_docs` | warn | Every public item is documented. |
| `unreachable_pub` | warn | Items not reachable from the crate root are `pub(crate)`, not `pub`. |
| `clippy::pedantic` | warn | The whole group, including `missing_errors_doc` and `missing_panics_doc`. |
| `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic` | deny | No panicking shortcuts outside tests (see below). |
| `clippy::todo`, `clippy::unimplemented`, `clippy::dbg_macro` | deny | Unfinished code and debugging leftovers never land, not even in tests. |
| `clippy::print_stdout`, `clippy::print_stderr` | warn | Log through the `log` facade instead of printing. |
| `clippy::allow_attributes_without_reason` | warn | Every lint suppression states its reason. |
| `rustdoc::broken_intra_doc_links` | deny | Doc links must resolve. |
| `rustdoc::missing_crate_level_docs` | warn | The crate has `//!` docs. |

`clippy.toml` adds the test exceptions below, and `rustfmt.toml` sets the formatting options (stable ones only).

### No panics outside tests

Library and binary code never calls `unwrap()`, `expect()` or `panic!`. Instead:

- propagate the error with `?` and let the caller decide;
- or recover with a safe value (`unwrap_or`, `unwrap_or_default`, `unwrap_or_else`, `let … else`), and log the failure if it is unexpected.

Tests may use `unwrap`, `expect` and `panic!` in `#[test]` functions and in anything under `#[cfg(test)]`. In an integration test (`tests/*.rs`) that exception covers only the `#[test]` functions, so start the file with `#![cfg(test)]` to let its helpers unwrap too.

### Suppressing a lint

Fix the code when you can. Otherwise, suppress the lint on the smallest item possible and give the reason in the attribute:

```rust
#[expect(clippy::cast_possible_truncation, reason = "the value is clamped to 0..=255 above")]
```

- Prefer `#[expect]`: Clippy reports it (`unfulfilled_lint_expectations`) as soon as the suppression is no longer needed.
- Use `#[allow(lint, reason = "…")]` only when the lint fires in some builds but not others (on one OS, or only under `cfg(test)`), where an `#[expect]` would go unfulfilled.
- A crate-wide `allow` goes in the `[lints]` table of `Cargo.toml`, with a comment explaining why.

## Conventions

- Follow the [Rust API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html). Reviews look closest at:
  - naming: casing (C-CASE), `as_`/`to_`/`into_` conversions (C-CONV), getters without a `get_` prefix (C-GETTER);
  - common traits: public types implement `Debug` (C-DEBUG) and, where it makes sense, `Clone`, `Default`, `PartialEq` and `Eq` (C-COMMON-TRAITS).
- Format with `cargo fmt`; rustfmt enforces the [Rust Style Guide](https://doc.rust-lang.org/style-guide/).

## Documentation

The developer documentation is generated from the code by `cargo doc`. It is mainly intended for developers, so it must be clear, understandable and easy to read.

- The crate docs in `src/lib.rs` are the front page: what VestaDesk is and how its modules fit together, with a link to each top-level module.
- Every doc comment starts with a one-sentence summary.
- A public function that returns a `Result` has an `# Errors` section that links each error it returns and says when.
- A public function that returns a value computed only from its arguments has an `# Examples` section. Examples use `?` rather than `unwrap` and run as tests with `cargo test --doc`.
- Link to other items with intra-doc links, not URLs.
