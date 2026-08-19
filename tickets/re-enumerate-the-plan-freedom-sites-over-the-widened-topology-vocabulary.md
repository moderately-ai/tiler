---
id: re-enumerate-the-plan-freedom-sites-over-the-widened-topology-vocabulary
title: Re-enumerate the plan freedom sites over the widened topology vocabulary
status: in-progress
priority: p2
dependencies: []
related: [repair-the-research-records-the-key-replacement-and-splits-falsified]
scopes: [research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [research, documentation, plan-freedom, enumeration]
claimed_from: todo
assignee: worker-freedom
lease_expires_at: 1787164923
---
## User-visible outcome

`docs/research/reference/plan-freedom-sites.md` states a freedom-site count over the vocabulary that exists, not over the vocabulary it was written against — so a reader deriving what a plan may vary from it is not working from a floor presented as a total.

## Why this exists

Filed 2026-08-19 as the mapped remainder of `repair-the-research-records-the-key-replacement-and-splits-falsified`, which repaired the record's counts and deliberately stopped short of re-running the enumeration. Verified by the coordinator at `bba2f0e1` before filing.

**Fact — two topology variants are unclassified freedom-site candidates.** `ReductionTopology` (`crates/tiler-ir/src/schedule/model.rs`, anchor `pub enum ReductionTopology {`) has seven variants; `LiveContraction` and `CooperativeContraction` joined the five the record enumerated. Both carry `permits_reassociation` and `permits_permutation` fields of their own — the coordinator counted six such field pairs across the enum's variants at this base — and that is precisely the shape which makes sites 4.1 through 4.4 witnesses. Each new variant is therefore a candidate for a row the table does not have, on the precedent of the record's own correction 4, which admitted site 4.9 for the ninth `ScalarProgram` variant.

**Fact — the record already says this about itself, so this ticket completes a stated gap rather than discovering one.** Its `Vocabulary repair — 2026-08-19` note states "the enumeration is deliberately not re-run against those two, and the headline count should be read as a floor rather than as current", and "twenty-five is therefore this record's count over the vocabulary it enumerated, not over the vocabulary at this base".

**Fact — the record's conclusion does not currently depend on the count.** The note records that what the Fact's own conclusion needs is only that the set is closed, and it still is. So this is accuracy work on a live research record, not a correctness defect in a derivation.

**Do not "repair" Part 7.5's drift row.** It records `ReductionTopology` as "five variants unchanged", which was true at base `61414b91` and is left standing as accurate history about that base. Rewriting it would destroy a true historical statement — the repair lane made this call deliberately and it should not be reversed.

**Observed and not yet resolved, stated so it is not rediscovered as new:** the record's closing section says "twenty-four sites" (anchors `the twenty-four sites are exhaustive over the vocabulary` and `compare the twenty-four sites' fields pairwise`) while Part 7.5's correction 4 says "the headline is twenty-five sites over the current vocabulary". That disagreement predates the widened topology and is a separate inconsistency inside the record. Resolve it as part of this work, since re-enumerating produces the number both sections should state.

## Fact audit at `f7a356de` by `worker-freedom`, before any edit

- **Fact 1 (two unclassified topology variants) — verified, with one imprecision.** `crates/tiler-ir/src/schedule/model.rs "pub enum ReductionTopology {"` has seven variants: `None`, `Serial`, `MultiPass`, `Contraction`, `LiveContraction`, `CooperativeWorkgroup`, `CooperativeContraction`. Six `permits_reassociation` / `permits_permutation` field pairs across them, matching the coordinator's count. **Imprecise:** "that is precisely the shape which makes sites 4.1 through 4.4 witnesses" overstates what the field pair settles. It is the candidate *test*, not the classification — site 4.4 carries the same pair with an empty spend population and site 5.3 carries the same-named field as a mirror. The two new variants carry identical pairs and land in different classes, so a worker reading this Fact literally would have classified them together and been wrong about one. The ticket's method was right; only this clause's confidence was.
- **Fact 2 (the record says this about itself) — verified, and both quoted anchors are defective as supplied.** The claims are in the record. But `the enumeration is deliberately not re-run against those two, and the headline count should be read as a floor rather than as current` returns 0 against `docs/research/reference/plan-freedom-sites.md`, because the source capitalizes the sentence-initial `The`; `The enumeration is deliberately not re-run against those two` returns 1. Likewise `the twenty-four sites are exhaustive over the vocabulary` in the Observed section returns 0 and `The twenty-four sites are exhaustive over the vocabulary` returns 1. Both failed as *absence*, the dangerous direction — a worker could have read them as text since removed. Second quoted fragment and `compare the twenty-four sites' fields pairwise` both resolve as written.
- **Fact 3 (the conclusion does not depend on the count) — verified.** `What the Fact's own conclusion needs is only that the set is closed, and it still is` returns 1, and the set is still closed at this base.
- **Observed disagreement — verified and resolved, and it was not the contradiction it looked like.** Twenty-four is the Part 2 table's own row count (the table has exactly 24 rows at this base); twenty-five is the count after Part 7.5's corrections 2 and 4. Both were accurate about their own subject. The genuine defect was that two *present-tense* statements — the refutation procedure and the closing bullet — kept the table's number where they needed the current one.

## Required work

- Re-audit every Fact above at your actual base before editing, per the stale-Facts rule, and re-derive the variant count by reading the enum body rather than trusting this ticket.
- Apply the record's own Part 1 classification rule to `LiveContraction` and `CooperativeContraction`, deriving each one's spend population, and either admit each as a numbered site with its class and field or state why it is not a witness. Follow correction 4's shape for site 4.9 — it is the worked precedent.
- Reconcile the headline count across every place the record states it, including the closing section and the refutation procedure, and state whether the number is now a total or still a floor. If it is still a floor, say what remains unenumerated.
- Where the reconciliation changes a bucket's membership, state the changed bucket rather than only the total — the record's value is the split, not the headline.
- Cite by searchable anchor, not line number, and run each anchor's grep against the file its citation names before writing it. The record's line pins have rotted once already.

## Non-goals

Any source change; `docs/decisions/**`; other research records; and re-litigating ADR 0112 or the topology vocabulary's widening. Do not rewrite Part 7.5's historical drift row.

## Outcome

Both variants classified by Part 1's rule at `f7a356de`, in `docs/research/reference/plan-freedom-sites.md`'s Part 2, following correction 4's shape.

- **Site 4.10 — `Reassociation` at `ReductionTopology::LiveContraction`'s contracted fold. Witness (empty spend population), reserved.** Field `live_access` + `live_axis` + `order`. `verify_live_contraction` cross-checks the permission fields but carries no `!*permits_reassociation` admission clause, so a strict contract admits the topology; the variant carries no partition, tile, or rounds, and `ContributorOrder` still has one variant, so no regrouping is representable. Site 4.4's class for site 4.4's reason.
- **Site 4.11 — `Reassociation` at `ReductionTopology::CooperativeContraction`'s tiled contracted fold. Witness, unevaluable, reserved.** Field `contracted_tile` + `tile.rounds`. `verify_cooperative_contraction` refuses unless the permission holds (`|| !*permits_reassociation`) and pins the regrouping exactly (`if tile.rounds != tile_count {`), so the plan records a real choice. Unevaluable by two named refusals: `ContractionF32PlanWitness::from_program` refuses a covering kernel that declares workgroup staging with `TopologyUnsupported`, and the strict route is closed because a realization permitting reassociation never reaches it.
- **Neither is reachable from the compiler**, so both are reserved: `grep -rn "ReductionTopology::" crates/tiler-compiler/src` shows no `CooperativeContraction` at all, and `LiveContraction` only under `measured_cost.rs`'s `#[cfg(test)]`.
- **Not new sites, checked:** the cooperative contraction's `accumulation` extends site 4.8's construct half under the same single `verify_accumulation_width` authority, class unchanged; `permits_permutation` and `order` leave sites 5.3 and 5.1 unchanged.

**Headline: twenty-seven, still a floor.** Split — evaluable witnesses **eight** (adds 4.10), unevaluable **four** (adds 4.11), mirrors five, undeclared four, unspendable six. Those two are the changed buckets. Reconciled against every present-tense statement of the count: the Outcome inference, Part 5's refutation procedure, and the closing bullet now name twenty-seven; the Part 2 table header and split keep twenty-four because they describe the table, which is unchanged. It stays a floor because `ContributorCoverage::IdentityPadded` (with its `ReductionPaddingIdentity`) and `TailPolicy::Predicated` entered outside the two construct enums this record enumerates and are unclassified — named in the record so the next re-enumeration has its subject.

Part 7.5's drift row recording `ReductionTopology` as "five variants unchanged" at `61414b91` was left untouched, as directed.

## Closes when

Both new topology variants are classified or explicitly excluded with the rule applied, the headline count agrees across every section that states it, whether it is a total or a floor is stated, changed buckets are named, and `make citations` is green.
