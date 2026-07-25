---
id: share-the-serial-sum-artifact-assembler
title: Make the serial-sum artifact assembler reachable from the runner
status: todo
priority: p0
dependencies: []
related: [route-the-runtime-proof-through-the-artifact-envelope]
scopes: [implementation/metal-aot, implementation/runtime, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, workspace, runtime, needs-tom]
---
`route-the-runtime-proof-through-the-artifact-envelope` cannot be done because the only artifact assembler in the workspace is private to a binary crate. This ticket makes it reachable. It is filed rather than decided because both ways of doing it cross a boundary a worker should not choose alone.

## Fact — the assembler exists exactly once, and it is unreachable

`grep -rn "ArtifactProgramBuilder" crates prototypes --include "*.rs"` returns one non-`tiler-artifact` user: `prototypes/serial-sum-compile/src/bundle.rs`. `prototypes/serial-sum-compile/Cargo.toml` declares `[[bin]]` and no `[lib]`, and `src/main.rs:23-25` declares `mod bundle; mod payload; mod target;` — private modules of a binary crate. Nothing outside that package can name them.

## Fact — the runtime proof needs it, and needs nothing else

`prototypes/serial-sum-run` already depends on `tiler-compiler`, `tiler-ir`, `tiler-metal`, `tiler-metal-aot`, `tiler-artifact`, `tiler-reference`, and `metal`, and after `admit-the-device-free-runtime-validation-crate` it can depend on `tiler-runtime`. Every input the loader needs is already in that process: `compilation.target_profile_key()` and `target_profile_descriptor()` give the `TargetProfileRef` for an `ExecutionEnvironment`; `"tiler.metal"`/`"metallib"` are the backend and representation keys; `VerifiedArtifactProgram::canonical_identity()` is the `expected` identity `DecodedProgram::preflight` binds against. The one missing value is the `VerifiedArtifactProgram` itself, and only the assembler produces one.

## Fact — a cold handoff does not avoid this

Writing the envelope to a file from the producer and reading it in the runner removes the need for an assembler in the runner and breaks two other things. The runner would have no `expected` identity to bind against except the one re-derived from the same bytes, which is vacuous; and it still could not obtain the entry symbol, because the payload-metadata section has no public parser (`decode_metadata` is `pub(crate)` at `crates/tiler-artifact/src/program/codec/payload.rs:292`). Binding by identity is only available to a consumer that holds the program it compiled, which is the single-process shape.

## The choice

**(a) Give `tiler-prototype-compile` a `[lib]` target** exposing `bundle`, `payload`, `target`, and the program/emit helpers, with `main.rs` a thin driver over it, and add a `tiler-prototype-run` -> `tiler-prototype-compile` edge. Cheapest and keeps one assembler. Costs: `scripts/check_workspace.py`'s `expected_member_manifest` and `expected_targets` both hard-code that a `tiler-prototype-*` package has exactly one `[[bin]]` target and no `[lib]`, so both must be relaxed; and it creates a public namespace on a package that has none, which is the category ADR 0075 routes to Tom even though the package is `publish = false`.

**(b) Promote the assembler to a library crate.** It needs `tiler_compiler::session` and `tiler_artifact::program` at once, and neither crate may depend on the other, so it cannot join either — it would be a new workspace member and therefore a new crate admission with its own ADR, on the pattern of ADR 0077 and ADR 0081.

**(c) Duplicate the assembler into `prototypes/serial-sum-run`.** Roughly 300-340 lines of identity-bearing logic — arena replay, target-profile and feasibility-rule minting, the checked capability-revision narrowing — in a second copy that can drift. Rejected here rather than left open: two independently maintained descriptions of one compilation is the exact defect the routing ticket exists to remove, and creating a second one to remove the first is not a trade.

**Recommendation: (a).** It is the smallest change that leaves one assembler, and the two prototypes are halves of one named vertical slice, both `publish = false`. The `check_workspace.py` relaxation should stay narrow — permit a `[lib]` on a prototype rather than dropping the target pin — so the shape stays checked.

## What closes this

The assembler is reachable from `prototypes/serial-sum-run` without a second copy, `scripts/check_workspace.py` pins whatever new shape results, and `route-the-runtime-proof-through-the-artifact-envelope` is unblocked. If (b) is chosen, this ticket also owns the admitting ADR.
