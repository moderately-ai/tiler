---
id: measure-executable-coverage-identity-growth-against-the-program-identity-bound
title: Measure executable-coverage identity growth against the program identity bound
status: done
priority: p3
dependencies: []
related: [bind-stage-coverage-to-index-refinement-identity, decide-whether-executable-coverage-evidence-folds-as-a-digest, add-the-identity-growth-experiment-rows-to-the-two-catalogs, widen-the-identity-growth-ladder-to-the-governed-operation-budget, rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [performance, measurement]
---
## User-visible outcome

The cost curve of proof-bound stage coverage is measured rather than extrapolated, so the point at which an ordinary compilation would hit `MAX_PROGRAM_IDENTITY_BYTES` is a known graph size with a recorded procedure — or the growth is shown to stay far from the bound across realistic programs.

## Why this exists

**Correction — 2026-08-10.** The filing-time mechanism below is historical. At this base, `encode_executable_coverage_identity` folds a fixed-width digest of the bound graph under `COVERAGE_GRAPH_DIGEST_DOMAIN` (`tiler.ir.index-refinement-coverage-graph.v1`) rather than embedding the full `SemanticGraphIdentity` per occurrence (`.digest(COVERAGE_GRAPH_DIGEST_DOMAIN, subject.graph.as_bytes())` in `crates/tiler-ir/src/index/refinement.rs`; ADR 0104 accepted and implemented 2026-08-06). Measured program-identity growth for the spike family is exactly linear: `program_bytes(n) = 3531n + 724` (residual 0 at all sixty-one points on the latest retained ladder). Rotten line citations in the filing text must not be followed; use the searchable anchors named below.

**Inference (structural) + Measurement (one point), recorded by the independent review of `bind-stage-coverage-to-index-refinement-identity` at `cd3119f5`.** `SemanticGraphIdentity` is a full canonical `Vec<u8>` encoding, not a digest (`pub struct SemanticGraphIdentity(Vec<u8>)` in `crates/tiler-ir/src/semantic/identity.rs`), and at filing `encode_executable_coverage_identity` (`fn encode_executable_coverage_identity` in `crates/tiler-ir/src/index/refinement.rs`) embedded one whole graph identity per covered occurrence, one record per operation. Program identity was therefore Θ(operations × graph-encoding size) — quadratic in graph size — against a hard `MAX_PROGRAM_IDENTITY_BYTES = 64 * 1024 * 1024` (`pub const MAX_PROGRAM_IDENTITY_BYTES` in `crates/tiler-ir/src/program/mod.rs`) that fails closed with a typed refusal (`KernelProgramDiagnostic::IdentityLimit`). The implementing worker measured a five-occurrence stage key at 21,366 bytes (~4 KB of evidence per occurrence) and ran no larger case.

The failure mode is fail-closed rather than silently wrong, which is why this is a measurement ticket rather than a defect: an over-large program refuses with a typed reason. What was unknown at filing was how far from realistic program sizes that refusal sits.

## What this ticket owes

- A bounded experiment under `spikes/program-planning/`: identity byte size as a function of operation count over generated programs of increasing size, with the exact generator, environment, and procedure retained, and the observed growth curve compared against the structural prediction (at filing, Θ(n²) under full-graph embedding; after ADR 0104, linear under the digest fold).
- The graph size at which `MAX_PROGRAM_IDENTITY_BYTES` refuses, measured or safely extrapolated with the extrapolation labelled as such.
- A recorded verdict: either the bound is unreachable for the program sizes the roadmap contemplates (state the margin), or a follow-up decision ticket for a digest form. **Discharged:** the spike retained both a margin and the follow-up; `decide-whether-executable-coverage-evidence-folds-as-a-digest` closed as ADR 0104 (accepted and implemented 2026-08-06), which replaced the per-record graph restatement with a fixed-width digest under `tiler.ir.index-refinement-coverage-graph.v1` and stepped the coverage tags to `v2`. The historical redundancy fact the digest decision rested on remains true in structure: program identity folds its one bound `SemanticGraphIdentity` separately and the builder proves every record names that same graph (`CoveredOccurrence.graph` is in-memory for the foreign-graph check); ADR 0104 is how the coverage projection stopped restating the full graph bytes per record.

## Closes when

The curve is measured and retained with its procedure, the refusal point is stated with its evidence class, and the verdict names either the margin or the follow-up decision ticket.

## Outcome

**Delivered 2026-08-05 at `5568bf19`.** Spike at `spikes/program-planning/identity-growth/` (frontmatter `ticket: "measure-executable-coverage-identity-growth-against-the-program-identity-bound"`), with generator, procedure, and retained results. Pre-fold exact fit was quadratic (`134n² + 3650n + …`); refusal-point **Extrapolation, labelled** under that encoding was n ≈ 695 against `MAX_PROGRAM_IDENTITY_BYTES`; margin against roadmap P2 (≥ 51 ops) was about ×125. Verdict named both the margin and a follow-up decision ticket for a digest form.

**Follow-up decision executed.** `decide-whether-executable-coverage-evidence-folds-as-a-digest` closed as [ADR 0104](../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) (`decision_status: accepted`, `implementation_status: implemented`), accepted and executed whole on 2026-08-06: `encode_executable_coverage_identity` writes a fixed-width digest of `subject.graph` under `tiler.ir.index-refinement-coverage-graph.v1` instead of the framed preimage; coverage tags step to `v2`.

**Current measurement (post-fold, re-baselined).** Latest retained ladder: `spikes/program-planning/identity-growth/results/2026-08-08-post-sourced-semantic-shape-apple-m4-max-macos27.0-26A5388g/growth.tsv`. Exact fit over 2..=62: `program_bytes(n) = 3531n + 724`, residual 0 at all sixty-one points, quadratic coefficient 0. Refusal point **Extrapolation, labelled**: n = 19,006 under the linear fit. Widest measured point: 62 ops / 219,646 B = 0.327% of 64 MiB. Margin against roadmap P2 at ≥ 51 ops is about ×371 in bytes. Catalog and ladder-widening continuation live under the related tickets named in frontmatter; this ticket's original measurement obligation is closed with no unsplit remainder.
