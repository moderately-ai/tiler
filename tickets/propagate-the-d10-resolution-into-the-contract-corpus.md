---
id: propagate-the-d10-resolution-into-the-contract-corpus
title: Propagate the D-10 resolution into the contract corpus
status: done
priority: p2
dependencies: [admit-the-reindex-and-broadcast-operation-families]
related: [design-attention-program-vertical, scope-the-sequence-extending-tensor-family, compose-rotary-position-embedding-from-reindex-and-broadcast, own-operation-family-support-matrix]
scopes: [contracts/foundation, research/program-planning, research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, semantics, structural]
---
## User-visible outcome

A reader who opens [IR](../docs/ir.md) to learn what a `Reindex` admits gets the same answer the registered definition gives, instead of a sentence that predates the decision.

## Why this is a separate ticket

**Fact — the resolution landed outside these files' scopes.** [`admit-the-reindex-and-broadcast-operation-families`](admit-the-reindex-and-broadcast-operation-families.md) settled decision D-10 in `tiler::reindex-f32@1`'s registered `NormativeDefinitionRef`, which is `implementation/ir`. The three documents that still state the question, or the pre-decision reading, are `contracts/foundation` and `research/*`. A ticket does not edit outside its declared scopes, and this remainder is narrow enough to carry its own.

**Inference — nothing checks the disagreement.** Nothing validates the documentation corpus, so a `Reindex` sentence that omits an admitted form costs a reader rather than a gate, and the cost is exactly the kind [AGENTS.md](../AGENTS.md) names: a doc comment is a claim, and an understated one makes reachable work look unreachable.

## Evidence prerequisite

**Fact — the resolution.** A within-axis coordinate permutation is admitted in exactly one named form, `reverse-axis`, the map `i -> extent − 1 − i`; no other is admitted, and one presented under any other name is refused as `reindex.form.unadmitted-kind`. The derivation and its four steps are recorded in the admission ticket's outcome and, in short form, in the registered normative reference itself.

**Fact — the three stale sites.**

- [`docs/ir.md`](../docs/ir.md) spells the initial forms as "bijective permutations/split/merge mappings or legal removal/insertion of unit axes". That sentence is now incomplete, and the same section's `Reindex` paragraph is where a reader looks first.
- [The L4 attention design](../docs/research/program-planning/first-attention-program-vertical.md) carries D-10 in its unresolved-decisions list and states it as open at two further points.
- [The sequence-extending family record](../docs/research/shapes/sequence-extending-tensor-family.md) carries the qualification that its "no slice and no concatenate inside a layer" result is conditional on D-10.

## Required delivery

- **`docs/ir.md`'s `Reindex` paragraph states the complete admitted set**, including the reversal, and states that a general within-axis permutation is a tensor-data-derived index the index-expression vocabulary rejects — which is the reason the admission is one named form rather than a class.
- **L4's D-10 entry moves out of the unresolved list** and records the answer and where it lives, rather than being deleted: an open question is removed only when its answer lives in a durable contract, and the derivation is worth keeping beside the measurement that motivated it.
- **The sequence-extending record's qualification is discharged**, so its result reads as unconditional with the resolution cited rather than as conditional on a decision that has been made.
- **No new claim.** This ticket propagates a decision already taken; it does not widen the family, revisit the rotation question, or restate the derivation as though it were being made here.

## Non-goals

Reopening D-10, admitting a within-axis rotation, and any change to `crates/`.

## Closes when

All three documents agree with the registered normative reference, and `grep -rn 'D-10' docs/` returns only settled statements.

## Outcome

**Fact — the propagated authority is quoted, not paraphrased.** Every sentence added below quotes `REINDEX_F32_NORMATIVE_DEFINITION` in `crates/tiler-ir/src/semantic/reindex.rs`, the registered `NormativeDefinitionRef` of `tiler::reindex-f32@1`, read verbatim: the admitted set is "permute-axes … split-axis … merge-axes … insert-unit-axis and remove-unit-axis … and reverse-axis, the within-axis coordinate map `i -> extent - 1 - i`", stated as "Admitted forms, and no others", and the ground is that "the affine within-axis bijections of an axis are exactly the identity and the reversal, while a general within-axis permutation is a tensor-data-derived index the accepted index vocabulary rejects". The refusal code is `reindex.form.unadmitted-kind`, read from `ReindexFormError::code`'s `UnadmittedFormKind` arm rather than from the ticket text.

**Fact — the three required deliveries.**

- [IR](../docs/ir.md)'s `Reindex` paragraph now states the complete admitted set including `reverse-axis`, names the construction-time refusal code for anything outside it, and states why the within-axis admission is one named form rather than a class: naming the reversal admits every affine within-axis bijection that does anything, while the general reading would admit a permutation *table* — a tensor-data-derived index the bounded initial index-expression vocabulary stated later in the same document rejects. The rotation stays expressible and deliberately unadmitted.
- [The L4 record](../docs/research/program-planning/first-attention-program-vertical.md)'s D-10 entry keeps its question and its motivating measurement in place and records `**Closed, 2026-07-31.**` beside them with the answer, where it lives, the four-step ground in short form, what stays refused, and the two consequences that resolved the other way. This is L3's D-8 idiom, and the entry deliberately stays in the numbered list under the `## Unresolved decisions` heading: [the roadmap's slice row](../docs/roadmap.md) links that exact anchor, and the section's opening sentence now says that a closed entry stays in place.
- [The sequence-extending record](../docs/research/shapes/sequence-extending-tensor-family.md)'s qualification is discharged: "no slice and no concatenate inside a layer" reads unconditionally, with the resolution and its ground cited, and the paragraph is relabelled **Fact** because it now reports a settled decision rather than inferring a conditional.

**Fact — the sweep, counted rather than eyeballed.** `git grep -c "D-10" 446f6fb -- docs/` returns seven mentions in three files: L4 4, the sequence-extending record 1, `docs/roadmap.md` 2, and `docs/ir.md` none. `git grep -c "D-10" HEAD -- docs/` returns eight in four files, the same three counts plus `docs/ir.md` 1 — the contract sentence that had no D-10 statement at all, which is why the ticket exists. The roadmap's two were already settled statements written by the admission ticket and are outside this ticket's scopes; the other six now read as settled — L4 at lines 126, 362, 374, and 399, `docs/ir.md` at 605, and the sequence-extending record at 119. The ticket anticipated the L4 entry plus "two further points"; the sweep found three. The two are the RoPE composition inference at 126 and the typed-refusal bullet at 362, which now says the RoPE swap is *not* a case that refusal fires on and that what it covers is every other within-axis permutation, a rotation included. The third is the delivery-ticket table row at 399, whose outcome for `compose-rotary-position-embedding-from-reindex-and-broadcast` no longer says D-10 "is settled either way" — it is settled, and what remains of that row is the composition.

**Fact — one stale D-10 statement outside this ticket's three files, reported rather than edited.** [`compose-rotary-position-embedding-from-reindex-and-broadcast`](compose-rotary-position-embedding-from-reindex-and-broadcast.md) still carries "**Settle decision D-10** … Answer it in the normative reference either way" as a required delivery, and its `Closes when` still asks for D-10 to be answered. Both are already satisfied, so a worker dispatched on it would either duplicate the resolution or reopen it. `grep -rln "bijective permutations/split/merge" docs/ tickets/` names that ticket as the only remaining file quoting the superseded `docs/ir.md` wording in the present tense. It is a peer ticket's own delivery list rather than a document this ticket owns, and rewriting another ticket's requirements from inside this one is the kind of silent scope expansion the process forbids — so it is reported for the coordinator to redispatch or amend.

**Fact — a fourth document stated the pre-decision admitted set without ever naming D-10, and a D-10 grep could not have found it.** Grepping the *phrasing* rather than the decision id — `grep -rn "removal/insertion of unit axes\|initial forms\|Initial reindexes" docs/` — surfaced [the L2 operation and shape surface](../docs/research/shapes/transformer-operation-and-shape-surface.md), which enumerates the admitted forms twice, in its structural support row and in the *No slice and no concatenate* section, both times stopping at unit-axis insertion or removal. It is in this ticket's `research/shapes` scope, and it is the same defect as the named three: a reader learning what a `Reindex` admits gets an answer the registered definition no longer gives. Both enumerations now carry the within-axis reversal, and the structural row says the RoPE half-split's swap is inside the admitted forms rather than merely near them. This is deliberately a different judgement from the maturity paragraph left alone above: an admitted-forms enumeration is a premise a later reader reasons *from*, while a rung is a claim about a moment.

**Fact — one further stale sentence in `docs/ir.md`, corrected.** The illustrative-built-ins section cited the support matrix as recording "that no `Cast`, `Reindex`, or `Broadcast` key exists". Two thirds of that is now false — `tiler::reindex-f32@1` and `tiler::broadcast-f32@1` are registered — while `Cast`'s absence still holds, per [the matrix's own cast row](../docs/roadmap.md). The sentence now records the one absent key and the two registered ones, and keeps its point: a placeholder in that list is not a support claim in either direction.

**Inference — one stale sentence deliberately left alone, and why.** The L4 record's closing maturity paragraph reads "Nothing moved. Contraction stays at R1 … `Reindex` and `Broadcast` at R2". Against today's support matrix, contraction is R3 and the two structural families are R5. Rewriting the numbers would introduce an error rather than remove one: the paragraph is a claim about what *this rung* moved, and the rung did leave those families at R2 — later delivery tickets moved them. Softmax and RMS normalization, which the same sentence rungs at R2, have no support-matrix row at all (`grep -n "Softmax\|RMS" docs/roadmap.md` returns nothing), so no current-state rewrite of the sentence is even available. A dated-record staleness convention for maturity claims is a corpus-wide question, not a D-10 one; it is reported to the coordinator rather than absorbed here.

**Fact — the dispatch said this ticket was claimed as `worker-propagate-th`; it was not.** `tkt claims` listed fifteen claims and none for this ticket, and `tkt show` reported `status: todo`. The claim was established (`tkt claim --as worker-propagate-th`) and the status moved to `in-progress` before this branch's commit landed, so the board now says what the dispatch assumed it already said.

**Fact — verification.** `tkt lint` clean; `git diff --check` clean; `tkt guard --base 446f6fb tkt/propagate-the-d10-resolution-into-the-contract-corpus` → `verdict: ok`, five changed files, affected scopes exactly the declared four, and every reported collision confined to the shared `project/tickets` scope, which is what `shared_scopes` declares it for — the set of colliding tickets moves as other workers claim, so the invariant rather than the count is the evidence. `make full` green: fmt, check, clippy with warnings denied, 1,855 nextest tests passed with 5 skipped, the doc-tests, rustdoc under `-D warnings`, 636 release-profile tests in `tiler-reference` and `tiler-compiler`, `ticketsplease lint`, and shellcheck. Local link targets in the changed hunks were resolved from each source file's own directory — seven resolved, with a deliberate non-existent path reported `MISSING` and a non-existent heading counted `0` against the real `## Unresolved decisions` heading's `1`, so the check is known to be able to say no.
