---
id: plan-the-materialized-attention-decomposition
title: Plan the materialized attention decomposition with its residency predicate
status: todo
priority: p1
dependencies: [assemble-the-causal-self-attention-block-program, realize-the-attention-contractions-on-metal, reclassify-language-model-work-as-a-conformance-track]
related: [design-attention-program-vertical, plan-the-recomputing-attention-decomposition, implement-general-dag-partitioning, implement-analytical-component-cost-model, implement-boundary-property-enforcers, integrate-the-attention-block-into-the-runtime]
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, fusion, feasibility, attention, language-model, class-generic-capability]
---
## User-visible outcome

The attention block gets a complete physical plan whose region cover, materializations, and boundary contracts are explained — and whose transient-memory requirement is a **hard feasibility predicate with an explainable reason** rather than a large number in a cost model.

## Evidence prerequisite

**Fact correction — the role-registration premise has been discharged.** `FusionNumericalCapabilities::governed` in `crates/tiler-compiler/src/fusion_legality.rs`, anchor `The table below is the complete set of families the governed provider declares a role for`, now registers the block's RMS normalization, softmax, structural reindex/broadcast/concatenate/slice, and strict tensor-contraction families in addition to the earlier source, elementwise, and ordered-reduction roles. The former four-role census and its inferred one-dispatch-per-operation baseline are false at this base. This ticket consumes those roles; it does not re-register them. A complete attention plan still does not exist: the Metal contraction realization is an unfinished dependency, and the block-specific cover, schedule, residency, and handoff feasibility decisions below remain this ticket's work. The difference between a role being registered and a whole plan being executable must stay visible in explain output.

**Fact — the transient requirement is `n · 16 · T · S · 4` bytes and `n` is a plan property.** From the [L4 design](../docs/research/program-planning/first-attention-program-vertical.md): `n = 4` with no fusion (`scores`, `scaled`, `masked`, `probs`); `n = 2` with the scale and mask fused as the contraction's epilogue; `n = 1` with a `StorageHandoff` additionally retiring the first tensor before the second is written.

| Row | `T = S` | One tensor | Unfused total | Epilogue-fused | With handoff |
| --- | --- | --- | --- | --- | --- |
| C1 prefill | 10 | 6,400 | 1,101,208 | 1,088,408 | 1,082,008 |
| B1-a prefill | 128 | 1,048,576 | 18,022,408 | 15,925,256 | 14,876,680 |
| B1-c prefill | 2,048 | 268,435,456 | 1,310,720,008 | 773,849,096 | 505,413,640 |
| B1-d prefill | 8,192 | 4,294,967,296 | **18,329,108,488** | 9,739,173,896 | 5,444,206,600 |

**Inference — L1's 4.00 GiB bound is this plan's best case, not its ordinary one**, and reaching it needs both the epilogue fusion and the allocation handoff. Neither exists.

**Fact — synchronization is dispatch ordering and nothing else.** [IR](../docs/ir.md)'s kernel verifier admits no barrier under the implemented zero-synchronization schedule profile. `crates/tiler-ir/src/program/model.rs` records the consequence at the split reduction: the pass boundary *is* the dispatch boundary, so an ordinary `Data` dependency is what makes a staged value visible.

## Required delivery

- **A complete cover of the block**, with the singleton coverage retained unconditionally and every larger candidate carrying a typed disposition. The whole block is inside the 32-member region budget and has three retained outputs inside the 8-output budget, so `EnumerateRegionCandidates` *will* propose it — its rejection is an explain event, not an absence. The reason is exact: the score contraction at `(g, r, t)` reads every key position, so a threadgroup owning one query tile would need a cross-threadgroup read of `k_rope`, which fusion legality forbids without an atomic or a multi-pass protocol.
- **The transient-residency feasibility predicate as a typed refusal**, carrying the exact byte requirement, the exact declared budget, and the plan's `n`. It is hard feasibility: an infeasible plan is rejected with a reason, never hidden behind an infinite or arbitrary cost. **Decision D-11 is that the budget does not exist yet** — no target profile in this repository declares a transient memory limit — so until one does, the verdict at B1-d is `Unknown`, and an `Unknown` feasibility verdict keeps a candidate in explain and search state only rather than in an executable frontier.
- **Boundary properties per handoff.** Every internal boundary in this block needs exactly *availability after producing dispatch* and *visibility readable on the requiring affinity*, both implemented and satisfiable. Record that the reserved values — availability after observed host completion, and visibility requiring an explicit coherence action — appear nowhere inside the program, and that the host readback of `h_out` is the separate boundary ADR 0033 governs. A plan that needed a reserved value would be refused rather than costed.
- **The materialization decision at `xn`, argued rather than defaulted.** It fans out to three contractions, and its producer is an RMS normalization over 1,024 contributors — a large reduction, which the contract names as usually not worth duplicating. Retain the multi-output alternative as a candidate blocked by [Q-PLAN-005](../docs/open-questions.md#q-plan-005--physical-multi-output-kernels) rather than dropping it.
- **The softmax's schedule set, bounded by the barrier rule.** One thread per row and a SIMD-group-cooperative row both survive; a threadgroup-cooperative reduction across several SIMD groups does not, and its rejection names the zero-synchronization schedule profile. A single-kernel softmax additionally carries two reductions in one schedule, which [Q-PLAN-004](../docs/open-questions.md#q-plan-004--coexisting-reductions-in-one-kernel) reserves, so the delivered form is a two-stage subprogram until that closes.
- **Explain output that distinguishes its refusals.** A missing fusion role, a budget stop, a target rejection, an `Unknown` feasibility verdict, and a dominance pruning are five different findings; none may render as "not fused". Every rejected candidate in the design's elimination table — the flash shape on distributivity, the opaque provider on its numerical guarantee, the whole-block region on the cross-threadgroup read, the rotary prologue on its duplication factor of `S` — must appear with its own reason.
- **Measured dispatch counts and per-stage times at the C1 prefill row and at least one B1 row**, so that [`plan-the-recomputing-attention-decomposition`](plan-the-recomputing-attention-decomposition.md) is compared against numbers rather than against an expectation.

## Non-goals

The recomputing decomposition, which is filed separately and must not be started before this one's numbers exist. The online single-pass form, which consumes distributivity and is rejected as a settled legality position. Any opaque provider. In-place execution, which [Q-PLAN-015](../docs/open-questions.md#q-plan-015--advanced-buffer-reuse-and-in-place-execution) defers — the allocation handoff above is reuse after last use, which is a different mechanism. Re-registering or redesigning the already-landed fusion roles.

## Closes when

The block has a checked complete plan at the C1 prefill row, the residency predicate refuses a deliberately undersized budget, every rejected candidate carries its own typed reason, and the plan's dispatch count and per-stage times are recorded at two rows.
