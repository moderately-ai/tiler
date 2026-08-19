---
id: split-the-artifact-program-test-monoliths-into-focused-modules
title: Split the artifact program test monoliths into focused modules
status: todo
priority: p2
dependencies: []
related: [keep-a-module-size-and-complexity-census-with-a-split-queue]
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [refactor, maintainability, artifact, tests]
---
## User-visible outcome

`crates/tiler-artifact/src/program/tests.rs` (7,936 lines, ~278 items at filing) and `crates/tiler-artifact/src/program/codec/tests.rs` (5,795 lines) become directories of focused test modules grouped by the property family they guard (identity pins, builder refusals, codec round-trips, decode validation, retained-environment subjects, …), so a reviewer auditing one property family reads one small file.

## Why this exists

Filed 2026-08-19 from Tom's module-size directive. These monoliths hold the artifact's correctness-bearing evidence — including identity pins the accepted symbolic-packaging migration (item 24) will soon need to re-derive — and their size makes the read-the-tests-in-full obligation needlessly expensive exactly where it matters most.

## Required work

- Read both files in full first; group tests by the property family each guards, keeping shared fixtures/helpers in one clearly named support module rather than duplicating them. Preserve every test byte-for-byte in assertion content — this is file reorganization, not test revision.
- Convert each `tests.rs` into a `tests/` directory module; the declaring `mod tests;` lines and `#[cfg(test)]` gating keep working unchanged. Production code is untouched.
- Test names must not change (they are cited from tickets and docs by name); if a helper must be renamed to resolve a collision, record it. Zero assertion, pin, or golden changes.
- Fence: only the two named test files (becoming directories) and any support module they extract may change; no production `.rs` file in the crate moves.

## Evidence and checks

`cargo nextest run -p tiler-artifact` with the same test count before and after (state both counts — a shrinking population is a defect), `cargo test -p tiler-artifact --doc`, clippy warnings-denied, `cargo fmt --check`, `tkt lint`, `git diff --check`, `tkt guard`. Report the module inventory and the before/after test-count equality.

## Non-goals

New tests, assertion changes, production-code edits, and other crates' test monoliths.

## Closes when

Both directories land with identical test populations, all gates green, and the inventory recorded.
