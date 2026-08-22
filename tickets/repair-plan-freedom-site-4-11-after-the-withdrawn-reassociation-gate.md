---
id: repair-plan-freedom-site-4-11-after-the-withdrawn-reassociation-gate
title: Repair plan-freedom site 4.11 after the withdrawn reassociation gate
status: in-progress
priority: p1
dependencies: []
related: [realize-the-tiled-contraction-schedule-and-its-metal-emission, re-enumerate-the-plan-freedom-sites-over-the-widened-topology-vocabulary, derive-staged-combine-structure-from-program-scope]
scopes: [research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift, scheduling, numerics]
claimed_from: todo
assignee: worker-site411
lease_expires_at: 1787430750
---
## User-visible outcome

`docs/research/reference/plan-freedom-sites.md` site 4.11 describes the gate the tree actually has, so a reader deriving numerical obligations from that record is not told a permission is required where none is.

## Why this exists

Found 2026-08-22 by the post-chain multi-lens audit and verified first-hand by the coordinator at `2cc3aefa`.

**Fact — site 4.11 asserts a gate that was withdrawn from the source the same day.** The record reads, verbatim, that `verify_cooperative_contraction` *"refuses the region unless the permission holds — the gate is the trailing `|| !*permits_reassociation` disjunct — so the topology is admitted only under a permitting contract, exactly as the variant's own doc claims."* That disjunct no longer exists. The tiled-contraction landing (`b3c07259`) removed it, on the ground that `ReductionTopology::CooperativeContraction` tiles the *memory* schedule and folds each output's contributors in ascending contracted order through a carried accumulator — so it consumes no reassociation, and requiring the permission refused a strict realization measured byte-identical to the direct fold. The variant's own doc no longer makes the claim the record cites it as making; it now carries a dated withdrawal.

**Fact — this is the load-bearing half of the site, not a detail.** 4.11's classification turns on *"Does the plan record the choice?"*, answered "Yes, and unlike 4.10 this one is a real spend." That answer rested on the withdrawn gate. Whether 4.11 is still a witness at all, and whether it still belongs in the unevaluable bucket, has to be re-derived rather than patched.

**Fact — a second citation in the same site is imprecise in the same direction.** 4.11 cites the witness refusal by the anchor `A kernel declaring workgroup staging combines inside the workgroup`. The staged-combine derivability spike found that prose sits in the doc comment of the `TopologyUnsupported` variant, several lines above the refusal, and that the actual predicate is `staging().len() != 0` — it refuses **any** workgroup staging, including staging carrying no combine structure. The record's description is narrower than the code.

**Fact audit — 2026-08-22 by `worker-site411` at `0e28564a`, re-read at source rather than inherited.** Fact 1 **verified**: `grep -rn '!\*permits_reassociation' crates/` returns two lines, both the conditional `|| (family.consumes_reassociation && !*permits_reassociation)` form in `crates/tiler-ir/src/schedule/builder/reduction.rs`, and neither in `verify_cooperative_contraction`, which carries only the sibling equality cross-check (`crates/tiler-ir/src/schedule/builder/contraction.rs "recorded and cross-checked against the region"`); the variant's dated withdrawal is at `crates/tiler-ir/src/schedule/model.rs "wording here said the tiling"`. Fact 2 **verified**, and the re-derivation confirms its worry was the right one: the class does move, to Witness (empty spend population), reserved. Fact 3 is **false in both of its halves** and is corrected below.

**Correction to Fact 3 — 2026-08-22 by `worker-site411` at `0e28564a`. The retired wording is preserved above; two of its claims do not hold.** *First*, the anchor does not sit "in the doc comment of the `TopologyUnsupported` variant, several lines above the refusal". `grep -n "A kernel declaring workgroup staging combines inside the workgroup" crates/tiler-ir/src/program/contraction_witness.rs` returns one line, a plain `//` comment **two lines directly above** the refusal it introduces — the predicate is on the next statement. That variant's own doc comment reads only "The covering realization's exact binary combine tree cannot be derived from program scope"; the prose the sibling ticket [`narrow-the-contraction-witness-refusal-to-staging-it-cannot-read`](narrow-the-contraction-witness-refusal-to-staging-it-cannot-read.md) describes — "including any kernel that declares workgroup staging" — is different wording again, and lives in the **enum-level** doc of `ContractionF32PlanWitnessError` rather than in the variant's. Three distinct pieces of prose were conflated into one. *Second*, "the record's description is narrower than the code" is false of the record. `plan-freedom-sites.md` site 4.11 says the witness "refuses **any** occurrence whose covering kernel declares workgroup staging", which is exactly `covering.kernel().staging().len() != 0`, quantifier included. What is narrower than the code is the *source comment the record anchors on*, not the record's own sentence. The repair that survives this correction is therefore smaller than the ticket states and is still worth making: name the predicate beside the prose anchor, so a reader who follows the citation reads the test rather than its rationale. That is what landed.

## Required work

- Re-audit all three Facts at your base and report a per-Fact verdict. Grep each quoted anchor against the file it names before relying on it; note that the *rendered* form of a wrapped `///` sentence greps to zero, which reads as removal.
- Re-derive site 4.11's classification against the current source rather than editing the sentence that named the gate. **If the withdrawal changes its bucket, the reconciled headline count and the bucket split in the same document must move with it** — the record states the split explicitly, and a repaired site with a stale split is worse than the drift it replaced.
- Repair the staging citation to name the real predicate, and say what the predicate actually covers.
- **Preserve the retired wording in a dated correction rather than deleting it**, per this repository's convention. Expect the document's grep counts not to shrink as a result; a shrinking count here is a false progress signal.
- Check the sibling sites for the same dependency. 4.1 through 4.4 are named in the same record as turning on the `permits_reassociation`/`permits_permutation` field pair, and one earlier note in this very document already records that the field pair is *the candidate test, not the classification*. Report both findings and clean results.

## Non-goals

Changing the withdrawal itself, which landed with its own evidence and two dated ADR status corrections; editing `crates/`; and re-running the whole enumeration, which is [`re-enumerate-the-plan-freedom-sites-over-the-widened-topology-vocabulary`](re-enumerate-the-plan-freedom-sites-over-the-widened-topology-vocabulary.md)'s subject.


## Sibling scan — 2026-08-22 by `worker-site411` at `0e28564a`

**Clean: no sibling site carries 4.11's defect.** The record never claims an unconditional gate for 4.1 through 4.4, so there is nothing of 4.11's kind to repair at them. Read at source: `crates/tiler-ir/src/schedule/builder/reduction.rs "fn verify_serial_semantics("` (4.1) cross-checks both permissions for *equality* against the region's realization and stops there; the multi-pass (4.2) and cooperative-workgroup (4.3) admissions add the **conditional** `|| (family.consumes_reassociation && !*permits_reassociation)`, which is ADR 0014's rule rather than 4.11's retired unconditional one; and `verify_contraction` (4.4) is equality-only, matching the record's existing statement that 4.4's spend population is empty. The record's own Part 3.3 census still holds at this base: `grep -rn '!= numerical.permits_reassociation()' crates/` and the `permits_permutation` form each return **six** lines, three in `reduction.rs` and three in `contraction.rs`.

**Both findings the ticket predicted are confirmed and are what decided the re-derivation.** The document already records, twice, that the `permits_reassociation` / `permits_permutation` field pair is the candidate test and not the classification — in Part 1's discharge note and in Part 2's vocabulary re-enumeration. Applying that stated rule to `CooperativeContraction` at this base is what moves 4.11, so the repair needed no new doctrine. **Both of those sentences support the rule with the claim that the two new topology variants "do not land in one class", and that supporting claim is now false** — with 4.11 re-derived, `LiveContraction` and `CooperativeContraction` land in the *same* class. Both are corrected in place: the framing is withdrawn, the rule survives on a larger population (4.4, 4.10, and 4.11 all carry the pair over an empty spend population, while 4.1–4.3 carry it over a real one).

**Out-of-scope drift found in passing, not repaired here.** `docs/research/reference/composed-realization-evaluation.md` (lines 26 and 198) and `docs/research/reference/permitted-divergence-oracle.md` (line 248) still state the twenty-four-site headline and the "six evaluable witnesses" bucket. That drift predates this repair and is not made newly wrong by it — both records frame the numbers as inherited from this enumeration at its own base — so it wants its own narrow ticket rather than an expansion of this one. The same count inside `plan-freedom-sites.md` itself (Part 2's closing Inference) *was* corrected here, because a reader meets it four paragraphs after the split this repair moves.

**`tickets/re-enumerate-the-plan-freedom-sites-over-the-widened-topology-vocabulary.md` is `status: done`** and restates 4.11's retired classification and split at lines 55 and 59. It is left standing as the record of what was derived at `f7a356de`, which is what the repository's terminal tickets are for; `make citations` skips terminal tickets, so nothing there is a live claim.

## Closes when

Site 4.11 describes the gate the tree has, its classification is re-derived rather than patched, any bucket or headline consequence moves with it, the staging citation names the real predicate, the retired wording is preserved in a dated correction, the sibling scan is reported with its clean results, and `make citations` is green.
