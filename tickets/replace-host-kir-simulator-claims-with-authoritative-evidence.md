---
id: replace-host-kir-simulator-claims-with-authoritative-evidence
title: Replace host KIR simulator claims with authoritative evidence
status: todo
priority: p1
dependencies: [promote-the-bounded-scalar-cpu-vertical-into-a-production-backend, execute-the-loop-carried-cooperative-kernel-on-a-real-backend]
related: [share-one-structured-kernel-interpreter, lower-a-loop-carried-cooperative-body]
scopes: [implementation/ir, implementation/compiler, implementation/reference, implementation/cpu, implementation/conformance, contracts/foundation, contracts/numerics, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [cpu, testing, correctness, conformance, cleanup]
---
## User-visible outcome

Every claim currently backed by either host `KirMachine` is backed by its owning authority: semantics by `tiler-reference`, structural refinement by IR/compiler verification, and actual execution by a real CPU or Metal backend. No claim becomes weaker or silently disappears merely to make deletion possible.

## Required census

At the exact implementation base, enumerate every call to `interpret_fused`, `interpret_fused_inputs`, `interpret_bf16`, the IR-local `KirMachine`, and `cooperative_reference`. Classify each assertion into exactly one primary evidence role:

1. **Semantic meaning** — independent `tiler-reference` evaluation.
2. **Structural refinement** — canonical KIR, verifier, identity, ownership, bounds, synchronization, or buffer/launch assertions in the owning IR/compiler crate.
3. **Executed realization** — artifact-backed execution through a real CPU or Metal backend, compared to the independent reference.
4. **Unsupported/unevidenced** — downgrade the maturity claim to `Unknown` or file a bounded successor; never substitute source compilation or the reference evaluator for execution.

Cross-layer conformance may deliberately combine all three positive roles, but each check must say which property it owns. Cover access relations, multi-input binding, staged materializations, split reductions, nontrailing axes, nonlinear expressions, BF16, NaN canonicalization, and synchronization as property dimensions rather than copying one test per old helper call.

## Correctness constraints

- Do not introduce another interpreter, executor trait, shared fake, compatibility wrapper, or deprecated alias.
- Do not use the same execution implementation as the semantic oracle.
- A backend source/golden check is structural evidence, not device execution.
- Preserve portable structural and reference tests on hosts without Metal; measured device cases report unavailable rather than silently passing.
- Use exact launch geometry from the scheduled program/artifact. Never infer participants from staging allocation.
- Bound every execution case and reference work explicitly.

## Acceptance

- A checked ledger names every former simulator consumer and its replacement or explicit unsupported disposition; the census count is asserted.
- Each replacement is independently perturbable at its subject and its failure text is retained.
- Documentation no longer calls host simulation device/backend execution or end-to-end agreement.
- No new code depends on either simulator while the deletion ticket is pending.
