---
id: compose-the-numerical-contract-from-its-decided-dimensions
title: Compose the numerical contract from its decided-independent dimensions
status: todo
priority: p1
dependencies: []
related: [realize-parallel-reduction-strategies-on-metal, calibrate-and-activate-parallel-reduction-selection, package-a-multi-entry-bundle-from-one-expansion]
scopes: [implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, public-boundary, contracts]
---
## User-visible outcome

A caller states a numerical contract by resolving its dimensions directly — subnormal handling, reassociation, permutation, and the rest of the typed vocabulary — instead of choosing from a closed list of named presets, so a hardware or workload corner no preset names is a stated combination rather than a filed blocker. The flush-and-reassociate combination every parallel reduction strategy on Apple hardware needs is the first consumer.

## Decision provenance

**Tom decided the direction on 2026-08-01, in the live session, relayed here by the coordinator who witnessed it.** Presented with a fifth preset, a composable interface, and an internal-only registration, he chose the composable interface, on the maintainability ground the elimination supports: the corpus has already decided the axes independent three times — [ADR 0011](../docs/decisions/0011-per-operation-numerical-permissions.md) holds one permission never implies another, [ADR 0014](../docs/decisions/0014-reassociation-vs-permutation.md) split the order dimensions on evidence, [ADR 0080](../docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) added a third independent dimension — and the target side already declares honourability and refuses per dimension, so the preset enum was the one point-shaped surface left in the stack, and it produced its predictable failure the first time real hardware needed an unnamed corner. **What is decided is the direction, not the Rust surface:** the exact public boundary comes back to Tom as a concrete draft under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) before acceptance.

## The shape decided

- The public `NumericalContract` enum is replaced outright by a composed contract type whose fields are the existing typed dimension enums. Pre-alpha, no external consumers: the superseded enum is removed, not deprecated.
- **Omission defaults to strict.** An unstated dimension resolves to its strict resolution, so omission never widens a contract — the fail-closed default.
- The four existing presets survive as named constants or constructors of the composed type, keeping their documentation value without the enum; `tiler.strict-f32.v1` and siblings remain the retained named points of the space.
- **Identity is a canonical, injective encoding of the dimension vector** as the governed contract key, replacing the four hand-named strings. This is an identity-domain change and carries the full ledger discipline: the version moves at its owning layer, the ledger documents move in the same commit, and every pinned identity is recomputed on the landing tree with each moved pin enumerated.
- **Coherence is enumerated, not discovered.** Any genuinely self-contradictory combination (candidates live among the exceptional-value-assumption interactions) is refused by name at construction; the design states the enumeration and its derivation so a reader can refute it.
- Statable will exceed tested, permanently, and the mitigation is the architecture's existing one: feasibility gates every dimension against measured target rows, so an unmeasured combination fails closed at the target with a typed refusal. State this boundary in the design rather than leaving it implicit.

## Carried obligations from the superseded ticket

- `crates/tiler-build/src/metal_plan.rs`'s `no_registered_contract_both_flushes_subnormals_and_permits_reassociation` is the activation trigger and fails the moment the combination becomes statable; replace it with the positive claim, keeping its two-halves record of what the gap was.
- The required evidence stands unchanged in substance: the flush-and-reassociate combination compiles a reassociating reduction against `BoundMetalCompileDeclaration::first_macos_apple9`, the portfolio retains the multi-pass split and single-workgroup tree beside the serial fold, and the reference oracle agrees with each retained alternative at its own declared order.
- `calibrate-and-activate-parallel-reduction-selection`'s stated measurement target names an environment/contract pair that cannot both hold today; correct it to name the composed combination in the same change.

## Closes when

The composed contract type is the public boundary and Tom has accepted its exact surface; the four presets are constants of it and the enum is gone; the dimension-vector identity is canonical with the ledger moved and every pin recomputed and enumerated; incoherent combinations refuse by name with the enumeration stated; the flush-and-reassociate combination reaches a portfolio on the authoritative Metal profile with the activation-trigger test flipped to the positive claim; and targeted checks plus `make full` pass.
