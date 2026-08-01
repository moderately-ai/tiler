---
id: register-opaque-calls-on-the-compile-path
title: Register opaque calls on the compile path
status: todo
priority: p2
dependencies: []
related: [accept-the-public-backend-provider-composition-boundary]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [physical-planning, extensions, implementation]
---
## User-visible outcome

A declared and registered opaque physical call can reach `enumerate_frontier` through `session::compile`, so the opaque-call admission path that already exists is reachable by some caller rather than by none.

## Why this slice exists

**Fact, at `e6a47d9`.** `implement-opaque-physical-call-providers` built the declaration, ABI, effect, aliasing, placement, failure-stage, and coherence contracts, and `integrate-opaque-calls-into-the-physical-frontier` made `enumerate_frontier` admit a registered call. `enumerate_frontier` takes an `&OpaqueCallRegistry`, and the sole production call site constructs an empty one inline: `crates/tiler-compiler/src/pipeline/planning.rs:228` passes `&crate::call_registry::OpaqueCallRegistry::new()`.

Reproduce with `grep -rn "OpaqueCallRegistry" crates/`. Every hit other than the definition module and the `use`/parameter in `frontier.rs` is inside a `#[cfg(test)]` module: `selection.rs` after its `#[cfg(test)]` at line 1689, `frontier.rs`'s own test module, and `pipeline/tests.rs`. The positive control for that reading is `GovernedPhysicalProvider`, which the same style of grep finds at a production site, `planning.rs:171`.

**Inference.** No opaque call can reach a compilation, so the admission path is implemented support behind an authority nothing populates, and a test asserting the frontier admits a registered call proves nothing about `compile()`.

This is a narrower statement than ADR 0078's 2026-07-31 correction, which records that opaque declaration and registration are compiler-owned and crate-private so that no *out-of-crate* provider registers a call. That classification is unaffected and is not reopened here: the gap is that no caller of any kind registers one.

## Implementation keys

- Decide where a compile-path registry is composed and by which authority; the call registry stays crate-private, so this is an internal wiring question and not a public seam.
- Keep an empty registry a legitimate state. A compilation with no declared call must behave exactly as it does now.
- Add one compile-path test that a registered call reaches an admitted alternative through `session::compile`, and one that an unregistered call named by a proposal is `FrontierRejection::OpaqueCall` with cause `Unregistered`.
- Run each new check against a case that must fail before relying on it.

## Closes when

A registered opaque call is admitted through `session::compile` by a test that fails when the registration is removed, the empty-registry path is unchanged, and targeted package checks pass.

## Graph maintenance

- Record the outcome against ADR 0078's opaque-call correction if the classification is affected; it should not be.
- **This ticket is `related` to [`accept-the-public-backend-provider-composition-boundary`](accept-the-public-backend-provider-composition-boundary.md) and deliberately does not depend on it.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) names this gap among its unsupported cases, so the ticket was considered as a conditional implementation ticket and rejected as one. The call registry is crate-private, which makes composing it internal wiring rather than a public seam, and a crate-private composition site can be moved for free while Tiler is pre-alpha; parking a live reachability gap — no caller of any kind registers an opaque call, so every test of the frontier's opaque-call admission proves nothing about `compile()` — behind a decision it does not need would cost more than the churn it saves. The interaction that does exist is worth knowing before starting: if ADR 0090's physical-provider seam is accepted, `CompileRequest` acquires an installation idiom, and an internal registry composed here should match that idiom rather than invent a third spelling.

## Outcome

**Fact.** The registry is composed with the provider list, in `crate::frontier::PhysicalAuthorities` — a crate-private pair of `Vec<&dyn PhysicalImplementationProvider>` and `OpaqueCallRegistry` — and passed down the compile path from `pipeline::compile`, which states `PhysicalAuthorities::governed()`: the governed provider and an empty registry. No public item was added, so ADR 0078's opaque-call correction is unaffected and is not reopened: declaration and registration remain crate-private, and nothing out of crate can register a call.

**The composition-point elimination.** Four candidates were tested. *`CompileRequest`* is eliminated by the crate-private classification itself — installing a call from outside the crate is exactly what ADR 0078's correction refuses, and the disclosure surfaces for such a seam are separately owned. *`CompilationRequest`/`VerifiedCompilationRequest`* is eliminated twice over: a provider is a borrowed statically linked implementation and the verified request is an owned, cloned, `Eq` value with no lifetime, so the pair cannot be carried there without a lifetime parameter on every downstream request type; and an authority the request *carries* but its canonical identity does not *bind* is an identity gap, while binding it would move the request subject of every compilation — including those registering nothing. *Construction inside `enumerate_complete_plans` from governed declarations alone* is eliminated by reachability: with no seam a caller can vary, the only way to register anything is to add a governed declaration, which is a production behaviour change (an admitted call has no lowering) and still leaves the registry unreachable by any caller who did not ship one. What survives is composing the pair where the governed provider array already lived and threading it from the entry, which is what landed. The providers are composed with the registry rather than left hardcoded because a registration alone is not reachable: only a provider proposes a call, so a seam that installs one without the other is a seam no test and no future backend can drive.

**Behaviour identity.** `PhysicalAuthorities::governed()` reproduces the previous `[&GovernedPhysicalProvider]` and `OpaqueCallRegistry::new()` exactly, and nothing in the change reaches an identity encoding. The explain request qualifier `bae4788d2fc79631` did not move (`explain::tests` passes unchanged), and the full workspace suite — including the deterministic-product, portfolio-identity, and conformance cases — passes with no rebaselined pin, golden, or fixture.

**Evidence.** `pipeline::tests::a_registered_opaque_call_is_admitted_through_the_compile_path` registers one call, installs a provider proposing it for the whole-program region, and compiles through `pipeline::compile_configured` — the same function `session::compile` reaches through `compile`, differing only in stating the authorities that `compile` defaults. The admission is observed twice: the compilation is refused with `ProgramError::Structure { rule: "unlowerable-opaque-body" }`, reachable only through an admitted opaque body in a retained plan, and the trace records `admitted-count = 2` for `region:whole-program` against `1` without the registration. `pipeline::tests::an_unregistered_opaque_call_named_on_the_compile_path_is_refused_by_name` compiles the same composition with an empty registry: the compilation succeeds, retains every governed alternative, and records `opaque-call.registration.v1` disproving `opaque-call.registered` with reason `opaque-call.unregistered` against the exact subject `test-owner/whole-program-call@1[x=input#0,y=output]` — the compile-path form of `FrontierRejection::OpaqueCall { cause: Unregistered }`.

**Watched failures.** Three perturbations were run and each failure observed. Restoring the inline empty registry in `planning.rs` while keeping everything else: the first test fails at its `Err` match because the compilation succeeds — so the test measures the wiring and not the fixture. Dropping the registration from the first test's authorities: same failure. Adding the registration to the second test's authorities: it fails at its `expect`, because a registered call is no longer refused as unregistered.

**Unsupported cases.** Lowering an opaque call remains unimplemented, so a registered call that reaches a retained plan fails the compilation closed; that is the current contract, not a defect this ticket introduces, and `plan_region_order` is where it is enforced. `PhysicalAuthorities::composed` has no production caller — the compile path still ships an empty registry — so this closes the reachability gap for callers *inside* the crate only, which is the whole of what the crate-private classification admits.
