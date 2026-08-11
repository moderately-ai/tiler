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
ADR 0108 was returned for revision, so no representation has been selected or
accepted. Acceptance must be followed by a separately scoped IR-admission ticket
that constructs, validates, proves or retains, identities, compacts, and explains
the selected form. Before this ticket leaves `blocked`, that admission ticket must
exist, be complete, and be added as a dependency. Acceptance by itself is not an
admitted IR.

**Fact — no integer storage carrier.** `pub enum StorageScalar` currently has
three variants, `U8`, `F32`, and `Bf16`; none carries the `tiler::u32@1` index
operand. [`admit-a-storage-carrier-for-integer-program-inputs`](admit-a-storage-carrier-for-integer-program-inputs.md)
owns that independent prerequisite and is now a direct dependency.

**Accepted sequencing — no implied validation.** ADR 0108's 2026-08-11 direction permits only a host-visible immutable-snapshot preflight lane in the first pass. [`admit-the-selected-data-dependent-index-representation`](admit-the-selected-data-dependent-index-representation.md) must first admit the accepted IR form. [`admit-an-invocation-scoped-gather-index-validation-receipt`](admit-an-invocation-scoped-gather-index-validation-receipt.md) must then carry and consume the exact mandatory obligation, and its public surface must be accepted separately. Emission cannot infer validation from forming an address and cannot make a device-side check part of this ticket.

The old dependency on
[`admit-the-indirect-access-class-into-the-index-layer`](admit-the-indirect-access-class-into-the-index-layer.md)
was removed because that completed research ticket drafted a proposal; it did not
accept or admit an access class.

## What this ticket delivers when unblocked

An emitted construct for the admitted relation, compiler and Metal explanation,
a golden, and a device comparison. Unsigned-index arithmetic must attribute the
exact bound and validation authority the accepted IR provides; emission must not
imply that a check happened merely because an address was formed.

## Non-goals

Scatter. Selecting or admitting the logical representation. Inventing an integer
carrier or a host-validation boundary inside the backend.
