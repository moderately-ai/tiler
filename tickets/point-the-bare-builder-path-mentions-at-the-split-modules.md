---
id: point-the-bare-builder-path-mentions-at-the-split-modules
title: Point the bare builder-path mentions at the split modules
status: in-progress
priority: p3
dependencies: []
related: [re-anchor-the-schedule-builder-line-citations, keep-a-module-size-and-complexity-census-with-a-split-queue]
scopes: [contracts/navigation, contracts/decisions, research/target-profiles, research/program-planning, research/scheduling, research/shapes, research/numerics, research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, maintainability]
claimed_from: todo
assignee: worker-bare-paths
lease_expires_at: 1787151418
---
## User-visible outcome

Live documents stop sending readers to the deleted `crates/tiler-ir/src/schedule/builder.rs`: each bare-path or ambiguous `builder.rs:NNNN` mention in a live document either names the split submodule that now holds the content or is de-pinned as a dated historical statement.

## Why this exists — filed 2026-08-19 from the re-anchoring lane's out-of-fence report

The pinned-citation population was repaired at the batch merge, but twelve live documents still mention the deleted path in forms the citation checker deliberately does not resolve (bare paths and prose suffixes): `docs/status.md`, `docs/roadmap.md`, ADRs 0074/0097/0100, and the research records `transformer-nonlinear-normalization-and-reductions`, `flash-class-capability-set`, `cpu-vector-lane-tier`, `multi-round-two-level-reduction-composition`, `scheduled-region-model`, `subgroup-execution-tier`, `two-level-subgroup-workgroup-reduction`. None fails a gate; each misleads a reader. Re-derive the population at the working base with `grep -rln "schedule/builder\.rs" docs/` filtered to live documents (dated snapshots stay historical).

## Required work

Per mention: read the surrounding claim; if it describes current code, point it at the owning `schedule/builder/` submodule (mod.rs, intrinsic, copy, contraction, elementwise, family, coverage, reduction, tile, proof, diagnostics, tests); if it is a dated statement about a past tree, leave the prose and add the smallest dated locator note only where a reader would otherwise search the wrong file. ADR edits are navigation maintenance only. Gates: `make citations`, `tkt lint`, `git diff --check`, `tkt guard`.

## Closes when

`grep -rln "schedule/builder\.rs" docs/` returns only dated historical records, and every live mention verified against the claim it carries.

## Fact audit at base `350a367e1672d7925c477f6b349af0662d8e4b1a`

**Fact 1 — "twelve live documents": false, the population is eighteen.** `grep -rln "schedule/builder\.rs" docs/` returns 78 files, 60 of them under the dated snapshot `docs/research/documentation/ticket-audit-2026-08-10/` which stays historical. The eighteen live ones are the twelve the section above names plus six it does not: ADRs [0012](../docs/decisions/0012-physical-reduction-topology.md), [0014](../docs/decisions/0014-reassociation-vs-permutation.md), and [0022](../docs/decisions/0022-reduction-identities-and-initial-values.md), and the research records [`permitted-divergence-oracle`](../docs/research/reference/permitted-divergence-oracle.md), [`plan-freedom-sites`](../docs/research/reference/plan-freedom-sites.md), and [`two-dimensional-cooperative-staging-relation`](../docs/research/scheduling/two-dimensional-cooperative-staging-relation.md). The undercount is benign: five of the six were already repaired by the re-anchoring lane and carry its dated notes, so only `plan-freedom-sites` added real work. The mention count is 30, not 18 — `plan-freedom-sites` and `two-dimensional-cooperative-staging-relation` carry six each, and ADR 0097 and `multi-round-two-level-reduction-composition` two each. Reproduce with `git grep -c "schedule/builder\.rs" 350a367e -- docs/ | grep -v ticket-audit`, whose per-file counts sum to 30.

**Fact 2 — the stated submodule list is incomplete.** `crates/tiler-ir/src/schedule/builder/` holds thirteen files, not the twelve the Required work names: `structural_relation_tests.rs` is a second `#[cfg(test)]` module beside `tests.rs`. No mention resolved to it.

**Fact 3 — "ADR edits are navigation maintenance only" held for 0097 and 0100 and could not hold for 0074.** ADR 0074's paragraph asserts both that `verify_access_and_semantics` is in `tiler-ir` (still true — `crates/tiler-ir/src/schedule/builder/reduction.rs "pub(super) fn verify_access_and_semantics("`) and that "it already carries a catch-all because it matches a three-way product" (false at this base: the function dispatches on `region.schedule.reduction` into three per-topology gates and has no `match` at all). The product match was last present at `f6da4c41`, the commit that wrote the paragraph. Repointing the path alone would have carried a false shape claim onto a live file, so the paragraph is left as a dated 2026-07-24 reading with a locator note beside it.

## Delivery note — 2026-08-19

Thirty live mentions across eighteen documents, dispositioned as follows; 9 + 6 + 15 accounts for all thirty.

**Repointed in place, claim re-read and verified unchanged (9 mentions, 9 documents).** [ADR 0097](../docs/decisions/0097-admit-a-two-dimensional-cooperative-staging-relation.md) ×2 — `verify_participant_space` to `builder/tile.rs` and the cooperative fixtures to `builder/tests.rs`, plus a bare `` `builder.rs` `` for `addressed_slots` in the same sentence that the population grep does not match, repointed to `builder/tile.rs` alongside; [ADR 0100](../docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md) — four named tests to `builder/tests.rs`, all four present; [`transformer-nonlinear-normalization-and-reductions`](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md), [`flash-class-capability-set`](../docs/research/program-planning/flash-class-capability-set.md), [`docs/status.md`](../docs/status.md), and [`docs/roadmap.md`](../docs/roadmap.md) — test and fixture citations to `builder/tests.rs`; [`scheduled-region-model`](../docs/research/scheduling/scheduled-region-model.md) — verification layers 1 and 2 to the `builder/` submodules through `builder/mod.rs`; [`subgroup-execution-tier`](../docs/research/scheduling/subgroup-execution-tier.md) — the empty-domain requirement to `builder/reduction.rs` through `builder/family.rs`.

**Left in place with a dated repair note, because something beyond the path drifted (6 mentions, 5 documents).** [ADR 0074](../docs/decisions/0074-use-explicit-public-api-conventions.md) — Fact 3 above; [`plan-freedom-sites`](../docs/research/reference/plan-freedom-sites.md) — the one paragraph the earlier pass missed, whose seven pin pairs are six live sites in `builder/reduction.rs` and `builder/contraction.rs`, with the seven-to-six move explicitly not attributed to the split (the count was already six at `46bf1319^`, and the older sites spelled the comparison `==` inside match guards); [`cpu-vector-lane-tier`](../docs/research/scheduling/cpu-vector-lane-tier.md) — "both split topologies consume reassociation as a hard admission condition" is now per family, not per topology, and the quoted source comment no longer matches; [`multi-round-two-level-reduction-composition`](../docs/research/scheduling/multi-round-two-level-reduction-composition.md) ×2 — `multi_pass_family` and `cooperative_family` were consolidated into one `split_family` at `eb0b7514` and the coverage arithmetic moved to `builder/coverage.rs`; [`two-level-subgroup-workgroup-reduction`](../docs/research/scheduling/two-level-subgroup-workgroup-reduction.md) — the three chained equalities now live in three different files and one no longer reads a `count` field.

**Left historical, untouched (15 mentions, 6 documents).** ADRs [0012](../docs/decisions/0012-physical-reduction-topology.md), [0014](../docs/decisions/0014-reassociation-vs-permutation.md), [0022](../docs/decisions/0022-reduction-identities-and-initial-values.md), [`permitted-divergence-oracle`](../docs/research/reference/permitted-divergence-oracle.md), and three of [`plan-freedom-sites`](../docs/research/reference/plan-freedom-sites.md)'s six mentions are the re-anchoring lane's own dated notes, where the deleted path is the subject of the rename sentence; two more are that record's corrections 2 and 3, already covered by that lane's note, which names their live sites. All six mentions in [`two-dimensional-cooperative-staging-relation`](../docs/research/scheduling/two-dimensional-cooperative-staging-relation.md) are declared facts about base `54833c9` under that record's own note, which states the pins are not repointed because repointing a spent plan would invent a claim.

**Out-of-fence drift found while reading, not repaired, needing tickets.** [ADR 0097](../docs/decisions/0097-admit-a-two-dimensional-cooperative-staging-relation.md)'s tested-guarantee paragraph names `the_staging_relation_step_moves_only_the_domain_separator` and `STRICT_F32_REGION_IDENTITY_HEX_V4`; `grep -rn 'the_staging_relation_step_moves_only_the_domain_separator\|STRICT_F32_REGION_IDENTITY_HEX_V4' crates/` returns nothing, because the schedule domain has stepped to `v7` with `STRICT_F32_REGION_IDENTITY_HEX_V6` as the retained comparison. [`cpu-vector-lane-tier`](../docs/research/scheduling/cpu-vector-lane-tier.md)'s two Facts either side of the repaired one are also false: `TailPolicy` now has `Exact` and `Predicated`, and identity padding is admitted through `ContributorCoverage::IdentityPadded` with derived two-sided neutrality. Both are recorded in the notes this ticket added rather than repaired, since neither is a builder-path mention.
