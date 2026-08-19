---
id: split-the-index-refinement-module-into-cohesive-submodules
title: Split the index refinement module into cohesive submodules
status: done
priority: p3
dependencies: []
related: [keep-a-module-size-and-complexity-census-with-a-split-queue]
scopes: [implementation/ir, research/documentation]
shared_scopes: [project/tickets]
paths: []
tags: [refactor, maintainability, ir]
---
## User-visible outcome

`crates/tiler-ir/src/index/refinement.rs` (7,101 lines at filing) becomes a `refinement/` directory of cohesive submodules with the public surface, behaviour, and every identity byte unchanged.

## Why this exists

Filed 2026-08-19 from Tom's module-size directive; second-tranche member kept off the immediate dispatch so review and integration capacity stays reserved while three sibling splits and a migration integration are in flight.

## Required work

The sibling split tickets' discipline verbatim: full read first; seams from cohesion; directory-module conversion leaving the declaring `mod refinement;` line untouched; pure code motion, zero public-surface movement, minimal recorded visibility widenings, zero identity/pin/test movement with zero test edits; per-submodule charter docs.

## Evidence and checks

`cargo check -p tiler-ir`, `cargo nextest run -p tiler-ir`, `cargo test -p tiler-ir --doc`, clippy warnings-denied, `cargo fmt --check`, rustdoc warnings-denied, `tkt lint`, `git diff --check`, `tkt guard` against the true base; submodule inventory and zero-test-edit confirmation in the delivery report.

## Closes when

The split lands with all gates green, zero test edits, and the inventory recorded.
