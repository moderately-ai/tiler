---
id: decide-how-the-oracle-independently-checks-a-gather-proof-identity
title: Decide how the oracle independently checks a gather proof identity
status: todo
priority: p2
dependencies: []
related: [admit-the-selected-data-dependent-index-representation, carry-the-gather-relation-through-the-compiler-vertical]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, identity, reference]
---
## User-visible outcome

The reference oracle can independently check that a static gather resolution's retained proof identity is the one the index layer minted, instead of being able to read the identity but never to derive anything to compare it against.

## Why this exists

Filed 2026-08-22 by the coordinator. Three successive gather lanes have now reached this and each stopped rather than work around it, which is the correct call every time.

**Fact — the boundary is deliberate and the missing constructor is the point.** `GatherIndexBoundsProofIdentity` is declared `pub(super) Vec<u8>` and its doc states there is no public constructor and no byte conversion, so `as_bytes` is the entire surface a downstream crate has. `tiler-reference` can therefore *read* a retained identity but cannot derive the bytes to compare it against without reimplementing the encoding — which would fork the identity domain the module exists to solely own, the exact defect the missing constructor prevents.

**Fact — a worker cannot mint the fix.** Closing this widens accepted public surface, and AGENTS.md reserves consequential public boundaries to Tom. Holding an identity one could not have constructed is precisely what makes it evidence the proof ran; handing out a constructor is not a mechanical convenience.

**Fact — the narrower slice was available and has been taken, so this ticket is not blocking correctness today.** The third gather pass landed a check of the retained `kind()` and `index_shape()` against an independent derivation from the operand shapes, written out rather than called, covering four cases including both arguments holding at once. It is the only check anywhere that catches a precedence inversion from outside the crate that decides it, and its perturbation proves it does: inverting the deriver's U32-before-empty precedence reddens it across the crate boundary. **What remains unchecked is the identity itself.**

## The decision

Two shapes, and they are not equivalent:

1. **A verifier-side re-derivation entry point** — the oracle recomputes the identity from the verified program and compares. Strongest, and the most surface: it publishes enough to mint an identity, which is what the current boundary refuses.
2. **An identity-comparison entry point** — the oracle hands back what it holds and the owning module answers whether it matches. Publishes a predicate rather than a constructor, so a caller still cannot mint one.

Enumerate at your base rather than treating this pair as closed, and include the status quo: the narrower slice already lands, so **"leave the identity unchecked and record why" is a real candidate**, not a placeholder.

## Required work

- Apply AGENTS.md's decision-packet readiness gate in full. Re-audit all three Facts at your base with a per-Fact verdict first.
- For each survivor, state exactly what public surface it adds, whether a caller could use it to mint an identity, and what that would let a wrong program claim.
- State the identity and schema consequence of each. **Expected: none** — these are read-side entry points over already-minted bytes — but derive it rather than copying that expectation.
- If one option dominates, recommend it rather than manufacturing a choice. If a real trade-off survives, ask Tom exactly one concrete question.
- For every survivor: strongest counterargument, the evidence that would reverse it, and the negative controls that would test it.

## Non-goals

Implementing whichever option is chosen; widening any other part of the gather surface; and the compiler vertical, which is [`carry-the-gather-relation-through-the-compiler-vertical`](carry-the-gather-relation-through-the-compiler-vertical.md).

## Closes when

Tom has accepted one route for the oracle to check a gather proof identity, or has accepted that it stays unchecked with the reason and a reconsideration trigger recorded — and in either case the surface each option would add is stated rather than implied.
