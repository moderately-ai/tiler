---
id: implement-opaque-physical-call-providers
title: Implement opaque physical-call providers
status: todo
priority: p1
dependencies: [implement-analytical-component-cost-model]
related: [prototype-physical-implementation-frontier]
scopes: [implementation/compiler, implementation/ir, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, extensions]
---
After optimizer conformance and the mature boundary-property and analytical-cost
authorities, implement reviewed opaque physical-call registration and
verification. Cover typed ABI, effects, aliasing, placement, numerical
guarantees, target/applicability requirements, failure stages, provider
provenance, additive coexistence with scheduled kernels, and deterministic
rejection/explain behavior. Keep three typed evidence classes separate:

- exact or proven-upper-bound `ResourceRequirements` used for hard feasibility;
- uncertain `ResourceEstimate`-class pressure estimates with provenance and an
  explicit `Unknown` state, including registers, occupancy, and source size;
- an analytical cost estimate with exact model provenance and an explicit
  unavailable/`Unknown` state.

Unknown resource estimates cannot establish hard feasibility. Unknown cost
cannot silently become zero, infinity, or an arbitrary winner. Calibration
remains deferred to the separate measurement/activation ticket. Opaque calls
remain explicit physical boundaries and may not smuggle unknown semantics or
effects into logical IR.

Any consequential public or cross-crate crate, module, trait, type, or call-site
boundary remains a draft until Tom reviews and accepts the exact implementation
commit. This ticket does not preselect that interface.

## Released, and one of its three evidence classes already exists (2026-07-27)

Unblocked by `implement-analytical-component-cost-model` closing. Two notes so the first worker does not rebuild what is there or overestimate what is.

**The third evidence class is built.** This ticket asks for "an analytical cost estimate with exact model provenance and an explicit unavailable/`Unknown` state". `crate::component_cost` is exactly that: `CostValue` keeps `Exact`, `Bounded { low, high }`, and `Unknown` as separate classes rather than one confidence scalar, attributed to the governed `tiler.cost.analytical.v1`, with `unknown_is_not_a_zero` pinning the substitution this ticket forbids. Reuse it rather than defining a parallel vocabulary; a second cost vocabulary is the failure that `dominates` returning `false` across model keys already punishes elsewhere.

**But the cost authority is a framework, not mature.** Two of nine components are modelled — `Allocation` and `Dispatch`, both exact sums over values the plan already carries. The other seven are `Unknown`, and `model-the-eight-unmodelled-cost-components` owns them. If this ticket's opaque-call providers need a *populated* cost model rather than a well-typed one, that dependency is on the child, not on the closed parent. Check which before starting.

**What this ticket unblocks structurally.** `MaterializationForm::OpaqueRuntimeValue` is marked `Reserved` in `crate::boundary` and names this ticket as the owner of its typed ABI, effect, aliasing, and placement contracts. It is also one of the eight reserved values that currently make `implement-boundary-property-enforcers` unstartable — see that ticket's deferral note and the trigger test `frontier::tests::the_bounded_profile_admits_no_undischarged_boundary`. Admitting `OpaqueCall` proposals will change what the bounded profile can guarantee, so expect that trigger to be part of this work rather than a surprise from it.

