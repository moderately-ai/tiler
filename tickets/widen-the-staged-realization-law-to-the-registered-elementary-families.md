---
id: widen-the-staged-realization-law-to-the-registered-elementary-families
title: Widen the staged realization law to the registered elementary families
status: done
priority: p1
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages, admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold, resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage, accept-the-root-mean-square-scale-realization-law, admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence, flip-the-normalization-law-wall-test-and-rebaseline-the-request-pin]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`tiler::rms-norm-f32@1` -- and, once its scalar lands, `tiler::softmax-f32@1` -- carries a registered `IndexRealizationLaw`, so `FrozenIndexRealizationLawRegistry::resolve` stops answering `MissingRealizationLaw` for them and refinement can prove a provider's emitted region sequence realizes the occurrence.

## Why this exists: the accepted staged template expresses neither family

**Fact, and it corrects a premise.** [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md) states that "the law registrations then use the accepted staged template (or a single-region law where the family is one region)". Read against `crates/tiler-ir/src/index/law.rs`, that is false for both registered elementary families. `IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32` (`law.rs:106-111`) is realized by `realize_staged_sum_then_pointwise` (`law.rs:953-1017`), and its exact shape is:

- **stage zero** is `SumPlan::for_boundaries` over operand zero with no prologue -- a plain strict left fold of the operand's own elements (`law.rs:967-982`);
- **stage one** is `emit_pointwise` applying one *binary* scalar to operand one and the published fold, with the fold legible only at the result shape or at rank zero (`law.rs:984-1004`).

The law's own doc-comment (`law.rs:98-105`) already says it "is deliberately *not* the normalization's own law". What the ticket above assumed is that the remaining gap was scalar keys; it is the template.

### The normalization

**Fact.** `rms_norm_f32_reference_semantics` (`crates/tiler-ir/src/semantic/rms_norm.rs:228-238`) pins `q_i = x_i * x_i`, `a = fold(q)`, `u = a / N`, `t = u + eps`, `r = Rsqrt(t)`, `y_i = w_i * (x_i * r)`. Three of those are outside the template:

1. the fold is over `x_i * x_i`, and the template's stage zero folds the operand's elements directly -- `SumPlan` has no prologue;
2. the published intermediate is transformed before the pointwise pass consumes it (`/ N`, `+ eps`, `Rsqrt`), and the template's stage one applies exactly one scalar;
3. stage one reads *three* values -- the weight, the normalized value, and the intermediate -- where `emit_pointwise` refuses any operand count other than two (`law.rs:707-709`, rule `pointwise-operand-arity`).

**And a silent-wrongness hazard if the template were registered anyway.** `reduction_axes` reads its attribute by field ID and tolerates extra fields (`law.rs:1396-1402`), while `realize_constant` refuses a record whose field set it does not expect. The normalization declares two attributes -- the axis and the exact `eps` bits -- so registering the staged template for it would drop `eps` with no refusal, and `eps` is part of the operation's identity (`rms_norm.rs:76-94`). Any widening must consume every declared attribute or refuse by name.

### The softmax

**Fact.** `softmax_f32_reference_semantics` (`crates/tiler-ir/src/semantic/softmax.rs:394-408`) pins a *maximum* fold, then `e_i = Exp(s_i - m)`, then a *second* fold summing `e`, then `c = 1.0 / d`, then `r_i = e_i * c`. That is at least three regions with two distinct folds, the first of which has no registered scalar combiner at all -- [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`](admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold.md) owns that. The template has one fold and one pointwise pass.

## Scope, and what is reachable now

The normalization half is reachable today: `rsqrt_f32_scalar_op` landed as a draft and every other scalar the reference names is registered. The softmax half is not, and waits on the maximum key.

The widening is a public surface: `IndexRealizationLaw` is `pub` and `#[non_exhaustive]`, so a new variant lands as a labelled draft with its own acceptance node. Its encoding tag must be appended -- tags `1..=9` are taken, and `the_staged_law_tag_is_append_only_and_distinct` (`law.rs`) is the pattern -- with per-tag injectivity reasoning recorded at the encoding site.

## Non-goals

Making the compiler *recognize* the family as a program stage. That is blocked on a separate fork -- see [`resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage`](resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage.md) -- and a registered law is useful without it: it flips `the_normalization_still_refuses_for_an_absent_law_and_not_for_the_vocabulary` (`crates/tiler-compiler/tests/two_region_occurrence_lowering.rs:1005-1049`) and lets refinement verify an emitted sequence.

## Closes when

At least the normalization's law is registered and realizes a verified `VerifiedIndexRegionSequence` whose stages match the pinned reference step for step, every declared attribute is consumed or refused by name, the new encoding tag is proved append-only and injective, and the wall test above is flipped rather than deleted.

## Outcome

The normalization half landed. The softmax half is further away than the graph said, and that finding is the second-most useful thing here.

### The law's shape, and the elimination that produced it

A **new appended variant**, not a generalized template:

```rust
StagedRootMeanSquareScaleF32 { axes_attribute: AttributeFieldId, eps_attribute: AttributeFieldId }
```

Four candidates were tested against `law.rs`'s own contract that "each variant is an atomic template whose complete interpretation is owned here", and three fail:

- **Reuse `StagedStrictSerialSumThenPointwiseF32`.** Refuted by this ticket's own body, and by measurement: it *realizes* a rank-one normalization occurrence successfully while never reading `eps`. That silent success is now a watched assertion.
- **Widen that variant in place with optional prologue/epilogue/arity fields.** It mutates an accepted variant's payload and therefore tag 9's encoding, and most field combinations denote no program. Discarded.
- **Carry the epilogue and the pass as law data.** Representing a chain of scalar applications in law data is a scalar-program language inside the law vocabulary — the universal IR the module header refuses.
- **A field-less variant, `PreciseSiluF32`-style.** Hard-codes two record-local attribute identifiers, which is the exact defect `scalar_attributes`' doc-comment records for the `f32`/`bf16` constants numbering their payload field alike.

The survivor fixes the chain (as `PreciseSiluF32` does, because the chain is what the template means) and names the attributes (as every parameterized variant does, because record-local identifiers are what a second row varies).

**Where the generality actually went, per the worked-examples discipline.** The three gaps this ticket names are closed as reusable *emitters*, not as one family's inline code: `SumPlan::contributor_square` (a fold over a per-contributor square), `SumPlan::fold` (a fold that returns its value so an epilogue can transform it inside the producing region, split out of `emit_serial_sum` with its emission order preserved byte for byte), and reading a reduced-rank published value at the kept coordinates of the consuming stage's point domain. The next staged family instantiates those; it does not instantiate this variant.

**Why the epilogue is in stage zero.** `r` is computed once per folded row and read once per point. Publishing `a` and putting `/N`, `+eps`, and `Rsqrt` in the pointwise pass evaluates each `N` times per row — a different scalar program, not a different schedule, by the same argument the staged template's own doc-comment already makes.

### Step-for-step realization evidence

`the_normalization_law_realizes_the_pinned_reference_step_for_step` derives an occurrence over `[3, 4]` reduced on axis 1 and reads the two verified stages. It pins the whole operation population of each stage first (so a walk that skipped a step could not pass), then walks the fold stage backwards from its published value — a verified region orders operations canonically rather than in emission order, so the walk navigates by definition rather than by position:

`r = Rsqrt(t)` is the published value; `t = add(u, eps)` with the eps constant's attribute record equal to the declared payload; `u = divide(a, N)` with the extent constant equal to `4.0f32`; `a` is a reduction seeded at a squared seed and combining a squared tail; each square is one *read* applied to itself at both sites. The scale stage is `multiply(w, multiply(x, r))`, with the boundary of each read asserted, because the value and the weight agree on element type and shape and only the boundary separates them. Sources: `[[Occurrence(0)], [Occurrence(0), Occurrence(1), Intermediate(0)]]`, and the handed value is `[3]` — one per folded row.

Three deliberate perturbations were run and watched fail: dropping the contributor square (step population), transposing the outer multiply's operands (read boundary), and moving the eps payload by one ULP (attribute record).

### The eps-consumption proof

`realize_root_mean_square_scale` requires the occurrence's attribute record to be **exactly** the two fields the law names, checked before `reduction_axes` is called, so that function's tolerance for a wider record cannot drop `eps` here. Aliased identifiers are refused first, which is what makes "two distinct names, two fields, both named" imply "each present exactly once".

`the_normalization_law_consumes_eps_where_the_staged_template_drops_it` asserts the hazard (the staged template realizes the same occurrence and never reads `eps`) and then watches four refusals fire: a wrong `eps` identifier and a wrong axes identifier both give `rms-scale-attributes`, aliased identifiers give `rms-scale-attribute-aliasing`, and the transposed pair — which passes the field-set check and reads each payload as the other — gives `rms-scale-eps-kind`.

### The appended tag and its injectivity

Tag 10; tags 1..=9 and their payloads are unchanged, so every sidecar byte any law registry has encoded is byte-identical. The first byte discriminates, so no other variant's encoding can be read as this one. Within the tag the payload is two fixed-width identifiers at disjoint fixed offsets, so the map from the pair to bytes is injective, and the pair is ordered, so the transposed row encodes as a third distinct row — which matters because the transposition is a real construction error the realizer must be able to refuse as a *different* law. `the_root_mean_square_law_tag_is_append_only_and_distinct` checks all four rows pairwise and every earlier variant against it.

### Two refusals that are not in the reference and are derived from it

- `rms-scale-extent-not-exact`: the reference divides by the extent, so a folded count whose nearest binary32 is not the count is refused rather than rounded. The representability test is integer-only (an integer is a binary32 value exactly when its odd part fits the 24-bit significand), so it does not depend on the rounding it detects. Watched: `[16_777_217]` refuses, `[16_777_216]` realizes.
- `rms-scale-empty-fold`: a fold seeded at the first contributor has no first contributor over an empty axis.

### Pins that moved, and the two sites this branch could not touch

The `FrozenIndexRealizationLawRegistry` identity moves, because the sidecar is a count-prefixed run over every registered law and it gained a row. The `FrozenSemanticRegistry` snapshot identity does **not** — it is computed without the sidecar (`refinement.rs:634-636` compares the two separately) — which is why no artifact, cache, or kernel-program pin moved.

The survey is empirical rather than argued: `cargo nextest run --workspace` reported 2844 tests, 2842 passing, and exactly two failures, both in `tiler-compiler` and both expected. They are transcribed into [`flip-the-normalization-law-wall-test-and-rebaseline-the-request-pin`](flip-the-normalization-law-wall-test-and-rebaseline-the-request-pin.md) with their exact repairs, because this ticket holds `implementation/ir` only. That ticket records both collision checks: the first was **vacuous** (the live `implementation/compiler` claim's branch carried zero commits, and an empty diff is not disjointness evidence), and by the second the branch had merged and been removed, with neither site moved on `main` since this base. The request pin must be recomputed on the merged tree, not copied from this branch.

### The rsqrt acceptance's code half

Folded in here on the coordinator's instruction, and executed against the *merged* record rather than the relay: `tickets/accept-the-governed-reciprocal-square-root-scalar-key.md` is `done` on `main` (`32d58a30`), its `## Accepted 2026-08-06` section carries the provenance, and its own "Closes when" routes the label flip to whichever branch holds `implementation/ir`. The draft label on `rsqrt_f32_scalar_op` is replaced by the accepted-boundary form the `PublishingCopy` acceptance established, naming who, when, where, that it was relayed rather than witnessed here, and that acceptance is not stabilization. Three stale "drafted" references in the same file's tests moved with it.

### The softmax remainder, scoped

The graph said the softmax waits on [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`](admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold.md). That key is necessary and **not sufficient**. `VerifiedIndexRegionSequence` requires a non-final stage to publish exactly one value and that value to be read by the immediately following stage and nothing else; the softmax needs `e_i` in both the summing fold and the final scale, or `m` and `d` together in the final stage. Every staging is refused, and the derivation is filed at [`admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence`](admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence.md). The softmax law is one ticket once both walls are down.

### The public boundary

`IndexRealizationLaw` is `pub` and `#[non_exhaustive]`, so the variant lands as a labelled draft with its own acceptance node, [`accept-the-root-mean-square-scale-realization-law`](accept-the-root-mean-square-scale-realization-law.md), parked at `awaiting-decision`. Nothing is self-accepted.

### Checks

`cargo fmt --check`, `cargo check -p tiler-ir --all-targets`, `cargo clippy -p tiler-ir --all-targets -- -D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-ir`, and `cargo nextest run -p tiler-ir` (872 passed) are all green. `cargo nextest run --workspace` is green except the two transcribed compiler-side sites.
