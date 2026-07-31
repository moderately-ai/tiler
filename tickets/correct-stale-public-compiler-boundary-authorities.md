---
id: correct-stale-public-compiler-boundary-authorities
title: Correct stale public compiler boundary authorities
status: todo
priority: p1
dependencies: []
related: [prototype-public-compiler-api, wire-capability-and-refinement-into-compile-path, correct-the-stale-post-vertical-implementation-status, record-the-arithmetic-type-in-the-numerical-honourability-contract, define-backend-device-and-execution-context-vocabulary]
scopes: [contracts/optimizer, contracts/foundation, contracts/numerics, contracts/decisions, research/program-planning, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, compiler, correctness]
---
## User-visible outcome

The compiler, extension, and correctness contracts describe the implemented public reviewed-draft compiler session and its exact bounded support instead of claiming that external capability installation, a public compile entry, or public explain results are unreachable.

## Why this is a correctness ticket

- **Fact:** `prototype-public-compiler-api` and `wire-capability-and-refinement-into-compile-path` are done, while `docs/compiler/optimizer.md`, `docs/correctness-and-testing.md`, `docs/operation-extensions.md`, `docs/research/program-planning/general-compilation-boundary.md`, accepted ADRs 0075, 0076, and 0078, and the module documentation in `crates/tiler-compiler/src/session.rs` retain statements from before those outcomes.
- **Fact:** the public session accepts caller-installed index-access lowering capabilities and an ordered numerical-contract preference, and exposes compilation and typed failure products. The scalar-lowering vocabulary is public but has no ordinary compile-path consumer, so the correction must not generalize the delivered extension seam.
- **Inference:** leaving the stale statements in normative contracts makes the public status correction internally contradictory and can cause new work to rebuild an already delivered boundary or rely on an extension path that is not delivered.

## Implementation keys

- Read every edited contract and source file in full, then derive the public surface from `tiler-compiler` construction and call sites rather than this ticket.
- Correct the stale passages located by `rg -n -i 'no public compile entry|public compiler.*later|compiler boundary is private|Nor does it expose the request|does not export its own entry point|request boundary is still crate-private|no way to configure it|installation is still not reachable|private bounded compiler slice|current executable model recognizes one exact graph' docs crates/tiler-compiler/src/session.rs`, including the current-tense claims in ADRs 0075, 0076, and 0078 and `general-compilation-boundary.md`.
- Preserve the exact support limit: one-input/one-output bounded F32 pointwise or scale-bias-strict-sum programs, caller-installed index-access capabilities, public caller-authored ordered `TargetRequest` profiles through the reviewed session boundary, and no ordinary scalar-lowering-provider consumption. The earlier “internal target-profile choice” premise is stale after the 2026-07-30 target-request acceptance.
- Correct `docs/numerical-semantics.md` and ADR 0076 to state that the implemented honourability declaration is keyed by a scalar-arithmetic policy subject and arithmetic type as well as dimension. Silence about one arithmetic type is `Unknown` even when the profile speaks about a neighbouring type.
- Reconcile the canonical scalar-policy sketch with all eleven governed dimensions, including operand permutation, signed zero, and the exceptional-value assumptions, and name the governed approximate-intrinsic envelope vocabulary. Do not generalize this dense scalar contract to integers, booleans, quantized compounds, or other future policy families.
- Keep reviewed alpha visibility separate from stabilization or publication. Correcting the module documentation is a public module-boundary change and requires Tom's review before acceptance.
- Prove every new absence or consistency check can fail, then run the targeted `tiler-compiler` tests, per-package Clippy, local documentation checks, `tkt lint`, and one batch `make full`.

## Closes when

Every named normative and source authority agrees with the live public session, the caller-installed registry success and missing-family failure are exercised, no unsupported scalar-provider or general-workload claim is introduced, Tom has reviewed the corrected public module boundary, and the targeted and full gates pass.

## Graph maintenance

- Link any additional stale current authority to the exact completed ticket that falsified it and update it in this scope or split a narrower follow-up.
- Preserve historical descriptions when they are explicitly historical; qualify them instead of rewriting the past as if the current boundary always existed.
- Close this ticket when the contracts and source documentation agree; implementation breadth remains owned by its existing tickets.
- `record-the-arithmetic-type-in-the-numerical-honourability-contract` is superseded after its exact scalar-policy requirements are preserved here.
- Treat deleted Python documentation tooling as history. Documentation coherence is verified by reading exact local targets and current construction sites, not by naming a renderer or gate that no longer exists.
