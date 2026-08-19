---
id: repair-the-stale-schedule-vocabulary-facts-in-two-live-records
title: Repair the stale schedule-vocabulary Facts in two live records
status: done
priority: p3
dependencies: []
related: [point-the-bare-builder-path-mentions-at-the-split-modules]
scopes: [contracts/decisions, research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift]
---
## User-visible outcome

ADR 0097 and `docs/research/scheduling/cpu-vector-lane-tier.md` stop asserting Facts the tree refutes: the ADR's named test and pin resolve again, and the lane-tier record's tail-policy and padding Facts state what the schedule vocabulary now admits.

## Why this exists — filed 2026-08-19 from the bare-path lane's out-of-fence report

Both finds verified by that lane with commands, recorded not repaired because they sat outside its navigation fence:

- **ADR 0097** names `the_staging_relation_step_moves_only_the_domain_separator` and `STRICT_F32_REGION_IDENTITY_HEX_V4`; both grep empty under `crates/`. The schedule domain has stepped to `v7`; the retained comparison is `STRICT_F32_REGION_IDENTITY_HEX_V6` and the one-step test is `the_elementary_dimension_step_moves_domain_and_payload()` (`crates/tiler-ir/src/schedule/builder/tests.rs`). Repair with a dated correction preserving the record's original claim as history.
- **`cpu-vector-lane-tier.md`** carries three stale Facts (the lane already added a dated flag naming them): the per-topology reassociation claim is now per-family (`family.consumes_reassociation`); `TailPolicy` now has `Exact` **and** `Predicated`; identity padding **is** admitted via `ContributorCoverage::IdentityPadded` with derived two-sided neutrality in `crates/tiler-ir/src/schedule/builder/coverage.rs`. Each needs a dated correction stating the current vocabulary with grep-verified anchors. `scheduled-region-model.md:496` repeats the one-variant `TailPolicy` claim — include it after re-reading its context.

## Fact audit at base `bda38064` — 2026-08-19 by worker-schedule-facts, before any edit

Every Fact above re-read at the working base. Two are wrong and the record of what the tree holds is wider than this ticket assumed.

- **The ADR 0097 bullet is stale: that repair already landed.** `grep -rn 'the_staging_relation_step_moves_only_the_domain_separator\|STRICT_F32_REGION_IDENTITY_HEX_V4' crates/` does return nothing, so the *finding* is right — but the record already carries it. `point-the-bare-builder-path-mentions-at-the-split-modules` repaired ADR 0097 at `c39ecb7b`, and the ADR's `docs/decisions/0097-admit-a-two-dimensional-cooperative-staging-relation.md "Navigation repair — 2026-08-19"` paragraph already names the empty grep, `STRICT_F32_REGION_IDENTITY_HEX_V6` as the retained comparison, and `the_elementary_dimension_step_moves_domain_and_payload()` as the one-step test, with the original claim preserved above it as history. All three of its anchors resolve at this base, as does its `builder/tile.rs "pub(super) fn verify_participant_space("`. The ADR was inside that lane's fence, not outside it. **No edit made** — a second dated correction restating a repaired claim in new words is the failure mode `AGENTS.md` names.
- **The schedule domain is `v7`**, not the `v6` a reader might infer from the retained pin name: `crates/tiler-ir/src/domains.rs "tiler.schedule.v7"`. `STRICT_F32_REGION_IDENTITY_HEX_V6` is the immediately preceding domain's pin, which is what makes the one-step comparison one step. The ADR's correction is consistent with this and needs no widening.
- **The `cpu-vector-lane-tier.md` bullet undercounts.** Its three named Facts are all genuinely stale and are now repaired. But the same census block carries three more that neither the reporting lane nor this ticket named: `ExecutionBinding` has three variants rather than one (`GlobalLinearInvocation`, `BlockedWorkgroup`, `FixedVectorMap`), `ReductionTopology` has six rather than five (`LiveContraction` joined), and `KernelType` has seven rather than five (`Bf16`, `U32`). The first is **this record's own first public-boundary item, landed**, which also falsifies the list preamble "Nothing below is implemented" for items 1, 2, and 4. All are repaired in one dated census note plus a second note beside the item list.
- **Two claims survived the audit and are left standing**, because a repair that rewrote them would be inventing drift: the `empty_identity_bits` Fact's substance (`+0.0` is still required at every admission — only its quoted sentence drifted from "a tile" to "a split"), and the whole `-0.0`-not-`+0.0` padding-value derivation, which `builder/coverage.rs "fn identity_is_two_sided_neutral("` reaches independently in source.
- **`scheduled-region-model.md` repeats more than the `TailPolicy` claim.** Its `ExecutionBinding` and `ReductionTopology` claims in the same paragraph, and the "Not implemented" paragraph's "The vector plan is absent entirely", drifted identically. Per this repair's fence only the `TailPolicy` claim is repaired; the rest is recorded in the same note and needs a narrow follow-up ticket. Both `model.rs` line numbers in that paragraph have rotted and are flagged rather than renumbered.

## Required work

Read each cited source in full at the working base before writing; dated corrections in each document's own convention, never silent restatement; every anchor grep-verified. Gates: `make citations`, `tkt lint`, `git diff --check`, `tkt guard`.

## Closes when

Both records' stale Facts carry dated corrections whose anchors resolve, and no named test or pin in either record greps empty against the tree.
