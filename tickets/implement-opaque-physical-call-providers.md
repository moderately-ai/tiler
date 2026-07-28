---
id: implement-opaque-physical-call-providers
title: Implement opaque physical-call providers
status: done
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

**But the cost authority is a framework, not mature. Corrected 2026-07-28.** The count has moved: **seven of nine** components are modelled — `MemoryTraffic`, `Allocation`, `Dispatch`, `RedundantWork`, `Indexing`, `Synchronization`, and `ThreadgroupMemory`. `ResourcePressure` and `CompileTime` are `Unknown` (`component_cost.rs:567`), and the ninth component of the original nine, `ArtifactSize`, was removed rather than modelled — unstateable for every candidate but the winner — with `ThreadgroupMemory` split out of `ResourcePressure` to take its place in the vocabulary. `model-the-eight-unmodelled-cost-components` owns the remainder. **Two of the seven are structurally zero**, so read the count carefully before depending on it: `ThreadgroupMemory` is `Exact(0)` because the bounded profile's only requirements derivation states zero local memory unconditionally, and `RedundantWork` is `Exact(0)` because `verify_cover` rejects a double-covered operation unconditionally and the enumerator cannot build an overlapping cover. A consumer needing a *populated* cost model therefore gets five components with varying values, not seven. If this ticket's opaque-call providers need that rather than a well-typed one, the dependency is on the child, not on the closed parent. Check which before starting.

**What this ticket unblocks structurally.** `MaterializationForm::OpaqueRuntimeValue` is marked `Reserved` in `crate::boundary` and names this ticket as the owner of its typed ABI, effect, aliasing, and placement contracts. It is also one of the eight reserved values that currently make `implement-boundary-property-enforcers` unstartable — see that ticket's deferral note and the trigger test `frontier::tests::the_bounded_profile_admits_no_undischarged_boundary`. Admitting `OpaqueCall` proposals will change what the bounded profile can guarantee, so expect that trigger to be part of this work rather than a surprise from it. **This prediction did not hold — see "Two delivered modules have no consumer" at the end of this ticket for the measured outcome.**

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

**Appended 2026-07-28 — nothing constructs or reads a `ResourceEstimate`.** The exact check: `grep -rn "ResourceEstimate\|PressureDimension\|EstimateProvenance" --include="*.rs" crates/ | grep -v "src/estimate.rs"` returns **one** line, and it is a doc-link at `crates/tiler-compiler/src/call_declaration.rs:169`, not a use. The opaque-call seam this module was built for carries a proven `ResourceRequirements` instead — the derivation is at `integrate-opaque-calls-into-the-physical-frontier.md`, in the three-way elimination under "Attempting the field swap", where "return the estimate instead" was rejected because routing around the absent conversion would defeat the type-level guarantee. That is the right outcome and it leaves this module with no caller, which is a different state from the one the section above describes and should not be read as a defect in either.

**The same holds for `crate::failure_stage`.** `grep -rn "CallFailureStage" --include="*.rs" crates/ | grep -v "src/failure_stage.rs"` returns nothing. Meanwhile `tiler_ir::program::RoutingCommitTransition::fallback_permitted` (`crates/tiler-ir/src/program/model.rs:277`, enforced at `crates/tiler-ir/src/program/builder.rs:283`) is the **live** authority over the same `AGENTS.md` rule this module encodes, and there is no link between the two. Two authorities over one rule with nothing relating them is the duplication `AGENTS.md` names; whoever wires the run path should decide which one governs rather than letting both stand.

**And in `crate::effects`, three accessors of seven have production consumers.** `declared()`, `elimination()`, and `aliasing()` are read at `call_declaration.rs:214`, `:222`, and `:371`. `unknown()`, `motion()`, `permits_more_than_unknown()`, and `meet()` have none outside tests. The doc-comment sweep at `588be6e` corrected the `#[allow(dead_code)]` reasons and rewrote `permits_more_than_unknown`'s doc to a capability statement, so most of the overstatement is gone; what remains is `meet`'s doc at `effects.rs:154-156` — "Used when a region contains more than one opaque call" — which is still present tense for something nothing does, and this ticket's own paragraph on `meet` above is likewise a description of intent rather than of a live path.

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

## Registration seam landed (2026-07-28)

`crates/tiler-compiler/src/call_registry.rs`. `OpaqueCallIdentity` (provider, call, output-affecting revision) keys an `OpaqueCallRegistry` of checked declarations.

**It is a separate type from `rewrite::RuleRegistry`, deliberately, despite identical shape.** `AGENTS.md` says two types with the same shape are not the same concept and names the construction site as the evidence. The sites differ in what registration *means*: a rewrite provider offers an optional transformation, so an entry that never fires costs nothing; an opaque call provider offers the only implementation of a call a program may already reference, so an entry that never fires is a program that cannot be built. Sharing one type would hide that, and the first person to add a "skip providers that propose nothing" convenience would apply it to both. The *reasoning* about duplicate refusal and canonical ordering is shared by reference rather than restated.

**Registration takes a checked `OpaqueCallDeclaration`, not its parts**, so an incoherent set cannot be registered — the coherence check is not something registration repeats or could skip.

**A revision change registers alongside its predecessor rather than being refused as a duplicate.** Two revisions must coexist: a program pinned to the old behaviour and one built against the new both have to resolve. This is where the opaque-call identity's revision behaves differently from a rewrite rule's, and it is tested, because the obvious duplicate check gets it wrong.

## Split, and closing on registration and verification (2026-07-28)

The ticket named "reviewed opaque physical-call **registration and verification**", then listed ten things to cover. Seven are delivered as checked, tested modules: uncertain pressure estimates, effects and aliasing, failure stages, the typed ABI, placement, cross-declaration coherence, and identity and registration. Applicability needed nothing — `frontier::TargetApplicability` already answers it.

The remaining three — additive coexistence with scheduled kernels, numerical guarantees, and explain records for rejections — are `integrate-opaque-calls-into-the-physical-frontier`, now live.

**The split is at a real boundary.** Every piece landed here was *additive*: new modules beside the frontier, with nothing existing modified. The remainder must change `frontier.rs` and the physical-planning path — starting by admitting `ProposalBody::OpaqueCall`, a variant the bounded frontier currently rejects explicitly. Different risk, different review surface, and a commit worth having on either side of.

**One consequence carried into the child rather than left to be discovered:** admitting `OpaqueCall` makes `MaterializationForm::OpaqueRuntimeValue` reachable, and that is one of eight `Reserved` values currently holding `implement-boundary-property-enforcers` closed. The trigger test `the_bounded_profile_admits_no_undischarged_boundary` is *expected* to fire during that work, and must not be repaired by widening the bounded property sets back into agreement — its firing is the signal that the enforcers ticket has become startable. **Measured after the child landed: this prediction was wrong. See below.**

Nothing in the child re-litigates what was decided here. The derivations for named-not-positional parameters, conservative-by-default effects, the absent estimate-to-requirement conversion, the fallback boundary, and why the call registry is a separate type from the rule registry are all recorded above and referenced rather than restated.

## Two delivered modules have no consumer (2026-07-28)

Two of the seven modules this ticket delivered are complete, checked, and tested, and nothing on the compile path reaches either. Recorded as a state to resolve deliberately, not as a defect: both are reasonable reserved seams, and the risk is that "delivered" reads as "wired".

Both checks are one line each and reproducible:

- `grep -rn "ResourceEstimate\|PressureDimension\|EstimateProvenance" --include="*.rs" crates/ | grep -v "src/estimate.rs"` → one line, a doc-link at `call_declaration.rs:169`.
- `grep -rn "CallFailureStage" --include="*.rs" crates/ | grep -v "src/failure_stage.rs"` → nothing.

**The child's fifth closing criterion is vacuous as a result.** `integrate-opaque-calls-into-the-physical-frontier` closes on "Unknown pressure estimates still cannot establish hard feasibility — the absence of a conversion from `ResourceEstimate` is preserved, not worked around at the integration point", and records it as met. It is met, but nothing could have failed it: no code outside `estimate.rs` names the type, so the absence of a conversion is preserved by the absence of any caller. The criterion pins a property no change to the integration point could have violated, and it should not be read as evidence the guarantee was exercised.

**Two further items are reachable only from their own module tests.** `CallPlacement::reaches` is called at `call_placement.rs:180` and `:182`, both in `mod tests`; `CallAbi::is_compatible_with` is called at `call_abi.rs:532-533`, `:553`, and `:563`, all in `mod tests`. Each is a real check with real derivations recorded above; neither is consulted by anything that compiles a program.

**The ask.** Either name the ticket that wires each of these four into the run path, or record them explicitly as reserved seams with the trigger that would consume them — `AGENTS.md` distinguishes a type-system reservation, an architectural seam, implemented support, and a tested guarantee, and these currently sit at "implemented and tested, consumed by nothing", which is none of the four as this ticket's prose describes them.

**Correction to the prediction carried into the child.** `MaterializationForm::OpaqueRuntimeValue` did **not** become reachable. `grep -rn "OpaqueRuntimeValue" --include="*.rs" crates/` returns only its declaration and encoding arms in `boundary.rs` (`:489`, `:498`, `:506`, `:519-522`) plus one test at `:2032`; nothing constructs it on a compile path. The variant that moved is `AliasView`: `guaranteed_properties_for` maps `Aliasing::Distinct` to `MaterializedBuffer` and `Aliasing::MayAliasInputs` to `AliasView` (`call_declaration.rs:371-374`), and there is no arm producing `OpaqueRuntimeValue` at all. So `the_bounded_profile_admits_no_undischarged_boundary` did not fire, and its continuing to pass is not evidence that the boundary question was settled by this work — the value it was watching for is still produced by nothing.
