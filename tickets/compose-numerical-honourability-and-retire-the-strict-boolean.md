---
id: compose-numerical-honourability-and-retire-the-strict-boolean
title: Compose per-dimension numerical honourability and retire the strict-f32 boolean
status: review
priority: p1
dependencies: [select-numerical-contract-and-compose-feasibility]
related: [declare-metal-numerical-honourability, draft-target-honourable-numerical-contract-adr]
scopes: [implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, feasibility]
claimed_from: todo
assignee: agent-numerics
lease_expires_at: 1784998939
---
The composition half of `select-numerical-contract-and-compose-feasibility`, split out when its Selection half landed. ADR 0076 items 2, 3, and 5.

A caller can now state one of two registered contracts and the compiler admits, plans, emits, and executes under either — measured end to end on an Apple M4 Max. What it cannot yet do is *assess honourability per dimension*, so the remaining work is real rather than cosmetic.

## What is missing

**The honourability authority.** Add it as a **peer** of `feasibility::CheckedTargetProfile`, not as new `CapabilityAxis` variants. The reason is decisive rather than aesthetic: `CapabilityAxis` is a quantitative space — a `u64` bound, a `Quantity` unit, an `AtMost`/`Exact`/`Implies` relation — and `SupportedWithExactEmulation` has no representation as a bound comparison. Emulation is honoured by *emitting different operations*, so it changes the program rather than the verdict; encoding it as a satisfied `Implies` would discard the one outcome that carries work.

**The composition into ADR 0043's outcomes.** Honoured exactly or by emulation is a satisfied hard predicate; honourable only under a relaxation the caller's contract does not authorize is a **disproved** predicate, not deferred and not unknown, because the caller's authorization is known at `CompileProfile` and cannot arrive later; declared unhonourable is disproved; and a dimension the profile does not speak to at all is `Unknown` in ADR 0043's exact sense. **Test that last clause directly** — it is the one most likely to be implemented as an accidental pass, and it is what makes an unenumerated dimension fail closed rather than defaulting to honoured.

**Retire the boolean.** `PrototypeTargetProfile::supports_strict_f32` and `CapabilityAxis::StrictF32Arithmetic` are replaced, not extended: a boolean cannot say which dimension failed and cannot express emulation. Fifteen sites reference them, counted at `a56bff8`. `physical::requires_strict_f32` is the interim summary that collapses four dimensions into one bit and should disappear with them.

**The caller preference list.** One resolved contract, or an explicitly ordered list resolved by the caller's stated order with the first honourable entry winning — deterministic, recorded, **never cost-ranked**. A single-entry list and a bare contract behave identically. ADR 0076 leaves list-versus-retry an open question and says the alternative was not rejected on evidence; if implementation gives evidence either way, record it and amend the ADR rather than silently choosing.

**The explain rejection shape.** A rejection must name the dimension, the required behaviour, the behaviour the target declares, the means the profile offers if any, and the declaring profile's versioned identity. `strict-f32: required 1, available 0` is the current state and is what this replaces.

## The honesty rule to enforce in code

No authority may narrow, weaken, or substitute the caller's stated contract to make a target feasible. When no stated contract is honourable, reject with a typed explainable error; never emit under a different contract, never fall back to a target default, never report the difference as a cost. The consequence: **the numerical contract is not a search dimension.** Cost-based selection ranks implementations of one contract and may never rank contracts against each other, because that prices meaning. The tempting mistake this forbids is treating a flush-tolerant plan as a cheaper alternative to a preserving one.

## Closes when

The authority exists as a peer, the four composition cases are implemented and each independently tested, the boolean and its axis are gone from all fifteen sites, explain carries the rejection shape, and `uv run --locked python scripts/check_repository.py` passes.

## Outcome

**Status: `review`, not `done`, and the reason is a single owner-reserved boundary.** Every closing condition above is met. What is not delivered is a *public* way to state the ordered preference list: `session::compile_governed` takes one contract, and widening it is a new `pub` item under ADR 0075 and therefore Tom's to review. `expose-the-numerical-contract-preference-list` owns that, and the two crate-internal items it would make reachable each carry a targeted `#[allow(dead_code)]` naming it. `record-adr-0076-honourability-implementation` owns recording this change in ADR 0076, which is in a scope this ticket did not hold.

### The authority, as a peer

`crates/tiler-compiler/src/honourability.rs` is new and owns the vocabulary: `NumericalDimension` over the four contract dimensions, `DimensionBehaviour` as the tagged union of the two behaviour spaces the dimensions range over, the four `HonouringMeans` `docs/numerical-semantics.md` already named, `RelaxationRequirement`, and a `NumericalHonourabilityFact` carrying the same availability phase, fact authority, validity scope, and declaring-profile provenance a `CapabilityFact` does.

It is a peer rather than a `CapabilityAxis` extension for the reason this ticket gave, and the code is arranged so the distinction cannot erode: the two vocabularies live in different modules, a numerical requirement has no `u64` and no `Quantity`, and the only thing they share is the outcome they compose into.

**One profile, two declarations, one identity.** `CheckedTargetProfile` holds both fact sets and encodes both into `canonical_descriptor`. Splitting the declaration into a second profile object was considered and rejected: it would mint a second identity that has to be kept in agreement with the first, which is the defect the descriptor's own doc comment warns about. The descriptor domain moved to `tiler.target-profile.descriptor.v3` because a `v2` descriptor could not distinguish two profiles declaring different honourable behaviours — exactly what the retired boolean could not say — and `the_canonical_profile_descriptor_separates_profiles_sharing_a_key` now asserts that three profiles differing only in one dimension's *means* have three descriptors.

The feasibility rule set key moved to `tiler.feasibility.phased-capability-and-numerical-honourability.v1`. The old key's own doc comment says a widened vocabulary mints a new key rather than bumping the revision, and this widened it in both directions: it added the numerical predicates and removed the `strict-f32` axis.

### The four composition cases, each independently tested

In `feasibility.rs`, all against `CheckedTargetProfile::assess`:

1. **Honoured exactly or by emulation ⇒ satisfied hard predicate.** `baseline_candidate_is_proven_with_canonical_resolved_predicates` covers exact support; `an_emulated_dimension_proves_and_retains_its_means` covers emulation and asserts the two are distinguishable in the proven evidence, because a verdict that lost the means would lose the emitted operations.
2. **Declared unhonourable ⇒ disproved.** `a_declared_unhonourable_dimension_rejects_with_the_full_shape` asserts all five parts of the rejection shape, including the behaviour the target *does* honour.
3. **Unauthorized relaxation ⇒ disproved, not deferred and not unknown.** `an_unauthorized_relaxation_is_disproved_and_authorizing_it_proves` runs one declaration against two proposals, so the same profile disproves and proves according to what the caller stated. That is the assertion that the authorization is read from the caller's contract rather than granted by the authority.
4. **Unenumerated ⇒ `Unknown`.** `an_unenumerated_dimension_is_unknown_and_never_honoured_by_default` tests three shapes of silence, because they fail closed for one reason and an implementation could get one right and another wrong: no declaration at all, three of four dimensions declared, and the dimension declared but not the behaviour required. The third is a finding this ticket did not anticipate — a profile can be partially silent about a dimension it has spoken about, and treating that as a refusal would assert knowledge the profile never supplied. All three are `Unknown`, and none is `Rejected`.

`a_later_phase_honourability_declaration_defers_then_resolves` covers the fifth case the ticket did not name: a declaration admissible only from a later phase defers, then proves once that phase is available. The governed profile never reaches it, but the phase machinery is shared with the capability axes and `Deferred` and `Unknown` are different claims.

**Rejection precedence, decided rather than inherited.** `Rejection::representative` returns a numerical cause before a capability one. A capability rejection says this plan does not fit and another plan might; an unhonourable dimension says the target cannot compute what the caller asked for, which no re-planning fixes, because the contract is not a search dimension. `a_numerical_cause_represents_a_rejection_that_is_also_capability_infeasible` pins it.

### The boolean and the axis are gone

`grep -rn "supports_strict_f32\|StrictF32Arithmetic\|requires_strict_f32" crates` now matches only two historical doc comments in `tiler-ir` describing the predicate that was replaced, and the retirement notes written by this change. No live code references any of the three.

`PrototypeTargetProfile::supports_strict_f32: bool` became `numerical: &'static [DeclaredBehaviour]`, keeping the profile `Copy` and keeping it in the request subject, where the single `u8::from(bool)` byte became a length-framed run of per-line declarations. `physical::requires_strict_f32` is deleted rather than relocated. `CapabilityAxis::StrictF32Arithmetic` is removed and its tag `0x06` is retired rather than reassigned, so a future widening cannot make a `v3` descriptor mean something a reader of the retirement would not expect.

**What the governed baseline declares, and what it deliberately does not.** It declares `Preserve` and `FlushToZero { PreservesSign }` on both subnormal dimensions and `Forbidden` and `Permitted` on both transform dimensions, each `SupportedExactly` — which is what admits both registered contracts. It declares nothing about `FlushToZero { AlwaysPositive }`, because nothing has measured a target that produces a positive zero for a negative subnormal and a neutral baseline must not claim a behaviour on no evidence. A contract requiring it is `Unknown`, and `a_target_that_honours_no_stated_contract_rejects_with_a_cause_per_entry` exercises that path.

### The caller preference list

`NumericalContractPreference` is an ordered nonempty list, resolved in `verify_request` **per target**, once, before any planning, by the caller's stated order with the first honourable entry winning. Resolution is the honourability authority applied to a proposal with no capability requirements — `physical::assess_contract` — so there is one authority, not a second one for contracts.

Four tests: `a_single_entry_preference_and_a_bare_contract_are_the_same_request` (the list is additive, not a second mechanism), `a_preference_list_resolves_by_the_callers_order_and_never_by_rank` (run in both orders against a baseline that honours both entries, so the winner is the caller's choice and nothing else), `the_stated_preference_separates_requests_that_resolve_alike`, and the rejection test above.

**The stated list is in the request subject, beside the resolved entry.** Binding only the winner would let two requests with different fallback intents share one subject, and an explain trace would then attribute a resolution to a preference it never saw. This is also the evidence ADR 0076's second open question was missing: the list is what lets the fallback enter identity, and a retry loop structurally cannot, because the compiler never sees the alternatives. Recorded in `record-adr-0076-honourability-implementation` rather than amended into the record, which is out of scope here.

**A structural constraint found while wiring it.** Semantic normalization is program-scoped and runs before per-target compilation, so it needs exactly one contract. When two targets resolve differently there is no defensible choice — a rewrite legal under one may be illegal under the other — so `uniform_resolved_contract` returns `None` and `compile` fails closed with `compile.unsupported.numerics.divergent-resolved-contracts`. Unreachable today, because `verify_request` admits one governed target profile; recorded because it is a real seam and silently normalizing under the first target's contract would be a correctness defect the moment a second profile lands.

### The explain rejection shape

`ExplainEvent::Feasibility` could not carry it: its `required` and `available` are `Quantity` values its validator compares by magnitude, and a means is not a magnitude. `ExplainEvent::NumericalHonourability` is new, at `ExplainStage::TargetFeasibility`, carrying the dimension, the required behaviour, a three-valued outcome (`Honoured { means }`, `Unhonourable { means, honoured }`, `Undeclared`), and the declaring profile. It is canonically encoded under a new event tag and rendered.

`Undeclared` maps to `DeferredUnsupported`, not `RejectedTarget`: the profile said nothing, and a trace that read as a refusal would let a downstream reader act on knowledge nobody supplied.

The **admitted** trace records the means too, not only the verdict, for the same reason the proven evidence does. Where three regions previously emitted one `target.strict-f32` record each, they now emit four `target.numerics.*` records each, and `end_to_end_explain_emitter_has_exhaustive_typed_conformance` asserts the exact set, the means, and the declaring profile.

### Identity, carried rather than dropped

The honoured dimensions flow through `ProvenEvidence` → `AdmittedImplementation::feasibility` → `SelectedPlan::honoured` → `encode_plan_identity`, encoding the dimension, behaviour, means, and declaring profile. Two plans that honour one dimension natively and by emulation emit different operations; two that rest on declarations from different profiles rest on different evidence. Either omission would give distinguishable plans one identity.

`FrontierRejection::Unhonourable` is a distinct variant from `Infeasible` and is canonically encoded, because the two say different things to a caller and a shared variant would force the numerical cause through a pair of `u64`s that cannot hold it.

### Measurement boundary

Everything above is implemented and tested; none of it is a measured target claim. No governed profile constructs `SupportedWithExactEmulation`, `SupportedOnlyUnderDeclaredRelaxation`, or `Unsupported` — the target-neutral baseline declares only `SupportedExactly` — so those three means are reserved-and-tested vocabulary, not observed behaviour. `declare-metal-numerical-honourability` is the first profile that will declare otherwise, and it is what this ticket was load-bearing for: the compiler can now assess honourability before emission, from a target profile fact, instead of discovering it afterwards from a backend-local check.

### Contract updated

`docs/numerical-semantics.md` — its normative owner per ADR 0076 — gained the selection point (required input, no default, ordered preference by stated order, never cost-ranked, one contract per program), the per-dimension honourability declaration and its composition into ADR 0043's four outcomes including the unenumerated-`Unknown` clause, and the converse honesty rule with its consequence that the numerical contract is not a search dimension. A stale sentence claiming `StrictF32NumericalContract::governed` is the only registered contract was corrected while preserving its conclusion, which the second registered contract does not disturb.

### Gate

`uv run --locked python scripts/check_repository.py` passes.
