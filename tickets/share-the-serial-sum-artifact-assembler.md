---
id: share-the-serial-sum-artifact-assembler
title: Make the serial-sum artifact assembler reachable from the runner
status: closed
priority: p0
dependencies: []
related: [route-the-runtime-proof-through-the-artifact-envelope]
scopes: [implementation/metal-aot, implementation/runtime, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, workspace, runtime]
closed_reason: obsolete
closed_note: the dispatch record made the cold handoff work; the runner never needed the assembler
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

## Closed 2026-07-25 — the premise is refuted; the runner never needed the assembler

Re-evaluated from `implementation/runtime` at `96fe032`, by reading rather than by assuming this ticket was either right or wrong. Two of its three "Fact" sections still hold. The third is the one everything else rests on, and half of it is now false.

**Retracted — "it still could not obtain the entry symbol, because the payload-metadata section has no public parser."** The cited fact is true and no longer decides anything. `decode_metadata` is still `pub(crate)` (`crates/tiler-artifact/src/program/codec/payload.rs:298`), but `decode_artifact` now *calls* it eagerly for every carried payload and stores the result on the view (`codec/view.rs:148-161`), and `DecodedEntry::backend_symbol` and `DecodedEntry::transport_slots` publish what it parsed. A consumer holding only bytes reads the symbol and the per-slot transport mapping without naming a producer type. `expose-the-dispatch-record-on-a-decoded-artifact` landed that after this ticket was written, and its test `a_decoded_artifact_carries_everything_one_dispatch_needs` (`codec/tests.rs:1325`) asserts exactly this end to end from `decode_artifact`, holding no `VerifiedArtifactProgram`, no semantic program, and no registry.

**Retracted — "the runner would have no `expected` identity to bind against except the one re-derived from the same bytes, which is vacuous."** True of a consumer given only an envelope, and it does not follow that the identity must come from an assembler. `DecodedProgram::preflight` documents two sources: an identity obtained by *building* the artifact, and one *recorded when the bytes were cached*. This ticket considered only the first. The second is a sidecar written by the producer beside the envelope, derived from the `VerifiedArtifactProgram` it built rather than from the encoded bytes, and it catches exactly the class it exists for — a stale artifact, a mixed-up path, a producer run that did not complete. It does not resist an adversary who rewrites both files, and nothing in an unsigned sidecar could; the check is worth what whatever wrote it is worth, which is stated rather than overclaimed.

**Standing — the assembler exists once, and is private to a binary crate.** Unchanged and still true. It is simply not on the runner's path.

## What replaced it

`route-the-runtime-proof-through-the-artifact-envelope` does the handoff this ticket called impossible: `prototypes/serial-sum-compile` writes the envelope and its identity sidecar to a path, and `prototypes/serial-sum-run` reads that file and dispatches from it, holding no producer module. Option (a) — a `[lib]` on `tiler-prototype-compile` — would have created a public namespace on a package that has none and relaxed two `scripts/check_workspace.py` pins, and it would have left the runner holding a `CompiledArtifact` it could load instead of the envelope's. **The cheaper option was the one that kept the bypass reachable**, which is the shape this ticket's own reasoning was set up to catch and did not.

`needs-tom` is dropped with it: no boundary is crossed, no ADR is needed, and nothing here is Tom's to decide.
