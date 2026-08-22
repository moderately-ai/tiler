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

## Required work

- Re-audit all three Facts at your base and report a per-Fact verdict. Grep each quoted anchor against the file it names before relying on it; note that the *rendered* form of a wrapped `///` sentence greps to zero, which reads as removal.
- Re-derive site 4.11's classification against the current source rather than editing the sentence that named the gate. **If the withdrawal changes its bucket, the reconciled headline count and the bucket split in the same document must move with it** — the record states the split explicitly, and a repaired site with a stale split is worse than the drift it replaced.
- Repair the staging citation to name the real predicate, and say what the predicate actually covers.
- **Preserve the retired wording in a dated correction rather than deleting it**, per this repository's convention. Expect the document's grep counts not to shrink as a result; a shrinking count here is a false progress signal.
- Check the sibling sites for the same dependency. 4.1 through 4.4 are named in the same record as turning on the `permits_reassociation`/`permits_permutation` field pair, and one earlier note in this very document already records that the field pair is *the candidate test, not the classification*. Report both findings and clean results.

## Non-goals

Changing the withdrawal itself, which landed with its own evidence and two dated ADR status corrections; editing `crates/`; and re-running the whole enumeration, which is [`re-enumerate-the-plan-freedom-sites-over-the-widened-topology-vocabulary`](re-enumerate-the-plan-freedom-sites-over-the-widened-topology-vocabulary.md)'s subject.

## Closes when

Site 4.11 describes the gate the tree has, its classification is re-derived rather than patched, any bucket or headline consequence moves with it, the staging citation names the real predicate, the retired wording is preserved in a dated correction, the sibling scan is reported with its clean results, and `make citations` is green.
