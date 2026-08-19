---
id: split-the-compiler-target-module-into-cohesive-submodules
title: Split the compiler target module into cohesive submodules
status: done
priority: p2
dependencies: []
related: [keep-a-module-size-and-complexity-census-with-a-split-queue]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [refactor, maintainability, compiler, identity-adjacent]
---
## User-visible outcome

`crates/tiler-compiler/src/target.rs` (8,633 lines, 209 top-level items at filing) becomes a set of cohesive submodules under the existing `target/` directory, with the public surface, the descriptor encoding, and every identity byte unchanged.

## Why this exists

Filed 2026-08-19 from Tom's module-size directive. The file carries the whole target-profile surface — keys, provenance sources, measurement contexts, fact rows, the complete-descriptor encoder, and resolution — in one unit; the encoder is identity-bearing (`tiler.target-profile.declaration.v11`, `fact-sources.v4`, checked `descriptor.v10`), which is precisely why its code should be a small, separately readable module rather than a stratum inside 8.6k lines.

## Required work

- Read the file in full first; derive the seams from cohesion (key/validation vocabulary, provenance and source types, context types, per-family row machinery, the descriptor encoder, resolution surface). The `target/` directory already exists (`target/feasibility.rs`); place new submodules beside it and keep `target.rs`'s `mod`/`pub use` spine as the single point of contact.
- **Touch only `target.rs` and new files under `target/`.** No other tiler-compiler file may change — a concurrent migration branch owns a disjoint file set in this crate and the two diffs must not meet. `target/feasibility.rs` stays unedited.
- Pure code motion: no public item added, removed, renamed, or re-signatured; minimal internal visibility widenings, each noted. Module docs move with their code; each new submodule gets a short charter doc.
- **The encoder is the hazard: no identity movement.** Every pinned digest, golden, and perturbation test (including `every_measurement_context_field_moves_the_descriptor` and the domain pins) must pass byte-identically with zero test edits. If the split cannot avoid moving a byte, stop and report.

## Evidence and checks

`cargo check -p tiler-compiler`, `cargo nextest run -p tiler-compiler`, `cargo test -p tiler-compiler --doc`, clippy warnings-denied, `cargo fmt --check`, rustdoc warnings-denied, `tkt lint`, `git diff --check`, `tkt guard` against the true base. Report the submodule inventory, visibility widenings, and zero-test-edit confirmation.

## Non-goals

Behaviour/API changes, editing `target/feasibility.rs` or any other compiler file, moving feasibility logic, and the `request.rs` split (own ticket, sequenced behind the contraction merge).

## Closes when

The split lands with all gates green, zero test-file edits, and the inventory recorded.
