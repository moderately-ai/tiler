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

## Started — the second evidence class landed (2026-07-27)

`crates/tiler-compiler/src/estimate.rs`. The ticket requires three evidence classes to stay separate; two now exist and this slice built the missing one.

| Class | Where it lives |
| --- | --- |
| exact / proven-upper-bound `ResourceRequirements`, used for hard feasibility | `tiler_ir::schedule`, already existed |
| **uncertain pressure estimates with provenance and an explicit `Unknown`, including registers, occupancy, and source size** | `crate::estimate`, this slice |
| analytical cost estimate with model provenance and its own `Unknown` | `crate::component_cost`, landed earlier this session |

**The invariant is a missing type, not a documented rule.** The ticket says an unknown resource estimate cannot establish hard feasibility. So there is deliberately **no conversion** from `ResourceEstimate` into `ResourceRequirements` or any feasibility input — not fallible, not documented-unsafe. A `TryFrom` would move the decision to each call site, and the failure mode is a caller who has an estimate, needs a requirement, and reaches for the conversion that exists. The absence is the enforcement. An estimate ranks and reports; a requirement decides.

**Provenance distinguishes otherwise identical estimates.** `ProviderAsserted` and `CompilerDerived` at the same number are different claims, and a calibration pass has to know which it is comparing a measurement against. Tested: two estimates with equal values and different provenance must not compare equal. `Measured` is reserved — no measurement path reaches here, and `calibrate-device-cost-models` owns that.

**Malformed values are refused at construction rather than reported as extreme estimates.** An occupancy above 100% is a provider fault; admitting it would rank a broken call above a working one. Registers and source bytes are unbounded counts and deliberately do not inherit occupancy's bound — tested, so a checker applying one rule to all three fails.

**`Unknown` is not zero**, and it keeps its provenance: "the provider was asked and does not know" and "nothing has asked yet" are different claims, and only the first says anything about the provider.

**Note for whoever takes `model-resource-pressure-from-a-register-and-occupancy-model`:** that ticket is deferred waiting for exactly this vocabulary. `PressureDimension::Registers` and `Occupancy` now exist. What is still missing there is a *target profile* declaring the axes — an estimate carrying a register count does not tell the cost model what the device's limit is. Check before assuming it is unblocked.

**Still not included:** the typed ABI, effect, aliasing, and placement contracts; provider registration and applicability; failure stages; and the additive coexistence with scheduled kernels. Those are the bulk of the ticket.

