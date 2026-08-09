---
id: admit-the-fused-multiply-add-pointwise-body-under-a-contracting-contract
title: Admit a fused multiply-then-add pointwise body under a contraction-permitting contract
status: awaiting-decision
priority: p2
dependencies: [admit-multi-input-tensors-in-the-scheduled-region-vocabulary]
related: [admit-multi-input-elementwise-programs-at-the-compiler-boundary, prototype-inline-aot-integration-proof]
scopes: [implementation/compiler, implementation/ir, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, numerics, decision, needs-tom, architecture]
---
## Why this exists

`admit-multi-input-tensors-in-the-scheduled-region-vocabulary` made the approved inline region `sym n; in a, b, c; out (a * b) + c` compile end to end on the governed profile. It compiles under `StrictF32`, `FlushSubnormalsToZeroF32`, and `ReassociateF32`, and not under `RelaxedF32`.

**Measurement — this worktree, 2026-07-31, `nightly-2026-07-19`, against `TargetProfile::governed()`.** Three programs, four contracts:

| program | Strict | FlushSubnormals | Relaxed | Reassociate |
| --- | --- | --- | --- | --- |
| `(a * b) + c`, three inputs | compiles | compiles | `NoFeasiblePlan` | compiles |
| `(a * 2.0) + 3.0`, one input | compiles | compiles | `NoFeasiblePlan` | compiles |
| `(a * 2.0) * 3.0`, one input | compiles | compiles | compiles | compiles |

**Inference — the refusal reads the multiply/add adjacency, not the input count.** `RelaxedF32` is the only registered contract that permits arithmetic contraction. A one-input program refuses identically to the three-input one, and the same one input with the same two constants multiplied twice compiles, so nothing about input cardinality participates.

**Fact — the refusal is a deliberate, measured decision and not a defect.** `derive_obligations` (`crates/tiler-compiler/src/fusion_legality.rs`, source anchor `fn derive_obligations`) discharges `FusionObligation::ArithmeticContraction` as a `SoundProof` only when `is_exact_governed_same_family_pointwise` holds — an add-only or multiply-only body, which provably has no multiply-plus-add pair to contract — and as a `NormativeGuarantee` when the contract forbids contraction. A body holding both families under a permitting contract falls to `unknown("unrealized-contraction")`, and the whole-program candidate is deferred before the frontier enumerates any implementation for it.

`a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction` (`crates/tiler-compiler/src/fusion_legality.rs`, source anchor `fn a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction`) records why the obvious widening was **eliminated rather than deferred**, under `admit-a-reassociating-contract-without-contraction`: the authority is handed the program, the budgets, the contract, the capabilities, and the candidate, and none of them names the realization that will be emitted or the backend that will emit it; and under a permitting realization the claim that the emission performs no contraction is *false* rather than merely unprovable, because `tiler_metal::emit::realization_requirements` names `NoFloatingPointContraction` only in the forbidden arm, so the artifact carries no contraction obligation at all and the measured Apple row fuses a written multiply/add pair under `-ffp-contract=fast`.

**Fact — the materialized cover exists but has no implementation.** The explain trace for the refused compilation shows `cover.enumeration.v1` retaining two covers, and then `schedule:region:unrecognized` with `admitted-count: 0`. `GovernedPhysicalProvider::propose` (`crates/tiler-compiler/src/frontier.rs`, source anchors `struct GovernedPhysicalProvider` and `impl PhysicalImplementationProvider for GovernedPhysicalProvider`) offers nothing unless the region's members are exactly the whole request's, so the singleton cover's one-operation regions reach no provider. A contract that loses the fused candidate therefore loses every candidate, where a serial-sum request under the same contract still has its materialized cover to fall back to.

## User-visible outcome

A caller stating `RelaxedF32` over a recognized pointwise body holding a multiply adjacent to an add gets a plan, or a typed refusal that names the contraction decision rather than an empty portfolio.

## Boundaries and what to watch

- The two routes are not equivalent and choosing between them is the work. **Realize the contraction**: give the physical pointwise vocabulary a form that *declares* whether it contracts, as `ScalarProgram::FusedMultiplyAddSerialSum`'s `contraction` field already does for reductions, and let the emitted body and the artifact's realization requirements carry that declaration. **Implement the materialized cover**: normalize and propose per *region candidate* rather than per request, so a one-operation pointwise region has an implementation and the singleton cover composes. The first changes what the compiler can express; the second changes what it can choose between, and is the larger architectural move.
- Do not relax `ArithmeticContraction` to discharge on a permitting contract without new evidence. The elimination above is recorded with a measurement, and reopening it needs a measurement that contradicts it — not an argument that permission is not obligation.
- Whichever route lands, `crates/tiler-compiler/tests/multi_input_elementwise_boundary.rs` carries the executable statement of the current boundary and its `the_contraction_permitting_contract_declines_a_mixed_body_at_any_input_count` pair is what must flip. Flip both halves together, or the file stops proving the refusal was ever about the adjacency.
- A third outcome is legitimate and must be stated rather than assumed away: that a contraction-permitting contract *should* refuse this body until a declaring physical form exists, in which case the refusal wants a typed reason naming the contraction obligation instead of a bare `NoFeasiblePlan`, and the frontend's `compile_error!` question in `crates/tiler-macros/src/region.rs` resolves against that.

## Decision packet — 2026-08-09

This is an architecture fork, not an implementation-ready ticket. The two positive routes move different boundaries and neither is correctness-dominant without Tom choosing what capability the first vertical is meant to prove.

- **Option A — add a physical pointwise form that explicitly declares contraction (recommended).** It directly represents the permission the contract grants, keeps the current fused cover, and makes artifact/backend obligations inspectable. It adds a physical vocabulary and identity surface.
- **Option B — implement singleton materialized pointwise regions.** It preserves the current fused-form vocabulary and gives the planner a non-contracting fallback, but widens planning from whole-request providers to per-region implementations and is the larger architectural change.
- **Option C — retain refusal, but classify the unmet contraction realization by a typed reason.** This is smallest and honest, but intentionally leaves a public permissive contract unable to compile the mixed body.

Tom needs to select the intended capability boundary. No worker should weaken `ArithmeticContraction` or silently select one architecture under this node.

## Closes when

Either a recognized pointwise body holding a multiply adjacent to an add compiles under `RelaxedF32` to a complete verified plan whose contraction behaviour is declared rather than inherited, or the refusal is preserved deliberately and carries a typed reason naming the contraction obligation — and in both cases the boundary test's contract pair is updated in the same change, with the one-input control retained so the result stays evidence about the adjacency.
