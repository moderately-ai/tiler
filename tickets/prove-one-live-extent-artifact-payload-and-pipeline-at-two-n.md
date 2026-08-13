---
id: prove-one-live-extent-artifact-payload-and-pipeline-at-two-n
title: Prove one live-extent artifact payload and pipeline at two N
status: done
priority: p1
dependencies: [carry-live-extent-operands-through-the-artifact-envelope]
related: [admit-live-extent-operands-to-payload-indexing, deliver-an-artifact-family-from-a-symbolic-region]
scopes: [implementation/artifact, implementation/build, implementation/metal, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, identity, runtime, metal]
---
## User-visible outcome

One compiled artifact, payload subject, and pipeline indexes dense F32 `[2,N]` from the bound input extent, so changing `N` changes the addressed byte without compiling another payload or pipeline.

## Exact gap

**Correction at `6ea5de7cd866edd296e39310cdb94163ca5c1a4c`.** The envelope row now exists: `8b52aa13` carries it, and `DecodedExtentOperand` / `EntryRef::extent_operands` are labelled drafts under `accept-the-live-extent-artifact-envelope-row`. Reproduce: `rg -n "Draft surface, not yet accepted" crates/tiler-artifact/src/program/codec/view.rs crates/tiler-artifact/src/program/model.rs`. What this ticket still owed is the `N = 14` / `N = 15` payload and pipeline execution evidence the parent named, which neither `9a8f53c9` nor the envelope row produced.

## Fact audit at `6ea5de7c`

- **Verified.** Semantic `(row = 1, column = 0)` on dense F32 `[2,N]` is element `N`, so bytes `4N`. The IR already states it as `dense_f32_row_major_bytes` and asserts `56` and `60`. Reproduce: `rg -n "dense_f32_row_major_bytes" crates/tiler-ir/src/kernel/tests.rs`.
- **Verified.** `DecodedExtentOperand` and `EntryRef::extent_operands` are labelled drafts. Reproduce: `rg -n "Draft surface, not yet accepted" crates/tiler-artifact/src/program/codec/view.rs crates/tiler-artifact/src/program/model.rs`.
- **False as written.** "the artifact envelope row does not exist yet" is stale after `8b52aa13`. The row exists; the two-N execution evidence did not.
- **Verified.** Baking neighbouring extents changes kernel identity. Reproduce: `rg -n "baking N = 14 must change identity" crates/tiler-ir/src/kernel/tests.rs`.

## Required work

- One artifact, payload subject, and pipeline handles dense F32 `[2,N]` at `N = 14` and `N = 15`.
- Semantic `(row = 1, column = 0)` addresses bytes 56 and 60 respectively from the bound input extent.
- Baking either value changes identity and fails the no-specialization assertion. The live value is excluded from artifact, payload, library, and pipeline identity.
- Existing range and launch expressions resolve from the same bound fact. A deliberate disagreement refuses before program work.

## Required evidence

- Both extents execute through one payload and one pipeline, with the two address oracles observed.
- Identity of the artifact, payload, library, and pipeline is equal across the two bindings and unequal to a baked neighbour.
- Targeted build, Metal, runtime, and artifact tests, exact identity blast radius, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Non-goals

Envelope construction is the dependency. `LiveContraction` contributor-loop evidence is [`prove-a-schedule-verified-live-contraction-consumes-s`](prove-a-schedule-verified-live-contraction-consumes-s.md). Inline AOT `deliver` lifting is [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md).

## Outcome

One live-extent artifact, one payload subject, and one pipeline subject index dense F32 `[2,N]` at `N = 14` and `N = 15`. The live value stays out of artifact, payload, library, and pipeline identity. Baking either neighbour is a failing identity assertion.

**Address oracles, executed.** Both N values dispatch through `route_with_adapter` against one assembled artifact and one payload. Semantic `(row = 1, column = 0)` is element N of the LiveRowMajor map:

- `N = 14` → input byte **56**, executed result `mapped_bits(input[14])`
- `N = 15` → input byte **60**, executed result `mapped_bits(input[15])`

Quoted: `assert_eq!(addresses, [56, 60], "semantic (row = 1, column = 0) at N=14 and N=15")`. The two executed results disagree.

**Perturbation.** Making `contributor_columns` ignore `RoutedExtentParameter` and use baked `ScalarEntry.columns` fails as:

`assertion \`left == right\` failed: N=14 must execute the LiveRowMajor read at byte 56`
`left: 0`
`right: 1106771968`

**Identity.** Across the two bindings the artifact, payload, library, and pipeline subjects are equal. Each is unequal to a baked `[2, 14]` / `[2, 15]` neighbour. The live MSL contains `constant ulong& e0 [[buffer(2)]]` and neither `14ul` nor `15ul`.

**Disagreement.** Binding only the static row axis refuses before program work as `runtime.abi-evaluation: entry 0's launch precondition 0 could not be evaluated: UnboundInputExtent { key: InputKey("input"), axis: Axis(1) }`.

**Identity blast radius.** `tiler.artifact-program.v16` and `tiler.kernel.v7` do not step. Empty extent lists still write nothing. A nonempty live declaration is a new subject. The bound *value* is not folded.

`DecodedExtentOperand` / `EntryRef::extent_operands` remain labelled drafts and were not self-accepted.

## Closes when

Both `N` values run from one identity, the two byte addresses are observed, and specialization is a failing identity check rather than a description.
