---
id: propagate-lowering-capability-wiring-into-contracts
title: Propagate the wired capability and refinement stage into governed contracts
status: done
priority: p1
dependencies: [wire-capability-and-refinement-into-compile-path]
related: []
scopes: [contracts/optimizer, contracts/numerics, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, optimizer, capability, milestone-0b]
---
`wire-capability-and-refinement-into-compile-path` was scoped to `implementation/compiler` and `implementation/ir`, so the governed contracts that describe the compile path were deliberately left untouched. They now under-describe it.

At least the following changed and must be represented:

- The compile path runs a lowering-capability resolution stage before cover enumeration. Resolution is unconditional and fails closed on a missing or contended capability; the artifact plan's lowering provenance is the set of providers that resolution returned, not a compile-time constant.
- Index-region refinement is attached per recognized occurrence as exhaustive finite evidence, and degrades to a typed `BudgetStop` at `ExplainStage::KernelRefinement` plus an `Unknown` assessment — never to a rejection — when `MAX_EXHAUSTIVE_PROOF_CELLS` cannot afford the emitted region's access proof.
- Refinement's scalar-authority conformance rule changed from equality to containment: a region must reach nothing beyond what its capability declared it may emit. `crates/tiler-compiler/src/legality.rs` records the reasoning; `docs/compiler/optimizer.md` and `docs/ir.md` may still state the equality form.
- The fused alternative is no longer gated by an installed fused-provider constant. Its availability is decided by fusion legality and target feasibility alone.
- `docs/correctness-and-testing.md` records that the optimizer conformance owner must drive an external operation through the ordinary capability and refinement path before the public compiler facade is accepted. `pipeline::conformance::an_externally_registered_lowering_provider_drives_the_compile_path` is that evidence; the record should cite it.

**Closing evidence.** `uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py` pass with the contracts updated, and no governed record still states the superseded equality rule or the constant-provider gate.

## Outcome

All five items landed across seven contracts. `docs/compiler/optimizer.md` gains the normative section; the other six state their own layer's consequence and delegate to it.

1. **Resolution is a stage, not a description.** The planning model gained `ResolveLoweringCapabilities` as stage 5 and renumbered the rest, with the placement argument stated: stages 5 and 6 run per recognized occurrence *before* the first cover, because grouping occurrences the installed authority cannot lower would enumerate plans nothing could realize. `installed_lowering_capabilities` is now a `CompilationRequest` field in `docs/architecture.md` and in the glossary's definition. Both failing dispositions are recorded as load-bearing rather than diagnostic taste: **absent** is a deferred capability (never extended to this occurrence), **contended** is a *disproved* checked predicate (two extensions contradict each other), and neither is resolved by a default provider or a priority order.
2. **Provenance is derived, not declared.** `docs/architecture.md` and the optimizer contract record that a plan's `lowering_providers` is re-derived from the request's own installed registry at build and at re-verification, so a receipt naming an authority the registry never resolved fails closed.
3. **The proof budget is a separate kind of budget, and that is the item with the widest blast radius.** It needed a distinction the corpus did not have: a *search* budget costs an alternative while complete coverage survives; a *proof* budget costs a proof, leaving one predicate open while the plan stands. The optimizer contract states it as a sixth thing that is deliberately not a failure class, `docs/compiler/fusion-and-scheduling.md` warns that the two must not be read as the same finding, `docs/ir.md` tells an index consumer to separate proof-resource diagnostics from refusals before deciding anything, and `docs/ir.md`'s `Unknown`-candidate rule was narrowed — it is about the *feasibility* verdict, which is a hard predicate over a target, and does not generalize to every `Unknown`. The explain section now also requires a budget stop never to stand alone, because a stop says nothing about its subject and a reader would otherwise infer a pass.
4. **Containment replaces equality**, in the optimizer contract with the worked reduction case (a `tiler.strict-serial-sum-f32` occurrence reaches no scalar operation with one contributor, the identity constant over an empty domain, and the add over many — three reached sets, one declaration), in `docs/ir.md` as the relation the evidence carries, and in the glossary and testing matrix. The recorded reason is that equality's extra content was a *completeness* claim about one occurrence rather than about the capability, and no correctness argument rested on it.
5. **The conformance evidence is cited** in `docs/correctness-and-testing.md`, along with the three fail-closed siblings, and the status line was updated.

**The maturity boundary is stated rather than elided, in three places.** An index/access provider written only against the public `capability` surface has driven a recognized occurrence end to end, and the artifact plan records it as the lowering authority — but *installing* such a registry from outside the crate is unreachable, because the request, its capability field, and the snapshot are all crate-private and `tiler-compiler` exports no compile entry point. Composing a registry and installing one are kept as different claims in the optimizer contract, `docs/operation-extensions.md`, and `docs/correctness-and-testing.md`, each pointing at `prototype-public-compiler-api` as what would close the second. `docs/correctness-and-testing.md` additionally records that ordered multi-output programs are a negative test rather than a covered row, so the gate is not read as closed.

Also corrected while in scope: `docs/compiler/optimizer.md` listed five deterministic budgets as though they were the compilation's; three more that bound downstream stages (`region_covers` 1,024, `region_cover_expansions` 100,000, `physical_plan_combinations` 4,096) were missing, and the forward-looking sixth was described as bounding an unimplemented stage rather than that stage's retention.

Verified by reading rather than inferred: `legality.rs:935` implements containment as `!declared.contains(reached)`; the three conformance tests exist at `pipeline.rs:4151`, `:4543`, and `:4670`, the last measuring `[70_000, 2]`; `a_refinement_the_proof_budget_cannot_afford_is_recorded_not_rejected` pins the budget-stop-plus-`Unknown` pair at 1,179,630 cells against a 1,048,576 limit; the three budget defaults are exact at `request.rs:133`–`135`; `ExplainStage::CapabilityResolution` and both rule keys exist; and no fused/materialized lowering-provider constant remains anywhere in `crates/`. A corpus grep for the superseded equality rule and the constant-provider gate returns only the two paragraphs that explicitly record their removal.

`uv run --locked python scripts/docs.py render` passes.
