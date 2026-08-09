---
id: correct-the-stale-bf16-backend-refusal-claim-in-the-kernel-type-doc
title: Correct the stale Bf16 backend-refusal claim in the kernel type doc
status: in-progress
priority: p3
dependencies: []
related: [re-read-the-bf16-and-elementary-support-rows-against-source, lower-bf16-to-metal]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [doc-claim, bf16, kernel]
claimed_from: todo
assignee: terra-bf16
lease_expires_at: 1786243732
---

## The defect (navigation re-read 2026-08-06, coordinator-verified)

`KernelType::Bf16`'s doc comment (`crates/tiler-ir/src/kernel/model.rs:111-115`) says "`crates/tiler-metal` refuses this type by name rather than spelling `bfloat`, because it carries no BF16 constant reinterpretation, canonicalization helper, or dispatch route. Verified and emittable are separate claims, and only the first holds here." All three named absences have since landed: `KernelType::Bf16 => Ok("bfloat")` at `crates/tiler-metal/src/emit.rs:993` (the arm's own comment says "and it did not before"), `CanonicalizeBf16Nan` with its `bfloat16` helper at `emit.rs:384-398`, and the `bfloat` operator emission at `emit.rs:1267`.

A doc comment is a claim the next worker acts on; this one makes a landed capability look absent — the inverse of the usual overstatement, and just as costly to a reader sequencing BF16 work.

## The work

Rewrite the paragraph to describe current behaviour: the backend spells `bfloat` with its canonicalization helper and dispatch route, and state what boundary actually remains for this type (read `lower-bf16-to-metal`'s outcome and the BF16 support-matrix row for the current residual — the offline-vs-dispatch and profile-row boundaries — rather than asserting from this ticket). Verify each claim at source before writing, per the corpus rule.

## Closes when

The doc describes what `tiler-metal` does now, with the remaining boundary stated from verified sources, and no reader can conclude the backend refuses the type.
