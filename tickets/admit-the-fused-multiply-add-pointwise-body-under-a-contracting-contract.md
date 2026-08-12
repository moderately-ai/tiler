---
id: admit-the-fused-multiply-add-pointwise-body-under-a-contracting-contract
title: Admit a fused multiply-then-add pointwise body under a contraction-permitting contract
status: blocked
priority: p2
dependencies: [admit-multi-input-tensors-in-the-scheduled-region-vocabulary, admit-a-scheduled-region-that-reads-two-materialization-edges]
related: [admit-multi-input-elementwise-programs-at-the-compiler-boundary, prototype-inline-aot-integration-proof, derive-physical-proposals-from-the-cover-region-subject, represent-an-explicit-pointwise-contraction-choice]
scopes: [implementation/compiler, implementation/ir, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, numerics, architecture]
---
## Why this exists

`admit-multi-input-tensors-in-the-scheduled-region-vocabulary` made the approved inline region `sym n; in a, b, c; out (a * b) + c` compile end to end on the governed profile. It compiles under `StrictF32`, `FlushSubnormalsToZeroF32`, and `ReassociateF32`, and not under `RelaxedF32`.

**Measurement — this worktree, 2026-07-31, `nightly-2026-07-19`, against `TargetProfile::governed()`.** Three programs, four contracts:

| program | Strict | FlushSubnormals | Relaxed | Reassociate |
| --- | --- | --- | --- | --- |
| `(a * b) + c`, three inputs | compiles | compiles | `NoFeasiblePlan` | compiles |
| `(a * 2.0) + 3.0`, one input | compiles | compiles | `NoFeasiblePlan` | compiles |
| `(a * 2.0) * 3.0`, one input | compiles | compiles | compiles | compiles |

A fifth public f32 preset, `FLUSH_AND_REASSOCIATE_F32`, was already registered when this table was taken and behaves like the other non-`Relaxed` contracts for this matrix (mixed bodies compile). The permanent restatement in `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs` walks the full `CONTRACTS` surface.

**Inference — the refusal reads the multiply/add adjacency, not the input count.** `RelaxedF32` is the only registered contract that permits arithmetic contraction. A one-input program refuses identically to the three-input one, and the same one input with the same two constants multiplied twice compiles, so nothing about input cardinality participates.

**Fact — the refusal is a deliberate, measured decision and not a defect.** `derive_obligations` (`crates/tiler-compiler/src/fusion_legality.rs`, source anchor `fn derive_obligations`) discharges `FusionObligation::ArithmeticContraction` as a `SoundProof` only when `is_exact_governed_same_family_pointwise` holds — an add-only or multiply-only body, which provably has no multiply-plus-add pair to contract — and as a `NormativeGuarantee` when the contract forbids contraction. A body holding both families under a permitting contract falls to `unknown("unrealized-contraction")`, and multi-member candidates carrying that unknown are marked illegal so covers that include them are skipped before useful frontier selection.

`a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction` (`crates/tiler-compiler/src/fusion_legality.rs`, source anchor `fn a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction`) records why the obvious widening was **eliminated rather than deferred**, under `admit-a-reassociating-contract-without-contraction`: the authority is handed the program, the budgets, the contract, the capabilities, and the candidate, and none of them names the realization that will be emitted or the backend that will emit it; and under a permitting realization the claim that the emission performs no contraction is *false* rather than merely unprovable, because `tiler_metal::emit::realization_requirements` names `NoFloatingPointContraction` only in the forbidden arm, so the artifact carries no contraction obligation at all and the measured Apple row fuses a written multiply/add pair under `-ffp-contract=fast`.

**Fact — the fused multi-member candidate dies at fusion legality, and the singleton cover still has no usable pointwise implementation.** Cover enumeration for a connected pointwise program still retains the fused cover and the fully-materialized cover. Multi-member mixed regions under a contraction-permitting contract fall to `unknown("unrealized-contraction")` and are skipped. Frontier subjects for placed cover regions are keyed by the cover region's occurrence label (`region.label()`), with presentation role carried as explain fact `region-role` (singleton fragments of a multi-op `Pointwise` partition get role `"unrecognized"` and `admitted-count: 0` when nothing is admitted). After `derive-physical-proposals-from-the-cover-region-subject`, `GovernedPhysicalProvider::propose` no longer answers placed cover regions with silent empty offers: `govern_spelling` → `spell_region`, and one-operation fragments of a multi-op pointwise chain are declined as `UnspellableRegion` with rule `region-partial-coverage` because `Pointwise` spelling requires `members == normalized.members` (the whole recognized expression). Under `RelaxedF32` the same unrealized-contraction wall hits multi-member candidates of the recognized serial-sum program as well, so no complete plan survives there either; reassociating restores a usable path by forbidding contraction rather than by leaving a materialized fallback under the permitting contract.

**Correction — 2026-08-10.** Retired wording on this Fact claimed `GovernedPhysicalProvider::propose` "offers nothing unless the region's members are exactly the whole request's, so the singleton cover's one-operation regions reach no provider", named the frontier subject as `schedule:region:unrecognized`, and contrasted that "a serial-sum request under the same contract still has its materialized cover to fall back to." Those three claims are obsolete or inverted: propose declines with typed `region-partial-coverage` rather than silence; the subject key is the occurrence label with `region-role: unrecognized`; and under the contraction-permitting contract serial-sum multi-member candidates also die at unrealized-contraction.

## User-visible outcome

A caller stating `RelaxedF32` over a recognized pointwise body holding a multiply adjacent to an add gets a plan, or a typed refusal that names the contraction decision rather than an empty portfolio.

## Boundaries and what to watch

- The two positive routes are not equivalent and choosing between them is the work. **Realize the contraction**: give the physical pointwise vocabulary a form that *declares* whether it contracts — `ScalarProgram::FusedMultiplyAddSerialSum` already carries a `contraction: bool` field as a template shape for reductions, but the schedule verifier admits that form only when `!contraction`, so verified production regions never carry `true` and it is not a working permitting-declaration path today — and let the emitted body and the artifact's realization requirements carry that declaration. **Implement the materialized cover**: spell (and assemble) sub-expression or per-op pointwise regions so singleton fragments of a multi-op pure-pointwise cover are not declined as `RegionVocabularyWall::PartialCoverage` / `region-partial-coverage`. Per-cover-region propose already landed under `derive-physical-proposals-from-the-cover-region-subject`; the live wall is region vocabulary / sub-expression pointwise spelling and multi-region pure-pointwise cover assembly, not silent whole-request providers. The first changes what the compiler can express; the second changes what covers can realize as complete plans.
- Do not relax `ArithmeticContraction` to discharge on a permitting contract without new evidence. The elimination above is recorded with a measurement, and reopening it needs a measurement that contradicts it — not an argument that permission is not obligation.
- Whichever route lands, `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs` carries the executable statement of the current boundary and its `the_contraction_permitting_contract_declines_a_mixed_body_at_any_input_count` pair is what must flip. Flip both halves together, or the file stops proving the refusal was ever about the adjacency.
- A third outcome is legitimate and must be stated rather than assumed away: that a contraction-permitting contract *should* refuse this body until a declaring physical form exists, in which case the refusal wants a typed reason naming the contraction obligation instead of a bare `NoFeasiblePlan`, and the frontend's `compile_error!` question resolves in `crates/tiler-macros` delivery/aot paths (`delivery.rs` / `aot.rs`) against that typed reason — not in `region.rs`, which only documents that `RelaxedF32` declines mixed bodies downstream with the compiler's reason.

## Decision packet — 2026-08-09

This is an architecture fork, not an implementation-ready ticket. The two positive routes move different boundaries and neither is correctness-dominant without Tom choosing what capability the first vertical is meant to prove.

- **Option A — add a physical pointwise form that explicitly declares contraction (recommended).** It directly represents the permission the contract grants, keeps the current fused cover, and makes artifact/backend obligations inspectable. It adds a physical vocabulary and identity surface. Treat `FusedMultiplyAddSerialSum.contraction` only as a field-shape precedent, not as evidence that permitting contraction is already expressible on verified regions.
- **Option B — spell and assemble sub-expression pointwise regions (PartialCoverage remainder).** It preserves the current fused-form vocabulary and aims to give the planner a non-contracting fallback via the materialized cover, but the stage-8 silence wall is already discharged (`GovernedPhysicalProvider` answers every placed cover region). The live work is widening `spell_region` / normalization so one-op or sub-DAG pointwise fragments are not `PartialCoverage`, plus assembling multi-region pure-pointwise covers into complete plans.
- **Option C — retain refusal, but classify the unmet contraction realization by a typed reason.** This is smallest and honest, but intentionally leaves a public permissive contract unable to compile the mixed body.

Tom needs to select the intended capability boundary. No worker should weaken `ArithmeticContraction` or silently select one architecture under this node.

## Public-boundary acceptance — 2026-08-12

**Decision — accepted by Tom in the live coordination session.** The apparent A/B fork was false: the materialized realization is the complete fail-closed baseline, and an explicit contraction realization is a later costed optimization. They serve different obligations and both belong in the planning portfolio.

The first vertical is the materialized baseline. It must spell checked pointwise fragments from the exact cover membership, and every external leaf must bind either a declared input or one specifically identified cross-region materialization edge. A region that consumes two materializations therefore depends on the canonical edge-ordinal boundary owned by [`admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region`](admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region.md) and the distinct-edge assembly owned by [`admit-a-scheduled-region-that-reads-two-materialization-edges`](admit-a-scheduled-region-that-reads-two-materialization-edges.md). It must not infer an edge from access position, extent, or cover order. A fully materialized cover is an ordinary compile-time alternative under `RelaxedF32`, selected by the same verified feasibility and cost machinery as every other plan; it is not a silent runtime fallback.

The later optimization is [`represent-an-explicit-pointwise-contraction-choice`](represent-an-explicit-pointwise-contraction-choice.md). A boolean permission is insufficient because one pointwise DAG may contain several or overlapping multiply/add sites. The physical program must identify every contracted site explicitly and carry that choice through schedule verification, kernel lowering and verification, emission, reference/conformance evidence, realization witnesses, explain, and canonical identity. This is ADR 0015's *permission over an unfused body*, not the deferred semantic FMA family whose single rounding is program meaning.

The existing contraction-legality rule remains load-bearing. Until one of these complete realizations exists, `unrealized-contraction` remains `Unknown`; no provider or backend may infer a contraction choice from a permissive contract, compiler flag, or emitted `multiply + add` spelling. Existing subject bytes remain stable only where the new carriers are append-only and preserve every old arm verbatim; each owning implementation ticket must prove that at its encoder rather than assume a domain step or its absence.

**Graph consequence.** This ticket now owns the materialized baseline and is blocked on the distinct-edge carrier. The explicit contracted realization is a separate later ticket blocked on this baseline, so availability lands before the performance alternative and neither can silently substitute for the other.

## Closes when

A recognized pointwise body holding a multiply adjacent to an add compiles under `RelaxedF32` through a complete verified materialized cover whose fragments bind every declared input and materialization edge exactly; the same provider still refuses a mixed fused region under `unrealized-contraction`; `(a * b) + (c * d)` proves that two distinct materialization edges cannot alias; and the boundary test's contract pair is updated in the same change with the one-input control retained so the result stays evidence about the adjacency. The explicit contracted optimization remains the dependent ticket above and is not required to close this baseline.
