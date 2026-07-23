---
id: promote-index-oracle-integration-test
title: Promote the index-region oracle test to an admitted target
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/workspace, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, testing, workspace]
claimed_from: todo
assignee: agent-promote-index-oracle-integration-test
lease_expires_at: 1784834740
---
The IndexRegion oracle's end-to-end cases live in `crates/tiler-reference/src/oracle.rs` rather than a dedicated integration target. That was a scope-respecting choice, not a shortcut: `scripts/check_workspace.py` holds a closed `EXPECTED_TESTS` allowlist that admits only `serial_sum_slice` for `tiler-reference`, so adding `crates/tiler-reference/tests/index_region_oracle.rs` fails the workspace contract, and amending the allowlist needs `implementation/workspace`, which the oracle ticket did not hold.

Move the oracle's integration-style cases into an admitted `tests/` target and register it in `EXPECTED_TESTS`. Keep genuinely unit-level cases (bounded arithmetic, registry identity) in their modules. Verify the moved target exercises the crate through its public API only, since that is the boundary the oracle is meant to prove. Run the complete repository gate before completion.

## Outcome

All eight `#[test]` cases in `crates/tiler-reference/src/oracle.rs`'s `mod tests` were end-to-end, driving the oracle exclusively through the crate's re-exported public surface. They now live in a new admitted integration target `crates/tiler-reference/tests/index_region_oracle.rs`; the in-crate `#[cfg(test)] mod tests` block was removed from `oracle.rs`.

Public-vs-private criterion applied: a case belongs in the integration target only if it compiles against the crate's public API alone. Every moved case does — the oracle re-exports its full surface from the crate root (`pub use oracle::{...}`), and the tests reach only `tiler_reference::*` and `tiler_ir::*` public items (verified: no `.0` private-field access, no crate-private constants such as `MAX_REFERENCE_*`, and no private helpers like `compute_scalar_reference_identity`/`encode_*`). The module's imports were rewritten from `use super::*; use crate::{...}` to explicit public paths; the moved bodies are otherwise byte-identical (extracted verbatim, then `cargo fmt`). No case required a private item, so none stayed behind. The crate's genuinely unit-level tests were never in this module: bounded-arithmetic cases remain in `src/arithmetic.rs` and the semantic reference-identity cases remain in `src/lib.rs`; both were left untouched.

The new target was registered in `scripts/check_workspace.py`'s closed `EXPECTED_TESTS["tiler-reference"]` allowlist, mirroring the existing `serial_sum_slice` entry exactly (`"index_region_oracle": "crates/tiler-reference/tests/index_region_oracle.rs"`); the allowlist was extended, not weakened. The workspace boundary contract, which reconciles this allowlist against resolved Cargo target metadata, passes with the new auto-discovered `[[test]]` target — no member `Cargo.toml` change was needed.

Verification: the eight moved tests pass under nextest; `uv run --locked python scripts/check_repository.py` passes end-to-end (workspace contract, formatting, Clippy, dev and optimized nextest runs of `index_region_oracle`, doctests, rustdoc); `git diff --check` is clean; `tkt guard` verdict `ok` within declared scopes.
