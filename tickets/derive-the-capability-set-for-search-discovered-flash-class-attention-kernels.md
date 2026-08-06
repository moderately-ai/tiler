---
id: derive-the-capability-set-for-search-discovered-flash-class-attention-kernels
title: Derive the capability set for search-discovered flash-class attention kernels
status: review
priority: p2
dependencies: []
related: [decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode, calibrate-and-activate-parallel-reduction-selection, accept-adr-0100-multi-round-reduction-composition, derive-the-rewrite-rule-declaration-and-admission-shape-for-the-online-softmax-fold, derive-the-oracle-for-a-permitted-divergence-candidate, derive-the-rescaled-cross-round-accumulator-a-streaming-attention-schedule-carries, probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary, calibrate-device-cost-models, admit-a-fusion-role-for-the-tensor-contraction, admit-elementwise-epilogues-over-a-materialized-intermediate]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-flash-capability
lease_expires_at: 1785986516
---
## User-visible outcome

A research record that answers, capability by capability, what Tiler must grow before a caller who states a naive Llama-style attention program — `QKᵀ`, softmax, `×V`, written as registered semantic operations with no flash/sage/streaming vocabulary anywhere in the IR — can have the optimizer *discover* an implementation in the FlashAttention class, with the numerical delta stated as contract rather than taken silently. The record is a map from each required capability to an existing seam, an existing ticket, or a newly filed one — not an implementation, and not a promise of parity.

## Why this exists, and why now

**Fact — the architecture already decomposes the target.** FlashAttention is not one algorithm; it is (1) a materialization decision (never build the `S = QKᵀ` intermediate), (2) a streaming two-level reduction schedule, and (3) an algebraic rewrite of the softmax normalization (the shifted-max rescaling identity) that changes the reduction structure. Component 1 is the cover/materialization search the compiler already enumerates. Component 2's vocabulary largely landed this week: cooperative tiles, loop-carried staging, the two-dimensional staging relation ([ADR 0097](../docs/decisions/0097-admit-a-two-dimensional-cooperative-staging-relation.md), implemented), and the multi-round two-level composition ([ADR 0100](../docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md), accepted 2026-08-05). Component 3 has no home yet, and it is the crux.

**Fact — the compiler searches declared rewrites; it does not invent algebra.** [ADR 0099](../docs/decisions/0099-project-an-elementary-familys-per-point-body-from-one-shared-statement.md) made single-statement projection the standing rule, and the optimizer contract admits logical alternatives "only when the effective permissions authorize the regrouping". [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md) shows the discipline refusing a freedom nothing needed. So "inventing" flash attention decomposes honestly into: operations *declare* local algebraic identities once (softmax's normalization commutes with rescaling by the exponential of a shifted max; the fold admits an online form under stated permissions), and the global algorithm emerges from search over fusion, materialization, schedule, and those declared rewrites. The record must say exactly what the declaration vocabulary is, who owns it, and what proves a declared identity sound.

**Fact — the cost layer is the largest absence.** The 2026-08-05 research-status audit recorded it against source: no cost model, cost estimate, or ranking type exists anywhere in `crates/`; selection is a legality join. Search cannot prefer the flash-shaped candidate over the naive one without a cost authority, and the roadmap's bootstrap-cost-model record is where that thread currently ends. Reproduce with the audit's own grep in [the scheduled-region model's implementation-status section](../docs/research/scheduling/scheduled-region-model.md). **Corrected 2026-08-06 — the audit's absolute form is stale.** `PhysicalCostEstimate` (`crates/tiler-compiler/src/frontier.rs:345`) carries `dispatch_count`, `launched_threads`, and `temporary_bytes` under `COST_MODEL_KEY = "tiler.cost.structural.v1"` with a `dominates` partial order used to prune strictly dominated feasible proposals; reproduce with `grep -rn 'PhysicalCostEstimate' crates/tiler-compiler/src/frontier.rs`. The narrowed absence that stands: nothing establishes whole-program ranking, no calibration exists, and the model is `pub(crate)` local pruning rather than a selection authority — but `temporary_bytes` is exactly the axis a no-materialization candidate wins on, so the gap is a calibration-and-ranking gap, not a vocabulary one. Found by the distributivity reassessment packet, which cites it.

**Fact — the numerical boundary is the differentiator, not an obstacle.** Every SOTA attention kernel changes bits relative to the naive spelling. Under this repository's contracts that change is only reachable when the caller grants it, which converts "our kernel is fast" into "our kernel is fast *and* states what numerical freedom bought it". The softmax family already separates its max-fold and sum-fold legality facts; the record must derive what additional permission vocabulary an online-softmax rewrite consumes, and whether SageAttention-class quantized attention is expressible as a quantization contract with accuracy obligations on the existing `require_elementary_accuracy` shape.

## The derivation the record owes, by axis

- **Rewrite and permission vocabulary.** What typed, operation-owned declaration lets softmax (or the exponential) state the shifted-max rescaling identity and the streaming-fold form? What numerical permission does firing it consume, and how does the permission compose with [ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md)'s declined distributivity and the order-contract dimensions? What evidence class proves a declared identity sound (`SoundProof`, exhaustive finite, empirical) — and what refuses an unsound declaration?
- **Fusion and materialization search.** Does the current cover enumeration already contain the no-`S` candidate for the three-operation attention chain once the fusion roles exist, or does the epilogue wall (`admit-elementwise-epilogues-over-a-materialized-intermediate`) and the fusion-role gap (`admit-a-fusion-role-for-the-tensor-contraction`) bound it first? Map the exact tickets.
- **Schedule composition.** Which of the flash schedule's remaining constructs are missing after ADR 0100 — the tile-blocked write map and bijectivity proof (`admit-a-two-dimensional-cooperative-staging-relation`'s successors), the tiled contraction realization, symbolic extents for the growing context axis — and which existing deferrals carry them?
- **Cost authority and calibration.** What is the smallest cost model that ranks naive-versus-streaming credibly on Apple Silicon: analytic (bytes moved, occupancy), calibrated (measured per-construct costs, the M3/M4 measurement discipline), or learned? What does `calibrate-and-activate-parallel-reduction-selection` already own, and what would a flash-shaped decision additionally need? Where does simulation sit — is a memory-traffic simulator worth its maintenance against direct measurement on the two host rows we own?
- **Search strategy and budget.** At what candidate-space size does exhaustive enumeration stop and guided search start, and what does the deterministic-budget vocabulary already bound? What does explain owe so a rejected flash-shaped candidate is a readable ledger entry rather than silence?
- **Conformance oracle.** What does the reference layer owe so a rewritten (bit-different) implementation is checkable — an enclosure-based oracle for the permitted delta, per the certified-arithmetic machinery in `tiler-reference::accuracy`, or per-contract golden regeneration? The decoder-layer assembly's zero-differing-elements evidence is the naive baseline; what is the oracle for the *permitted-divergence* case?
- **Information the system lacks.** Target facts not yet in any profile (shared-memory bandwidth rows, simdgroup matrix capabilities, occupancy limits), and which ledger owns each.

## The expected outcome class, stated so nobody overdelivers

This is a graph-augmentation ticket, and that is its success condition, not its consolation prize. The likely deliverable is a record whose every axis ends in *filed, refined, or re-edged tickets* — new research questions, corrected dependencies on existing ones, deferrals with honest triggers — and possibly zero code, zero ADRs, and zero experiments run. A worker who maps all seven axes to well-edged tickets and defers every question that lacks evidence has completed this ticket; a worker who forces a premature ADR or experiment to satisfy the research-outcome discipline has misread it — a deferred question with a reconsideration trigger *is* one of the four sanctioned outcomes, and for a capability derivation this far ahead of the implementation frontier it is the expected one on most axes. Where an axis genuinely resolves to something stronger (an existing seam already suffices, or a bounded experiment is cheap and decisive), take it; nothing here forbids substance — the point is that substance is not owed.

## Explicit non-goals

Implementing any of it; adding any IR vocabulary; an attention-specific anything in the compiler (the record must show the general capability and treat attention as the worked example); performance claims (nothing here measures a kernel); reopening accepted numerical decisions — the record works within ADR 0095's decline and derives what *additional* permissions a caller would grant, not a relaxation of the defaults.

## Closes when

The record exists under `docs/research/program-planning/` with contract-conforming frontmatter and catalog rows, every required capability is mapped to an existing seam, an existing ticket, or a ticket filed by this work with correct edges (deferrals carrying trigger logs), the worked example walks the naive attention chain through each axis showing where today's tree refuses and what closes each refusal, the four maturity claims are kept apart per capability, and the record ends in the AGENTS.md research outcomes — each axis reaching a contract update, an accepted decision, a bounded experiment, or an explicitly deferred question with a reconsideration trigger, never an open-ended note.

## Outcome — 2026-08-06

The record is [`docs/research/program-planning/flash-class-capability-set.md`](../docs/research/program-planning/flash-class-capability-set.md). All seven axes are mapped, each ends in one of AGENTS.md's four outcomes, and the four maturity claims are stated per capability. **This is a graph-augmentation outcome exactly as the ticket's expected-outcome section frames it:** zero code, zero ADRs, zero experiments run, three tickets filed, two deferrals' trigger logs extended, and two premises of this ticket's own body corrected from source.

### Three findings that change what the next work is

**The cost layer is a smaller gap than this ticket states, and the 2026-08-06 correction only half-repaired it.** Four cost models exist, not one: `CoverCost` (`tiler.cost.partition-structural.v1`, `crates/tiler-compiler/src/cover.rs:581`), `PhysicalCostEstimate` and `PlanStructuralCost` (both `tiler.cost.structural.v1`), and `AnalyticalPlanCost` (`tiler.cost.analytical.v1`, nine governed components). More consequentially, `derive_cover_cost` (`cover.rs:2166`) charges `recomputed_elements` only for a member's *repeated* occurrences, and in the naive attention chain `S` and `P` each have exactly one consumer — so the whole-chain fused cover scores `(1, 0, 0, 0)` and **strictly dominates every materializing partition today**. The search does not need a new cost authority to prefer not materializing the score matrix; it needs one only to compare two implementations of that one cover.

**The crux is one named, ownerless prerequisite rather than a research area.** ADR 0095's 2026-08-06 reaffirmation states three prerequisites for its joint reopening condition. Two have owners (`expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate`, `todo`; `derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound`, live). The third — a rule in the certified-bounds admission shape — had none, and filing it is this work's most consequential graph edit.

**Axis 6 is worse than the ticket assumes, and it already bites.** `ReferenceNumericalConformance::from_realization` (`crates/tiler-reference/src/conformance.rs:166`) *refuses* any realization permitting contraction, reassociation, permutation, signed-zero elimination, or an exceptional-value absence assumption, rather than accepting one and ignoring it. The whole-program oracle is structurally bit-exact-only, and `NumericalContract::FLUSH_AND_REASSOCIATE_F32` (`crates/tiler-compiler/src/session.rs:1490`) is a registered contract a caller can state today. Nothing owned an oracle for a permitted-divergence candidate; that is why its ticket is `todo` and not `deferred`.

### Tickets filed

- [`derive-the-rewrite-rule-declaration-and-admission-shape-for-the-online-softmax-fold`](derive-the-rewrite-rule-declaration-and-admission-shape-for-the-online-softmax-fold.md) — `todo`, p2, `research/numerics`. The only ownerless prerequisite of ADR 0095's joint reopening condition; presumes neither decision's outcome, because it serves a continued decline (a checkable refusal) and an admission (the consuming rule) equally.
- [`derive-the-oracle-for-a-permitted-divergence-candidate`](derive-the-oracle-for-a-permitted-divergence-candidate.md) — `todo`, p2, `research/reference`. Its trigger has already fired: a reassociating contract is registered and reachable.
- [`derive-the-rescaled-cross-round-accumulator-a-streaming-attention-schedule-carries`](derive-the-rescaled-cross-round-accumulator-a-streaming-attention-schedule-carries.md) — `deferred`, p3, `research/scheduling`, with a trigger check log. ADR 0100 decision 4 discharges its three identity sites on the premise that "the outer fold and the round accumulator are the same binary operation as the inner combine"; a streaming attention output accumulator's `O ← O·r + P·V` breaks that premise, and nothing asked the question.

### Tickets updated

- [`probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary`](probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary.md) — one dated trigger-check-log line. The record's axis 5 supplies the five-rule set its stop condition (a) named as missing, and checks that stop condition (b) does not fire while naming R4 as the rule to watch. **The ticket is left `deferred`**: the rules are a research Proposal rather than a declared vocabulary in the tree, and whether that satisfies "declared" is the dispatcher's judgement, not this worker's.
- [`calibrate-device-cost-models`](calibrate-device-cost-models.md) — one dated trigger-check-log line. Trigger 2 acquires its first named firing condition (a flash-versus-naive pair is the first candidate pair for which `ResourcePressure` would not be constant) and does not fire, because no such pair is enumerable. The entry also records the four-cost-model fact and the simulator elimination so a future claimant does not re-derive either, and replaces a line number that has now drifted three times with a grep.

### Catalog row for the coordinator to apply

**Correction to the dispatch brief, checked rather than assumed: there is one catalog row, not two.** `docs/research/README.md` is the only navigation document carrying a per-record row; `docs/README.md`, `docs/status.md`, `docs/design-map.md`, and `docs/roadmap.md` carry none for research records. Reproduce: `grep -rn 'multi-round-two-level-reduction-composition\|elementary-identity-rewrite-dimension' docs/README.md docs/status.md docs/roadmap.md docs/design-map.md docs/research/README.md docs/open-questions.md` returns two lines, both in `docs/research/README.md`. `spikes/README.md` takes no row either, because this record retains no experiment.

The row belongs in the **Physical planning and lowering** group, immediately before `- [The CPU vector-lane tier]`, verbatim:

```text
- [The capability set for search-discovered flash-class attention](program-planning/flash-class-capability-set.md) — pending; primary-source-synthesis; informs: [Optimizer model](../compiler/optimizer.md), [Fusion and scheduling](../compiler/fusion-and-scheduling.md)
```

**Its three paths are `docs/research/README.md`-relative and therefore do not resolve from this file**, which is stated here rather than repointed, exactly as AGENTS.md requires of a verbatim-landable span: repointing would break the identity the transfer claim rests on. A link check run over this branch reports those three and nothing else — 95 of 98 local links resolve, and the three are the row's own.

### Recommended follow-ups the coordinator owns, not filed here

- [The rewrite-search formalism](../docs/research/region-search/rewrite-search-formalism.md)'s Part 0 already names this ticket as the derivation its tractability frame waits on; a reciprocal link from that record to the new one would close the loop, and `research/region-search` was outside this ticket's scopes.
- The two owning tickets on axis 2 (`admit-a-fusion-role-for-the-tensor-contraction`, `admit-elementwise-epilogues-over-a-materialized-intermediate`) should carry the cover-domination correction in their dispatch briefs: the flash-shaped cover is enumerated and preferred already, so their walls are the whole remaining distance rather than one obstacle among several. Neither ticket's stated outcome changes, which is why neither was edited.

### Checks

`tkt lint` clean. `git diff --check` clean. **No gate input was touched** — the branch changes only `docs/research/program-planning/` and `tickets/`, and nothing under `crates/`, `prototypes/`, `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, or `deps.sh`, so no cargo gate is owed under AGENTS.md's stated narrowing.
