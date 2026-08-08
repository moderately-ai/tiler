---
id: emit-the-indirect-gather-on-metal
title: Emit the indirect gather on Metal
status: blocked
priority: p3
dependencies: [accept-adr-0108-data-dependent-index-coordinate-siting, admit-a-storage-carrier-for-integer-program-inputs]
related: [admit-an-indirect-gather-family-for-tied-embedding-lookup, admit-a-storage-carrier-for-integer-program-inputs]
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
