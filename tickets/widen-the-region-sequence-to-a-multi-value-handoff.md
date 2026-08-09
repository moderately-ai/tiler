---
id: widen-the-region-sequence-to-a-multi-value-handoff
title: Widen the region sequence to a multi-value handoff
status: deferred
priority: p3
dependencies: []
related: [admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence, accept-the-multi-region-index-realization-surface, register-the-softmax-realization-law]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, indexing]
---
## User-visible outcome

A non-final stage of a `VerifiedIndexRegionSequence` publishes more than one value, so a family whose single region computes two independent results — two folds over one pass, most plainly — has a chain its law could be written as.

## Why this is deferred rather than todo

**Fact — it is the arm of a fork that lost, and it lost on the softmax rather than in general.** [`admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence`](admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence.md) named the fork: a value surviving multiple stages, or a multi-value handoff. Both were run against the softmax's four refused stagings and against generality. The multi-reader arm landed; this one did not, for reasons that are specific to the softmax and do **not** generalize to a refusal:

- Reaching the softmax through a handoff needs one stage to publish `(e, d)`. In one region the parallel dimension and the reduction dimension are distinct, so a region writing `e_i` per point *and* folding `e` reads the scores at both and evaluates the exponential twice per element — a different scalar program by the standard `StagedStrictSerialSumThenPointwiseF32`'s own doc-comment sets, and it doubles the operation's one inexact step.
- The alternative handoff staging has the folding stage republish `e` verbatim beside `d`, which costs a full-size identity copy and puts an output boundary and a write that are no part of what the operation means inside a region's canonical identity.

**Fact — the capability is nevertheless real and the multi-reader arm cannot express it.** A stage producing two *independent* values consumed by one later stage — a sum and a sum of squares in one pass, which is the two-fold layer-normalization shape — is not a retention question at all. No amount of widening the reader vocabulary reaches it, because it is the *publication* vocabulary that admits one value.

**Fact — nothing registered asks for it.** `NotChained` is what refuses it today, at `crates/tiler-ir/src/index/sequence.rs`, and the deliberate narrowness is documented at that site. The softmax takes the multi-reader arm; the normalization publishes one value; no other family is registered. Building the vocabulary now would harden a publication contract, an identity encoding, and a refusal set against a caller that does not exist — the premature-API failure `AGENTS.md` names.

## What it would cost, so the trigger is priced

The publication side of `try_new` becomes a list per stage rather than one value, `StagedInputSource::Intermediate` needs a second ordinal selecting *which* of the producer's values (or a new variant), and that ordinal enters `encode_sequence_identity` — which is the one change here that is **not** identity-neutral by construction, unlike the multi-reader widening. Existing chains would have to keep their exact bytes through a framing that did not previously write the ordinal, so the encoding needs an explicit compatibility argument rather than the "the map did not change" argument the retention widening could make. It is a public-boundary redesign and would land as a labelled draft with its own acceptance node.

## Trigger

A registered semantic family whose canonical realization has a stage producing two or more values that a later stage consumes, and for which no multi-reader chain over single-value publications expresses the same scalar program. A layer normalization computing a mean and a mean of squares in one fold pass is the concrete candidate; the softmax's online single-pass form is **not**, because `SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM` records that it consumes distributivity and the exponential's functional equation, which ADR 0095 declines and no declared dimension names.

## Closes when

The trigger fires and the vocabulary lands, or the trigger is re-evaluated and the capability is shown unreachable.

## Trigger check log

- **2026-08-06 — not fired.** No registered family has a multi-value stage. Reproduce: `grep -n 'realizes_region_sequence' -A 8 crates/tiler-ir/src/index/law.rs` names the two staged variants, and both publish one value per non-final stage; `cargo nextest run -p tiler-ir -E 'test(a_non_final_stage_publishing_two_values_refuses)'` passes, which is the refusal still standing.
- **2026-08-09 — not fired.** RMS normalization and softmax now register region-sequence laws, but neither has a non-final stage publishing multiple values. `a_non_final_stage_publishing_two_values_refuses` still exercises `NotChained { stage: 0 }`; no registered sum-and-sum-of-squares single-pass family requires the wider publication identity.
