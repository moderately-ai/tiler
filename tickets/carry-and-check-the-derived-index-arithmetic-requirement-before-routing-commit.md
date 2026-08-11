---
id: carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit
title: Carry and check the derived index-arithmetic requirement before routing commit
status: todo
priority: p1
dependencies: []
related: [emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable, check-synchronization-realization-before-the-routing-commit, separate-metal-launch-index-from-index-and-address-width, declare-a-required-gpu-family-in-the-artifact]
scopes: [implementation/ir, implementation/artifact, implementation/compiler, implementation/metal, implementation/runtime, implementation/build, contracts/artifacts, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, fail-closed, index-arithmetic, artifact-schema, metal, public-boundary, decision, needs-tom]
---
## User-visible outcome

A delivered Metal artifact carries the verified program's complete unsigned-64 index-arithmetic requirement once, as part of its derived dispatch/resource record, and a device outside the sourced Apple-family support refuses before routing commitment instead of reaching pipeline creation.

## Why this is a direct requirement and not a route row

**Fact — construction. Line citations corrected 2026-08-08 at base `209013bd`; every claim's substance held and every line number was wrong.** (Historical pre-landing construction shape.) `crates/tiler-compiler/src/physical.rs` added `index_arithmetic_requirement(KernelType::Index)` to every region proposal at **`:3957`**, not `:2060-2082`, and mapped that governed type to `CapabilityAxis::IndexArithmeticU64` at **`:4002-4004`**, not `:2119` — a drift of roughly +1,890 lines. `crates/tiler-metal/src/emit.rs` declares the type at **`:297`** and spells it `uint64_t` at **`:988`**, not `:261-266,811-818`. And the golden population is **ten** files, not six: all ten of `crates/tiler-metal/goldens/*.metal` carry the explicit widening, and the operation-bearing ones exercise addition, multiplication, division, and modulo.

**Correction — 2026-08-10.** The KernelType call form above is pre-landing and is **not** live. `region_proposal` now takes `requirements: ResourceRequirements` and calls `index_arithmetic_requirement(requirements.index_arithmetic)`; the helper is `const fn index_arithmetic_requirement(index_arithmetic: IndexArithmetic)` matching only `IndexArithmetic::CompleteU64 → CapabilityAxis::IndexArithmeticU64`. Emission still spells KIR Index as `uint64_t` (`KernelType::Index => Ok("uint64_t")` under `// Structured index arithmetic:`); all ten goldens still widen explicitly. Anchors, not line numbers. Reproduce: `rg -n 'index_arithmetic_requirement\\(requirements\\.index_arithmetic\\)|KernelType::Index => Ok\\("uint64_t"\\)|Structured index arithmetic' crates/tiler-compiler/src/physical.rs crates/tiler-metal/src/emit.rs`.

**Fact — authority. Line citation corrected 2026-08-08: the row is at `:108`, not `:90-97`; the substance held exactly, including the labelled operation-completeness inference at `:111`.** `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md` section `### Index arithmetic — \`CompleteU64\`` reads Apple's 2025-10-20 Metal Feature Set Tables row `64-bit integer math` as `Metal3 | Apple3 | —`: MSL 4.0 clears the language-version half, Apple3 is the minimum Apple family, and no Mac-family guarantee exists. A macOS artifact family or spellable `uint64_t` therefore cannot discharge the live-device half.

**Fact — accepted ownership rule. Verified 2026-08-08, with one citation corrected.** [`declare-a-required-gpu-family-in-the-artifact`](declare-a-required-gpu-family-in-the-artifact.md) states the structural rule exactly. The historical `docs/artifact-abi.md:280` citation named the `bf16` carrier-table Fact, not the derivability rule; live prose is under the live-device route-requirements section (`A row belongs only when the selected route consumes it *and* the verified program does not already state it`; index arithmetic `mints no route row`). Both sources require facts already derivable from the verified program to remain direct requirements. Copying one into `RouteRequirement::BackendFeature` creates a second producer authority that can contradict the program. This ticket is the direct-check owner; [`emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable`](emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable.md) is deferred until emission has an *additional*, non-derivable feature.

## The missing mechanism, traced exactly

**Historical pre-landing gap (filing-time problem statement; not live).** **The producer has the requirement and the decoded consumer loses its interpretable form.** (Citations corrected 2026-08-08; every claim's substance held as of that pre-landing audit.) `ResourceRequirements` was then described at `crates/tiler-ir/src/schedule/model.rs:1161-1203` — not `:846-887` — as the verified schedule's derived direct-requirement record. Artifact model/codec code encodes it (`push_resources`), decodes it, and publishes it through `DecodedEntry::resources()` (`pub fn resources`). The paths were given as `codec/…` rather than `program/codec/…` and every line was wrong. Its fields at that time covered bindings, workgroup threads, local memory, device-memory use, synchronization, and numerical dimensions; none stated index arithmetic. The envelope did not carry KIR operations, so a consumer holding decoded bytes could not reconstruct an omitted `KernelType::Index` derivation.

**Correction — 2026-08-10.** That absence is closed. `ResourceRequirements` carries non-optional `pub index_arithmetic: IndexArithmetic`, derived once as `REGION_INDEX_ARITHMETIC` in `derive_requirements`, encoded via `push_resources` / `index_arithmetic_tag`, decoded into the resources run, and published by `DecodedEntry::resources()`. The compiler classifies the carried `IndexArithmetic` rather than re-deriving from `KernelType`. Delivered truth is the Outcome below. Reproduce: `rg -n 'pub index_arithmetic: IndexArithmetic|REGION_INDEX_ARITHMETIC|push_resources|pub fn resources' crates/tiler-ir/src/schedule/model.rs crates/tiler-artifact/src/program`.

**The backend has the observation but no comparison seam for this direct requirement.** (Pre-landing observation; discharge landed under Outcome.) `prototypes/serial-sum-run` already observed the device's Apple family and directly checked decoded accessible windows and `local_memory_bytes` before commitment. It had no decoded index requirement to check. `tiler-metal::applicability` owns the family observation vocabulary, but `MetalGpuFamily` begins at Apple5 while the normative threshold is Apple3, and its public items remain reviewed drafts rather than an accepted consumer boundary.

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

## Outcome — built 2026-08-08, tested draft, awaiting Tom's acceptance of the surface

A delivered Metal artifact now carries the verified program's index-arithmetic requirement once, as part of its derived dispatch record, and the Metal adapter compares it against the bound device before any pipeline is created.

**The shape that landed.** `tiler_ir::schedule::IndexArithmetic` is a nominal, exhaustive, single-variant vocabulary (`CompleteU64`), carried as a non-optional `ResourceRequirements::index_arithmetic`. Non-optional deliberately: every scheduled region computes coordinates, so a region deriving its absence does not exist, and an `Option` would add an unreachable absence a producer could encode to skip the device check. It is derived once — `REGION_INDEX_ARITHMETIC`, a constant of the scheduled-region layer's own `u64` coordinate space — and bound to the KIR index role by a `const` assertion in `crate::kernel::model`, the only module that can see both. `tiler-compiler`'s `index_arithmetic_requirement` stopped re-deriving from `KernelType` and now classifies the value it receives, removing the second producer authority inside the compiler.

**The Apple3 threshold is represented without touching the observation vocabulary, and the ticket's authorization is what decided that.** The authorization scopes the draft to "a family vocabulary that starts at Apple5". An earlier attempt widened `MetalGpuFamily` down to `Apple3`/`Apple4`; it broke `crates/tiler-conformance/src/dispatch.rs`'s `MetalGpuFamily::COUNT == 5` assertion — a crate this ticket does not own, with a live sibling branch — and was reverted. `tiler_metal::direct_requirement::AppleFamilyFloor` is instead its own type below the observation vocabulary. Any named family clears the floor, licensed by a `const` assertion rather than by reading the enum; `NoneNamed` is genuinely `Unknown` against an `Apple3` floor and refuses under ADR 0043's disposal rather than being reported as an unsupported device.

**Identity step, whole.** `tiler.kernel.v6`→`v7`, `tiler.artifact-program.v15`→`v16`, manifest schema `15.0`→`16.0`. `docs/artifact-abi.md` carries the step's Fact, its measurement, and the corrected ledger row in the same commit.

**Recomputed pin table.** Nine pins were checked; seven moved.

| pin | site | before | after |
| --- | --- | --- | --- |
| `KERNEL_DOMAIN` | `crates/tiler-ir/src/kernel/model.rs` | `tiler.kernel.v6` | **`tiler.kernel.v7`** |
| kernel-domain test pin | `crates/tiler-ir/src/kernel/tests.rs` | `tiler.kernel.v6\0` | **`tiler.kernel.v7\0`** |
| `ARTIFACT_DOMAIN` | `crates/tiler-artifact/src/program/model.rs` | `tiler.artifact-program.v15` | **`…v16`** |
| artifact-domain test pins | `…/codec/tests.rs`, `crates/tiler/src/route/tests.rs` | `…v15\0` | **`…v16\0`** |
| `MANIFEST_SCHEMA` | `…/codec/encode.rs` + its test | `(15, 0)` | **`(16, 0)`** |
| `DIFFERING_CARRIER_POSITIONS` | `…/codec/tests.rs` | `67` | **`68`** |
| standard Metal artifact identity | `crates/tiler-build/src/metal_plan.rs` | `e16ce9264f7f4fe6…` | **`39e765637a7e014adac2b8a30788798758ca46584b558732c2bda41b7639ddda`** |
| standard Metal cache subject | `crates/tiler-build/src/metal_plan.rs` | `287df9823c146b71…` | **`7e00d9fa0ce90749e6f7d3d42e0f2aaabe5670e0359a0c20d1580a09bb967130`** |
| `FIXED_CONTENT_BYTES` | `crates/tiler-build/src/metal_plan.rs` | `65_308` | **`65_313`** |

Unmoved by *this* identity step, and checked rather than assumed at landing: `tiler.schedule.v5` and `STRICT_F32_REGION_IDENTITY_HEX` (requirements are not folded into the scheduled-region identity — confirmed by every golden's `scheduled region identity digest` line being byte-identical while the kernel digest moved); `tiler.kernel-program.v11` and `tiler.artifact-program.stage.v3` (their grammars are unchanged; only folded content moved); `tiler.semantic-graph.v3`; the explain request qualifier (landing-time hex `940c09e0821665a6`; this step did not move it — later work rebaselined the live pin to `7ba3d77a66f04638`); the `index/law.rs` chain byte counts and digests; `POPULATION = 649`.

**`DIFFERING_CARRIER_POSITIONS` returning to 68 is coincidence, not a revert.** The four tag positions and sixty-four digest positions never moved; only how many digest bytes coincide did. Recorded at the constant and in `docs/artifact-abi.md`.

**The +5 envelope growth is decomposed, not accepted as a delta.** Measured by byte-aligning against an envelope built at `209013bd`: manifest 41,113→41,116, non-object sections 24,134→24,136. Five insertions of the literal `0x01` tag — one entry-row `resources`, four embedded kernel identities. Five and not six is the property `FIXED_CONTENT_BYTES` pins; six would mean one record encodes the requirement twice.

**Deliberate failures, each perturbing the subject.** (1) Moving the floor enumerator from `1003` to `1006` fails the build at the `const` assertion that licenses the admit-without-comparing arm. (2) Making `NoneNamed` admit instead of refuse fails three `direct_requirement_tests`. (3) Deleting the pre-pipeline check from the routing path *only warned*, and `prototypes/` is excluded from the style gate by design — so no gate would have gone red. That is why `DirectRequirementsDischarged` exists: a witness with a private field, minted only inside the `discharge` module, consumed by value by `prepare_pipelines`. Re-running the perturbation now fails with `E0423`. Introducing it also surfaced a **second, unchecked** `prepare_pipelines` call site in the device-probe harness, which the witness forced to discharge the requirement too.

**Live evidence, Apple M4 Max (Apple9), macOS 27.0 build 26A5388g, arm64.** `cargo run -p tiler-prototype-run -- --artifact …` prints `the derived index-arithmetic requirement clears on this device: family Apple9` immediately before `the unperturbed route prepares: every stage cleared before the commit`, and the whole proof matrix passes: 30 cases across 6 members, 6 contraction cases across 2 members. No hardware-negative family run is claimed; both reachable devices are Apple9, and every negative case is device-free.

**Route-row population is zero, asserted.** `the_standard_metal_path_publishes_its_recorded_identities` now checks the variant count is one and that variant's `route_requirements()` is empty. `accept-the-public-route-requirement-answer-boundary` is neither unblocked nor accepted.

## Scope added 2026-08-08: `implementation/frontend`

Scheduling metadata, and the narrowest possible reason. `crates/tiler/src/route/tests.rs` restates the artifact identity domain as its own `IDENTITY_DOMAIN` constant — deliberately, its own comment explains, because the restatement is self-detecting — so the `v15`→`v16` step falsifies it and every case in that file that expects to get past the identity restatement fails with `MalformedRouteFacts`. It is a mandatory pin of this identity step, not a widening of the work: **one file, one line, one domain string**, and nothing else under `crates/tiler/**` or `crates/tiler-macros/**` is touched.

`tkt guard` reports a *direct* declared-area collision with `correct-the-stale-fallbackonly-claims-in-tiler-macros-family-cfg`, which shares the scope glob. The two branches touch **disjoint files** — that ticket is in `crates/tiler-macros/src/family_cfg.rs` and this one is in `crates/tiler/src/route/tests.rs` — so the overlap is the glob's granularity rather than a real conflict, and it is a non-failing WARN under this repository's `gate_collisions = false`. Flagged here rather than left for the integrator to rediscover.

## Draft public boundary — Tom's acceptance required

New surface, all of it a labelled ADR 0075 draft in `tiler_metal::direct_requirement`:

- **Included:** `AppleFamilyFloor` (exhaustive, `Apple3`) and `AppleFamilyFloor::apple_constant_value`; `minimum_gpu_family`; `MetalIndexArithmeticRefusal` (`#[non_exhaustive]`: `UndecidableBelowVocabulary`, `Unobserved`) with `rule`, `required`, `floor`, `Display`, `Error`; `evaluate_index_arithmetic`.
- **Excluded, deliberately:** `evaluate_against` is `pub(crate)` — a caller choosing its own floor would be a second authority over what an arithmetic requires. No `BelowMinimumFamily` variant is published, because no reachable floor can produce one; `#[non_exhaustive]` admits it when a floor rises above `Apple5`.
- **In `tiler_ir::schedule`:** `IndexArithmetic` (exhaustive, `CompleteU64`) and the new public field `ResourceRequirements::index_arithmetic`. `IndexArithmetic::of` is a public method on that schedule type; its `impl` lives in `crates/tiler-ir/src/kernel/model.rs` (the module that can see both schedule and KIR). `REGION_INDEX_ARITHMETIC` is `pub(crate)` and is not surface.
- **Unchanged:** `MetalGpuFamily` and everything in `tiler_metal::applicability`.

**Recommendation: accept the draft as built.** The non-optional neutral `CompleteU64` requirement states the one arithmetic every scheduled region already derives, while `AppleFamilyFloor` keeps Apple vocabulary in the Metal-owned comparison and prevents callers from minting their own threshold. The private witness makes removal of the pre-pipeline discharge a compile error. **Strongest counterpoint:** the single-variant public vocabularies reserve names before a second arithmetic or floor is reachable, and `AppleFamilyFloor` sits beside an observation vocabulary that begins at Apple5; Tom may prefer a private comparison until a second floor exists. That would require revising the public included set without removing the carried artifact requirement or the pre-commit check.

## Accepted with visibility narrowing — 2026-08-11

**Decision.** Tom accepted the exact surface below in the Codex coordination thread by replying `sounds good, accept` to the coordinator's decision packet. The relay source is Tom's direct response in that thread. The accepted surface preserves the tested draft's requirement, comparison, refusal, witness, artifact identity, and manifest schema, while narrowing two unused derivation details before the boundary is labelled accepted in source. This moves the ticket to `todo`; it is not complete until those visibility changes and their gates land.

The accepted public surface is:

- exhaustive `tiler_ir::schedule::IndexArithmetic`, currently `CompleteU64`;
- non-optional `ResourceRequirements::index_arithmetic`;
- exhaustive `tiler_metal::direct_requirement::AppleFamilyFloor`, currently `Apple3`;
- `minimum_gpu_family` as the one readable map from a neutral arithmetic to its sourced Apple-family floor;
- non-exhaustive `MetalIndexArithmeticRefusal`, its `rule`, `required`, and `floor` accessors, and its `Display` and `Error` implementations; and
- `evaluate_index_arithmetic` as the sole public comparison of a carried requirement against a normalized live-device observation.

The following remain or become crate-private:

- `evaluate_against`, because a caller choosing a floor would create a second threshold authority;
- `IndexArithmetic::of(KernelType)`, because an external consumer must read the requirement already derived into `ResourceRequirements` rather than deriving it again from KIR; and
- `AppleFamilyFloor::apple_constant_value`, because exposing the raw SDK enumerator invites a caller to bypass the normalized observation path and create another device-comparison route.

`REGION_INDEX_ARITHMETIC`, the private discharge witness, `MetalGpuFamily`, and everything in `tiler_metal::applicability` remain unchanged. The already-landed `tiler.kernel.v7`, `tiler.artifact-program.v16`, manifest schema `16.0`, artifact bytes, route-row population, ledger, and derived pins do not move for these visibility-only corrections.

**Included behaviour.** Every entry carries the derived requirement once; the Metal-owned comparison maps `CompleteU64` to the sourced Apple3 floor; every named observable family clears it; `NoneNamed` remains `Unknown` and refuses as `UndecidableBelowVocabulary`; no observation refuses as `Unobserved`; and the private discharge witness makes pipeline preparation unreachable until the comparison succeeds.

**Excluded surface.** This acceptance does not widen `MetalGpuFamily` to Apple3/Apple4, publish raw Apple-family constants, let callers supply a comparison floor, mint a backend route row, teach the neutral runtime Apple vocabulary, claim a below-Apple3 hardware measurement, or change any feasibility, identity, or schema authority.

**Strongest counterpoint accepted with the decision.** Both narrowed methods are deterministic and tested, so leaving them public would not create an immediate arithmetic error. The smaller surface is preferred because neither has an out-of-crate consumer and each exposes a construction detail the one-authority design tells consumers not to use.

## Support-matrix rung

This advances **no** support-matrix or dtype row. It moves one row of the *maturity* ladder for the derived-requirement class only: index arithmetic goes from **reserved type** (a compiler-internal `CapabilityAxis` with no artifact carrier) to **tested guarantee** for the producer-to-decoder carry and the device comparison, and to **empirical evidence** bounded to one Apple9 host row for the positive live case. The negative device case has **`Unknown`** authority and no measurement, by construction: no device below Apple3 is reachable.

## What the next semantic-graph step must account for

**Historical coordination note (landing-time).** This step did **not** touch `tiler.semantic-graph`, so `remove-the-workload-shapes-from-the-concatenate-normative-definition`'s `v3`→`v4` was independent of it. That ticket was told to rebaseline from `tiler.artifact-program.v16` / manifest schema `16.0` and `DIFFERING_CARRIER_POSITIONS` **68**, not `v15`/`15.0`/67.

**Correction — 2026-08-10.** [`remove-the-workload-shapes-from-the-concatenate-normative-definition`](remove-the-workload-shapes-from-the-concatenate-normative-definition.md) is `status: done`. The forward rebaseline obligation above is closed; no open semantic-graph work remains on this ticket.

## Out-of-scope drift — corrected by its owners

The original outcome found unasserted BF16 byte lengths in `docs/dtype-support.md` and requested a narrow owner. That owner already landed: [`recompute-the-unasserted-bf16-byte-lengths-in-the-dtype-support-matrix`](recompute-the-unasserted-bf16-byte-lengths-in-the-dtype-support-matrix.md) and [`replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin`](replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin.md) are both `done`. The old digits remain searchable only inside struck historical measurements and ticket evidence; the live dtype-support contract now states checked length and identity relationships rather than unpinned absolute sizes. No new owner is needed.

## Closes when

The two accepted visibility narrowings land with source labels and tests aligned; the acceptance provenance remains recorded; the one-authority and pre-routing-discharge invariants are preserved; and targeted IR/Metal checks plus repository publication gates pass.
