---
id: select-numerical-contract-and-compose-feasibility
title: Make the numerical contract a stated request input and compose its feasibility
status: todo
priority: p1
dependencies: [widen-numerical-vocabulary-and-complete-identity]
related: [draft-target-honourable-numerical-contract-adr, prototype-optimizer-conformance-gate]
scopes: [implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, feasibility]
---
ADR 0076 items 2, 3, and 5. This is where a permanent refusal becomes a choice — and the point of the record is that the choice belongs to the caller, not to the planner.

## Selection: the contract is a required request input

Today `StrictF32NumericalContract` in `crates/tiler-compiler/src/request.rs` is `pub(crate)`, its only constructor `governed()` is a hardcoded constant, its only assembly site `CompilationRequest::governed` is `#[cfg(test)]`, and `pipeline::compile` is itself `pub(crate)`. There is no non-test path by which any caller states a numerical contract at all, and `verify_request` rejects any contract that is not the constant with `compile.unsupported.numerics.strict-f32`.

Make the resolved contract a required, typed input at the compilation-request boundary. No `Default`, no ambient fallback, no implicit strictest reading. A request that does not state it does not compile, and the diagnostic says the contract is *unstated* rather than naming a dimension.

**A strict default was considered and rejected**, and the reason is specific rather than stylistic: on the measured Apple row the strictest reading is unhonourable, so defaulting to it would make every Apple compilation fail with a rejection the caller never asked for, and the caller's only route to the knob would be reading a rejection. `MetalTargetFacts` sets the in-repo precedent for a no-`Default` typed target statement and documents its reasoning.

A caller may state one resolved contract, or an explicitly ordered preference list of contracts it declares equally acceptable. With a list, resolution is by the caller's stated order and the first honourable entry wins; it is deterministic, recorded, and **never cost-ranked**. A single-entry list and a bare contract behave identically. Note that ADR 0076 leaves the list-versus-retry shape as an explicit open question and says the alternative was not rejected on evidence — if implementation gives you evidence either way, record it and amend the ADR rather than silently choosing.

**This shapes a boundary before it is public.** Under ADR 0075 a `pub(crate)` → `pub` promotion of `CompilationRequest` or `compile` is Tom's to approve. Do not promote them here; state the shape and leave the visibility as it is unless you are separately authorized.

## The honourability authority

Add a per-dimension honourability authority as a **peer** of `feasibility::CheckedTargetProfile`, not as new `CapabilityAxis` variants. The reason is decisive rather than aesthetic: `CapabilityAxis` is a quantitative space — a `u64` bound, a `Quantity` unit, an `AtMost`/`Exact`/`Implies` relation — and `SupportedWithExactEmulation` has no representation as a bound comparison. Emulation is honoured by *emitting different operations*, so it changes the program rather than the verdict; encoding it as a satisfied `Implies` predicate would discard exactly the outcome that carries work.

Implement the composition ADR 0076 §3 states, into ADR 0043's `Proven`/`Rejected`/`Unknown`:

- honoured exactly or by emulation → a satisfied hard predicate;
- honourable only under a relaxation the caller's stated contract does not authorize → a **disproved** predicate, not deferred and not unknown, because the caller's authorization is known at `CompileProfile` and cannot arrive later;
- declared unhonourable → a disproved predicate;
- a dimension the profile does not speak to at all → `Unknown` in ADR 0043's exact sense (no admissible proof path), which may appear in search and explain state but never in an executable frontier.

That last clause is what makes an unenumerated dimension fail closed rather than defaulting to honoured. Test it directly — it is the clause most likely to be implemented as an accidental pass.

## Retire the boolean

`PrototypeTargetProfile::supports_strict_f32` and `CapabilityAxis::StrictF32Arithmetic` are **replaced, not extended**. A boolean cannot say which dimension failed and cannot express emulation, and it is today wired to a requirement predicate that omits the subnormal dimensions entirely, so extending it would preserve a defect. The requirement side of that wiring is repaired by `widen-numerical-vocabulary-and-complete-identity`, which is why this ticket depends on it.

## The honesty rule, both directions

`docs/numerical-semantics.md` already states that target defaults cannot expand the program's permissions. Add the converse, which is stated nowhere: **no authority may narrow, weaken, or substitute the caller's stated numerical contract in order to make a target feasible.** When no contract the caller stated is honourable, reject with a typed, explainable error naming the dimension, the required behaviour, the behaviour the target declares, the means the profile offers if any, and the declaring profile's versioned identity. Never emit under a different contract, never fall back to a target default, and never report the difference as a cost.

The consequence to enforce in code, not only in prose: **the numerical contract is not a search dimension.** Cost-based selection ranks implementations of one contract and may never rank contracts against each other, because that would price meaning. The tempting mistake this forbids is treating a flush-tolerant plan as a cheaper alternative to a preserving one.

Give `explain` the rejection shape item 5 requires. A rejection that reads `strict-f32: required 1, available 0` is the current state and is exactly what this ticket exists to replace.

## Boundaries

Keep hard feasibility separate from estimated cost; never hide an infeasible plan behind an infinite cost. One contract per program — a preference list resolves to exactly one contract before planning begins, it does not become a per-region choice, and two regions of one program never honour different contracts.
