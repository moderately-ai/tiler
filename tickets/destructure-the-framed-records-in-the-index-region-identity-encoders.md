---
id: destructure-the-framed-records-in-the-index-region-identity-encoders
title: Destructure the framed records in the index-region identity encoders
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, identity]
claimed_from: todo
assignee: worker-regionenc
lease_expires_at: 1787456482
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

## Coordinator correction, 2026-08-22 — my third anchor failure of this session, same shape each time

The ticket text above cites the `numerics.rs` convention *"at the anchor `a field added to a provenance record is then a build error at the encoder`"*. Retired wording preserved. **That anchor returns 0.** A line break falls after `build error at`, so the full sentence never appears in the bytes. The shortest resolving fragment is `a field added to a provenance record is then a build error at`, which returns 1. Verified by the coordinator at `5c104c59`.

**This is the third time in one session I have handed a worker a full-sentence anchor lifted from a rendered view without running its grep first** — after `it can also say what it deliberately withheld` on the typed-declines ticket and the `pub`/`pub(crate)` mis-citation on the flash-class ticket. AGENTS.md is not ambiguous about the obligation and does not need strengthening: *"run its grep against the file the citation names before handing it to anyone"*, and the coordinator section repeats it as *"run it yourself first — a supplied command that has never been executed is a claim, not a check."* The failure is mine, not the document's. Every instance failed in the dangerous direction, reading as *the text was removed* when the text was plainly there, and every instance was caught by a worker rather than by me.

**Two counts in the same ticket were also imprecise, and the lane repaired both.** "Thirteen `let Self { … }` encoders below it" is 13 sites in the file, **12** below the anchor, of which only **7** are inside `pub fn encode`; the other 6 are `render`. And my brief's "nine per-element loops" is 8 in `encode_region` and 7 in `encoded_region_len`. Neither changed the substance.

## Coordinator verification of the landed work, 2026-08-22 at `5c104c59`

The fourth site the ticket left unverified is **real**: `alpha_access_key` in `compact.rs` frames the same `GatherReadAccessData` under `tiler.index.access-read.alpha.v1`, and its Direct arm additionally elided `mode` with nothing recording the omission.

**The finding worth carrying forward is about what region identity can discriminate.** Perturbing `access_read_key` and `alpha_access_key` initially moved **zero** of 117 dumped records, because a consistent field reordering inside a structural or alpha key is a bijection on keys — interning and canonical order are unchanged, so every region identity stays put. Region identity therefore **cannot discriminate those two encoders at all**, and a byte-identity harness that only dumps region identities would have reported a clean comparison while proving nothing about them. The lane found this by extending the harness to dump the draft keys directly, turning 0 into 6/1/7/1. Any future byte-identity demonstration over this layer must dump the keys, not only the identities.

Confirmed by the coordinator: `INDEX_REGION_DOMAIN` has **0** occurrences in the diff, no golden or pinned identity file is in the commit, and `git grep` for `drift_probe` and `regionenc_harness` at the landed commit both return nothing.
