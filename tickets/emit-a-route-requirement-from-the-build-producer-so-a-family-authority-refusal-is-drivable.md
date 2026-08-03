---
id: emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable
title: Emit a route requirement from the build producer so a family-authority refusal is drivable
status: awaiting-decision
priority: p2
dependencies: [accept-the-public-route-requirement-answer-boundary]
related: [realize-parallel-reduction-strategies-on-metal, declare-a-required-gpu-family-in-the-artifact, select-executable-variants-across-registered-backend-families, design-the-adapter-owned-route-requirement-answer-channel, admit-the-bf16-type-and-carrier-into-every-total-map, lower-bf16-to-metal]
scopes: [implementation/build, implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
tags: [implementation, artifacts, build, route-requirements, evidence-gap]
claimed_from: todo
assignee: agent-route-requirement
lease_expires_at: 1785786692
---
## User-visible outcome

A produced artifact carries the route requirement its payload actually needs of a device, so a consumer refusing on missing family or feature authority can be driven against a real produced artifact rather than only against a hand-built one.

## Why this exists

**Fact, reproducible in one line.** The build producer emits no route requirement at all:

```sh
git grep -n 'RouteRequirement' -- crates/tiler-build/src
```

returns nothing at `dc13abb`. A control pattern confirms the search works — `git grep -c 'DeviceFacts' -- prototypes/` returns hits — so this is an absence that was read, not a search that failed.

**Fact — the consumer side exists and the vocabulary is done.** [`declare-a-required-gpu-family-in-the-artifact`](declare-a-required-gpu-family-in-the-artifact.md) and [`select-executable-variants-across-registered-backend-families`](select-executable-variants-across-registered-backend-families.md) are both `done`. The refusals exist and are exercised device-free — `each_undecidable_route_requirement_refuses_by_its_own_class` and `a_foreign_owner_is_refused_without_consulting_an_adapter` — and the production offer path refuses on `metal.host-applicability.unknown-translation-authority` before any routing commit.

**Inference — so the gap is producer-side, and it is what keeps one negative fixture off hardware.** [`realize-parallel-reduction-strategies-on-metal`](realize-parallel-reduction-strategies-on-metal.md) drove three of its four required negative fixtures on the qualified host and recorded the fourth-of-three — missing family/feature authority — as not re-driven on hardware for exactly this reason: no produced artifact carries a required family, so there is nothing for a real device to be asked about. That ticket recorded the reason rather than converting a device-free case into hardware evidence, which is the substitution its own body forbids.

## Required work

- Decide what the Metal producer actually knows about the payload it emits that constitutes a requirement of a device, and emit that — not a placeholder row added so a test can pass. A requirement the producer cannot derive from the payload is a requirement it must not state.
- State the requirement in the **backend-neutral** vocabulary. Do not add Apple vocabulary to the neutral artifact; that constraint is the reason the route-requirement layer exists.
- Keep the one-way preparation/commit boundary: a requirement is checked before routing commit, never after allocation, partial encoding, or submission.

## Required evidence

- A produced artifact carries a route requirement, read back from the encoded artifact rather than from the builder's own value.
- A consumer on a device lacking the required family refuses **before routing commit**, driven on hardware, with the refusal quoted.
- A consumer on a device that satisfies it proceeds — so the check is not refusing unconditionally.
- Watch each refusal fail: perturb the device facts and the declared requirement independently, and confirm the refusal names the axis that actually moved.

## Identity discipline

Adding a field to the artifact's encoded form moves artifact identity. If it does, that is an identity step: move the ledger in the same commit, recompute every pinned identity on the tree the change lands into, and enumerate each moved pin in the report. If the requirement rides in an already-encoded optional position and moves nothing, say so and show the pinned values unchanged rather than asserting it.

## Explicit non-goals

The adapter-owned answer channel ([`design-the-adapter-owned-route-requirement-answer-channel`](design-the-adapter-owned-route-requirement-answer-channel.md)) and its public boundary ([`accept-the-public-route-requirement-answer-boundary`](accept-the-public-route-requirement-answer-boundary.md)). No new resource dimension — [`correct-the-subgroup-threads-route-dimension-meaning`](correct-the-subgroup-threads-route-dimension-meaning.md) and [`rename-the-route-resource-floor-vocabulary-for-its-corrected-relation`](rename-the-route-resource-floor-vocabulary-for-its-corrected-relation.md) own dimension changes.

## Closes when

A produced artifact carries a derived route requirement, a family-authority refusal is driven on hardware before routing commit with the refusal quoted, the positive case proceeds, each refusal has been watched failing under an independent perturbation, and any moved identity is enumerated.

## Graph maintenance

Filed 2026-08-02 at integration of `realize-parallel-reduction-strategies-on-metal`, which found the absence and recorded it rather than absorbing it.

## Dispatch outcome and correction — 2026-08-03

**The first dispatch conclusion was wrong and is withdrawn. A truthful requirement is already derivable: every emitted Metal entry requires Apple3-or-newer 64-bit integer math.** The first pass audited floating-point types and incorrectly concluded that no current emission used a family-scoped feature. That population omitted the governed index type, even though the same pass quoted it in the exhaustive `msl_type` map. The reviewer supplied the missing authority; the checks below reproduce it from construction through consumption.

**Fact — every region proposal requires complete unsigned-64 index arithmetic.** `crates/tiler-compiler/src/physical.rs:2060-2082` constructs every region feasibility proposal with `index_arithmetic_requirement(KernelType::Index)`, and `:2119` maps that exact type to `CapabilityAxis::IndexArithmeticU64`. This is unconditional beside the proposal's grid, workgroup, binding, device-memory, and local-memory requirements; it is not inferred from a particular semantic family.

**Fact — the emitted payload consumes the same governed family.** `crates/tiler-metal/src/emit.rs:261-266` declares the structured index type in every translation unit, `:811-818` maps `KernelType::Index` to `uint64_t`, and the production goldens exercise addition, multiplication, division, and modulo over it. Reproduce the population with `rg -n 'uint64_t' crates/tiler-metal/goldens/*.metal`: every current golden contains the explicit launch-index widening and the operation-bearing reduction and contraction goldens contain the governed arithmetic. This is payload content, not a profile-name inference.

**Fact — the minimum family is sourced and is lower than the measured profile label.** `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md:90-97` reads Apple's 2025-10-20 Metal Feature Set Tables row `64-bit integer math` as `Metal3 | Apple3 | —`: minimum Metal version 3, minimum Apple family 3, and no Mac-family guarantee. The ledger explicitly says the Mac artifact family does not imply support and that the Apple-family applicability predicate bounds the compile-profile claim. The producer can therefore truthfully state `minimum-gpu-family = Apple3`; it must not copy `Apple9` from the measured profile key or `Apple8` from ADR 0092's illustrative program.

**Fact — the production construction gap is exactly located.** `accept_or_publish_metal_plan` calls the same neutral `assemble_plan_artifact` three times — pending, cache-miss carried, and read-back carried — at `crates/tiler-build/src/metal_plan.rs:283-346`. That function's complete builder sequence is `crates/tiler-build/src/plan_artifact.rs:152-227`: providers, payloads, compiler-minted prepared-entry predicates, entries, `push_variant`, and `build`; it never calls `require_route`. Reproduce with `rg -n 'assemble_plan_artifact|require_route' crates/tiler-build/src/{metal_plan,plan_artifact}.rs`, reading the production function rather than counting its later tests.

**Fact — the existing neutral consumer checks the row before commitment.** `crates/tiler-runtime/src/load.rs:759-782` binds each decoded row and refuses a foreign owner; `crates/tiler-runtime/src/load/route.rs:439-474` resolves every row and returns `Unowned`, `Misanswered`, or `Unsatisfied` before producing `RoutePreparation`; routing commitment is only available later at `route.rs:740`.

### Elimination

**Rely on compile-profile feasibility alone — eliminated on correctness.** The compiler proved the route against a declaration whose `CompleteU64` row is valid only for Apple3-or-newer. A macOS artifact family says nothing about that predicate because the normative Mac-family column is absent. Without a live-device row, a Mac device outside the Apple3 validity scope can reach pipeline creation carrying operations the plan assumed available.

**Infer the requirement at runtime from the target-profile key or payload provenance — eliminated on correctness and ownership.** A profile key is a producer declaration, not a device observation; its `apple9` spelling would overstate the actual Apple3 payload minimum. The profile descriptor is intentionally comparable rather than readable, and provenance explains what produced bytes rather than what a live device must support. Decoding either into a capability would make the check a tautology over producer-supplied facts.

**Hard-code an unconditional Apple check beside the consumer — eliminated on maintainability and the accepted architecture.** It duplicates the feature threshold outside the payload that consumes it, cannot distinguish routes with different needs, and makes every adapter restate the governed key, version, payload codec, and ordering. ADR 0092 items 1-4 explicitly keep opaque bytes in the neutral artifact and give the owning backend the mint/decode/ordering authority.

**Add a neutral numeric resource floor — eliminated by the accepted ADR.** Apple-family ordering is backend vocabulary, not a quantity. ADR 0092's neutralization elimination names the lexicographic `Apple10 < Apple9` counterexample and rejects moving the backend's rank table into neutral data.

**Mint `RouteRequirement::BackendFeature` through the owning `tiler-metal` vocabulary and resolve it from the bound device before commitment — survives.** It carries the exact Apple3 threshold derived from the verified KIR/emitted payload, keeps Apple vocabulary opaque to `tiler-artifact`, and uses the loader's already-accepted pre-commit comparison.

### Exact decision now blocking reachable work

ADR 0092 accepted this model and no public shape. Its status paragraph and implementation boundary explicitly reserve the minting constructor, the governed key/version ownership, the decision function/answer type, and the consumer-nameable `tiler-metal` surface for Tom. `rg -n 'BackendFeatureRequirement::new|minimum-gpu-family' crates/tiler-metal/src` still returns zero matches at `2fdb71e`. Moreover, `MetalGpuFamily` currently starts at `Apple5`, while the sourced requirement is Apple3, so the accepted shape must say how the owning vocabulary represents and compares the lower threshold rather than asking `tiler-build` to spell bytes directly.

[`accept-the-public-route-requirement-answer-boundary`](accept-the-public-route-requirement-answer-boundary.md) is therefore a dependency, not merely a related design record, and its "first compiler-minted route requirement" trigger is now concrete. The minimum decision slice this producer needs is: **accept or revise a `tiler-metal` minting API that owns the key/version and can express the sourced Apple3 threshold; the complete accepted packet must also settle the matching decoder/answer surface before a dispatching consumer can proceed instead of correctly returning `Unrecognized`.** This ticket does not self-accept either surface or widen the public family vocabulary under an implementation claim.

### Reachable work after the decision

With the boundary accepted, the implementation can derive one Apple3 minimum-family row from the invariant IndexArithmeticU64 requirement, attach it to all three pending/carried assembly paths, read it back from encoded bytes, and perturb the declared requirement independently of device facts. The positive case is available on the Apple9 host. A truthful hardware-negative case still needs either a device below Apple3 or a separately admitted higher-family perturbation the Apple9 host genuinely lacks; a synthesized device fact remains device-free evidence and must not be relabelled as hardware.

BF16 carrier and lowering remain related but are not prerequisites for this row: the index requirement already exists. BF16 may reveal an additional family-scoped requirement only if its own primary-source or measurement work establishes one; BF16 support alone does not change or widen the Apple3 index row.
