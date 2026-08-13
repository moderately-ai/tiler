---
id: repair-the-records-the-symbolic-region-construction-landing-falsifies
title: Repair the records the symbolic-region construction landing falsifies
status: done
priority: p2
dependencies: [construct-a-symbolic-region-as-a-semantic-program]
related: [carry-symbolic-extents-into-the-semantic-program, deliver-an-artifact-family-from-a-symbolic-region, admit-symbolic-extents-at-the-compiler-request-boundary]
scopes: [research/shapes, implementation/frontend, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, correction, shapes]
---
## User-visible outcome

Live research records and the AOT `deliver` diagnostic stop stating that a symbolic inline region is still `ProgramEvidence::DeferredSymbolicExtent` or that `construct-a-symbolic-region-as-a-semantic-program` is still `todo`. Dated corrections record what the 2026-08-12 landing actually changed, and what it did not.

## Why this exists

[`construct-a-symbolic-region-as-a-semantic-program`](construct-a-symbolic-region-as-a-semantic-program.md) is `done` at `d258698f` / `92f89ad0`. Re-read at `cd1f76da`:

- **Verified.** `crates/tiler-macros/src/region.rs` no longer contains `DeferredSymbolicExtent`. `ProgramEvidence` is a single `Verified(SemanticProgram)` arm. Reproduce: `rg -n "DeferredSymbolicExtent" crates` is empty.
- **Verified.** `elementwise_axes` no longer restates the match-or-scalar rule. On disagreement it keeps the left axes as a placeholder so binding can proceed; the registry refusal surfaces as `RegionError::Program`. Anchor: `a second independent refusal would be a second authority`.
- **Verified.** AOT delivery still refuses a symbolic interface. `deliver` constructs the program, then `program_interface_is_symbolic` returns `AotRefusal::SymbolicExtent`. Anchor: `AOT delivery still needs every extent known at expansion time`.
- **Verified, and still wrong as user-facing text.** The `AotRefusal::SymbolicExtent` Display names `carry-symbolic-extents-into-the-semantic-program` as the work that removes the restriction. That parent ticket is `done`. The remaining owner is [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md).

## The repairs

### `docs/research/shapes/symbolic-semantic-extents.md`

1. **Fact — what the expansion holds today** still ends `ProgramEvidence::DeferredSymbolicExtent`. The environment/binding half remains; the evidence variant is gone. Date a correction beside it: approved `sym n` regions now produce `ProgramEvidence::Verified` carrying sourced shapes; `deliver macos;` still refuses through `AotRefusal::SymbolicExtent`.
2. The 2026-08-08 superseded note still says the `return Ok(ProgramEvidence::DeferredSymbolicExtent)` arms are unchanged. They are gone.
3. The delivery-table follow-up still treats row 5 as `todo`. Row 5 is `done`. Rows 6 and 7 remain the live remainder. Do not promote row 7.

### `docs/research/shapes/transformer-operation-and-shape-surface.md`

The 2026-08-08 correction still says `The frontend clauses above remain current — the inline frontend still defers a symbolic region`. The deferral variant is gone. The workload conclusion that every distinct `T`/`S` is still a separate compiled artifact, and that `deliver` still refuses a symbolic interface, survives.

### `crates/tiler-macros/src/aot.rs`

Replace the Display sentence that names `carry-symbolic-extents-into-the-semantic-program` with the ticket that actually owns remaining delivery. Do not change the refusal itself.

## Out of scope

Do not rewrite dated 2026-08-10 audit snapshots under `docs/research/documentation/ticket-audit-2026-08-10/`. Do not admit symbolic AOT delivery. Do not touch `CompilationRequest` (owned by the live `admit-symbolic-extents` claim).

## Closes when

Each live present-tense claim above is dated or replaced; `rg -n "DeferredSymbolicExtent" crates` stays empty; the AOT diagnostic names the remaining delivery ticket; and `rg -n "still defers a symbolic region|DeferredSymbolicExtent" docs/research/shapes` finds only dated historical sentences plus the correction that retires them.

## Outcome

Dated 2026-08-13 corrections landed on `docs/research/shapes/symbolic-semantic-extents.md` and `docs/research/shapes/transformer-operation-and-shape-surface.md`. `AotRefusal::SymbolicExtent` Display now names `deliver-an-artifact-family-from-a-symbolic-region`; the matching trybuild golden and `docs/integration/frontends.md` were retargeted. The refusal itself is unchanged.

**Support-matrix navigation note.** This advances no operation-family support-matrix row and no dtype-maturity cell. It repairs present-tense records and a consumer-facing remedy id after a landed frontend construction change.

`contracts/integrations` was added because `docs/integration/frontends.md` carried the same stale remedy id.

Commands: `cargo test -p tiler --test facade` (worktree; `deliver_selects_an_artifact_family.rs` compile-fail ok); `tkt lint`; `git diff --check`; `rg -n "DeferredSymbolicExtent" crates` empty; `rg -n "carry-symbolic-extents-into-the-semantic-program" crates` empty.

