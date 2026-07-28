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

## Effect and aliasing contract landed (2026-07-27)

`crates/tiler-compiler/src/effects.rs`. An opaque call's body is not modelled, so every question an optimizer would answer by inspection — may this be reordered, fused, eliminated, run twice — has to come from a declaration.

Three independent axes, deliberately not collapsed: `Elimination` (removable when results are unused, or required), `Motion` (free to reorder and re-execute, or ordered against other effects), and `Aliasing` (results distinct from inputs, or may alias them). Motion and aliasing are separate because the questions are independent — a pure call can still return a view onto an input, and an ordered call can return storage aliasing nothing. Collapsing them would make one declaration answer a question it was never asked.

**Every axis's conservative value is the undeclared value**, and `CallEffects::unknown()` is all three at once. **There is deliberately no `Default` impl**: a `Default` reads as "the ordinary case", and for an opaque call the ordinary case is *not knowing*, which a caller should have to write down rather than receive by omission. That is this ticket's "may not smuggle unknown semantics or effects into logical IR" expressed as a type rather than a rule.

`meet` takes the conservative value **per axis**, for a region containing more than one opaque call — the region may only be optimized as far as its most restrictive member allows. Written as an explicit match per axis rather than a numeric minimum, so a third value on any axis is a build error here instead of silently ordering itself against the others. Tested that one undeclared call constrains a whole region, that the meet is symmetric so the result does not depend on member order, and that an axis both declarations agree on is *not* needlessly constrained — the last is what catches a `meet` that just returns `unknown()`.

## Failure stages landed (2026-07-27)

`crates/tiler-compiler/src/failure_stage.rs`. Seven stages in sequence order — applicability, preflight, validation, program construction, allocation, partial encoding, submission — and the thing that matters about a stage is which side of the commit point it falls on, not how severe it is.

**`fallback_permitted` encodes `AGENTS.md`'s rule directly:** "preflight before routing commit, fallback only before program work, and no fallback after allocation, partial encoding, submission, or semantic validation failure." It is a property of the stage, not a policy a caller may override, and there is no stage at which a caller may opt back in.

*Why the rule bites hardest here.* Once resources are allocated or a command buffer is partly encoded, a fallback must reason about what the abandoned attempt already did to device state — and that is precisely what nobody can do for an **opaque** call, since the compiler does not model its body and cannot know what it touched before failing. A fallback there is not a slower correct path; it is a guess.

*Written as an exhaustive match rather than a `<=` comparison*, so inserting a stage forces a decision about which side of the boundary it belongs on instead of inheriting an answer from where it happens to sit in the declaration order. The `Ord` derive makes declaration order meaningful, which is exactly why it must not silently decide this. A test pins that the declaration order *is* the sequence order, because a reordering that looked cosmetic would otherwise move the boundary; another checks the exhaustive match and the named `LAST_FALLBACK_STAGE` agree, since they are written independently and the match would silently win a disagreement.

**Only applicability is an ordinary failure.** A provider declining a target is not an error; everything else must be explained. Treating a preflight rejection as routine is how an infeasible plan becomes a silent one.

**Still not included:** the typed ABI and placement contracts; provider registration and applicability resolution; and the additive coexistence with scheduled kernels. Those remain the bulk of the ticket.
