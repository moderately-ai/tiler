---
id: carry-live-extent-operands-through-the-artifact-envelope
title: Carry live extent operands through the artifact envelope
status: done
priority: p1
dependencies: [accept-the-live-extent-operand-public-surface]
related: [admit-live-extent-operands-to-payload-indexing, deliver-an-artifact-family-from-a-symbolic-region]
scopes: [implementation/artifact, contracts/artifacts, implementation/runtime, implementation/ir, implementation/compiler, implementation/metal, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, identity, public-boundary]
---
## User-visible outcome

A verified artifact program carries the accepted live-extent operand row through construction, codec, decode, and validation, so a routed preflight binds the same `AbiRoot::InputExtent` fact the kernel body reads.

## Exact gap

**Fact at `209e0f9fd5a18486039d859a5f47ccf260f0f8cf`, re-read this session.** Main has no live-extent kernel operand and no artifact operand row. The labelled draft at `9a8f53c937dc9b9f777a1d4b361cadc1a0b0316e` adds kernel, schedule, Metal, and routed-runtime spellings and explicitly does not add an envelope feature. Reproduce on that branch: `rg -n "InputExtentParameter|RoutedExtentParameter" crates/tiler-artifact` reports no construction or codec hit.

[`admit-live-extent-operands-to-payload-indexing`](admit-live-extent-operands-to-payload-indexing.md) required carrying the root declaration, use sites, type, phase, canonical order, and read-only parameter transport through artifact construction, codec, decode, and validation. The worker correctly refused to invent that row before Tom accepted the public surface.

## Required work

- After [`accept-the-live-extent-operand-public-surface`](accept-the-live-extent-operand-public-surface.md) closes, integrate the accepted kernel/runtime spelling and add the envelope row it implies. If acceptance names a different spelling, implement that spelling rather than the draft.
- Construct, encode, decode, and validate the operand row. Missing, reordered, duplicated, wrong-axis, wrong-type, and backend-misbound rows fail at the named layer.
- Runtime binds the row from the same authoritative `AbiFacts` input extent used by range and launch evaluation. Callers do not provide another list. A deliberate disagreement between host-side and payload use refuses before program work.
- Freeze canonical parameter bytes before `RoutingCommit`. The live extent *value* stays out of artifact, payload, library, and pipeline identity.
- If the accepted row is a new public artifact view or schema field, produce that exact included/excluded surface as a labelled draft and stop for Tom rather than self-accepting it.

## Required evidence

- Encode/decode round-trip of a program that declares one live input-axis extent, with subject perturbation of omit, swap, wrong axis, and misordered transport.
- Existing range and launch expressions resolve from the same bound fact as the new row.
- Exact identity blast radius. Empty extent lists must not move previously encodable artifact bytes unless a justified domain step is recorded.
- Targeted artifact and runtime tests, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Non-goals

The `N = 14` / `N = 15` payload and pipeline execution evidence is [`prove-one-live-extent-artifact-payload-and-pipeline-at-two-n`](prove-one-live-extent-artifact-payload-and-pipeline-at-two-n.md). Schedule-verified `LiveContraction` is [`prove-a-schedule-verified-live-contraction-consumes-s`](prove-a-schedule-verified-live-contraction-consumes-s.md). `compile()` through strategy selection and region formation is [`admit-symbolic-extents-through-compiler-region-formation`](admit-symbolic-extents-through-compiler-region-formation.md).

## Outcome

The accepted kernel/runtime spelling is integrated on this branch, and the envelope row it implied is constructed, encoded, decoded, and validated. The row is a new public artifact view (`DecodedExtentOperand`, `DecodedEntry::extent_operands`, `EntryRef::extent_operands`) and is labelled a draft. [`accept-the-live-extent-artifact-envelope-row`](accept-the-live-extent-artifact-envelope-row.md) is filed at `awaiting-decision` and was not self-accepted.

**Conflict port of `9a8f53c9`.** Both-add: HEAD's `ParametricBroadcast` / subgroup / `CooperativeContraction` stay; the accepted live-extent types stay. `LiveRowMajor` is schedule tag `0x09` (HEAD already consumed `0x08` for parametric broadcast). `LiveContraction` is topology tag `0x38` because accepted `0x36` stays reserved for `CooperativeContractionSplit` and HEAD already uses `0x37` for `CooperativeContraction`. Empty live-extent lists write nothing after staging and before the subgroup requirement, so previously encodable kernel and artifact bytes do not move.

**Quoted perturbations.**

- Omit the backend placement for a declared row: `EntryTransportCardinality { payload: 0, bindings: 2, extents: 1, transports: 2 }`.
- Reorder two rows of one input: `NonCanonicalOrder { subject: ExtentOperand }`.
- Duplicate a row: `DuplicateItem { subject: ExtentOperand }`.
- Wrong axis: `ExtentOperandAxis { key: "input", axis: 99, rank: 2 }`.
- Wrong type: `ExtentOperandType { key: "input", axis: 1 }`.
- Misordered transport: `ExtentOperandTransport { payload: 0, operand: 0, declared: 0, expected: 2 }`.
- Unbound runtime fact: `runtime.unbound-input-extent: entry 0 requires input input axis 1 from the same AbiFacts used for range and launch`.

**Identity blast radius.** `tiler.artifact-program.v16` and `tiler.kernel.v7` do not step. Empty extent lists write nothing. A nonempty declaration is a new subject. The live value is not in artifact, payload, library, or pipeline identity.

## Closes when

The accepted operand exists on the envelope end to end, every named negative is fail-capable, identity consequences are recorded, and no second scalar authority was introduced.
