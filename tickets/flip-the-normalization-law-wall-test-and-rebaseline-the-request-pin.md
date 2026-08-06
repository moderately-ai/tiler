---
id: flip-the-normalization-law-wall-test-and-rebaseline-the-request-pin
title: Flip the normalization law wall test and rebaseline the request pin
status: done
priority: p1
dependencies: [widen-the-staged-realization-law-to-the-registered-elementary-families]
related: [implement-stage-level-cover-atoms-for-multi-region-occurrences]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
assignee: coordinator
lease_expires_at: 1786040703
---
## User-visible outcome

`cargo nextest run --workspace` is green again after `tiler::rms-norm-f32@1` gained a registered realization law. Two `tiler-compiler` assertions record the old state and must move with it; both are stated exactly below, so this is transcription rather than derivation.

## Why it is a separate ticket

[`widen-the-staged-realization-law-to-the-registered-elementary-families`](widen-the-staged-realization-law-to-the-registered-elementary-families.md) holds `implementation/ir` only, and both sites are under `crates/tiler-compiler/**`.

The collision question was checked twice and resolved into "not applicable" rather than "disjoint". At first check, `implementation/compiler` was held by the live claim [`implement-stage-level-cover-atoms-for-multi-region-occurrences`](implement-stage-level-cover-atoms-for-multi-region-occurrences.md), and the verification was **vacuous**: `git diff --name-only 20ed0fbf...tkt/implement-stage-level-cover-atoms-for-multi-region-occurrences` was empty because the branch carried zero commits, and an empty diff is not disjointness evidence. At final check, that branch and its worktree were gone and its work had merged (`911fea5a`, `b714d868`), and `git diff --name-only 20ed0fbf..main -- crates/tiler-compiler/src/explain.rs crates/tiler-compiler/tests/two_region_occurrence_lowering.rs` is empty, so neither site moved on `main` either.

So the reason these edits are split out is scope, not collision: the widening ticket never declared `implementation/compiler`, and declaring a second exclusive scope to carry two transcribed edits is worse than filing them where a `tkt` query can find them.

## The two sites

### 1. `crates/tiler-compiler/tests/two_region_occurrence_lowering.rs`

`the_normalization_still_refuses_for_an_absent_law_and_not_for_the_vocabulary` (around line 1005) asserts `IndexRefinementVerificationError::MissingRealizationLaw` and that the provider is driven zero times. Observed on the widening branch, the refusal is now:

```text
Emit { stage: 0, source: Occurrence { rule: "fixture-never-reached" } }
```

because the law resolves and the host then drives `RecordingProvider`, whose `lower` increments its counter and returns that fixture error. **Flip, do not delete**: the assertion's subject moves from "this family has no law" to "this family has a law and the ceiling is now what a provider emits against it". The body becomes the resolution succeeding, `laws.resolve(&subject).is_ok()`, the refusal above, and `driven.load(Ordering::Relaxed) == 1` — the counter that used to read zero is what shows the wall moved.

The file's module doc (lines 24-30 and its `admit-the-rms-normalization-family` link at line 40) states the normalization "is still held, by something else ... no governed scalar operation spells either". Both halves are now false: `tiler.scalar::rsqrt-f32@1` is registered and the family carries `IndexRealizationLaw::StagedRootMeanSquareScaleF32`. A doc comment is a claim, so it moves in the same change.

### 2. `crates/tiler-compiler/src/explain.rs`

`deterministic_trace_is_sealed_and_rendered_separately` (assertion around line 3769) pins the request qualifier. It must move: the request subject binds the frozen semantic-realization authority, whose identity folds the count-prefixed law sidecar, and the sidecar gained a row. This is the ledger sentence that comment already carries ("even this unrelated multiply request must miss an authority snapshot that predates the new row") being kept, not a surprise. Only the law half of the subject moves — the semantic snapshot identity is computed without the sidecar and is unchanged, which is why no artifact, cache, or kernel-program pin moved with it.

Observed on the widening branch alone: `b88654bff9b673c1` becomes `ce6f9106c1c5933b`.

**Do not copy that value.** Recompute it on the merged tree — this pin has been rebaselined by two branches independently before, and its own comment records that both were stale on the merge. Regenerate with:

```sh
cargo nextest run -p tiler-compiler -E 'test(deterministic_trace_is_sealed_and_rendered_separately)'
```

and take the `left` value the assertion reports. Append the reason to the pin's ledger comment in the same change.

## The population, so a partial fix cannot look complete

Two, and they are the complete set: a full `cargo nextest run --workspace` on the widening branch (2844 tests) reported exactly these two failures and 2842 passes.

## Closes when

Both sites are updated, the wall test is flipped rather than deleted, the request pin is recomputed on the tree it lands in rather than copied from the widening branch, and `cargo nextest run --workspace` is green.

## Outcome — 2026-08-06, executed by the coordinator on the merged tree

Both sites repaired at the merge of `d88ebdb8`; `cargo nextest run --workspace` green (2847 passed, 7 skipped — the population grew by the widening branch's own tests plus the flipped one).

**The wall test** is flipped and renamed `the_normalization_resolves_its_law_and_is_held_by_what_a_provider_emits`: `laws.resolve(&subject).is_ok()` pins that the refusal is not the law's absence, the refusal is `Emit { stage: 0, source: Occurrence { rule: "fixture-never-reached" } }`, and `driven == 1`. The module doc's stale bullet and closing paragraph were rewritten in place.

**The request pin** was recomputed on the merged tree per this ticket's own instruction: observed `left` = `b88654bff9b673c1`, replacing `ce6f9106c1c5933b`. The transcription's prose transposed the two values ("b886… becomes ce6f…" — ce6f was the base pin, b886 the observed); the operative instruction (take `left`, never copy) resolved it, and b886 appears in no prior ledger entry, so no replay ambiguity. Cause: the law sidecar gained the normalization's tag-10 row and the request subject folds the sidecar; the semantic snapshot identity is computed without it, so no other pin moved.

**Deviation from the transcription, on Tom's live direction:** the pin's ~480-line append-only rebaseline ledger was condensed in place to its invariant (what the subject folds, the recompute-on-merged-tree rule, the version-step boundary). Per-landing causes now belong in the commits that move the value, starting with this one.

Also corrected the one stale "drafted" expect string at `crates/tiler-ir/src/index/scalar.rs:2852` the widening branch's classifier refused.
