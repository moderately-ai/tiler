---
id: carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit
title: Carry and check the derived index-arithmetic requirement before routing commit
status: awaiting-decision
priority: p1
dependencies: []
related: [emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable, check-synchronization-realization-before-the-routing-commit, separate-metal-launch-index-from-index-and-address-width, declare-a-required-gpu-family-in-the-artifact]
scopes: [implementation/ir, implementation/artifact, implementation/compiler, implementation/metal, implementation/runtime, implementation/build, contracts/artifacts, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, fail-closed, index-arithmetic, artifact-schema, metal, public-boundary]
claimed_from: todo
assignee: w-carry-and
lease_expires_at: 1786165341
---
## User-visible outcome

A delivered Metal artifact carries the verified program's complete unsigned-64 index-arithmetic requirement once, as part of its derived dispatch/resource record, and a device outside the sourced Apple-family support refuses before routing commitment instead of reaching pipeline creation.

## Why this is a direct requirement and not a route row

**Fact — construction. Line citations corrected 2026-08-08 at base `209013bd`; every claim's substance held and every line number was wrong.** `crates/tiler-compiler/src/physical.rs` added `index_arithmetic_requirement(KernelType::Index)` to every region proposal at **`:3957`**, not `:2060-2082`, and mapped that governed type to `CapabilityAxis::IndexArithmeticU64` at **`:4002-4004`**, not `:2119` — a drift of roughly +1,890 lines. `crates/tiler-metal/src/emit.rs` declares the type at **`:297`** and spells it `uint64_t` at **`:988`**, not `:261-266,811-818`. And the golden population is **ten** files, not six: all ten of `crates/tiler-metal/goldens/*.metal` carry the explicit widening, and the operation-bearing ones exercise addition, multiplication, division, and modulo.

**Fact — authority. Line citation corrected 2026-08-08: the row is at `:108`, not `:90-97`; the substance held exactly, including the labelled operation-completeness inference at `:111`.** `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md:108` reads Apple's 2025-10-20 Metal Feature Set Tables row `64-bit integer math` as `Metal3 | Apple3 | —`: MSL 4.0 clears the language-version half, Apple3 is the minimum Apple family, and no Mac-family guarantee exists. A macOS artifact family or spellable `uint64_t` therefore cannot discharge the live-device half.

**Fact — accepted ownership rule. Verified 2026-08-08, with one citation corrected.** [`declare-a-required-gpu-family-in-the-artifact`](declare-a-required-gpu-family-in-the-artifact.md):23-25 is exact. The `docs/artifact-abi.md` citation was **`:280`** and the derivability rule is at **`:352`**; `:280` is the `bf16` carrier-table Fact and states no such rule. Both sources require facts already derivable from the verified program to remain direct requirements. Copying one into `RouteRequirement::BackendFeature` creates a second producer authority that can contradict the program. This ticket is the direct-check owner; [`emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable`](emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable.md) is deferred until emission has an *additional*, non-derivable feature.

## The missing mechanism, traced exactly

**The producer has the requirement and the decoded consumer loses its interpretable form.** (Citations corrected 2026-08-08; every claim's substance held.) `ResourceRequirements` at `crates/tiler-ir/src/schedule/model.rs:1161-1203` — not `:846-887` — is the verified schedule's derived direct-requirement record. Artifact model/codec code encodes it (`crates/tiler-artifact/src/program/model.rs:2199`, `program/codec/encode.rs:698`), decodes it (`program/codec/decode.rs:1049`), and publishes it through `DecodedEntry::resources()` (`program/codec/view.rs:692`). The paths were given as `codec/…` rather than `program/codec/…` and every line was wrong. Its complete fields cover bindings, workgroup threads, local memory, device-memory use, synchronization, and numerical dimensions; none states index arithmetic. The envelope does not carry KIR operations, so a consumer holding decoded bytes cannot reconstruct the omitted `KernelType::Index` derivation.

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

Unmoved, and checked rather than assumed: `tiler.schedule.v5` and `STRICT_F32_REGION_IDENTITY_HEX` (requirements are not folded into the scheduled-region identity — confirmed by every golden's `scheduled region identity digest` line being byte-identical while the kernel digest moved); `tiler.kernel-program.v11` and `tiler.artifact-program.stage.v3` (their grammars are unchanged; only folded content moved); `tiler.semantic-graph.v3`; the explain qualifier `940c09e0821665a6`; the `index/law.rs` chain byte counts and digests; `POPULATION = 649`.

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
- **In `tiler_ir::schedule`:** `IndexArithmetic` (exhaustive, `CompleteU64`), `IndexArithmetic::of`, and the new public field `ResourceRequirements::index_arithmetic`. `REGION_INDEX_ARITHMETIC` is `pub(crate)` and is not surface.
- **Unchanged:** `MetalGpuFamily` and everything in `tiler_metal::applicability`.

## Support-matrix rung

This advances **no** support-matrix or dtype row. It moves one row of the *maturity* ladder for the derived-requirement class only: index arithmetic goes from **reserved type** (a compiler-internal `CapabilityAxis` with no artifact carrier) to **tested guarantee** for the producer-to-decoder carry and the device comparison, and to **empirical evidence** bounded to one Apple9 host row for the positive live case. The negative device case has **`Unknown`** authority and no measurement, by construction: no device below Apple3 is reachable.

## What the next semantic-graph step must account for

This step does **not** touch `tiler.semantic-graph`, so `remove-the-workload-shapes-from-the-concatenate-normative-definition`'s `v3`→`v4` is independent of it. What that ticket must account for is that the artifact and manifest domains have moved underneath it: it will rebaseline `tiler.artifact-program.v16` and manifest schema `16.0`, not `v15`/`15.0`, and its own recomputation of `DIFFERING_CARRIER_POSITIONS` starts from **68**, not 67.

## Out of scope, and needing a ticket

`docs/dtype-support.md` (scope `contracts/navigation`, not this ticket's) states artifact byte lengths — `97,060`, `90,806`, `45,457`, `73,556`, `36,832` — for the BF16 producer path. No test asserts them, so nothing went red, but every one is now stale by this step's per-record growth. `docs/artifact-abi.md:283`'s copies of the forged-pair figures are in scope and were left as the historical measurements they are labelled as. A narrow ticket should recompute the `dtype-support.md` figures on the merged tree.
