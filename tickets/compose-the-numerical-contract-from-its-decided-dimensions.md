---
id: compose-the-numerical-contract-from-its-decided-dimensions
title: Compose the numerical contract from its decided-independent dimensions
status: review
priority: p1
dependencies: []
related: [realize-parallel-reduction-strategies-on-metal, calibrate-and-activate-parallel-reduction-selection, package-a-multi-entry-bundle-from-one-expansion, decide-the-inline-frontend-numerical-contract, restore-the-spikes-against-the-composed-numerical-contract]
scopes: [implementation/compiler, contracts/numerics, implementation/frontend, implementation/build, implementation/metal-aot, implementation/runtime, contracts/artifacts, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, public-boundary, contracts]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785626991
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

## Scopes added while implementing, with the verification

**Fact — 2026-08-01, read from `ticketsplease.toml` and `tkt claims` on the working tree.** Removing the public enum breaks every call site that named a variant, and four of them are workspace members whose build the gate covers, so the scopes below were added rather than the change left half-landed. Each edit is the same one-token rename from a variant to the associated constant of the composed type.

- `implementation/frontend` — `crates/tiler-macros/src/{aot.rs,aot/tests.rs,region.rs}`. Held by `package-a-multi-entry-bundle-from-one-expansion`, which is `blocked`, depends on this ticket, and has an expired lease; `git diff --name-only 2119b20...tkt/package-a-multi-entry-bundle-from-one-expansion` resolves to no branch, so there is no worker diff to be disjoint from.
- `implementation/build` — `crates/tiler-build/**`. Unheld; `tkt claims` names no live holder.
- `implementation/metal-aot` — `prototypes/serial-sum-compile/src/main.rs`. Unheld.
- `implementation/runtime` — `prototypes/serial-sum-run/src/proof.rs`, two sites (one doc line, one call). Held live by `route-or-refuse-the-device-translation-execution-policy`. File-level disjointness verified against that worker's actual branch: `git diff --name-only 2119b20...tkt/route-or-refuse-the-device-translation-execution-policy` printed nothing, so its branch is at the base and touches no file this branch does. That check is a point in time and is recorded as such.
- `contracts/artifacts` — `docs/artifact-abi.md`, the identity-ledger row and the domain-step paragraph. Unheld.
- `contracts/navigation` — `docs/status.md`, two sentences: the identity ledger and a stale public type name. Held live by `land-the-two-level-reduction-adr`. Verified disjoint against its actual branch diff, which is `docs/decisions/0096-*.md`, `docs/decisions/README.md`, `docs/research/README.md`, `docs/research/scheduling/two-level-subgroup-workgroup-reduction.md`, `tickets/accept-adr-0096-two-level-reduction.md`, and `tickets/land-the-two-level-reduction-adr.md` — `docs/status.md` is not among them.

`spikes/` was deliberately **not** edited: three spikes name a removed variant, no `make` target reaches them, and one of the three is the live subject of `restore-the-scalar-cpu-vertical-spike-against-the-current-crates`. `restore-the-spikes-against-the-composed-numerical-contract` carries the exact sites.

## Closes when

The composed contract type is the public boundary and Tom has accepted its exact surface; the four presets are constants of it and the enum is gone; the dimension-vector identity is canonical with the ledger moved and every pin recomputed and enumerated; incoherent combinations refuse by name with the enumeration stated; the flush-and-reassociate combination reaches a portfolio on the authoritative Metal profile with the activation-trigger test flipped to the positive claim; and targeted checks plus `make full` pass.

## What landed, and what is outstanding

**Delivered.** The composed type, its builder, the five named constants, the canonical injective key under `tiler.contract.f32.v2`, the coherence enumeration with its one survivor and five stated eliminations, the ledger rows in `docs/artifact-abi.md` and `docs/status.md`, the statable-exceeds-tested boundary in `docs/numerical-semantics.md`, and the positive Metal claim as `a_flush_and_reassociate_contract_reaches_a_parallel_portfolio`.

**Fact — a second compiler defect was on the path and is fixed here.** The host ABI declared one literal `1` as every stage's workgroup width and reused whichever element count happened to equal a stage's work items as its grid. Both hold for a region running one independent invocation per result element and both are false for a single-workgroup tree, so the first tree to reach a kernel program failed the whole compilation with `ThreadsPerWorkgroupDisagreement { expected: 2, actual: 1 }`. `HostAbi::launch` now reads both quantities from the schedule the stage lowers, and `a_cooperative_region_declares_its_own_launch` pins it, watched failing against the restored literal. It is fixed here rather than filed because the composed contract's own closing evidence runs through it.

**Outstanding, and Tom's.** The exact public Rust surface is a draft under ADR 0075 and is not self-accepted. `decide-the-inline-frontend-numerical-contract` carries the frontend choice this work activated. `restore-the-spikes-against-the-composed-numerical-contract` carries the three spikes left naming a removed variant.

**The two pinned identities in `crates/tiler-build/src/metal_plan.rs` are deliberately not rebaselined.** `ARTIFACT_IDENTITY` and `CACHE_SUBJECT` both move — the contract key is folded through every identity beneath them — but a sibling branch may move the same pins from its own base, and two branch-local rebaselines cannot compose. The integrator recomputes them on the merged tree; the enumeration and the observed values are in this branch's report.
