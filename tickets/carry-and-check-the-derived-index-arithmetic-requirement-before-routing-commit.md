---
id: carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit
title: Carry and check the derived index-arithmetic requirement before routing commit
status: todo
priority: p1
dependencies: []
related: [emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable, check-synchronization-realization-before-the-routing-commit, separate-metal-launch-index-from-index-and-address-width, declare-a-required-gpu-family-in-the-artifact]
scopes: [implementation/ir, implementation/artifact, implementation/compiler, implementation/metal, implementation/runtime, implementation/build, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, fail-closed, index-arithmetic, artifact-schema, metal, public-boundary]
---
## User-visible outcome

A delivered Metal artifact carries the verified program's complete unsigned-64 index-arithmetic requirement once, as part of its derived dispatch/resource record, and a device outside the sourced Apple-family support refuses before routing commitment instead of reaching pipeline creation.

## Why this is a direct requirement and not a route row

**Fact — construction.** `crates/tiler-compiler/src/physical.rs:2060-2082` adds `index_arithmetic_requirement(KernelType::Index)` to every region proposal, and `:2119` maps that governed type to `CapabilityAxis::IndexArithmeticU64`. `crates/tiler-metal/src/emit.rs:261-266,811-818` declares and spells the same type as `uint64_t`; all six files under `crates/tiler-metal/goldens/*.metal` carry the explicit widening, and the operation-bearing goldens exercise addition, multiplication, division, and modulo.

**Fact — authority.** `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md:90-97` reads Apple's 2025-10-20 Metal Feature Set Tables row `64-bit integer math` as `Metal3 | Apple3 | —`: MSL 4.0 clears the language-version half, Apple3 is the minimum Apple family, and no Mac-family guarantee exists. A macOS artifact family or spellable `uint64_t` therefore cannot discharge the live-device half.

**Fact — accepted ownership rule.** [`declare-a-required-gpu-family-in-the-artifact`](declare-a-required-gpu-family-in-the-artifact.md):23-25 and [`docs/artifact-abi.md`](../docs/artifact-abi.md):280 require facts already derivable from the verified program to remain direct requirements. Copying one into `RouteRequirement::BackendFeature` creates a second producer authority that can contradict the program. This ticket is the direct-check owner; [`emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable`](emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable.md) is deferred until emission has an *additional*, non-derivable feature.

## The missing mechanism, traced exactly

**The producer has the requirement and the decoded consumer loses its interpretable form.** `ResourceRequirements` at `crates/tiler-ir/src/schedule/model.rs:846-887` is the verified schedule's derived direct-requirement record. Artifact model/codec code encodes it (`crates/tiler-artifact/src/program/model.rs:2085`, `codec/encode.rs:443`), decodes it (`codec/decode.rs:999`), and publishes it through `DecodedEntry::resources()` (`codec/view.rs:673`). Its complete fields cover bindings, workgroup threads, local memory, device-memory use, synchronization, and numerical dimensions; none states index arithmetic. The envelope does not carry KIR operations, so a consumer holding decoded bytes cannot reconstruct the omitted `KernelType::Index` derivation.

**The backend has the observation but no comparison seam for this direct requirement.** `prototypes/serial-sum-run` already observes the device's Apple family and directly checks decoded accessible windows and `local_memory_bytes` before commitment. It has no decoded index requirement to check. `tiler-metal::applicability` owns the family observation vocabulary, but `MetalGpuFamily` currently begins at Apple5 while the normative threshold is Apple3, and its public items remain reviewed drafts rather than an accepted consumer boundary.

## Elimination

- **Compile-profile feasibility only:** eliminated; the profile is a producer declaration, not a fact about the bound device.
- **Infer from target-profile key, descriptor, or provenance:** eliminated; the key's Apple9 label overstates Apple3, the descriptor is comparable rather than readable, and provenance is not device evidence.
- **Copy into a backend route row:** eliminated by the accepted derivability rule; two independently editable statements could disagree about one KIR requirement.
- **Teach neutral runtime about Apple families:** eliminated by backend neutrality and ADR 0092's ordering counterexample.
- **Carry one nominal index-arithmetic requirement in the verified resource/dispatch record and let the Metal adapter map that neutral requirement to its live device vocabulary:** survives. It preserves one producer authority and one backend observation authority.

## Public and schema boundary — Tom's stop

The exact carrier is consequential: adding a public field or nominal type to `ResourceRequirements` changes a cross-crate public record; encoding it changes the artifact manifest/schema and canonical identity; publishing a reusable Metal comparison must decide how an Apple3 threshold is represented when the current public family vocabulary begins at Apple5. These choices survive the correctness elimination but their exact public shapes do not follow mechanically. Tom must accept or revise them before implementation. A tested draft may be prepared only if separately authorized; this ticket does not treat ADR 0092's accepted backend-ownership model as acceptance of these interfaces.

## Required implementation after the decision

- Add the accepted nominal index-arithmetic requirement to the verified schedule/resource record and derive it from the governed KIR type rather than restating a raw width.
- Encode, decode, validate, and expose it as one direct entry requirement. Execute the artifact identity step whole: advance the owning manifest/component/artifact domains, update the artifact ledger in the same commit, and recompute every affected pin on the landed tree.
- Add the owning Metal comparison from `CompleteU64` to the sourced Apple3-or-newer live-device predicate without putting Apple vocabulary in the neutral artifact/runtime layers.
- Check every selected entry before pipeline preparation or routing commitment, beside the direct accessible-window and local-memory checks. Unknown or unsupported device authority refuses with a typed cause naming index arithmetic.
- Keep compiler feasibility and runtime validation distinct: the former proves the plan against its declared target profile; the latter proves the bound device satisfies the derived route.

## Required evidence

- Encoded-artifact readback shows the derived requirement, not a builder-local value.
- An Apple9 positive case clears the direct check and reaches the existing preflight neighbour.
- A device-free lower-family/unknown observation refuses before commitment with the exact index-arithmetic cause; perturb both the carried requirement and device observation independently and watch each check fail.
- Do not claim a hardware-negative family run without a device below Apple3. Both currently reachable measured devices are Apple9.
- Prove the route-row population stays zero for a program with no additional emitted feature.
- Targeted IR, artifact, compiler, Metal, build, and runtime checks; Clippy and doc-tests; `tkt lint`; true-base guard; then one full gate on the integrated identity step.

## Graph maintenance

- Related to [`check-synchronization-realization-before-the-routing-commit`](check-synchronization-realization-before-the-routing-commit.md), which has the same pre-commit placement but already carries its own whole synchronization subject and uses a different comparison authority. Neither subsumes the other.
- Related to [`separate-metal-launch-index-from-index-and-address-width`](separate-metal-launch-index-from-index-and-address-width.md), whose separation and Apple-family authority this completes at delivery time without recombining launch width, arithmetic, and address width.
- Do not unblock or accept [`accept-the-public-route-requirement-answer-boundary`](accept-the-public-route-requirement-answer-boundary.md): no backend route row is minted here.

## Authorized — tested draft, 2026-08-05

Tom authorized the tested-draft route at the live decision review in the coordination session, witnessed first-hand by the coordinator: a worker builds the surviving candidate's exact shapes — the nominal requirement on the verified resource record, the artifact schema/identity step, and the Metal adapter's Apple3-threshold comparison against a family vocabulary that starts at Apple5 — with tests, and the exact surface returns to Tom for acceptance before it is treated as accepted. This discharges this ticket's "only if separately authorized" condition and nothing else; the public shapes remain Tom's to accept.
