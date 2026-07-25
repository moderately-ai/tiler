---
id: update-adr-0071-schedule-builder-boundary
title: Update ADR 0071 implementation-boundary notes (schedule builders and closure convenience)
status: done
priority: p2
dependencies: [add-checked-closure-convenience-for-shared-ir-builders]
related: [prototype-scheduled-region-ir, add-checked-closure-convenience-for-shared-ir-builders]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, decisions]
---
ADR 0071's implementation-boundary note states that schedule builders remain
unimplemented. `prototype-scheduled-region-ir` merged `tiler_ir::schedule` with a
real `ScheduledRegionBuilder` and opaque `VerifiedScheduledRegion` following the
same checked-builder discipline the ADR governs, so that note is now partially
superseded.

A second part of ADR 0071's accepted ergonomic layer — the closure-based
convenience over the shared IR builders — was also described as unimplemented.
`add-checked-closure-convenience-for-shared-ir-builders` implements it (for
`IndexRegionBuilder` first) but is scoped to `implementation/ir` and deliberately
defers the ADR 0071 decision-doc status edit here, so this is the single
consolidated owner of ADR 0071's implementation-status updates. This is why it
depends on that ticket: the ADR should reflect the implemented state.

Update ADR 0071 to record BOTH the implemented schedule builder/verifier
(`tiler_ir::schedule`) AND the implemented closure convenience, superseding the
durable "unimplemented" statements explicitly rather than silently (per the
documentation contract for superseding accepted decisions). Keep the ADR's
original rationale intact; note only what evidence changed. If the edit makes any
new normative contract a genuine `applies_to` destination, extend the frontmatter
edge so prose and typed edge agree, and regenerate `docs/decisions/README.md`
(the catalog is a generated view — edit source metadata, not list items).

Run `uv run --locked python scripts/docs.py render` and the full documentation
gate before completion.

## Outcome

ADR 0071's Implementation boundary is rewritten as a clause-by-clause maturity record. Both statements this ticket named are superseded in place, and four further facts the ticket did not name were found by reading the tree and are recorded rather than left for a reader to discover. Two of them are owed engineering and are split out.

**Fact — the ticket's first premise understated what landed.** It says `prototype-scheduled-region-ir` merged a real `ScheduledRegionBuilder`, so ADR 0071's "Schedule, kernel, and program builders remain unimplemented" is "partially superseded". Reading `crates/tiler-ir/src/` shows all three are implemented, not one of three: `schedule/builder.rs` has `ScheduledRegionBuilder` and `VerifiedScheduledRegion`, `kernel/builder.rs` has `build()` returning `VerifiedKernel`, `program/builder.rs` has `build()` returning `VerifiedKernelProgram`, and all three failure types carry `diagnostics()` plus an `into_parts()` that recovers the intact builder. All three are on the compile path — `crates/tiler-compiler/src/physical.rs` verifies schedules and lowers kernels through them and `crates/tiler-compiler/src/program.rs` drives the program builder — so this is production use, not a declared surface. The sentence is superseded outright rather than partially.

**Fact — the closure convenience is narrower than "implemented".** `IndexRegionBuilder::build_with` and the crate-root `CheckedBuildError<Admission, Verification>` exist and behave as the dependency ticket's Outcome describes. `grep -rn 'pub fn build_with' crates/tiler-ir/src/` returns one line. The schedule, kernel, and program builders reuse the error type's *shape* through a crate-private combinator and expose no `build_with`, so the ADR now records a delivered convenience on one layer and a reusable shape on the rest, which are different claims.

**Fact — `VerifiedProgramPortfolio` does not exist.** ADR 0071's Decision names five verified types that cross boundaries. Four exist in `tiler-ir`. `grep -rn 'VerifiedProgramPortfolio' crates/` returns nothing; `ProgramPortfolio` at `crates/tiler-compiler/src/pipeline.rs:154` is `pub(crate)` and is a compiler-internal aggregate, not a verified IR product with a checked builder. Recording the builders as implemented without this would have let a reader take the whole five-type clause as delivered.

**Fact — one of the two identity-retention edges is missing, and it is the one this ticket's subject was supposed to establish.** Kernel to schedule is realized: `crates/tiler-ir/src/kernel/model.rs` stores `schedule_identity: CanonicalScheduledRegionIdentity`, exposes it, and folds its bytes into `CanonicalKernelIdentity`. Schedule to index region is not. `crates/tiler-ir/src/schedule/model.rs:178` declares its own public-field `IndexRegion` struct — a different type from `tiler_ir::index::VerifiedIndexRegion`, with different content and a separate canonical identity — and `encode_identity` at `:674` folds that struct's content into `CanonicalScheduledRegionIdentity` without ever referencing `CanonicalIndexRegionIdentity`. The exact check is `grep -rn 'crate::index' crates/tiler-ir/src/schedule/`, which returns one line: a doc-comment cross-reference in `error.rs`. There is no code path from the schedule module into the index module, so `tiler-ir` carries two index-region representations and the schedule layer refines the one it declares itself. Split into [`bind-the-scheduled-region-to-the-verified-index-region-identity`](bind-the-scheduled-region-to-the-verified-index-region-identity.md), which requires deciding whether the duplication is a defect or a deliberate asymmetry *before* coding, because if the asymmetry is deliberate then ADR 0071's clause is what needs correcting rather than the code.

**Fact — one Decision mechanism is unimplemented and its guarantee currently holds vacuously.** ADR 0071 states that artifact decoding reconstructs values through the same IR builders. `crates/tiler-artifact/src/program/codec/view.rs:85`'s `decode_artifact` validates framing, digests, schema, canonical order, and arena closure and re-derives identity, then returns a `DecodedArtifact` read view; nothing in `tiler-artifact` calls a `tiler_ir` builder, and the dependency runs the other way. "Deserialization cannot manufacture a verified value" is therefore true because deserialization manufactures no IR value at all — a stronger position than the clause describes, and free only until something needs an IR value back out of an artifact. Split into [`settle-adr-0071-artifact-decoding-through-ir-builders`](settle-adr-0071-artifact-decoding-through-ir-builders.md), which must land one of two outcomes and not a third that leaves the clause decorative.

**Fact — the negative-compile-test consequence reaches one layer.** `crates/tiler-ir/tests/` carries `index-region`, `shape-evidence`, and `typed-handles` `trybuild` suites and none for schedule, kernel, or program. Those three layers' verified products are opaque by construction — private fields, `pub(super)` constructors — which is implemented support; that no out-of-crate forgery compiles is not a tested guarantee for them. The ADR keeps the two apart.

**Amended rather than superseded — the `ShapeEnv` premise.** ADR 0071 justified deferring symbolic extents "because the accepted `ShapeEnv` authority does not yet exist". `crates/tiler-ir/src/shape/env.rs` landed the scoped shape-symbol and typed-root-binding half as a `pub(crate)` ADR 0074 convention 7 draft, with the constraint environment deliberately split out. The conclusion survives and its reason is now stronger — nothing on the compile path constructs one, the module's crate-level `#![allow(dead_code)]` reason says so, and the index module still invents no competing binding system — so the premise is corrected in place rather than the clause being superseded. `implement-shapeenv-core` was `in-progress` in another worktree while this ran, so the state recorded is the one at `43f685f` and is dated.

**`implementation_status` unchanged at `partial`, and `decision_status` untouched.** Four builders, four verified products, one closure convenience, and both recoverable error boundaries are implemented; one named verified type does not exist, one identity edge is missing, and one mechanism is unimplemented. `partial` is the honest high-water mark and a bump would misreport exactly what `close-remaining-adr-status-drift` warned about.

**Scope.** No new `applies_to` destination arose — the edits are status statements about existing destinations, not new normative homes — so the frontmatter edge set is unchanged and `docs/decisions/README.md` regenerated with no diff, the catalog being a view over frontmatter that did not move. All edits are inside the declared `contracts/decisions` and shared `project/tickets`; `contracts/navigation` was declared and not needed.

**Measurement.** `uv run --locked python scripts/docs.py render` reported "documentation render passed (182 records)". `uv run --locked python scripts/check_repository.py` exited 0 with "complete repository validation passed". Host macOS arm64, toolchain `nightly-2026-07-19`.
