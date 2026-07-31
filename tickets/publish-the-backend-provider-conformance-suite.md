---
id: publish-the-backend-provider-conformance-suite
title: Publish the backend-provider conformance suite
status: todo
priority: p2
dependencies: [exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio]
related: [compile-extension-spike-fixtures-in-the-gate]
scopes: [implementation/compiler, implementation/build, implementation/artifact, implementation/runtime, contracts/numerics, contracts/foundation, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, testing, conformance]
---
## User-visible outcome

Third-party backend authors receive a reusable conformance harness that proves the host can reject invalid providers and that a passing provider composes deterministically through compilation, artifacts, routing, and execution.

## Implementation keys

- Extract tests only after the three-provider vertical identifies the real public contracts; do not design a mock-only alternate API.
- Cover provider identity/revision stability, deterministic registration/freeze, duplicate and ambiguous authority, empty offers, malformed proposals, verifier bypass attempts, forged provenance/resources, unstable emission, payload/entry mismatch, missing runtime adapters, incompatible target/representation, backend-aware routing, routing commit, and asynchronous resource lifetime.
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
