---
id: publish-the-backend-provider-conformance-suite
title: Publish the backend-provider conformance suite
status: deferred
priority: p1
dependencies: [exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio, decide-the-backend-provider-conformance-harness-public-surface, package-selected-physical-implementation-provenance-in-artifact-identity, carry-required-compilation-selection-identity-on-compile-profile-contexts, make-explain-dispositions-assertable-by-a-conformance-suite]
related: [compile-extension-spike-fixtures-in-the-gate, audit-backend-authoring-against-all-thirteen-responsibilities]
scopes: [implementation/conformance, implementation/compiler, implementation/build, implementation/artifact, implementation/runtime, contracts/numerics, contracts/foundation, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, testing, conformance]
---
## User-visible outcome

Third-party backend authors receive a reusable conformance harness that proves the host can reject invalid providers and that a passing provider composes deterministically through compilation, artifacts, routing, and execution.

## Exact-current Fact audit — 2026-08-17 at `d002cd55406522922e5eb750c8c4d9033dde4469`

- **False — an accepted reusable owner already exists.** ADR 0106 and the complete `tiler-conformance` crate header deliberately make every module test-only and export no public item. The crate is the correct cross-layer evidence owner, but publishing a third-party harness is a consequential new facade rather than extraction under an accepted surface. `decide-the-backend-provider-conformance-harness-public-surface` is the required prerequisite, and this implementation now declares `implementation/conformance`.
- **False — a missing runtime-adapter registry is malformed.** ADR 0090 deliberately makes the runtime adapter an explicitly supplied, independently selected value. `route_with_adapter` cannot be called without one; there is no ambient registry to be empty or missing. Conformance must test an explicitly supplied wrong, refusing, or incompatible adapter, not invent discovery.
- **False — an empty physical-provider offer is invalid.** `ProviderOffer` documents an empty offer as a legitimate local result. Malformed provider output, a typed decline, an empty offer, and a globally absent feasible plan are distinct subjects and must remain distinct.
- **Imprecise — all thirteen historical matrix rows are current public responsibilities.** The matrix remains the audit index, but scalar lowering is retired under ADR 0105 and opaque-call registration remains compiler-owned. The exact current conformance packet must count the active external rows, print the retired/internal exclusions, and fail if either population silently moves; it must not report a uniform thirteen-row pass.
- **Verified prerequisite gap.** The reusable end-to-end suite cannot truthfully bind complete selected physical provenance or compilation-selection provenance until the two carrier tickets named in `dependencies` land. Explain dispositions must either gain a structured assertion surface or be explicitly excluded by the accepted conformance facade before this suite reports its bounded coverage.

## Implementation keys

- Extract tests only after the three-provider vertical identifies the real public contracts; do not design a mock-only alternate API.
- Cover provider identity/revision stability, deterministic registration/freeze, duplicate and ambiguous authority, legitimate empty offers, malformed proposals, verifier bypass attempts, forged provenance/resources, unstable emission, payload/entry mismatch, explicitly supplied adapter refusals or incompatibility, incompatible target/representation, backend-aware routing, routing commit, and asynchronous resource lifetime.
- Separate semantic-equivalence obligations that require provider-supplied reference/conformance evidence from structural properties Tiler can rederive.
- Require each check to have a deliberate perturbation that fires; count the discovered test population so a glob matching nothing cannot pass.
- Supply an external-provider-shaped passing fixture and multiple failing fixtures that compile/run only against public surfaces.
- Document exact maturity: passing this suite is conformance to the bounded provider contract, not certification of arbitrary mathematical correctness or performance.
- Keep nextest as the test runner and retain required doc-tests separately.
- Present the exact public conformance-harness facade, types, and call sites to Tom before acceptance.
- Prefer an existing accepted public crate or module only when the accepted composition ADR assigns it that ownership. If a new crate is required, file and complete a separate crate-admission ticket before scaffolding it.

## Closes when

Every public provider component has positive and negative conformance coverage, every new check has demonstrated its failure path, the harness is consumer-neutral, documentation states its limits, targeted nextest and per-package Clippy pass, and one final `make full` passes.

## Graph maintenance

- Link the suite from the provider-composition contract, public API docs, correctness contract, and example providers.
- File backend-specific performance qualification separately; conformance must not turn cost measurements into correctness authority.
- Keep untrusted/dynamically loaded plugin certification deferred.


## Deferred — 2026-08-18

The owning decision (`decide-the-backend-provider-conformance-harness-public-surface`) was accepted as an exact typed deferral: no public conformance spelling now. This carrier defers on that decision's named reopening trigger rather than dispatching on the decision ticket's completion.

## Trigger check log

- 2026-08-18 — **not fired.** The trigger is one second independently authored backend fixture sharing the portfolio's neutral, non-self-certifying structural and execution subjects. No such fixture exists in-tree. Reproduce: enumerate independently authored fixtures under `crates/tiler-conformance/` and compare their structural/execution subjects for a shared non-self-certifying pair.
