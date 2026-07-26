---
id: record-the-accessmode-total-mapping-site-under-convention-5b
title: Record the AccessMode total-mapping site under ADR 0074 convention 5b
status: todo
priority: p3
dependencies: []
related: [implement-boundary-property-model, harden-public-enums-non-exhaustive]
scopes: [contracts/decisions]
shared_scopes: []
paths: []
tags: [contract, adr, api-conventions]
---
ADR 0074 convention 5b governs public enums that are exhaustively matched from outside their defining crate, and enumerates the sites where that holds. `implement-boundary-property-model` created a new one and could not record it, holding only `implementation/compiler`.

**Fact.** `tiler_ir::schedule::AccessMode` is now mapped totally onto an identity tag from `crates/tiler-compiler/src/boundary.rs` at two sites.

**Fact.** It carries no `#[non_exhaustive]`, and under convention 5b it must not gain one: a total mapping from outside the crate is exactly what `#[non_exhaustive]` would break, and the mapping is deliberate — an identity encoding must be exhaustive with no wildcard arm, so that adding a variant is a compile error at every encoder rather than a silently mis-encoded subject.

**Inference.** This is the *reason* convention 5b exists rather than an exception to it, so the correct action is to add the site to the ADR's enumeration, not to reconsider the enum's attributes.

**Check before writing.** Confirm both call sites still exist and are still total, and confirm the ADR's enumeration is a normative list rather than an illustrative one — if it is illustrative, adding a site is not sufficient and the ADR needs to say how a reader finds the complete set. `AGENTS.md`: a failed search is evidence the search was wrong until the file has been read, and a doc comment that overstates what is enumerated makes unreachable work look reachable.

**Related but distinct:** `harden-public-enums-non-exhaustive` decides which growth-expecting public enums *gain* `#[non_exhaustive]`. `AccessMode` is a case that must not, so the two tickets must not contradict each other; whichever lands second should check the first.

## Closes when

The site is enumerated in ADR 0074, the must-not-gain-`#[non_exhaustive]` reason is stated where a future editor will see it, any catalog block quoting ADR 0074 is updated by hand in the same change, and `make full` passes.
