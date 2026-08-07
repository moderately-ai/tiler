---
id: admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence
title: Admit a handed value with more than one reader in the region sequence
status: review
priority: p2
dependencies: []
related: [widen-the-staged-realization-law-to-the-registered-elementary-families, admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold, accept-the-multi-region-index-realization-surface, accept-the-multi-reader-index-realization-retention, register-the-softmax-realization-law, widen-the-region-sequence-to-a-multi-value-handoff, carry-a-multi-reader-intermediate-through-region-formation]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-softmax-law
lease_expires_at: 1786070865
---
## User-visible outcome

`VerifiedIndexRegionSequence` can express a realization in which one stage's published value is read by more than one later stage, so `tiler::softmax-f32@1` has a chain its law could be written as. Today it does not, and this is the second wall holding the softmax's law — not the one the graph currently records.

## Why this exists: the softmax is unrealizable under the current chain rules, with or without the maximum key

**Fact, and it corrects a premise.** [`widen-the-staged-realization-law-to-the-registered-elementary-families`](widen-the-staged-realization-law-to-the-registered-elementary-families.md) recorded the softmax half as waiting on [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`](admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold.md). That key is necessary and not sufficient. The blocking fact is in `crates/tiler-ir/src/index/sequence.rs`, and it is checked rather than documented:

- a non-final stage publishes **exactly one** value, or `try_new` answers `NotChained` (`sequence.rs:306-317`);
- a handed value is read by the **immediately following** stage and by **nothing else**, or `try_new` answers `UnavailableIntermediate` — the `owed` slot is cleared by its one consumer and a second claim finds nothing (`sequence.rs:248-301`).

**The derivation.** `softmax_f32_reference_semantics` (`crates/tiler-ir/src/semantic/softmax.rs:394-408`) pins `m = max fold over x`, `e_i = Exp(x_i - m)`, `d = sum fold over e`, `c = 1.0 / d`, `r_i = e_i * c`. Every staging is refused:

1. **Publish `e`.** `S0 -> m`, `S1` reads `x` and `m` and publishes `e`, `S2` folds `sum(e)` and publishes `d`, `S3` computes `r_i = e_i * c`. `S3` needs `e`, whose producer is `S1` and whose one reader was `S2`. `UnavailableIntermediate`.
2. **Publish `d`.** `S0 -> m`, `S1` reads `x` and `m`, computes `e_i` internally and folds it to `d`. `S2` then needs `m` to recompute `e_i`, and `m` was consumed by `S1`. Nothing available.
3. **Publish the pair.** `S1` hands `(m, d)` on as two values. `NotChained`.
4. **Recompute `m` per point in the final stage.** This is a different scalar program, by exactly the argument `StagedStrictSerialSumThenPointwiseF32`'s own doc-comment already makes about a fold read more than once: the reference computes `m` once per row.

So the softmax needs either a value that survives more than one stage, or a multi-value handoff, and neither is expressible. This is a vocabulary gap, not a missing scalar.

## Scope

The design question is which of the two to admit, and it is a real fork rather than an implementation detail: a value with several readers makes the intermediate's *lifetime* span stages, which the current model deliberately refuses ("A value handed further down the chain would have to stay live across a stage that does not mention it, which the sequence deliberately cannot express rather than leaving the retention implied by stage order" — `sequence.rs:75-78`). Whatever lands must state where retention is recorded rather than implying it from stage order, and must move `CanonicalIndexRegionSequenceIdentity` coherently: the identity encodes each stage's source list, so a source naming a non-adjacent producer is a new preimage and needs its own injectivity reasoning at the encoding site.

The sequence surface is public and accepted ([`accept-the-multi-region-index-realization-surface`](accept-the-multi-region-index-realization-surface.md)), so a change to it is a public-boundary redesign and lands as a labelled draft with its own acceptance node.

## Non-goals

Writing the softmax's law. That additionally needs the maximum scalar key, and it should be one ticket once both walls are down.

## Closes when

A realization in which one published value has more than one reader is expressible and checked, its retention contract is stated rather than implied, the sequence identity encodes the wider source vocabulary injectively with the reasoning recorded at the encoding site, and every existing one-reader chain's identity is unchanged byte for byte.

## Outcome — 2026-08-06

The multi-reader arm landed and the fork **resolved rather than parked**: exactly one arm survives the elimination this ticket set. Commit `c02d4f7d` on `tkt/admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`, base `dd9def76` — the tree `make full` was run against; the branch tip is one further commit correcting this hash reference and touching no gate-carry path. Shared with [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`](admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold.md).

### The fork's resolution

Run against the softmax's four refused stagings **and** against generality.

**The multi-value handoff reaches the softmax only through a copy-through, and both of its shapes are disqualified by the law layer's own standards.**

- **One stage publishes `(e, d)`.** In a single region the parallel dimension and the reduction dimension are distinct `DomainRole`s on distinct dimensions, so a region writing `e_i` per point *and* folding `e` must read the scores at both and evaluate `Exp(s_i - m)` twice per element. That is "a different scalar program, not a different schedule" by the standard `StagedStrictSerialSumThenPointwiseF32`'s own doc-comment sets and `realize_root_mean_square_scale`'s split argument repeats — and here it doubles the operation's *one inexact step*, the one carrying the resolved ADR 0042 accuracy contract. This is the same argument that eliminated staging 4 in this ticket's body, applied to a different recomputation.
- **The folding stage republishes `e` verbatim beside `d`.** Structurally sound, and it costs a full-size identity copy: an output boundary and a write that are no part of what the operation means, carried inside a region's canonical identity. It expresses retention by duplication rather than as a lifetime, against `AGENTS.md`'s "represent … lifetimes explicitly" and "keep the public graph about **what** operations mean, not **how** hardware runs them".

**The multi-reader arm generalizes and the handoff arm does not, for this shape.** Layer normalization's `x - m` is read by the variance fold and again by the output pass; any log-sum-exp sibling has the same shape. Nothing was found that the multi-reader arm cannot express and the handoff can — **except a genuinely different capability**: one region producing two *independent* results consumed by one pass (a sum and a sum of squares in one fold). That is a publication-vocabulary question, not a retention one, and no reader widening reaches it. It is filed and deferred with a priced trigger at [`widen-the-region-sequence-to-a-multi-value-handoff`](widen-the-region-sequence-to-a-multi-value-handoff.md) rather than left as an unresolved fork, because no registered family asks for it.

### What landed

`VerifiedIndexRegionSequence::try_new` was restructured into three passes — collect each non-final stage's single publication, validate every declared source against it, then check every publication has a reader — replacing the single in-flight `handoff` slot that made adjacency and single-readership structural. `StagedInputSource::Intermediate(p)` now admits any earlier stage, and several reads of one value are admitted at one stage or across stages.

**Where retention is recorded.** `StagedIntermediate` gains `retained_through()`: the last stage across which the published value stays live, so its lifetime is `producer()..=retained_through()`. It is *derived* from the declared readers and then recorded on every record of that value — the module's own "derived and checked, never declared and believed", rather than an exception to it. A separately declared span would be a second authority over one fact that could disagree with the readers. The contract is stated in the module header under **The retention contract** and in `StagedIntermediate`'s rewritten **Lifetime** paragraph, which previously said the model "deliberately cannot express" it.

**Three rules bound the chain**, stated in the header: a source names a strictly earlier stage (acyclicity, which is what adjacency used to provide); a non-final stage publishes exactly one value (`NotChained` unchanged, and its comment now says why widening *that* is a separate capability); a published value has at least one reader, checked over the whole chain rather than at the following stage.

**`StagedIntermediate` stays per read, not per value.** For a one-reader chain — every chain any registered law spells — the record set is exactly what it always was. This is stated as a choice worth objecting to on the acceptance node: per value is arguably cleaner, but `consumer()`/`consumer_input()` are read from `crates/tiler-compiler/src/region.rs`, which this scope could not edit, and per-boundary is the granularity this record has always had.

### Identity — unchanged by construction, and pinned anyway

`encode_sequence_identity` is **untouched**. `Intermediate` already wrote its producer ordinal in full under tag `2`, and `push_len` is injective over the whole `usize` range rather than over the range the chain rules happened to admit — so the admitted preimage set widened while the map did not. Every chain expressible before encodes byte for byte as before, and injectivity over the wider domain follows from the unchanged length-prefixed, tagged, ordered argument. That reasoning is recorded at the encoding site, including why the ordinal is now load-bearing where it used to be redundant.

`the_landed_one_reader_chain_identities_are_unchanged_byte_for_byte` (`crates/tiler-ir/src/index/law.rs`) pins exact length plus a SHA-256 under a test-local domain for three realized sequence identities — the normalization's own law at `[3,4]` axis 1 and at `[4]` axis 0 (the live instance), and the plain staged template — captured on base commit `dd9def76` before either widening landed. **Zero pins moved**, as expected. The workspace suite is 2899 passing against a base of 2892 (seven new tests), with one expected exception owned by the sibling ticket: the compiler request digest, moved by the *scalar* registration and not by anything here.

### The softmax's staging #1, expressible and checked

`the_softmax_staging_publishing_the_exponentials_chains` builds the four-stage chain at the sequence layer with real reduced-rank interfaces — two `row_fold_region` stages (`[3,4] -> [3]`, a genuine `reduce`) and two `row_pointwise_region` stages (`[3,4]` and `[3]` in, `[3,4]` out) — sourced `[[Occurrence(0)], [Occurrence(0), Intermediate(0)], [Intermediate(1)], [Intermediate(1), Intermediate(2)]]`. It asserts the four reads as `(producer, consumer, retained_through)` = `[(0,1,1), (1,2,3), (1,3,3), (2,3,3)]`: the exponentials are published by stage one, read by stage two and again by stage three, and stay live across a stage that publishes something else. The regions stand for the pinned formula's steps in their *boundaries*, which is where the chain checks anything; emitting the softmax's scalar programs is [`register-the-softmax-realization-law`](register-the-softmax-realization-law.md)'s work.

### Watched failures, and one that found a hole

Four perturbations, each run and read:

1. **Adjacency restored** (`*producer + 1 == position`): only `the_softmax_staging_publishing_the_exponentials_chains` fails, at `UnavailableIntermediate { stage: 3, slot: 0 }` — the second read of `e`. 891/892 pass.
2. **Single readership restored** (`reads[*producer] == 0` added): the softmax test *and* `one_published_value_read_twice_by_one_stage_chains` fail, and only those two. 890/892.
3. **The acyclicity bound dropped** (`published.get(*producer)` with no `< position` filter): **the whole suite passed.** The bound was uncovered. Both of this module's existing "wrong producer" assertions used two-stage chains, where naming stage one is already an *out-of-range* ordinal, so an implementation with no ordering rule at all refuses them. The fix is a new test, `a_value_read_at_or_before_its_producing_stage_refuses`, over three-stage chains where both the self-reference and the forward reference name a *live* producer whose element type and shape agree — plus the strictly-forward wiring of the same three regions, admitted, so the refusals prove the bound rather than proving something else was wrong. Re-run under the same perturbation, that test fails.
4. **The retention span reduced to the single read** (`retained_through = consumer`): the softmax test fails with `(1, 2, 2)` where `(1, 2, 3)` is required — the exact claim that a value's span is a property of the value and not of one read.

### Public boundary

The sequence surface is public and accepted, so this widening lands as a labelled draft with its own acceptance node, [`accept-the-multi-reader-index-realization-retention`](accept-the-multi-reader-index-realization-retention.md), parked at `awaiting-decision`. Nothing is self-accepted. The module header now points at both nodes.

### Also filed

[`carry-a-multi-reader-intermediate-through-region-formation`](carry-a-multi-reader-intermediate-through-region-formation.md) — `crates/tiler-compiler/src/region.rs` synthesizes one `GraphValue` per `StagedIntermediate`, so a value read twice would become two independent synthetic values. Nothing is wrong today (per-read and per-value coincide for every registered law, and the workspace suite is green); the gap opens with the first multi-reader law.

### Checks

`cargo fmt --all --check`; `make lint` (workspace clippy less the three prototypes, `-D warnings`); `make doc` (`RUSTDOCFLAGS="-D warnings"`); `cargo nextest run --workspace` — 2899 passed, 7 skipped; `cargo test --workspace --doc`; `make full` (green end to end: 2899 workspace, 1003 release over `tiler-reference` and `tiler-compiler`, `ticketsplease lint`, `shellcheck`); `tkt lint`; `git diff --check`.

**Scope evidence, and how to read it.** Guard the branch against the *co-resident* ticket: `tkt guard tkt/admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold` returns exit 0, `"conflict": false`, `"under_declared": []`, with only non-failing declared-area overlaps against open siblings. Guarding against **this** ticket alone reports an escape, and that is correct rather than a defect: every file this ticket caused is inside its declared `implementation/ir`, and the escape is entirely the co-resident ticket's — `crates/tiler-ir/src/index/scalar.rs`, `crates/tiler-ir/src/index/mod.rs`, and the one `crates/tiler-compiler/src/explain.rs` pin, which that ticket declares `implementation/compiler` for. The two claims are held by one agent on one branch, so the union of their declarations is the branch's declaration.
