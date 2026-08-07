---
id: relocate-the-sourced-extent-vocabulary-to-the-shape-module
title: Relocate the sourced-extent vocabulary from the index module to the shape module
status: done
priority: p1
dependencies: []
related: [carry-symbolic-extents-into-the-semantic-program, promote-the-symbolic-index-profile-to-a-public-boundary]
scopes: [implementation/ir, implementation/compiler, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, extents, api]
---
## User-visible outcome

The crate's one constant-or-symbol extent vocabulary lives where its components live, so the semantic layer can consume it without minting a second copy. The index layer's accepted behaviour, identity bytes, and diagnostics are unchanged.

## Why this exists

**Fact.** `SourcedExtent`'s own documentation calls it "the crate's *one* constant-or-symbol vocabulary for an index-layer magnitude", and states the ground: "a second divisor enum would give a frontend two ways to spell the same fact, two encodings to fold into identity, and two places to extend when a third source kind arrives." Reproduce with `grep -n "constant-or-symbol vocabulary" crates/tiler-ir/src/index/sourced.rs`, which returns line 144.

**Inference.** The scoping phrase "index-layer" is a fact about where the type currently sits, not about what it is: `SourcedExtent` is `Extent | ShapeSymbol` and both live in `crate::shape`. [The symbolic-semantic-extents record](../docs/research/shapes/symbolic-semantic-extents.md) eliminates a semantic-layer mirror on exactly this ground, which leaves relocation as the only way for two layers to share one vocabulary.

## Implementation keys

- Move `SourcedExtent`, `SourcedShape`, `ExtentSources`, `ExtentSourceError`, `SymbolicExtentError`, and `EXTENT_PHASE_CEILING` into `crate::shape`, re-exported flat as the `ShapeEnv` vocabulary already is.
- Decide and state whether `tiler_ir::index` keeps re-exports at the accepted paths or whether callers move. Either is defensible; leaving both live without saying which is canonical is not.
- Change no encoding. `SourcedExtent::encode`, `SourcedShape::encode`, and their `encoded_len` companions must produce identical bytes, so no identity domain moves. A domain that advanced for a relocation alone would make two identical subjects carry different domains, which the index promotion already refused for a visibility change.
- `EXTENT_PHASE_CEILING`'s documentation currently derives the boundary case by inference from a rule the accepted contract states for *semantic* extents. Once the type is shared, that inference becomes the quoted case for one of its two consumers; correct the comment rather than leaving it describing the narrower situation.

## Evidence

- Every existing `tiler-ir`, `tiler-compiler`, and `tiler-reference` test passes unchanged, including the index-region identity assertions.
- A canonical-bytes comparison over a symbolic and a static region before and after the move, showing equality.
- `git diff --check`, `tkt lint`, per-package Clippy with warnings denied, `make full`.

## Public boundary

The exported paths change, which is ADR 0075's always-ask category. Tom accepted the current paths on 2026-07-31; this is a re-acceptance and the packet must say so, listing the before and after path for each of the six items.

## Worker notes (2026-08-07, `agent-relocate`)

Three of this ticket's stated Facts no longer matched the tree at base `0ee647ee`, and are repaired here rather than worked around.

**The item list was five, not six.** `SymbolicExtentError` did **not** move, and the ticket's key above (and A1 of the symbolic-semantic-extents record) is wrong to list it. It is `Source(ExtentSourceError) | Structural(IndexBuildError) | ShapeVocabulary(ShapeError)`, so siting it in `crate::shape` would make the crate's base vocabulary name `crate::index::IndexBuildError` — an inversion of the layer the whole relocation is arguing for. It also would not deliver the sharing this ticket exists for: a second consumer refusing a sourced extent puts *its own* build error in the structural slot, so it needs its own union and would not reuse this one. Only `Source` is the shared authority, and `ExtentSourceError` is what moved. This is the same "collapsing them would report one limit's rejection under the other's name" argument the `ShapeVocabulary` variant already carries one level down; `decide-symbolic-extent-error-siting` records the containment direction.

**`SourcedIndexInteger` exists and also did not move.** It was added after this ticket was filed (`admit-symbolic-index-expression-coefficients`, index-region domain `v9` → `v10`) and is `IndexInteger | ShapeSymbol`, so the same inversion applies. Its symbol half is not duplicated: it converts from `SourcedExtent` and admits through the one `ExtentSources::admit`.

**The phase-ceiling key described the wrong direction, and the ladder was missing a rung.** The comment at base made an index-layer *tensor boundary* the quoted case and an index *domain* the inference. The accepted clause is about semantic initial output shapes, so the corrected ladder is: a semantic output extent is the quoted case (no implementation at this commit), an index boundary inherits it directly, an index domain is the inference, and a divisor **or a linear-combination coefficient** follows the domain. The coefficient rung did not exist when this ticket was written and reaches the ceiling through the same `ExtentSources::admit`.

**Re-export decision: callers move; `tiler_ir::index` keeps no re-export.** AGENTS.md — "complete replacements should remove superseded internal paths rather than preserve unneeded compatibility" — and a compatibility re-export would restore the second spelling the relocation removes. Fourteen call sites across `tiler-ir`, `tiler-compiler`, and `tiler-reference` were moved to `tiler_ir::shape`.

**Encoding evidence.** `SourcedExtent::{tag, encode, encoded_len}` and `SourcedShape::{encode, encoded_len}` moved verbatim; `INDEX_REGION_DOMAIN` stays `tiler.index-region.v11`. Canonical bytes compared at `0ee647ee` and at this branch over one wholly static region and one region carrying a symbolic boundary, a symbolic divisor, and a symbolic coefficient: **1,090 and 1,466 bytes respectively, byte-identical across the move** (probe run in both trees, then removed). `tiler-build`'s pins are unmoved: `7a2bfe51619c05a13fe86cd973e1dfa85c7353da33e4e75af0531068b774357d` / `8bdcde644d7df6d4ca95736f445a011b2d163efdfb3ba93a5c0a954d139b1aa2` / 65,294 bytes.

**Populations.** 3,056 `#[test]` sites at base and at HEAD; 3,049 run, 7 skipped, in both. The 26 tests in `index/sourced.rs` stay there because every fixture builds an `IndexRegionBuilder`.

**Refusal observed failing.** Replacing `available > EXTENT_PHASE_CEILING` with `false` in the relocated `ExtentSources::admit` fails four tests (`a_source_after_the_phase_ceiling_is_refused_at_the_dimension`, `a_boundary_source_after_the_phase_ceiling_is_refused_where_it_is_written`, `a_divisor_source_is_refused_under_the_authority_that_refused_it`, `a_symbolic_coefficient_source_is_refused_under_the_authority_that_refused_it`); restored before commit.

**Out of scope for this worker.** `docs/ir.md` (`contracts/foundation`), `docs/roadmap.md` and `docs/open-questions.md` (`contracts/navigation`, held by a parallel worker), and `docs/research/shapes/symbolic-semantic-extents.md` (`research/shapes`) all name `tiler_ir::index::SourcedExtent` and now carry stale paths; A1 of that record also lists the six-item set corrected above. Its line 32 Fact — that grepping the semantic module for `SourcedExtent` "returns nothing" — was already false at base (`semantic/slice.rs`, `semantic/softmax/tests.rs`). These need a follow-up ticket in the owning scopes.

## Outcome — delivered 2026-08-07 at `f32813da`

Five items moved from `tiler_ir::index::` to `tiler_ir::shape::`: `SourcedExtent`, `SourcedShape`, `ExtentSources`, `ExtentSourceError`, `EXTENT_PHASE_CEILING`. No signature, field, variant or behaviour changed. Fourteen call sites moved with them. **No compatibility re-export was left**, deliberately — a re-export would reinstate the second spelling this relocation exists to remove, and `AGENTS.md` requires a complete replacement to remove the superseded path.

**Purity was proved, not argued.** The worker materialized base `0ee647ee` as a plain directory, installed an identical throwaway probe in both trees, and compared canonical encodings — one wholly static region (1,090 bytes) and one carrying a symbolic boundary, a symbolic divisor and a symbolic coefficient (1,466 bytes). `diff` reports identical, same SHA-256 both sides. `INDEX_REGION_DOMAIN` stays `tiler.index-region.v11`; the pinned population was enumerated at 8 files and 35 literals, none touched; and the standard Metal identity test was **run explicitly** rather than inferred from a green suite. Test-site count is 3,056 at base and at head and matches per-file across all eleven touched files, so nothing was silently dropped in the move.

### The ticket named six items; five moved, and the sixth was argued rather than forgotten

**`SymbolicExtentError` stays in `index`.** It is `Source(ExtentSourceError) | Structural(IndexBuildError) | ShapeVocabulary(ShapeError)`, so moving it to `shape` would make the crate's *base* vocabulary name `crate::index::IndexBuildError` — **inverting the exact layering this ticket argues for.** And it would not deliver the sharing: a second consumer refusing a sourced extent puts its own build error in the structural slot and needs its own union. Only `ExtentSourceError` is the shared authority, which is what moved. That is the same argument the `ShapeVocabulary` variant already carries one level down.

**`SourcedIndexInteger` stays for the same reason**, and this ticket could not have named it — it arrived after filing, with the `v9 → v10` step.

Two further ticket Facts were repaired rather than worked around: the phase-ceiling key had its direction backwards, and its ladder was missing the linear-combination-coefficient rung that postdates the ticket.

### Released

- [`accept-the-sourced-extent-vocabulary-at-its-shape-module-paths`](accept-the-sourced-extent-vocabulary-at-its-shape-module-paths.md) — five changed public paths are a boundary under ADR 0075 even with no signature change, so they park for Tom. The node carries the objection worth making: `ExtentSources` is consumed almost entirely by index-region construction, so these items now live away from every one of their callers.
- [`repoint-the-sourced-extent-paths-in-the-four-documents-that-name-them`](repoint-the-sourced-extent-paths-in-the-four-documents-that-name-them.md) — four documents name paths that no longer exist, each outside this ticket's scopes. The shapes record additionally lists the wrong six-item set, and carries a Fact — "grepping the semantic module returns nothing" — that was **already false at base**, contradicted by `semantic/slice.rs` and `semantic/softmax/tests.rs`.

**Stated read boundary.** Five files were read in full; for `law.rs` (4,439 lines), `refinement.rs` (6,750) and the eight cross-crate consumers the change is confined to `use`-list membership and doc-link paths, so each import block and every occurrence site of the moved symbols was read rather than the whole file — with the compiler and the 3,049-test suite verifying the substitution exhaustively. Recorded rather than left implicit.

`make full` exit 0 on the branch and again on the merged tree.
