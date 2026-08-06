---
schema: "tiler-doc/v1"
id: "tiler.spike.target-profiles.scalar-cpu-vertical"
kind: "experiment"
title: "Bounded scalar CPU backend vertical"
topics: ["target-profiles", "backends", "cpu", "artifacts", "runtime", "pluggability"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.target-profiles.physical-feasibility-model", "tiler.research.extensions.backend-provider-composition"]
entrypoints: ["spikes/target-profiles/scalar-cpu-vertical/src/main.rs"]
last_verified: "2026-08-05"
verified_at_commit: "55d1d09f"
ticket: "prototype-a-bounded-scalar-cpu-backend-vertical"
---

# Bounded scalar CPU backend vertical

A second backend, materially different from Metal, carried end to end through the same public boundaries: a declared CPU target profile, the compiler's verified physical work, an independently identified executable representation, a real artifact payload, device-free validation, a live host execution context, the one-way routing commit, execution, and a bitwise comparison against `tiler-reference`.

It exists to answer one question before a generic provider interface is specified: **does anything in the accepted target-profile, artifact, and runtime contracts silently encode Metal's execution hierarchy?** The answer this run supports is *almost nothing does, and the exceptions are nameable*. They are listed under "Findings" and are the payload this spike owes `specify-the-consumer-neutral-backend-provider-composition-contract`.

`docs/backends/cpu.md` stays **proposed** and Q-PLAN-011 stays open. Nothing here is a production CPU backend, an implementation commitment, or a performance claim, and no `tiler-cpu` crate is scaffolded.

## Running it

From this directory. `rust-toolchain.toml` is resolved by directory ancestry from the repository root, so no selector is passed and this spike deliberately carries no toolchain file of its own.

```sh
cd spikes/target-profiles/scalar-cpu-vertical
CARGO_TARGET_DIR=./target cargo run
```

Passing a path records the result fixture instead of only printing the narrative:

```sh
CARGO_TARGET_DIR=./target cargo run -- results/2026-07-31-macos-arm64.json
```

The binary's only product is a verdict: every stage that fails exits non-zero with the stage named, and there is no partial success. `CARGO_TARGET_DIR` is set explicitly because this is a nested workspace and sharing one target directory across unrelated workspaces is forbidden.

## What one run does, in order

1. **Declares a bounded scalar CPU target profile** (`src/profile.rs`) through `tiler_compiler::target::TargetProfileBuilder`: governed key `tiler.target.cpu-scalar-host-aarch64-darwin`, triple `aarch64-apple-darwin`, the AAPCS64/Darwin data layout, a 64-bit address model, one thread per workgroup, zero staged local memory, two buffer bindings per entry, complete unsigned-64 index arithmetic, `f32` dispatchable, and a numerical table that preserves subnormals exactly and declares **every** reshaping freedom unsupported. Vector width, mask and tail support, scalable-vector length, thread count, task granularity, and oversubscription are undeclared and therefore `Unknown` — the builder's sparse-profile rule, used as the mechanism rather than worked around.
2. **Compiles** the smallest scalar program this build's semantic, normalization, and reference layers all admit, under `NumericalContract::STRICT_F32`, against that profile alone.
3. **Translates** each verified structured kernel into `tiler.cpu.scalar-image-v1` (`src/image.rs`) and serializes it. The translation is a real consumer of KIR: launch builtins, typed constants, index arithmetic, comparisons, the named NaN canonicalization, typed loads and stores, predicated blocks, and bounded serial loops, all read through `VerifiedKernel`'s public views.
4. **Observes the translator refusing** every buffer parameter this backend cannot bind, against an accepted neighbour, before the positive path is claimed.
5. **Packages** a real artifact: one carried payload whose `code` is the serialized image, whose compilation subject names the kernel identities it was translated from, and one variant bound to the compiler's own kernel program.
6. **Encodes and decodes** the envelope through `tiler_artifact`, as the sole delivery position this artifact declares a payload for, then runs the fail-closed probe set against those exact bytes.
7. **Decodes the payload** through the image decoder — which knows nothing about `VerifiedKernel` — and runs the payload-level probe set.
8. **Binds a live host execution context** (`src/host.rs`) by *measuring* this process: architecture, system, pointer width, byte order, and the subnormal behaviour of its actual floating-point arithmetic. It refuses a route whose declared realization this process does not deliver.
9. **Commits** the route, then executes (`src/interpret.rs`): one invocation at a time, in ascending grid index, on the calling thread.
10. **Compares** the output bits against `tiler-reference`'s independent evaluation of the same semantic program, exactly, and reports the artifact identity and the reference registry identity as two separate numbers.

## Result

**Measurement**, Apple M-series arm64 macOS, `rust-toolchain.toml`'s pinned nightly, base commit `d5960e81`:

The vertical executed. Twelve `f32` elements agreed **bit for bit** with `tiler-reference`, including a negative zero whose sign survived, the least positive and least negative subnormals preserved through a multiply, a non-canonical NaN payload canonicalized to the realization's exact `0x7fc00000`, and both infinities. The retained fixture is [`results/2026-07-31-macos-arm64.json`](results/2026-07-31-macos-arm64.json).

Recorded quantities from that run: profile descriptor 865 bytes, payload 265 bytes, envelope 87,338 bytes, artifact identity 42,832 bytes, reference registry identity 1,420,906 bytes, **zero** deferred prepared-entry predicates.

### Three identity sizes moved between `488efac` and `63f9259`

**Measurement.** This spike was restored to a running state on 2026-08-01 after `TensorRole::Input` gained an `ordinal` payload and left it uncompilable. Re-running is what detects drift from the source beside it, and it detected some — recorded against the previous run at `488efac`:

| Quantity | `488efac` | `63f9259` |
| --- | --- | --- |
| selected plan | `program-alternative:506a3f9171c1b383` | `program-alternative:5ef3467e50acb6f7` |
| envelope bytes | 20,327 | 20,953 |
| artifact identity bytes | 9,464 | 9,753 |
| reference registry identity bytes | 80,104 | 438,805 |

**What did not move is the part the spike exists to claim.** The twelve output bit patterns are byte-identical to the earlier fixture, and so are the profile descriptor (797), the payload (265), the element count, and the zero deferred predicates. The four quantities above are *identity and encoding sizes*, not numerical results: they moved because the content folded into those identities changed as `tiler-ir`, `tiler-artifact`, and `tiler-reference` gained operations and fields between the two commits. The reference registry identity grew most because it enumerates the reference implementations, and that set grew.

**Inference.** No claim [ADR 0090](../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) cites from this spike depends on the moved numbers — it cites the bit-for-bit agreement, the named exceptional values, the `CanonicalizeF32Nan` perturbation, and that no physical provider was installed, all of which reproduced. The moved numbers were recorded here and in the fixture only, so this table is the correction rather than an amendment to an accepted record.

### Two more quantities moved between `63f9259` and `e2da98f`

**Measurement.** The re-run recorded above was taken at `e2da98f` on 2026-08-01, after the loader's compatibility refusals were retired (next section). Recorded against the previous run at `63f9259`:

| Quantity | `63f9259` | `e2da98f` |
| --- | --- | --- |
| selected plan | `program-alternative:5ef3467e50acb6f7` | `program-alternative:986779d4106ea633` |
| reference registry identity bytes | 438,805 | 446,768 |

**What did not move.** The twelve output bit patterns, the profile descriptor (797), the payload (265), the envelope (20,953), the artifact identity (9,753), the element count, and the zero deferred predicates all reproduced. The envelope and artifact-identity numbers held across this interval where they moved across the last one.

**Measurement boundary, because five of those seven are byte *counts*.** What reproduced for them is a length rather than a value: an identity whose content changed without changing size would read the same here, and this spike retains no identity bytes to tell the difference. Only the output bits are recorded exactly, and those are byte-identical to the earlier fixture. **Inference**, held to that boundary: what folds into an artifact's identity and what folds into a plan's stable id are at least not the same set, because the plan id moved while the artifact-identity length did not — which is weaker than saying the artifact identity is unchanged, and is as much as this fixture can support.

### Four host-relative refusals moved class between `63f9259` and `e2da98f`

**Measurement.** `select-executable-variants-across-registered-backend-families` inverted the loader's selection order: host-relative ineligibility is now a *filter* applied to a variant before any applicability guard is evaluated, rather than a terminal mismatch reported after one. `LoadRejection::UnexecutablePayload`, `LoadRejection::IncompatibleTarget`, and `TargetDeclaration` were the terminal spelling and are removed, so the four envelope probes below stopped compiling; they were rewritten against `LoadRejection::NoEligibleVariant` and `VariantIneligibility` and re-run. What each probe perturbs is unchanged, and so is the fact that each is refused; the class the loader reports for it moved.

| Perturbation | class at `63f9259` | class at `e2da98f` |
| --- | --- | --- |
| another profile descriptor | `IncompatibleTarget` / `TargetDeclaration::Variant` / `DescriptorMismatch` | `runtime.no-eligible-variant` / `AssessedProfile` / `DescriptorMismatch` |
| the Metal profile family | `IncompatibleTarget` / `TargetDeclaration::Variant` / `ProfileKeyMismatch` | `runtime.no-eligible-variant` / `AssessedProfile` / `ProfileKeyMismatch` |
| a host executing `metallib` | `runtime.unexecutable-payload` | `runtime.no-eligible-variant` / `UnsupportedRepresentation` |
| a host consuming `tiler.cpu.scalar-image-v2` | `runtime.unexecutable-payload` | `runtime.no-eligible-variant` / `UnsupportedRepresentation` |

**Inference, and it is why the last two rows are pinned further than their class.** The two payload perturbations reported one class before and still do, but that class now carries the host's own stated backend and representation, and this artifact packages a single variant — so all four probes arrive as "the whole portfolio was filtered" and only the carried reason tells them apart. `probe_fail_closed` therefore asserts the exclusion's fields rather than only its class: the Metal-host probe requires `tiler.metal`/`metallib` and the representation probe requires `tiler.cpu.scalar`/`tiler.cpu.scalar-image-v2`. Substituting either host pair for the other was made, and the run exits non-zero naming the probe. This is stronger than the retired code was rather than a restoration of it — those two probes matched `UnexecutablePayload { .. }` and were interchangeable then too, and the class move is only what made the weakness visible. The first two rows are separated by their classification as they were before.

### Two API steps landed while this spike could not compile, and the delivery position is a decision

**Fact.** Between `e2da98f` and `2119b20` the spike stopped compiling twice over, in ten places, and neither landing was caught by anything: no `make` target reaches `spikes/`, so the only detector is a reader running it by hand. `BackendEntryRef::payload` became `payloads`, a counted run of payload references rather than one, and `DecodedProgram::decode` gained a `delivery: usize` argument at nine call sites. [`restore-the-scalar-cpu-vertical-spike-against-the-current-crates`](../../../tickets/restore-the-scalar-cpu-vertical-spike-against-the-current-crates.md) repaired both; the provenance ticket that preceded it had deliberately stopped short of the second, because passing a position is a decision rather than a mechanical rename.

**The position this spike resolves is zero, and it is derived rather than convenient.** A delivery position is the ordered slot a consumer's build target resolves to — `crates/tiler-artifact/src/program/model.rs`'s `BackendEntryRef` and `docs/artifact-abi.md` both state that two positions are one compilation, one plan, one kernel program, and two separately compiled objects, and that the artifact layer deliberately carries no name for what a position *is*. `assemble` pushes exactly one carried payload and names it once per entry, so this artifact declares one position, `DecodedProgram::decode` refuses every other index outright, and zero is not a default but the only member of the set. There is no default in the API for exactly the case this artifact is not in: an artifact carrying several objects has no "the" payload, and taking the first would hand a consumer the object built for another target. The constant is named `SOLE_DELIVERY` and documented at its definition, which is the spelling `prototypes/serial-sum-run`, `prototypes/candle-metal-adapter`, and the runtime's own integration tests already use for the same single-target case.

### Five quantities moved between `e2da98f` and `2119b20`

**Measurement.** Recorded against the previous run at `e2da98f`, on the re-run the repair above made possible:

| Quantity | `e2da98f` | `2119b20` |
| --- | --- | --- |
| selected plan | `program-alternative:986779d4106ea633` | `program-alternative:72d49e71d668fff8` |
| profile descriptor bytes | 797 | 865 |
| envelope bytes | 20,953 | 21,296 |
| artifact identity bytes | 9,753 | 9,969 |
| reference registry identity bytes | 446,768 | 912,256 |

**What did not move.** The twelve output bit patterns are byte-identical to every earlier fixture, and so are the payload (265), the element count, the zero deferred predicates, the host string, and every governed key. The profile descriptor moved for the first time across any of these intervals, having held at 797 through the previous three runs.

**No delta here is attributable to any single landing, and the interval is why.** 232 commits separate the two bases, 64 of them touching `crates/` across 158 files, so this table records *that* the numbers moved and not *what moved them*. The byte-count boundary the previous section states applies unchanged: four of these five rows are lengths, and an identity whose content changed without changing length would read the same.

**A correction to the ticket that ordered this run.** It predicted `payload_bytes` would move from 265, reasoning that the provenance record's canonical subject lost three SDK text runs and gained a platform tag. It did not move, and the prediction rested on reading the fixture's field as the payload's canonical subject: `payload_bytes` records the length of the serialized scalar image — the payload's `code` — which no provenance field is part of. Provenance folds into the payload descriptor's canonical key and therefore into artifact identity and the envelope, and those are two of the numbers that did move.

### The composed numerical contract, a widened kernel type, and four quantities moved between `2119b20` and `d5960e81`

**Fact — two drifts, and the second was not the one the ticket enumerated.** [`restore-the-spikes-against-the-composed-numerical-contract`](../../../tickets/restore-the-spikes-against-the-composed-numerical-contract.md) named one site here: `NumericalContract` stopped being a preset enumeration, its named points became associated constants on a composed record, and `NumericalContract::StrictF32` is now `NumericalContract::STRICT_F32`. A clean `cargo check --workspace --all-targets` reported **two** errors. The second is `KernelType::Bf16`, a variant added after that enumeration was written, and it surfaced here as a non-exhaustive `match` in `ImageType::from_kernel` — which is the enum working as designed rather than an accident: `KernelType` is deliberately not `#[non_exhaustive]` precisely so that widening it is a build error at every backend that has to decide what the new variant means.

**This backend's decision is to refuse it, and the type's own definition is what asks for that.** `KernelType::Bf16` is admitted in KIR as a *type* rather than a lowerable one, and its documentation states that a backend which cannot yet lower it refuses it by name rather than spelling it. This profile declares `f32` dispatchable and says nothing about `bf16`, so `Bf16` joins `U8` and `I32` in the refusing arm and the refusal path carries the type: `TranslationError::UnsupportedValueType { found: KernelType }`. No wildcard arm was added — a catch-all is exactly what would stop the next widening from being a build error, which is the property that caught this one.

**Measurement.** Recorded against the previous run at `2119b20`:

| Quantity | `2119b20` | `d5960e81` |
| --- | --- | --- |
| selected plan | `program-alternative:72d49e71d668fff8` | `program-alternative:f6c5c487fbfbd8fa` |
| envelope bytes | 21,296 | 82,918 |
| artifact identity bytes | 9,969 | 40,622 |
| reference registry identity bytes | 912,256 | 1,420,906 |

**What did not move.** The twelve output bit patterns are byte-identical to every earlier fixture, and so are the profile descriptor (865), the payload (265), the element count, the zero deferred predicates, the host string, and every governed key. The profile descriptor held at 865 across this interval, having moved for the first time in the last one.

**No delta here is attributable to any single landing, and the interval is again why.** 517 commits separate the two bases, 125 of them touching `crates/` across 198 files. The envelope and the artifact identity roughly quadrupled and the reference registry identity grew by half, which is a larger step than any previous interval recorded here, and this table states *that* they moved rather than *what* moved them. The byte-count boundary two sections above applies unchanged: three of these four rows are lengths, and an identity whose content changed without changing length would read the same.

**The comparison was re-proved able to say no, because the oracle moved again.** The reference registry identity grew from 912,256 to 1,420,906 bytes, so the reference this run is checked against is not the one the previous re-run used and this README's own rule required the perturbation again. Replacing the `CanonicalizeF32Nan` arm in `src/interpret.rs` with an identity made the run exit 1 naming exactly one differing element — the backend returning the operand's own `0x7fc01234` where the reference requires `0x7fc00000`, with all eleven other elements still agreeing. Perturbation reverted; `git diff` over `src/interpret.rs` is empty.

**Determinism, re-measured rather than inherited.** Four runs at this base — one of them after the rebuild the perturbation forced — produced byte-identical fixtures and byte-identical 47-line run narratives, `diff` exit 0 on every pair.

### The delivered-realization record, and two quantities moved between `d5960e81` and the wiring landing

**Fact — one drift, and it is a required field rather than a rename.** `wire-the-delivered-realization-record-into-the-artifact` made every executable artifact carry a delivered-realization record: `ArtifactProgramBuilder::build` refuses a draft that never called `declare_realization`, so `assemble` here stopped producing an artifact rather than stopping compiling — a state no `cargo check` detects and only running the spike reports. The repair is one `declare_realization` call and one `realization_record` helper beside it, deriving the eleven governed resolutions from the packaged program's own scheduled realization rather than restating them, so a contract change in the plan cannot leave this spike describing the old one.

**Measurement.** Recorded against the previous run at `d5960e81`, on the tree of that ticket based at `55d1d09f`:

| Quantity | `d5960e81` | with the record wired |
| --- | --- | --- |
| envelope bytes | 82,918 | 87,338 |
| artifact identity bytes | 40,622 | 42,832 |

**What did not move, and here the interval is short enough to attribute the delta.** The twelve output bit patterns are byte-identical to every earlier fixture; so are the selected plan (`program-alternative:f6c5c487fbfbd8fa`), the profile descriptor (865), the payload (265), the element count, the zero deferred predicates, the host string, and every governed key. Unlike every interval above, this one is a single landing rather than hundreds of commits, and the two moved rows are exactly what that landing adds: the record's canonical bytes are folded into the artifact identity, and the manifest carries the same run — so the identity grew by 2,210 bytes and the envelope by 4,420, the second being the first counted twice, once inside the folded identity the manifest also carries.

**The comparison was proved able to say no.** The `CanonicalizeF32Nan` perturbation in `src/interpret.rs` was applied again at this base and the run exited 1 naming exactly one differing element, the backend returning `0x7fc01234` where the reference requires `0x7fc00000` and the other eleven still agreeing. Perturbation reverted; `git diff` over `src/interpret.rs` is empty.

## Findings

Each is what a consumer-neutral backend-provider contract has to account for. **Fact** means inspected source or this run's output; **Inference** is derived from those.

1. **A second backend needs no production edit.** (Fact) The whole vertical runs against `crates/` unmodified. `TargetProfileBuilder`, `CompileRequest`/`TargetRequest`, `ArtifactProgramBuilder`, and `tiler_runtime::load` are together sufficient to declare a target, compile for it, package a payload for it, and route to it.

2. **Governed keys are open text, and that is what makes a new backend expressible.** (Fact) `BackendKey`, `RepresentationKey`, and `TargetProfileKey` validate length and alphabet only. `tiler.cpu.scalar` / `tiler.cpu.scalar-image-v1` were minted without touching a registry. The cost is the other side of the same coin: nothing prevents two producers from minting the same key for different things, and a provider contract has to say who governs the namespace.

3. **The device-free loader is genuinely device-free, and a CPU backend needs no deferred predicate.** (Fact) Because this profile declares its workgroup bound as an available compile-time fact rather than a prepared-pipeline query, the plan carries zero deferred prepared-entry requirements and `DecodedProgram::preflight` — not `prepare` — is sufficient. Metal cannot do this: only a built pipeline knows its own `maxTotalThreadsPerThreadgroup`. **Inference:** the `prepare`/`resolve_target_properties` stage is correctly *optional* rather than universal, and a provider contract must not make it mandatory.

4. **A CPU backend still has a real second stage, and it is numerical rather than structural.** (Fact) This spike's host preflight measures whether the running process preserves subnormals and refuses a route whose image declares otherwise; the four host refusals in the run output are that check saying no. **Inference:** "device-bound facts" is the wrong name for this stage. The general shape is *facts about the execution context that no artifact can assert*, and on a CPU that set is dominated by the floating-point environment rather than by resource limits.

5. **`AvailabilityPhase` has no host-process spelling.** (Fact) Its five variants are compile profile, artifact evidence, `LiveDevicePreflight`, `PreparedKernelPreflight`, and launch preflight. A fact known once a *host process* is bound has to borrow `LiveDevicePreflight`. This is a naming seam rather than a functional one today, and it becomes a real one the moment a profile wants to distinguish "known once a device exists" from "known once a process exists".

6. **`ArtifactExecutionPolicy` is a two-valued GPU dichotomy.** (Fact) `NativeImage` or `RequiresDeviceTranslation`. The scalar image is legitimately the former — the bytes are decoded and executed as they stand, with no target-specific translation and no pipeline object — but the vocabulary offers no way to say "an interpreted image", "a JIT input", or "a dynamically linked object", which are three different things a CPU backend family will eventually want to distinguish.

   **The second value was retired on 2026-08-01 by [`route-or-refuse-the-device-translation-execution-policy`](../../../tickets/route-or-refuse-the-device-translation-execution-policy.md), and the half of this finding that mattered is the half that survives.** The observation above is left as measured: it is what the vocabulary was when this vertical ran. What moved is the enum rather than this spike's answer — the retirement eliminated a route for the second value against [ADR 0086](../../../docs/decisions/0086-require-attributable-or-attested-native-translation.md)'s uninhabited native-translation authority instead of building one, so `NativeImage` stands alone and wire tag `0x02` is retired without reassignment. Nothing here is re-measured, and nothing needed to be: the scalar image was and is legitimately a native image, and the gap this finding names is untouched, because both values it lists are GPU-shaped and neither was ever a spelling for an interpreted image, a JIT input, or a dynamically linked object. Read the dichotomy framing as a record of the vocabulary this run met; what a CPU backend family still needs is what the finding's second half states.

7. **`PayloadProvenance` carries Apple-shaped required fields.** (Fact, as measured) `deployment_major`/`deployment_minor` and `PayloadSdkIdentity` are mandatory and have no CPU meaning; this spike states its representation version and its own name in them, which is honest but is not what the fields are called. A neutral provenance record would make the platform-versioned fields optional or backend-owned.

   **Closed 2026-08-01 by [`generalize-payload-provenance-beyond-the-apple-shape`](../../../tickets/generalize-payload-provenance-beyond-the-apple-shape.md).** `PayloadProvenance` now carries a `PayloadPlatform`, and this spike's source states `PayloadPlatform::Unversioned` rather than minting an SDK. The answer landed as *backend-owned* rather than optional, which is the stronger of the two the finding offered: which fields a payload owes follows the shape it declares, so a Metal payload still owes the deployment minimum and all three SDK fields and an omission is refused by field name. **Re-measured 2026-08-01 at `2119b20`**, once [`restore-the-scalar-cpu-vertical-spike-against-the-current-crates`](../../../tickets/restore-the-scalar-cpu-vertical-spike-against-the-current-crates.md) repaired the two unrelated API steps that had left the spike uncompilable. Every number under `results/` is now this source's own output: the envelope and artifact identity moved, the payload did not, and the table above gives both the values and the reason the moves cannot be attributed to the provenance change alone.

8. **Two profile axes have no CPU referent and must still be answered.** (Fact) `WorkgroupThreads` and `LocalMemoryBytes` are compared against every kernel's derived `ResourceRequirements`, so a CPU profile has to declare `1` and `0` rather than omit them. Omitting them leaves them `Unknown`, which is not the same claim. **Inference:** the quantitative axis set is a *GPU* axis set with a neutral spelling; a CPU profile's real axes — vector width, mask and tail support, scalable-vector length, cache levels, thread count — have no home in it at all, which is exactly what `docs/backends/cpu.md` proposes and what this vertical could not express.

9. **The target triple has nowhere typed to live.** (Fact) `CapabilityAxis` has no triple, ABI, or data-layout axis, so both survive only inside the profile *key* string and the payload provenance. The host check that compares the declared triple against `std::env::consts` is therefore parsing a key this spike itself formatted.

10. **The transport mapping is the identity here, and that is why the artifact must carry one.** (Fact) A scalar entry's transports are its ABI slots, because storage is bound by signature position and there is no argument table. Metal's mapping is not the identity in general. A provider contract that assumed either would be wrong for the other backend.

11. **The artifact layer cannot validate a payload, and the backend must.** (Fact) A payload's `code` bytes are opaque to every check `DecodedProgram` performs. The six payload-level refusals in the run output — a foreign domain separator, truncation, trailing bytes, an out-of-range slot, an access-mode violation, and the accepted neighbour they are all measured against — are checks *this backend* owns. **Inference:** a provider contract has to require a payload-validation obligation of every provider, and say that it runs while the preflight is still held, or the first backend that skips it discovers a malformed payload after the routing commit.

## Perturbation evidence

Every check below was run against a case that must fail, and observed failing. Each perturbs exactly one thing, and each is paired with an accepted neighbour so a refusal is evidence about the perturbation rather than about a harness that refuses everything.

**Envelope**, against the exact bytes this run packaged: a flipped interior byte (`artifact.integrity: ManifestDigestMismatch`), truncation (`artifact.malformed: TotalLengthMismatch`), a foreign expected identity (`runtime.program-mismatch`), another profile descriptor (`runtime.no-eligible-variant`, the variant filtered as `AssessedProfile` / `DescriptorMismatch`), the *Metal profile family* (`runtime.no-eligible-variant`, `AssessedProfile` / `ProfileKeyMismatch`), a host that executes `metallib` (`runtime.no-eligible-variant`, `UnsupportedRepresentation` naming `tiler.metal`/`metallib` as what the host stated), and a host consuming `tiler.cpu.scalar-image-v2` (`runtime.no-eligible-variant`, `UnsupportedRepresentation` naming `tiler.cpu.scalar`/`tiler.cpu.scalar-image-v2`). The last four are exclusions rather than terminal mismatches, and they are the outcome here only because this artifact packages one variant; each is pinned to its carried reason as well as its class, for the reason the drift section above gives.

**Payload**, against the exact image bytes: a changed domain separator, truncation, one appended byte, an instruction naming a slot past the declared value space, and a store redirected into the read-only input parameter. The last two are constructed by modifying the decoded image and re-encoding it, so the perturbation is exactly the one claimed rather than a byte flipped at a guessed offset.

**Translation**: four buffer parameters this backend cannot bind — workgroup space, constant space, invocation-private space, and a `u8` element type — each refused by a real call into the translator's own decision, against an accepted `f32` device buffer.

**Host context**: an image declaring flushed input subnormals, an image declaring flushed result subnormals, an artifact declaring a 32-bit address model, and an artifact declaring `x86_64`, each refused by the measured context, against the unperturbed declaration it admits.

**The comparison itself**, which is the one that matters most, because every probe above would still pass if the final check could not fail. Deleting the `CanonicalizeF32Nan` arm from `src/interpret.rs` — replacing it with an identity — makes the run exit non-zero at the comparison, naming exactly one differing element: the backend returns the operand's own `0x7fc01234` payload where the reference requires the realization's canonical `0x7fc00000`. Every other element still agrees. That perturbation was made, observed, and reverted; it is the evidence that the agreement above is a result rather than a tautology, and that the NaN canonicalization the kernel names is load-bearing rather than decorative. It was re-run at `63f9259`, at `e2da98f`, and at `2119b20`, all on 2026-08-01, and again at `d5960e81` on 2026-08-05, and produced that same single-element failure each time — exit 1, `0x7fc01234` where the reference requires `0x7fc00000`, every other element still agreeing — so the comparison is still a check that can say no on the current tree rather than one inherited from a run nobody repeated. None of the re-runs is redundant, and the rule is the same each time: the reference registry identity moved across every one of those intervals, so the oracle the comparison is against is never the one the previous re-run used.

## Measurement boundary

- **One host, one run.** Every arithmetic answer is a fact about this process on this machine in the interval the probe was taken in. A process can change its own floating-point control state after the probe, so the binding is evidence about that interval and the spike re-measures per run rather than caching. Nothing here is a portable claim about `aarch64-apple-darwin`.
- **One process, so the artifact-identity check is a tautology.** `prototypes/serial-sum-compile` and `prototypes/serial-sum-run` are separate crates, and their agreement is evidence about a delivery mechanism. This spike computes the identity it then checks against. The rest of the load path is not a tautology — the envelope really is encoded to bytes and decoded back, and the payload really is decoded by code with no access to `VerifiedKernel` — but splitting producer from consumer is the obvious next increment and has not been done.
- **The operation-level translation refusals are not observed.** Packed-nibble extraction, barriers, the dequantization conversions, and `I32Subtract` are refused by an exhaustive match in `src/image.rs`, and no case here watches one refuse: constructing an `OperationView` needs a `VerifiedValueId`, which only `tiler-ir` mints, and no program this profile admits produces a kernel containing them. A build error at that match is a weaker guarantee than an observed refusal, and it is what this spike has.
- **One program shape, one dtype, one stage.** The admitted program is pointwise and fuses to a single stage, so the multi-entry route, the shared-allocation pairing, and the serial-loop interpreter path are **implemented and unexercised by a run**. The reduction program that would exercise them is one operation larger and was not packaged.
- **No performance claim.** Nothing here is timed, and a per-invocation interpreter is not a fast CPU backend and is not trying to be.
- **No concurrency.** The execution model is one invocation at a time. A threaded CPU backend is a different backend and would need its own evidence.

## Retained evidence

- [`results/2026-07-31-macos-arm64.json`](results/2026-07-31-macos-arm64.json) — the identities, byte counts, and exact output bit patterns of the run recorded above. Re-running with the same argument overwrites it; a diff is drift from the source beside it. The date in the name is the fixture's origin, deliberately not bumped per run: the path is stable so that `git diff` is the drift signal, and the run it currently holds is dated by `last_verified` above. The 2026-08-01 restoration made four of its numbers move, the loader-vocabulary re-run later the same day moved two more, the delivery-position repair later still moved five, the 2026-08-05 contract-and-`KernelType` restoration moved four, and the delivered-realization wiring moved two; all five are tabled under "Result" with the superseded values kept rather than overwritten. The run this file currently holds is deterministic across invocations: four consecutive runs at `d5960e81`, one of them after a rebuild, produced byte-identical fixtures and byte-identical 47-line run narratives.
- `Cargo.lock` is tracked, so the dependency set a recorded run was taken under is recoverable.
