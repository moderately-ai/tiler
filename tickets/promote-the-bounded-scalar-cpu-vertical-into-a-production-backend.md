---
id: promote-the-bounded-scalar-cpu-vertical-into-a-production-backend
title: Promote the bounded scalar CPU vertical into a production backend
status: todo
priority: p1
dependencies: [accept-the-production-boundary-for-the-bounded-scalar-cpu-backend]
related: [prototype-a-bounded-scalar-cpu-backend-vertical, exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio, earn-cpu-feature-level-execution-environments-from-host-observation]
scopes: [implementation/cpu, implementation/ir, implementation/artifact, implementation/build, implementation/runtime, implementation/conformance, implementation/workspace, implementation/cargo-lock, contracts/artifacts, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [cpu, backend-providers, implementation, artifacts, runtime]
---
## User-visible outcome

An explicitly selected CPU attempt can compile, package, route, and execute the accepted scalar profile through production crates and compare bit-for-bit with `tiler-reference`; it no longer depends on an out-of-gate spike or a test-only KIR simulator.

## Work

- Implement exactly the crate/responsibility boundary accepted by `accept-the-production-boundary-for-the-bounded-scalar-cpu-backend`.
- Promote the spike's versioned scalar-image translation and independently validated decoder rather than executing compiler-owned handles at runtime.
- Preserve its explicit host-process observation and pre-commit numerical/layout refusals; do not trust an artifact's restatement of its own target as host evidence.
- Route publication through the neutral build/cache/correspondence seam and route execution through the accepted artifact/runtime adapter seam.
- Retain `tiler.cpu.scalar` and the accepted representation spelling under one governed owner; step the representation if the production grammar is not byte-for-byte the spike grammar.
- Keep every unsupported KIR type, operation, address space, barrier, vector, thread, and numerical realization a typed refusal. Never widen the target profile because the executor happens to implement an operation.
- Add cross-layer CPU conformance using independent `tiler-reference` results and subject perturbations for payload identity, decoder grammar, host profile, routing environment, numerical realization, launch geometry, buffer offsets/extents, and operation semantics.
- Update `docs/backends/cpu.md`, architecture/component ownership, catalogs, and the support matrix to distinguish scalar production support from vector/threaded reservations.
- Add the admitted CPU crate to the workspace and atomically map `implementation/cpu` to its package in `ticketsplease.toml`.

## Acceptance

- One retained scalar program reaches a real host result through production artifact bytes, not `VerifiedKernel` or a test helper, on every supported CPU host.
- The reference perturbation that removes NaN canonicalization still fails on the exact differing element, and independent host/profile/representation perturbations each reach their named refusal.
- The artifact can be decoded and executed without linking the compiler or holding compiler objects.
- Metal and CPU are exercised as separate explicit attempts; neither silently falls back to the other.
- The spike remains reproducible until the production path subsumes its bounded evidence, then is archived or removed according to the accepted evidence-retention policy rather than becoming a second maintained implementation.
- Targeted package checks, workspace nextest and doc tests, Clippy, rustdoc, `tkt lint`, `tkt guard`, citation checks, and the exact merged-tree gate pass.
