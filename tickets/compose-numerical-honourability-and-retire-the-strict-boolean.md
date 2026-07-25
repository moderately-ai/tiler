---
id: compose-numerical-honourability-and-retire-the-strict-boolean
title: Compose per-dimension numerical honourability and retire the strict-f32 boolean
status: in-progress
priority: p1
dependencies: [select-numerical-contract-and-compose-feasibility]
related: [declare-metal-numerical-honourability, draft-target-honourable-numerical-contract-adr]
scopes: [implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, feasibility]
claimed_from: todo
assignee: agent-numerics
lease_expires_at: 1784996925
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
