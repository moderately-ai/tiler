---
id: select-numerical-contract-and-compose-feasibility
title: Make the numerical contract a stated request input and compose its feasibility
status: todo
priority: p0
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

## This is now the hard blocker to first execution — measured, not predicted

**Measurement, 2026-07-25, `29a26ba`, Apple M4 Max under macOS 27.0.** `cargo run -p tiler-prototype-compile` drives a `[4, 1]` scale-then-reduce program from a `SemanticProgram` through the public compiler boundary, selects the fused plan `selected-plan:59399b7f985a0859`, emits Metal source for it — and then stops:

```text
target profile: tiler.prototype-target-neutral-baseline.v1
selected alternative: selected-plan:59399b7f985a0859 (fused)
serial-sum offline producer failed: the target cannot honour the kernels'
declared numerical contract: subnormal-flush-in-arithmetic
```

Everything before that line works. The compiler compiles, the emitter emits, and `MetalTranslationUnit::require_declared_realization` refuses because the governed contract declares subnormal **preservation** while `MetalSubnormalArithmetic::FlushesToZero` states the measured Apple fact that `f32` arithmetic flushes subnormal operands and results to zero on every governed family in every math mode.

**Inference — no `metallib` can be produced for the governed contract on Apple hardware, so no execution proof can begin.** The offline path is not missing a component; it is complete and correctly refusing. Reaching hardware requires exactly what this ticket owns: making the resolved contract a stated request input with more than one expressible value, so a caller who accepts flushing can say so in advance as part of what the program means. `StrictF32NumericalContract::governed` is currently the only contract the compiler registers and it is not deliverable on the only backend that exists.

**What must not be done instead.** Relaxing `require_declared_realization`, widening the emitter's honourability, or compiling under `strict_baseline` while declaring preservation would each reach hardware and return wrong numbers. ADR 0076 item 5 forbids delivering anything other than the declared contract, and the accepted goal names a shortcut that reaches hardware by hard-wiring the first profile as a *loss* rather than a win.

`prototypes/serial-sum-compile/src/main.rs::the_governed_contract_is_not_honourable_on_the_governed_apple_target` pins the refusal and its exact gap key, and is deliberately written as an assertion of failure so that the day a contract becomes selectable, it breaks and forces its reasoning to be re-derived rather than passing silently under a new meaning.

Raised to p0 on this evidence: every Metal and runtime p0 sits behind it.

## Measured: admission is the easy half — the proof machinery is contract-specific

A working prototype of the Selection half was built and then **reverted deliberately**, because it produced a public surface that lied: `NumericalContract::FlushSubnormalsToZeroF32` was accepted at the request boundary and then failed for every program. What it measured is recorded here so the real implementation starts from evidence rather than from the same discovery.

**What the prototype did.** Added a second governed contract `tiler.flush-f32.v1` — `SubnormalMode::FlushToZero { zero_sign: PreservesSign }` on both dimensions, contraction and reassociation still `Forbidden`, its own versioned key so identities differ — plus `governed_profile()`, `governed_under()`, and a `NumericalContract` selector on the public boundary.

**Two hardcoded equality checks had to be widened, not one.** `verify_request` compares against `StrictF32NumericalContract::governed()`, and so does `VerifiedCompilationRequest::for_target`, which raises `UnverifiedTargetSelection` — a diagnostic that names target selection for what is actually a contract rejection. Both are at `crates/tiler-compiler/src/request.rs`.

**Then the compile failed inside the compiler, and the trace says exactly where.** Under `tiler.flush-f32.v1`, request verification, normalization, region formation, capability resolution, and index-region refinement all succeed and `cover.enumeration` retains 16 covers. Then **every** fused candidate defers:

```text
31..42 capability-resolution deferred-unsupported rule=fusion.legality.v1@1
       provider=tiler.fusion-strict-f32@1 subject=candidate:region:…
       event=deferred:fusion.obligations-discharged:unproven-exceptional-values
44 intrinsic-scheduling compiler-failure rule=compile.failure@1
   subject=schedule:implementation-frontier
   event=compiler-failure:frontier-malformed-proposal
```

with `Frontier(MalformedProposal { provider: tiler/prototype-serial-sum-physical@1, source: Intrinsic { rule: "request-subject", region: RegionId(1) } })`.

**Inference — two distinct pieces of work, neither of them the request boundary.** The fusion numerical proof provider is literally named `tiler.fusion-strict-f32` and cannot discharge exceptional-value obligations for a contract it was not written for, so it defers every region rather than proving or refusing one. And the physical provider then rejects with an `Intrinsic { rule: "request-subject" }`, which is a hard compiler-output error rather than a feasibility outcome — something below is still keyed to the strict contract's subject.

**Second pass, 2026-07-25: the three admission checks are now unified and the remaining blocker is a single obligation.**

Landed on `main`: `governed_flush_to_zero()` (key `tiler.flush-f32.v1`, `FlushToZero { zero_sign: PreservesSign }` on both dimensions, contraction and reassociation still `Forbidden`), `governed_profile()`, `is_governed()`, and `governed_under()`. All three sites that hardcoded equality with `governed()` — `verify_request`, `VerifiedCompilationRequest::for_target`, and `physical::verify_schedule_with_feasibility` — now share that one authority. The public selector was **not** landed; see below.

With those in place the failure moved from `InvalidCompilerOutput` to an honest `NoFeasiblePlan(Selection(Structure { rule: "no-complete-plan" }))`. The physical path accepts the flush contract; the trace shows `frontier.enumeration` admitting one implementation for the reduction region and **zero** for the other, then `selection.complete-plan` with `plan-count: 0`.

**The single remaining cause.** `fusion_legality.rs` discharges `FusionObligation::ExceptionalValues` only when both subnormal dimensions are `SubnormalMode::Preserve`. Under any flush contract it returns `unknown("unproven-exceptional-values")`, so every fused candidate defers, the whole-program region is never legal, and no complete plan forms. The provider is named `tiler.fusion-strict-f32` — it was written for one contract and correctly declines to speak for another.

**The argument that would discharge it, to be made carefully rather than assumed.** Both alternatives compile under the *same* contract, and the measured target flushes at each arithmetic operation while a store/load round trip preserves subnormals unchanged. So a value that lives in a register in the fused form and in memory in the materialized form is flushed at the same operations either way, and fusion does not change observable exceptional-value behaviour. That reasoning needs checking against the NaN-canonicalization dimension too, and it needs its own test — it is a fusion-legality correctness claim, not a plumbing change.

**The public selector was deliberately withheld.** A `NumericalContract::FlushSubnormalsToZeroF32` on the public boundary would compile no program at all until the obligation above is dischargeable, which is the same lying-API failure this ticket's first pass already reverted once. Expose it in the same change that makes fusion contract-aware.

**Consequence for whoever takes this ticket.** Widening admission is roughly twenty lines and is *not* the work. The work is the ticket's honourability-authority and composition sections: the fusion proof must be contract-parameterized so it proves or refuses rather than deferring, and the physical path's request-subject binding must accept any admitted contract. Until both hold, admitting a second contract only moves the failure from a clear rejection at the boundary to an opaque `InvalidCompilerOutput` deep inside — strictly worse for a caller.

The reverted prototype is not in the tree; this record is what it produced.
