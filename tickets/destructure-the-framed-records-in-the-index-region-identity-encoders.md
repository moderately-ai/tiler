---
id: destructure-the-framed-records-in-the-index-region-identity-encoders
title: Destructure the framed records in the index-region identity encoders
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, identity]
---
## User-visible outcome

Adding a field to a compacted index-region record becomes a build error at the encoders that frame it, instead of compiling and silently producing a narrower `tiler.index-region.v11` identity.

## Why this exists

Filed 2026-08-22 by the coordinator from the sibling sweep of [`destructure-the-gather-bounds-subject-in-its-identity-encoder`](destructure-the-gather-bounds-subject-in-its-identity-encoder.md), which landed as `f197697f` and closed the same defect on the gather **bounds** subject. That lane reported this remainder and correctly declined it as more than mechanical.

**Fact — the defect is live and the coordinator verified it at `3291b105`.** `crates/tiler-ir/src/index/builder/identity.rs` reads the gather access record entirely by field at **three** sites: the `AccessData::GatherRead(gather)` arm (`gather.source`, `.index`, `.axis`, `.domain`, `.source_coordinates`, `.index_coordinates`), the `CompactedAccess::GatherRead(gather)` arm in the region encoder, and the corresponding length arm. `GatherReadAccessData` and `CompactedGatherReadAccess` in `crates/tiler-ir/src/index/model.rs` each declare exactly **six** fields. Nothing in any of those arms is exhaustive over the struct, so a seventh field enters the record while never entering the identity bytes, and no check fails. The delivering lane names a fourth site, `alpha_access_key`, which the coordinator has **not** verified.

**Why this is worse than the case already closed.** The gather bounds subject was one encoder over one struct. This is a **paired encoder and length invariant** that must stay in agreement, across nine per-element loops, on `tiler.index-region.v11` — the index crate's central identity. A repair that moves a byte here moves far more than a repair there did.

**Fact — the convention already exists in this repository and is documented.** `crates/tiler-ir/src/numerics.rs` carries it with a rationale at the anchor `a field added to a provenance record is then a build error at the encoder`, with thirteen `let Self { … }` encoders below it. `encode_region` and `encoded_region_len` in the same `identity.rs` **already** destructure `CompactedRegion` exhaustively with no rest pattern. So this is applying an established local convention to the arms that have not adopted it, not inventing one.

## Required work

- Re-audit the Fact at your base with a per-Fact verdict, and **re-derive the site list yourself** — the coordinator verified three sites and the fourth is unverified. Say which spellings you searched for and why that set is complete.
- Destructure each framed record with **no `..` rest pattern**, and write the bytes from the bindings. The byte output must be identical; this is a build-time guarantee, not an encoding change.
- **The paired length invariant is the risk.** The encoder and its length function must stay in agreement. Demonstrate that they still do, and say how.
- **Demonstrate byte-identity rather than asserting it**, across a subject population wide enough to discriminate — the sibling lane dumped 14 subjects covering both records, both proof kinds, both fact sources, ranks 2 and 3, several axes, an empty index shape, and an empty result extent, then showed the comparison could say *no* by swapping two writes and moving all 14. Copy that method.
- Confirm no identity domain steps: no pinned identity, golden, or ledger row may move. **If one does, stop** — the encoding changed and the change was not the intended one.
- Perturb by adding a field and quoting the build error. rustc identifies the site by span, so confirm the span lands in the encoder rather than only at a construction site.

## Non-goals

Changing what any identity encodes, its field order, or its domain tag. Any public surface change. The separate question of whether `IndexRefinementSubject::environment` belongs in the refinement subject identity, which is [its own ticket](decide-whether-the-refinement-subject-identity-should-carry-its-environment.md).

## Closes when

Every framed record in the index-region identity encoders is destructured exhaustively, the paired length invariant is shown to hold, the emitted bytes are demonstrated unchanged over a discriminating population, no identity value has moved, and a field-addition perturbation is watched failing at an encoder span.
