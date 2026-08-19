---
id: re-anchor-the-schedule-builder-line-citations
title: Re-anchor the schedule-builder line citations
status: done
priority: p2
dependencies: []
related: [split-the-schedule-builder-into-cohesive-submodules, keep-a-module-size-and-complexity-census-with-a-split-queue]
scopes: [contracts/decisions, research/reference, research/scheduling, research/documentation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, citations, maintainability]
---
## User-visible outcome

The fifteen pinned line-only citations naming `crates/tiler-ir/src/schedule/builder.rs:NNN` are re-anchored as quoted-fragment citations against the split `schedule/builder/` submodules, so `make citations` is green with the builder split merged and the citations survive future code motion instead of rotting silently.

## Why this exists — filed 2026-08-19 at the builder split's delivery

The split (`56d95195`, delivered, held for batch integration) deletes `builder.rs`, and `check-citations.sh` reports `has 0 lines` for every line-only citation naming it. The citing documents, recorded by the split worker (also durably in its ticket's delivery note) — each contains one or more pinned citations naming `crates/tiler-ir/src/schedule/builder.rs` plus a line number; the numbers after each document below are the cited *builder.rs* lines, not lines in the document: three accepted ADRs — `docs/decisions/0012-physical-reduction-topology.md` (builder line 391), `docs/decisions/0014-reassociation-vs-permutation.md` (831), `docs/decisions/0022-reduction-identities-and-initial-values.md` (403); `docs/research/reference/permitted-divergence-oracle.md` (416, 4767); `docs/research/reference/plan-freedom-sites.md` (620–632, 906, 909, 1029, 1516); `docs/research/scheduling/two-dimensional-cooperative-staging-relation.md` (1147, 1671, 4484); and two `docs/research/documentation/ticket-audit-2026-08-10/reports/` snapshots (1516; 664). Re-derive the exact population at the working base with `grep -rn "schedule/builder.rs" docs/` rather than trusting this transcription — this ticket's first filing mis-stated the pairs as document-line locations and the citation checker caught three past end-of-file, itself a demonstration of why the repair must read each citing claim rather than relocate numbers.

This is exactly the line-only rot AGENTS.md's anchor discipline exists to prevent — the citations passed at base only because the checker verifies a line exists, not what it says.

## Required work

- For each citation, read the citing document's claim, find where the cited content now lives under `schedule/builder/`, and replace the line-only citation with a quoted distinctive fragment (verified by grep against the named file before commit) — several of the old cited lines are non-distinctive (`}`, `|| write.mode != AccessMode::Write`), so mechanical relocation is wrong; the claim decides the anchor.
- The two dated audit-snapshot reports are historical records: follow the repository's convention for them (a dated correction note beside the stale citation, or the checker's documented skip form, whichever the snapshots' own convention uses — read a sibling snapshot repair first rather than inventing one).
- ADR edits here are citation maintenance only; no decision content moves.
- `make citations` green on the tree containing the builder split is the check; run it and quote the summary.

## Coordination

This repair only makes sense on a tree containing the builder split; the coordinator lands it in the same integration batch as (or immediately after) the split merge. Until then `main` stays green because `builder.rs` still exists there.

## Closes when

All fifteen citations resolve as quoted anchors (or documented historical-skip forms), `make citations` is green with the split merged, and no citation names the deleted path.

## Delivery note — 2026-08-19, over base `56d951953f0d4367e655f9dfb3dbec044811596c`

**The population was re-derived rather than taken from the transcription above, and the transcription is right.** `grep -rn "schedule/builder\.rs" docs/` returns 151 matching lines across 78 files, almost all of them bare-path mentions the checker deliberately does not check; the *failing* population is what `make citations` names, and it is exactly the fifteen pinned citations in the eight files this ticket lists. Two counts in the ticket body were confirmed the same way: three ADRs, two research records, one scheduling record, two audit snapshots.

**Fact — several of the fifteen had already rotted before the split, which is why none of them was relocated mechanically.** Read against the pre-split file at `9bcc2d86`, the cited lines were: `:391` = `}`, `:831` = `|| write.mode != AccessMode::Write`, `:403` = `ReductionTopology::CooperativeContraction { .. }`, `:1147` = `&mut contracted_covered,`, `:1671` = `if !empty_domain_is_satisfied(...)`, `:4484` = `builder.build().unwrap_err().diagnostics(),`. Not one of those lines carried the claim its citing sentence made. Every replacement anchor was derived from the citing claim and then verified against the file it names.

**The fifteen, and what each became.**

| Citing document | Retired pin | What the sentence claims | New anchor |
| --- | --- | --- | --- |
| [ADR 0012](../docs/decisions/0012-physical-reduction-topology.md) | `:391` | every topology's recorded permissions equal the declared realization | `builder/reduction.rs "*permits_reassociation != numerical.permits_reassociation()"` + the same in `builder/contraction.rs` |
| [ADR 0014](../docs/decisions/0014-reassociation-vs-permutation.md) | `:831` | a split is admitted only when reassociation is permitted | `builder/reduction.rs "family.consumes_reassociation && !*permits_reassociation"` |
| [ADR 0022](../docs/decisions/0022-reduction-identities-and-initial-values.md) | `:403` | a contracted space with no points is refused | `builder/contraction.rs "contracted space with no points has no result to commit"` |
| [Permitted divergence oracle](../docs/research/reference/permitted-divergence-oracle.md) | `:416` | the permission cross-check is code, not a doc claim | the two `permits_reassociation` anchors above |
| Same | `:4767` | `tiler-ir`'s fixtures construct and verify a `rounds: 2` tile | `builder/tests.rs "fn multi_round_tile_fixture() -> CooperativeTile {"` |
| [Plan freedom sites](../docs/research/reference/plan-freedom-sites.md) | `:906` | `permits_contraction`'s one non-definition read | `builder/elementwise.rs`, anchored on the `permits_contraction()` disjunct |
| Same | `:909` | `permits_signed_zero_elimination`'s read site | `builder/elementwise.rs`, anchored on the `permits_signed_zero_elimination()` disjunct |
| Same | `:620-632` | the structural verifier reading `input_count` and `is_valid` | `builder/elementwise.rs "pub(super) fn verify_pointwise_f32("` |
| Same | `:1516` | the accumulation-width rejection, at `61414b91` | de-pinned as a dated fact; live site named as `builder/reduction.rs "pub(super) fn verify_accumulation_width("` |
| Same | `:1029` | the `!contraction` gate for the FMA fold, at `61414b91` | de-pinned as a dated fact; live site named as `builder/family.rs "(!contraction).then_some(SplitFamily {"` |
| [Two-dimensional cooperative staging](../docs/research/scheduling/two-dimensional-cooperative-staging-relation.md) | `:1147` | the participant-count check and nothing else about the space | de-pinned to a fact about base `54833c9` |
| Same | `:4484` | the domain-separator test | de-pinned; successor named as `builder/tests.rs "fn the_elementary_dimension_step_moves_domain_and_payload()"` |
| Same | `:1671` | prose the `v5` step must move | de-pinned to a fact about base `54833c9` |
| [Witness-vocabulary audit snapshot](../docs/research/documentation/ticket-audit-2026-08-10/reports/implement-the-realization-witness-vocabulary/a88f715fd68c_c99ac54950f2.md) | `:1516`, `:1640` | a `Raw source anchor:` field recording line drift | de-pinned to a historical filename plus line numbers |
| [Two-arithmetic-types audit snapshot](../docs/research/documentation/ticket-audit-2026-08-10/reports/subject-the-numerical-realization-when-a-region-carries-two-arithmetic-types/072ecde48afb_c99ac54950f2.md) | `:664` | the verdict's own subject, an imprecise ticket citation | de-pinned to a historical filename plus line number |

Every anchor was checked with `grep -cF` against the file its citation names before the commit, and each returned at least one.

**The snapshots follow the convention their own directory already set**, not a new one. [`repair-the-ticket-audit-report-citation-and-link-breakage`](repair-the-ticket-audit-report-citation-and-link-breakage.md) restated failing report citations "as a historical filename plus line range" in `4887cd3f`, changing only citation syntax and provenance and never the audit finding. Both snapshots here were repaired with a single-line edit each, at the one code span the checker failed; their prose, verdict labels, and other `builder.rs:NNNN` mentions are untouched.

**Six claims turned out stale in substance, and each carries a dated correction rather than a rewrite.**

1. **ADR 0012 — the permission cross-check is at six sites, not the five the pins named**, and *"the two reassociating strategies"* is now three: `builder/contraction.rs "|| !*permits_reassociation"` requires the permission outright for the cooperative contraction, a topology that postdates the section.
2. **ADR 0014 — the same third strategy**, recorded as obeying the rule the section already states rather than as a new one.
3. **ADR 0022 — the five separate `+0.0` comparisons are one shared authority**, `empty_domain_is_satisfied`, which each fold gate calls. "At every admission site" survives; five copies of a constant do not.
4. **Permitted divergence oracle — the same six-versus-three undercount**, which [Plan freedom sites](../docs/research/reference/plan-freedom-sites.md) Part 6 had already recorded against this record before the split.
5. **Plan freedom sites — two reproductions no longer return what they claim.** `grep -rn "permits_contraction" crates/` returns **three** lines, not two: `crates/tiler-ir/src/schedule/witness.rs:401` reads it to classify a freedom site. `permits_signed_zero_elimination` is read at **two** sites, not "exactly one site repo-wide": the second is `builder/coverage.rs "if numerical.permits_signed_zero_elimination() {"`, which widens `f32` equality while deriving a padding identity's two-sided neutrality — a live consumer, so the heading holds only in the narrower sense the paragraph's own last sentence gives it. Neither conclusion is re-derived here; both counts are corrected. Part 7.5's corrections 2 and 3 also had their subjects consolidated: two width rejections became one authority called from three admissions, spelled `scalar_arithmetic_type` rather than `region_arithmetic_type`, and three `!contraction` gates became one `split_family` table row.
6. **Two-dimensional cooperative staging §5 is a spent plan.** The `v4` → `v5` step it enumerates landed under ADR 0097, and [the identity ledger](../docs/artifact-abi.md) now records `tiler.schedule.v7`. `STRICT_F32_REGION_IDENTITY_HEX_V3` is now `STRICT_F32_REGION_IDENTITY_HEX_V6`, and `the_round_step_moves_only_the_domain_separator` is now `the_elementary_dimension_step_moves_domain_and_payload`; the section's own **Proposal** — rebaseline the retained constant at each step so the comparison keeps proving exactly one step — was adopted and is restated in that constant's doc comment. The section's remaining `builder.rs:NNNN` references are left as facts about base `54833c9` rather than repointed, because repointing a spent plan would invent a claim about work already done.

**Perturbation — the new anchors were shown to fail, by breaking the source rather than the assertion.** Three independent subjects were perturbed in the worktree and reverted: `verify_accumulation_width` renamed, the contraction comment's `no result to commit` changed to `no result to publish`, and `multi_round_tile_fixture` renamed. `./check-citations.sh` went to exit 1 with four failures across four documents, each naming the anchor and the file — for example `FAIL docs/decisions/0012-physical-reduction-topology.md` / `anchor occurs nowhere in crates/tiler-ir/src/schedule/builder/contraction.rs`. The crate files were restored with `git checkout --` and the run returned to exit 0.

**Gates.** `make citations` → exit 0, `1171 pinned citation(s) resolved across 1545 live ticket/comment/document file(s)`, forms `1368 line-only, 121 anchor-only, 1 line+anchor` (99 anchor-only at base), `untracked 0`. `tkt lint --format json` → `"ok": true`. `git diff --check` and `git diff --cached --check` → no output. No crate file is in the commit, so no Cargo gate applies.

**Out of fence, reported not repaired.** `grep -rl "schedule/builder\.rs" docs/` returns **78** files at this commit. Six are the live documents repaired above, which now name the path only inside dated historical statements. Sixty are 2026-08-10 audit snapshots, whose mentions are records of what a past base contained. The remaining twelve are live and outside this ticket's fence: `docs/status.md`, `docs/roadmap.md`, ADRs 0074, 0097, and 0100, and the research records `transformer-nonlinear-normalization-and-reductions`, `flash-class-capability-set`, `cpu-vector-lane-tier`, `multi-round-two-level-reduction-composition`, `scheduled-region-model`, `subgroup-execution-tier`, and `two-level-subgroup-workgroup-reduction`. Every one is a bare path or an ambiguous `builder.rs:NNNN` suffix, which the checker deliberately does not resolve, so none of them fails a gate — and each would still send a reader to a file that no longer exists. Two line citations inside the documents repaired here were also found drifted but resolve and were left alone: `crates/tiler-ir/src/schedule/numerics.rs:292` and `:310` in Plan freedom sites are at `:319` and `:337`.
