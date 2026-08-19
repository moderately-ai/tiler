---
id: emit-the-indirect-gather-on-metal
title: Emit the indirect gather on Metal
status: blocked
priority: p3
dependencies: [accept-adr-0108-data-dependent-index-coordinate-siting, admit-the-selected-data-dependent-index-representation, admit-a-storage-carrier-for-integer-program-inputs, admit-an-invocation-scoped-gather-index-validation-receipt, accept-the-invocation-scoped-gather-validation-public-surface]
related: [admit-an-indirect-gather-family-for-tied-embedding-lookup, admit-a-storage-carrier-for-integer-program-inputs, validate-device-resident-gather-indices-before-dispatch, admit-a-zero-copy-exclusive-lease-for-validated-gather-indices]
scopes: [implementation/metal, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, gather, language-model]
---
## User-visible outcome

A verified scheduled region containing the eventually accepted and admitted
indirect-gather relation reaches a `VerifiedKernel`, and Metal emits it with a
reference and device comparison.

## Why this remains blocked

**Fact — no admitted IR.** `AccessData` carries one tensor ordinal and no
`IndexNode` reads tensor data; `LogicalAccess` has no indirect-read relation.
Acceptance must be followed by a separately scoped IR-admission ticket
that constructs, validates, proves or retains, identities, compacts, and explains
the selected form. Before this ticket leaves `blocked`, that admission ticket must
exist, be complete, and be added as a dependency. Acceptance by itself is not an
admitted IR.

**Correction — 2026-08-19 (ADR 0108 clause withdrawn; the IR half re-verified).**
The Fact above originally continued "ADR 0108 was returned for revision, so no
representation has been selected or accepted". That is **false at this base** and
the clause is withdrawn. `docs/decisions/0108-site-a-data-dependent-index-coordinate-on-the-expression.md`
carries frontmatter `decision_status: "accepted"` and the body states acceptance on
2026-08-12; the dependency [`accept-adr-0108-data-dependent-index-coordinate-siting`](accept-adr-0108-data-dependent-index-coordinate-siting.md)
is `done`, and the ADR selects the **append-only tagged access representation** with a
narrow gather-specific first variant. The clause's *conclusion* nonetheless survives on
its other leg, which is why the surrounding paragraph is retained rather than deleted:
the admission ticket [`admit-the-selected-data-dependent-index-representation`](admit-the-selected-data-dependent-index-representation.md)
now exists and is already a dependency, but it is not complete, so no IR is admitted.
The IR half of the Fact was re-read and **remains true**: `AccessData` in
`crates/tiler-ir/src/index/model.rs` still declares `pub tensor: u32` — one tensor
ordinal; `pub(super) enum IndexNode` in that file still has exactly five forms
(`Constant`, `Dimension`, `LinearCombination`, `FloorDiv`, `Modulo`), none reading
tensor data; and `pub enum LogicalAccess` in `crates/tiler-ir/src/schedule/model.rs`
now carries **eleven** variants, none data-dependent. `implementation_status: "none"`
on ADR 0108 is the frontmatter that agrees.

**Fact — no integer storage carrier.** *(Withdrawn 2026-08-19 — this premise has
lifted; retained with its correction because it is one of the two grounds this
ticket's `blocked` state was argued from.)* The Fact read: "`pub enum StorageScalar`
currently has three variants, `U8`, `F32`, and `Bf16`; none carries the
`tiler::u32@1` index operand."

**Correction — 2026-08-19.** At this base `pub enum StorageScalar`
(`crates/tiler-ir/src/program/model.rs`) has **four** variants — `U8`, `F32`,
`Bf16`, and `U32` — the last documented "An unsigned 32-bit integer carrier" whose
"natural access type is the exact-width [`KernelType::U32`]". Its prerequisite
[`admit-a-storage-carrier-for-integer-program-inputs`](admit-a-storage-carrier-for-integer-program-inputs.md)
is `done`. The carrier's own doc comment keeps the boundary this ticket still owes:
it "is physical storage and not an integer-arithmetic capability", with "arithmetic,
conversion, and backend support" left as separate decisions — so the carrier
existing does not by itself supply the unsigned-index arithmetic and bound
attribution the delivery section below requires.

**Accepted sequencing — no implied validation.** ADR 0108's 2026-08-11 direction permits only a host-visible immutable-snapshot preflight lane in the first pass. [`admit-the-selected-data-dependent-index-representation`](admit-the-selected-data-dependent-index-representation.md) must first admit the accepted IR form. [`admit-an-invocation-scoped-gather-index-validation-receipt`](admit-an-invocation-scoped-gather-index-validation-receipt.md) must then carry and consume the exact mandatory obligation, and its public surface must be accepted separately. Emission cannot infer validation from forming an address and cannot make a device-side check part of this ticket.

The old dependency on
[`admit-the-indirect-access-class-into-the-index-layer`](admit-the-indirect-access-class-into-the-index-layer.md)
was removed because that completed research ticket drafted a proposal; it did not
accept or admit an access class.

## Readiness determination — 2026-08-19

Delivered by [`repair-the-ticket-population-facts-the-splits-and-retirements-falsified`](repair-the-ticket-population-facts-the-splits-and-retirements-falsified.md)
at base `f08281a1`. **This ticket remains blocked, but on a different edge than the one it states.**
The finding is recorded here; the state change, if any, is the coordinator's.

Both Facts this ticket argued `blocked` from have been re-audited above: the integer-carrier
premise has **fully lifted**, and the ADR-0108-unaccepted premise is **false**. What still
blocks it is the admission ticket the Fact itself names as mandatory. Dependency states at
this base, read from each file's own frontmatter:

| dependency | status | bears on readiness |
| --- | --- | --- |
| `accept-adr-0108-data-dependent-index-coordinate-siting` | `done` | satisfied — ADR 0108 accepted 2026-08-12 |
| `admit-a-storage-carrier-for-integer-program-inputs` | `done` | satisfied — `StorageScalar::U32` landed |
| `admit-the-selected-data-dependent-index-representation` | `blocked` | **the live blocker** |
| `admit-an-invocation-scoped-gather-index-validation-receipt` | `todo` | unstarted; depends on the blocker above |
| `accept-the-invocation-scoped-gather-validation-public-surface` | `todo` | unstarted; depends on the receipt |

So two of five dependencies are satisfied and three are not. The "Accepted sequencing" paragraph
below still holds unchanged and is the operative statement of what is owed: the accepted IR form
must be admitted, then the invocation-scoped obligation carried and consumed, then its public
surface accepted separately.

**Finding the coordinator should act on separately, not here.**
[`admit-the-selected-data-dependent-index-representation`](admit-the-selected-data-dependent-index-representation.md)
is `status: blocked` while **both** of its declared dependencies —
`accept-adr-0108-data-dependent-index-coordinate-siting` and
`decide-the-data-dependent-index-representation-public-surface` — are `done`. Its blocked state
therefore has no surviving declared ground either, and it is the ticket that actually gates this
one. This lane's stop conditions forbid changing any ticket's state, so it is reported rather than
altered; whether it is genuinely ready turns on reading its Required-boundary section against the
accepted ADR, which is not this lane's scope. **Unverified here:** whether an undeclared
prerequisite justifies that ticket's `blocked` state.

## What this ticket delivers when unblocked

An emitted construct for the admitted relation, compiler and Metal explanation,
a golden, and a device comparison. Unsigned-index arithmetic must attribute the
exact bound and validation authority the accepted IR provides; emission must not
imply that a check happened merely because an address was formed.

## Non-goals

Scatter. Selecting or admitting the logical representation. Inventing an integer
carrier or a host-validation boundary inside the backend.
