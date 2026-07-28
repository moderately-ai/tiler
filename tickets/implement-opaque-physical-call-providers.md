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

## Typed ABI landed (2026-07-28)

`crates/tiler-compiler/src/call_abi.rs`. Parameters are **named with a typed role**, not positional.

*That is settled by a mistake already recorded in `AGENTS.md`*, in the section on eliminating options that do not survive: an artifact "binding buffers by slot position could not verify the position meant what it assumed", and the consequence named there is a silently wrong result rather than a trade-off. A positional ABI is checkable only for arity — two calls taking three buffers agree positionally whatever those buffers are for, so swapping an input for an output passes every check a position can support.

Slots still exist, because a binding table is ordered, but a slot is **derived from declaration order rather than supplied by the provider**, and nothing matches two parameters by comparing slots. `parameter(name)` is the only lookup; there is deliberately no lookup by slot, since a caller holding a slot and wanting a parameter would be reintroducing exactly what this prevents.

The tests are built around the distinction the positional form cannot make: the same names declared in either order **are** compatible (and their slots differ, asserted), while the same positions with swapped roles are **not**. A positional ABI would get both of those backwards.

`NoWrittenParameter` is refused at declaration: a call writing nothing produces nothing observable, and if that is genuinely intended it belongs in the effect declaration rather than in the absence of every output.

## Placement contract landed (2026-07-28)

`crates/tiler-compiler/src/call_placement.rs`. A call declares the affinity it runs on and the memory-domain classes it may address, and an undeclared placement is **refused rather than defaulted** — the same reasoning as the effect declaration: the compiler cannot see the call's body, so a placement it did not state is a placement nobody knows.

**It adds no new vocabulary.** `ExecutionAffinity` and `MemoryDomainClass` already exist in `crate::boundary`, built for the same ADR 0047 contract. A second set here would be two authorities over one concept, which is the failure `AGENTS.md` names directly — two types with the same shape are not the same concept, and matching one against the other produces a confident wrong conclusion.

`declare` takes the supported classes as an argument rather than reading a constant, so a widened profile needs no edit here and a test can drive the rejection path without a profile that permits it.

**Two things in the first draft overstated, and both were corrected before commit rather than shipped:**

- A `NoAdmittedDomain` error variant could never fire — `AdmittedMemoryDomains` already refuses an empty set at construction. An unreachable error variant reads as a check while being none, which is worse than its absence, so it was removed and the reason recorded where it was.
- A test doc-comment claimed both halves of `reaches` were covered. Only the domain half is: the bounded profile has one symbolic affinity, so no test can supply a second to fail the affinity half against, and a `reaches` ignoring its affinity argument entirely would still pass. Both the method and the test now say so, so a green run is not read as verifying the conjunction.

## Cross-declaration coherence landed (2026-07-28)

`crates/tiler-compiler/src/call_declaration.rs`. The ABI, effects, and placement each validate on their own; this checks the thing none of them can see — whether they **agree**.

**Applicability resolution is deliberately not here.** `frontier::TargetApplicability` already answers which providers apply to a target profile, over governed `TargetProfileKey`s with canonical deduplicated ordering. An opaque-call provider uses it rather than a second predicate over the same question — the same reuse decision the placement contract made for memory domains, and for the same reason.

**Two contradictions checked, each derived rather than invented:**

- *An `InOut` parameter beside `Aliasing::Distinct`.* An in-place parameter **is** a result occupying an input's storage, so `Distinct` beside one is not a stricter promise but a false one, and a caller trusting it would reuse storage the call overwrote.
- *A written parameter beside `Elimination::Removable`.* A call that writes storage the caller handed it is observable through that storage whether or not anything reads a returned value, so declaring it removable would let dead-result elimination discard a write the caller relies on.

**A contradiction is a defect, not a rejection.** A provider whose declarations disagree has not described a call this compiler cannot run — it has described no call at all, since no single behaviour is consistent with what it said. Same distinction `rewrite::ProviderDefect` draws; conflating them would let a caller counting infeasible candidates count broken providers among them.

**Every contradiction is reported, not the first**, mirroring `boundary::unsatisfied_properties`, so a provider author fixing one does not resubmit to discover the next. The test for that uses a declaration violating both rules and asserts two faults — a `check` returning early passes the two single-fault tests and fails only this one. A fourth test admits a consistent declaration, without which all three rejection tests would pass against a `check` that refused everything.

**Still not included:** the registration seam itself (its shape is already proven by `rewrite::RuleRegistry` — duplicate refusal, canonical iteration order) and the additive coexistence with scheduled kernels.
