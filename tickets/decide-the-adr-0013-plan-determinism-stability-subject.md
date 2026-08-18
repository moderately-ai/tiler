---
id: decide-the-adr-0013-plan-determinism-stability-subject
title: Decide the ADR 0013 plan-determinism stability subject
status: awaiting-decision
priority: p1
dependencies: []
related: [decide-the-semantic-order-contract-for-relaxed-contractions]
scopes: [implementation/ir, implementation/artifact, implementation/compiler, implementation/runtime, contracts/numerics, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: [.ticketsplease/decision-queue.md]
tags: [decision, numerics, identity, public-boundary, needs-tom]
---
## Outcome

Decide the exact identity-bearing public subject that realizes ADR 0013 plan determinism before any relaxed contraction topology can claim deterministic evaluation. This is decision research, not implementation authorization.

## Exact decision

The accepted semantic scope is fixed: identical input bits and runtime bindings, the same object-bearing artifact-envelope digest, selected route coordinate of routing rank plus delivery position, and declared target-environment compatibility identity must produce identical output bits; recompilation, another artifact envelope, another route coordinate, or another declared target environment may choose a different legal result. Decide the currently unresolved declared target-environment compatibility identity, its construction and verification owners, the public projection of the already transitively identity-bound selected topology, public types and errors, durable schema/domain/version/pin cascade, and runtime refusal path.

## Exact-base Fact audit and repairs

Audit base: `dc105234df0f9fe9bf76461d7fde911aaeb12ffc`. This audit preceded the decision packet below. The repairs do not change the ticket's purpose.

- **Verified:** ADR 0013 fixes the initial promise and leaves the environment subject open at `The accepted initial guarantee is` and `The exact fields forming the declared target-environment compatibility identity` in [`docs/decisions/0013-scoped-determinism.md`](../docs/decisions/0013-scoped-determinism.md). Its `Artifact identity bounds` section also permits a separately compiled artifact to choose another legal result.
- **Verified:** ADR 0012 leaves `Unrealized — the explicit stability scope for deterministic order` and requires that scope before a relaxed topology claims deterministic evaluation in [`docs/decisions/0012-physical-reduction-topology.md`](../docs/decisions/0012-physical-reduction-topology.md). The implementation and relaxed-contraction admission tickets retain that dependency.
- **Imprecise, repaired above:** `same artifact digest` cannot mean `CanonicalArtifactProgramIdentity`, the manifest identity digest, or the artifact cache key. `CanonicalArtifactProgramIdentity` says `This is a pre-compilation subject` and expressly excludes emitted object bytes in `crates/tiler-artifact/src/program/model.rs`. Only `ENVELOPE_DIGEST_DOMAIN` in `crates/tiler-artifact/src/program/codec/encode.rs` hashes the exact encoded envelope and therefore the executable sections.
- **Imprecise, repaired above:** routing rank alone does not identify the selected executable payload. `DecodedProgram::select_route` reads `DecodedVariant::routing_rank`, but `route_candidate` drops it, while `delivery_position` can select different payload object bytes and explicitly is not an `ExecutionEnvironment`. The exact route coordinate is `(routing_rank, delivery_position)`.
- **Imprecise, repaired above:** selected topology is not absent from identity. Schedule-model `fn push_schedule` exhaustively encodes `ReductionTopology`; the resulting schedule identity is carried through kernel, kernel-program, and artifact-program identity. What is absent is a proof-bound public projection and its retention in the runtime stability subject.
- **False as a candidate solution:** compiler-private `TargetProfileIdentity { key }` in `crates/tiler-compiler/src/target.rs` is checked-fact attribution, not a target-environment compatibility identity. `TargetProfileRef { key, descriptor }` is the complete compile-profile half but supplies no runtime compatibility authority.
- **False as a candidate solution:** `TargetExecutionEnvironment` and `ExecutionEnvironmentIdentity` are exact measurement provenance, not executable compatibility. The latter's encoder writes each of its five fields once; an apparent duplicate was overlapping source output, not a source defect.
- **False as a candidate solution:** device-free `tiler_runtime::load::ExecutionEnvironment` is a caller statement, not an adapter-bound live attestation. It also contains `dtype_dispatch`, omitted by the glossary's claim that the host declaration is `exactly` a profile/backend/representation triple. Dtype dispatch remains eligibility information rather than part of the compatibility identity; the glossary wording is a separate bounded documentation defect for the implementation carrier to heal.
- **Verified unsupported population:** ADR 0086 records the Metal runtime translation authority and provenance as `Unknown`. No current Metal adapter can mint a positive plan-determinism compatibility attestation without new accepted evidence.

## Readiness gate

Use the strongest reasoning model. Re-audit ADRs 0012 and 0013 at the exact base and read target, schedule, kernel-program, artifact manifest/codec, explain, cache, and runtime construction/consumption/refusal paths. Apply the full Pareto-complete decision gate: status quo typed refusal, the narrowest exact target-environment subject, a complete replacement if current target identity is insufficient, bounded research, and deferral. Eliminate any option that invents/defaults environment compatibility, leaves selected topology unbound, conflates artifact identity with live device identity, or claims a schema-complete outcome with unresolved fields.

## Required evidence

- Exact public fields, constructors, accessors, verification errors, and owner for the stability subject.
- Complete request/schedule/kernel/artifact/explain/cache/runtime identity and schema consequences.
- Subject perturbations for artifact digest, selected variant, target environment, and topology, plus a negative execution control for run-dependent selection.
- Strongest counterargument and reversal evidence for every frontier survivor.

## Boundary

Do not implement the result or authorize relaxed contraction semantics. The already-filed [`implement-the-adr-0013-plan-determinism-stability-subject`](implement-the-adr-0013-plan-determinism-stability-subject.md) remains blocked on this decision and may implement only the exact subject Tom accepts. [`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md) depends on that carrier, so a relaxed-contraction implementation cannot proceed until both the decision and its implementation are complete.

## Source-read boundary and current pipeline

The Fact audit read the complete ticket, root `AGENTS.md`, ticketsplease `SKILL.md`, [`docs/README.md`](../docs/README.md), ADRs [0012](../docs/decisions/0012-physical-reduction-topology.md), [0013](../docs/decisions/0013-scoped-determinism.md), [0081](../docs/decisions/0081-admit-tiler-runtime-as-a-device-free-artifact-loader.md), [0086](../docs/decisions/0086-require-attributable-or-attested-native-translation.md), and [0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), [`docs/numerical-semantics.md`](../docs/numerical-semantics.md), [`docs/glossary.md`](../docs/glossary.md), the related semantic-order decision, implementation, admission, and ADR-0086 decision tickets, and the applicable target, schedule/topology/witness/verifier, kernel/kernel-program, compiler request/selection/explain, artifact model/builder/error/manifest/codec/proof, build assembly/cache, and runtime host/load/route/adapter construction, validation, identity, refusal, and test sites. There is no descendant `AGENTS.md` below any target path.

**Fact — current identity cascade.** `CompileRequest` reaches `VerifiedRequestSubject`, selected physical plan, scheduled region, kernel, kernel program, artifact program/manifest, composed cache subject/key, and finally a runtime route. The current domains are `tiler.compiler.request-subject.v6`, `tiler.target-profile.declaration.v11`, `tiler.compiler.physical-implementation-proposal.v3`, `tiler.compiler.selected-physical-plan.v2`, `tiler.program-alternative.v2`, `tiler.schedule.v6`, `tiler.kernel.v8`, `tiler.kernel-program.v12`, and `tiler.artifact-program.v18`; artifact manifest schema is `18.0`, compiler explain trace/renderer are `11`/`9`, and the cache composed subject/key are both v1. The searchable anchors are `fn push_schedule`, `The identity folds the exact canonical identity of the scheduled region`, and `What a composed subject determines, and what it does not`.

**Fact — current decode and refusal order.** The artifact codec checks the 256 MiB envelope bound, header/framing, manifest digest, manifest schema/features, section counts and digests, trailing bytes, model validity, canonical artifact identity plus the carried identity-digest declaration, and canonical re-encoding. `DecodedProgram::decode` then checks delivery position. The adapter route binds a context; `DecodedProgram::prepare` evaluates retained shapes before its expected pre-compilation artifact-identity comparison; stable-priority selection checks target profile, each selected payload's profile/backend/representation, and dtype before evaluating the guard; route and ABI obligations follow; then the adapter validates payload bytes, resolves live-device requirements, prepares entries, resolves prepared-entry facts, sizes the dispatch, and only then commits. Allocation and dispatch are post-commit. Search `fn select_variant`, `fn variant_eligibility`, `fn route_with_adapter`, and `What a committed route names`.

**Fact — current route loses two required coordinates.** `DecodedVariant::routing_rank()` exists during selection and `DecodedProgram::delivery_position()` exists on the decoded program, but `RouteCandidate` retains neither. `LiveDeviceQualification`, `Preflight`, and `RoutedDispatch` expose only the pre-compilation artifact identity and selected kernel-program identity. Two variants may legally share a kernel program while differing in guard or other variant metadata, and one envelope may carry different object sections at different delivery positions. Kernel-program identity is therefore topology evidence, not the selected-variant coordinate.

## Recommendation — one generic subject, no current positive provider

**Proposal — recommend the provider-versioned exact subject below.** It is the sole complete frontier member. The generic layer decides every outer field, bound, canonical encoding, owner, comparison, and refusal. A provider-specific schema decides the inner descriptor fields and canonicalization under its own governed identity and exact schema version. No provider is compatible merely because its bytes parse: positive support requires an independently selected adapter to expose that exact schema and produce an observation after binding a live context. This relocates the backend-specific field vocabulary to its authority; it does not default or infer it.

The public execution-class identity is exactly:

```text
PlanDeterminismSubject
  artifact_envelope_digest: ArtifactEnvelopeDigest
  selected_variant:
    routing_rank: u32
    delivery_position: u32
  declared_target_environment: CanonicalTargetEnvironmentCompatibilityIdentity

CanonicalTargetEnvironmentCompatibilityIdentity
  target_profile: TargetProfileRef       # key + full exact descriptor
  backend: BackendKey
  representation: RepresentationKey
  provider: ProviderIdentity             # namespace + name + nonzero revision
  descriptor_schema: SchemaVersion       # exact major + minor; no widening
  descriptor: TargetEnvironmentDescriptor
```

The `PlanDeterminismSubject` equality and canonical identity contain those three top-level fields and no hidden host choice. Identical input bits and runtime bindings remain independent premises of ADR 0013 rather than fields copied into this carrier. `kernel_program_identity()` is a verified accessor projection, not a fifth equality input: envelope digest plus routing rank already fixes the variant and its complete kernel-program identity, while delivery position fixes the executable objects. This is the narrowest nonredundant topology binding.

The target-environment identity deliberately excludes dtype-dispatch statements, delivery position, live-device handles or serial numbers, queue/context identity, capacities, timing, cost, and input bindings. Dtype is checked independently as route eligibility; delivery is in `SelectedPlanVariant`; device/context objects are neither portable nor compatibility classes; capacities are feasibility. A provider must put every output-affecting runtime/compiler/device/process arithmetic condition into its descriptor schema. If it cannot prove that equality of those bytes is sufficient, it cannot register positive support.

## Exact public surface and owners

All types below are a proposed, labelled public boundary until Tom accepts the included and excluded surface. No raw constructor certifies execution.

### IR proof owner

`tiler_ir::kernel` owns:

- `PlanDeterminismWitness<'program>`, with private fields and no unchecked constructor;
- `verify_plan_determinism(&VerifiedKernelProgram) -> Result<PlanDeterminismWitness<'_>, PlanDeterminismRefusal>`;
- witness accessors `kernel_program_identity()` and `scheduled_region_identities()`; and
- non-exhaustive `PlanDeterminismRefusal::{UnfixedContributorArrival { stage }, OutputAffectingAtomic { stage }, RuntimeDependentSelection { stage }, UnverifiedOpaqueStage { stage }}`.

The witness borrows its `VerifiedKernelProgram`, so it cannot be replayed against another owner. Verification is exhaustive over every stage, schedule topology, contributor-arrival spelling, synchronization, execution edge, multipass dependency, and opaque stage. It accepts only choices fixed by canonical program bytes. Current `NondeterministicArrival`, `AtomicAccumulation`, and `SynchronizationKind::Atomic` already fail before a verified schedule exists; the new verifier retains an independent whole-program backstop so a future admitted construct cannot inherit plan determinism silently.

No `tiler.schedule.v6`, `tiler.kernel.v8`, or `tiler.kernel-program.v12` field is added. The witness is a proof over identities already complete for the current vocabulary, not a second topology encoder.

### Artifact declaration and digest owner

`tiler_artifact::program` owns:

- `ArtifactEnvelopeDigest([u8; DIGEST_BYTES])`, privately constructed by successful encode/decode, with `as_bytes()`;
- `RecordedArtifactEnvelopeDigest`, constructible from exactly `DIGEST_BYTES`, with `as_bytes()` and `matches(&ArtifactEnvelopeDigest)`; proof sidecars store this assertion type rather than raw `[u8; 32]`;
- `TargetEnvironmentDescriptor`, private bounded bytes, with `new`, `as_bytes`, and the same 64 KiB ceiling as a target-profile descriptor;
- `TargetEnvironmentDeclaration`, private fields `provider: ProviderIdentity`, `descriptor_schema: SchemaVersion`, and `descriptor: TargetEnvironmentDescriptor`, with a raw `new` and read-only accessors; this is explicitly a declaration, never an attestation;
- `TargetEnvironmentDescriptorSchema`, the provider contract with `provider()`, `schema_version()`, and `validate_canonical_descriptor(&[u8])`; validation accepts exactly one byte spelling and returns a bounded governed reason code on failure;
- opaque `ValidatedTargetEnvironmentDeclaration`, minted only by `TargetEnvironmentDeclaration::validate(&dyn TargetEnvironmentDescriptorSchema)` after exact provider/revision/schema agreement and canonical validation;
- `CanonicalTargetEnvironmentCompatibilityIdentity`, derived only from a validated declaration plus exact `TargetProfileRef`, `BackendKey`, and `RepresentationKey`, with `as_bytes()` and structured accessors;
- exhaustive `PlanDeterminismScope::{Unclaimed, Plan}`; and
- `PayloadPlanDeterminismVerifier::verify(&PlanDeterminismWitness<'_>, &BackendPayloadDescriptor, object_bytes: &[u8], &ValidatedTargetEnvironmentDeclaration) -> Result<PayloadPlanDeterminismReceipt, PayloadPlanDeterminismRefusal>`; and
- `PayloadPlanDeterminismReceipt`, privately minted by the backend's installed payload verifier from the exact verified kernel-program identity, payload compilation subject, emitted object section digest, declared target environment, and absence of run-dependent translation choices.

`BackendPayloadDescriptor` stores an optional `TargetEnvironmentDeclaration`. Each variant stores one `PlanDeterminismScope` per delivery position. `Plan` at one position is buildable only when the IR witness exists and every entry's selected payload has a receipt and resolves to the same complete target-environment compatibility identity. Other positions may remain `Unclaimed`; a missing provider for one family must not erase an independently supported family. The existing ceilings make the table at most `MAX_ARTIFACT_PAYLOADS = 16` descriptors and `MAX_ARTIFACT_VARIANTS * MAX_DELIVERY_POSITIONS = 1,024` scope cells.

The standard `tiler-build` path owns the join: it accepts owner-linked compiler witness and backend payload receipts and asks `ArtifactProgramBuilder` to publish `Plan`. Low-level construction cannot pass a bool or raw declaration as proof. Exact new build refusals are:

- `TargetEnvironmentDeclarationError::{ZeroSchemaMajor, DescriptorTooLong { bytes, limit }, ProviderMismatch { declared, registered }, SchemaMismatch { declared, registered }, NoncanonicalDescriptor { provider, schema, reason }}`;
- `ArtifactBuildError::{MissingPlanDeterminismWitness { variant }, MissingPayloadPlanDeterminismReceipt { variant, delivery, entry }, PlanDeterminismProgramMismatch { variant, delivery, entry }, PlanDeterminismPayloadMismatch { variant, delivery, entry }, MissingTargetEnvironmentDeclaration { variant, delivery, entry }, PlanDeterminismEnvironmentMismatch { variant, delivery, first_entry, entry }}`; and
- backend-owned receipt refusal `PayloadPlanDeterminismRefusal::{RunDependentTranslation, KernelProgramMismatch, PayloadSubjectMismatch, ObjectDigestMismatch, TargetEnvironmentMismatch}`.

Existing governed-key and `ProviderIdentity` constructors retain their current empty/length/alphabet/nonzero-revision errors; this packet does not duplicate that grammar.

### Runtime schema registration, attestation, and subject owner

ADR 0090 forbids a runtime-adapter registry, so this design adds none. The independently selected `RuntimeAdapter` must instead implement two required methods:

```text
target_environment_support() -> TargetEnvironmentSupport<'_>
observe_target_environment(&mut self, &LiveExecutionContext)
    -> TargetEnvironmentObservation
```

`TargetEnvironmentSupport` is exhaustive: `Unsupported` or `Registered(&dyn TargetEnvironmentDescriptorSchema)`. There is no permissive default. `TargetEnvironmentObservation` is exhaustive: `Observed(TargetEnvironmentDescriptor)` or `Unavailable { reason }`; both are assertions, not attestations. `route_with_adapter` calls it only after `bind_execution_context` and only when this delivery contains a `Plan` cell, checks an observed descriptor with the registered schema, and privately mints `LiveTargetEnvironmentAttestation`. `Unavailable` filters claimed cells but does not prevent a lower-ranked `Unclaimed` cell from routing. Neither `ExecutionEnvironment`, `TargetEnvironmentDeclaration`, `TargetEnvironmentObservation`, nor their public constructors can mint the attestation type.

`tiler_runtime::load` owns:

- `SelectedPlanVariant`, private fields and constructor, with `routing_rank() -> u32` and `delivery_position() -> u32`;
- `LiveTargetEnvironmentAttestation`, public read-only accessors but no public constructor;
- `PlanDeterminismSubject`, private constructor and fields, with `artifact_envelope_digest()`, `selected_variant()`, `declared_target_environment()`, `kernel_program_identity()`, and `canonical_identity()`;
- `CanonicalPlanDeterminismSubjectIdentity`, under `tiler.runtime.plan-determinism-subject.v1\0`, with `as_bytes()`; and
- `Preflight::plan_determinism_subject() -> Option<&PlanDeterminismSubject>` and the same accessor on `RoutedDispatch`, carrying one value unchanged across commit.

The canonical plan-subject encoding is the domain, fixed-width envelope digest, big-endian `u32` routing rank and delivery position, then the length-framed complete target-environment identity. The kernel-program accessor is re-derived from the privately retained selected variant and must agree; it is not encoded twice.

Device-free `DecodedProgram::preflight` and `prepare` can continue routing `Unclaimed` variants and return `None`. They filter a `Plan` cell as unverified because a caller-stated `ExecutionEnvironment` cannot mint a live attestation. `route_with_adapter` is the only positive path.

`VariantIneligibility` gains `PlanDeterminismEnvironment { delivery_position, reason }`, where non-exhaustive `TargetEnvironmentIneligibility` has exact classes `Unattested`, `ProviderUnavailable`, `ProviderMismatch { declared, registered }`, `SchemaMismatch { declared, registered }`, `ObservationUnavailable { provider, schema, reason }`, `InvalidDeclaredDescriptor { provider, schema, reason }`, `InvalidObservedDescriptor { provider, schema, reason }`, and `EnvironmentMismatch { declared, observed }`. Unknown provider/revision/schema, dynamic observation unavailability, and byte mismatch are therefore stable-priority filters before the guard; if no candidate survives, existing `LoadRejection::NoEligibleVariant` carries every filtered rank. Only `bind_execution_context` failure remains `AdapterRouteFailure::Context`; an absent stability attestation cannot suppress a viable `Unclaimed` candidate. Nothing stability-related may first fail after `Preflight::commit`.

## Construction, decode, and refusal precedence

The exact order is part of the recommendation:

1. The compiler verifies the complete kernel program and derives the IR witness. Missing authority or any run-dependent selection refuses before artifact assembly.
2. Each backend compiles an exact payload and its installed verifier binds the object section digest, program identity, payload compilation subject, and validated target-environment declaration into a receipt. No receipt means `Unclaimed`, never an inferred claim.
3. `tiler-build` joins the witness and every selected payload receipt per `(variant, delivery)`; disagreement refuses transactionally before the builder mutates or freezes the variant.
4. The artifact codec preserves its present cheap-to-deep order. Manifest 19 parses and generically bounds provider/schema/descriptor runs before model validation; semantic provider validation remains unavailable to a neutral decoder. The full envelope digest is derived once from the exact input bytes but becomes observable only on successful complete decode.
5. `DecodedProgram::decode` refuses an out-of-range delivery position as today. `route_with_adapter` then binds a context, obtains and schema-validates its raw observation, and mints the live attestation. `Unsupported` mints none.
6. Retained shapes and the expected pre-compilation artifact identity retain their current precedence. Stable-priority eligibility then checks profile, payload profile, backend/representation, dtype, and finally the exact attested compatibility identity, all before the candidate's guard.
7. A selected route retains envelope digest, routing rank, delivery position, kernel-program identity, and the matched declaration through `RouteCandidate`, `LiveDeviceQualification`, `RoutePreparation`, `Preflight`, and `RoutedDispatch`.
8. Entry/ABI checks and backend payload validation precede subject minting. Live requirements, prepared-entry requirements, and dispatch sizing remain pre-commit; they may refuse feasibility but do not rewrite the subject. The subject is first exposed on `Preflight`, after all identity and payload obligations are discharged, then carried unchanged through the one-way commit.
9. Allocation and dispatch remain post-commit. No compatibility fallback, re-selection, or subject mutation is permitted there.

The provider schema is checked twice for different authorities: build validates what the producer declares; runtime validates both the received declaration and live observation through the independently selected adapter's exact registration. A neutral decode can frame an unknown provider but never turn it into executable compatibility.

## Identity, schema, explain, cache, and pin cascade

| Layer | Required consequence |
| --- | --- |
| Semantic/request | No caller-selectable stability field. `REQUEST_SCHEMA_VERSION = 2` and `tiler.compiler.request-subject.v6` stay. If the related semantic-order decision replaces the operation definition, complete semantic identities and request values move transitively under their existing grammar. |
| Target profile | Compiler-private `TargetProfileIdentity` is not widened or promoted. `tiler.target-profile.declaration.v11` stays; runtime compatibility is not compile-profile truth. |
| Selection/schedule/kernel | Selected-plan/proposal/program-alternative domains, `tiler.schedule.v6`, `tiler.kernel.v8`, and `tiler.kernel-program.v12` stay. The proof and topology accessor derive from already complete bytes. A new schedule/topology construct steps its owner independently if its encoding changes. |
| Environment identity | Add `tiler.target-environment-compatibility.v1\0`, enumerated in `crates/tiler-artifact/src/domains.rs`. It length-frames profile key/descriptor, backend, representation, provider namespace/name/revision, exact schema major/minor, and descriptor bytes. Provider-specific schemas are separately versioned; no minor-version widening is implicit. |
| Artifact model | Step `ARTIFACT_DOMAIN` v18 to v19 because payload environment declarations and per-delivery scope cells are identity-bearing. Step manifest `18.0` to `19.0` major and `ArtifactSchema::GOVERNED.guard_and_routing` from `1.0` to `2.0`. Program/ABI/target-requirement components stay `1.0`/`1.0`/`3.0`. Old schema-18 artifacts refuse rather than silently become `Unclaimed`; pre-production fixtures are rebuilt. |
| Envelope/proof | Envelope format, canonical encoding, section/manifest/envelope digest domains, and proof-sidecar schema `1.0` stay because their subjects and framing are unchanged. `ArtifactEnvelopeDigest` promotes the existing v1 algorithm; sidecars change only from raw bytes to `RecordedArtifactEnvelopeDigest`. Envelope/pin values move with manifest/object changes. |
| Explain | Compiler explain schema `11`, compilation schema `1`, and renderer `v9` stay if the implementation uses an existing detail-event shape to report IR witness eligibility/refusal; affected trace values/pins move. It must not print a not-yet-known envelope digest or delivery. Artifact views expose scope/declaration, and runtime `PlanDeterminismSubject` renders the exact post-build subject. |
| Expansion cache | Composed-subject/key/bundle domains stay v1. The v19 artifact identity and declared environment move relevant keys and standard pins. Object-only relinking may still hit the same pre-compilation key; the returned envelope's independently derived digest distinguishes the stability subject, so no false equality follows. |
| Runtime cache | No such cache exists in `tiler-runtime` now. A future prepared/dispatch cache must include the canonical plan-determinism subject plus its existing live-context specialization; a live-device identity may scope resources but cannot replace the declared compatibility identity. |
| Ledgers/tests | Update the artifact domain enumeration, manifest/component-schema pins, standard Metal artifact/cache/envelope pins, proof-sidecar typed accessor tests, and exact request/explain pins whose values actually move. Schedule/kernel pins must remain byte-identical. Every domain census must be sized from its enum. |

## Pareto-complete option analysis

| Option | Disposition | Strongest counterargument / reversal evidence |
| --- | --- | --- |
| Status quo typed refusal | Safe executable fallback, not a complete answer to this ticket. It has zero new surface and all present claims remain unsupported, but it leaves ADR 0013's accepted carrier and dependents unresolved. | It becomes the product answer only if Tom retires the accepted plan-determinism capability or evidence shows no provider can ever state/attest a bounded exact class. |
| Reuse compiler-private `TargetProfileIdentity` | Eliminated. A key-only checked-fact token omits the profile descriptor, payload backend/representation, runtime compiler/process/device conditions, object bytes, and live attestation. It can silently merge different executions. | Reverse only if its owner and encoding are completely replaced by all fields and authorities below; that is no longer reuse. |
| Reuse `TargetExecutionEnvironment` / `ExecutionEnvironmentIdentity` | Eliminated. Those five strings are measurement provenance, caller-constructible, and do not establish executable compatibility. | Reverse only with accepted evidence that the measurement row is a complete compatibility class for every admitted backend and a live authority can observe it canonically. None exists. |
| Treat current host `ExecutionEnvironment` as the subject | Eliminated. It is caller-stated, producer-restated on current paths, omits runtime arithmetic authority, and includes dtype eligibility not class identity. It would let a caller self-certify. | Reverse only if device-free loading is superseded and construction becomes exclusively adapter-bound with complete provider evidence; the recommended design supplies that distinction without erasing device-free load. |
| One fixed backend-neutral OS/device/toolchain field list | Eliminated. No accepted evidence makes one finite list complete across native objects, runtime translation, interpreters, CPUs, or future backends; adding guessed fields does not create authority. | Reverse with a closed, independently derived list proven sufficient for every admitted provider and canonical observation sources for every field. |
| One concrete Apple/Metal row now | Eliminated by ADR 0086. Its runtime translation authority/provenance remains `Unknown`, so a detailed row would be validity scope without warrant. | Reopen only when ADR 0086's named observer/authority trigger fires and the exact provider schema is accepted. |
| Replace the compiler target-profile identity with compile + live environment | Eliminated. It conflates facts fixed at compilation with facts available only after adapter bind, forces the compiler to claim a delivery/object it has not seen, and duplicates target-profile bytes. | Reverse only if compilation and execution become one authoritative phase, contradicting the accepted AOT/artifact/runtime split. |
| Provider-versioned declaration + build receipts + adapter-bound attestation + exact route subject | **Sole complete frontier survivor and recommendation.** Exact and fail-closed at every layer; provider-specific without a global adapter registry; no current unsupported provider is widened; topology and object bytes are bound; host cost is bounded. | Strongest objection: opaque provider bytes reduce cross-provider explainability and can look like postponing the exact field decision. Reverse if independent evidence proves a single closed neutral vocabulary is complete, or if provider schemas cannot be reviewed, canonically validated, and observed under the stated bounds. Until then, the provider key/revision/schema and exact-field contract make the authority explicit rather than guessed. |
| More generic research | Stop condition met for the outer subject. It cannot manufacture the missing Metal observer. Provider-specific research remains a prerequisite to enabling each provider, not a reason to leave the generic type/refusal boundary ambiguous. | Reopen the generic choice for a concrete provider that cannot fit one exact bounded descriptor/attestation or a multi-context execution model that invalidates the singular environment class. |
| Defer | Process fallback only. Execution stays soundly `Unclaimed`, but the implementation and relaxed-topology graph remain blocked. | Appropriate only if review finds a named missing invariant; unchanged external evidence is not itself a new trade-off. |

One answer dominates among solutions that satisfy the accepted carrier: the provider-versioned subject. Tom still owns acceptance of this consequential public and wire boundary, so the packet needs only an accept-or-retain-typed-refusal question rather than a manufactured multi-option choice. It is queued behind the already presented LiveRow question and is not presented concurrently.

## Host cost, unsupported population, and graph boundary

The implementation adds one governed hash over the exact envelope, `O(envelope_bytes)` and at most 256 MiB, performed once during successful decode; the 32-byte result is retained. A descriptor is at most 64 KiB and there are at most sixteen payloads, so new raw descriptor retention is at most 1 MiB plus bounded framing. The selected subject retains one route coordinate, digest, kernel-program reference, and one canonical environment identity; an implementation may share immutable identity storage, and even a single unshared exact encoding is bounded by the two 64 KiB descriptors plus small keys. Scope cells are at most 1,024. Runtime comparisons are linear in one selected descriptor after ordinary eligibility; no device memory, kernel instruction, dispatch count, or kernel-performance claim changes.

The following remain typed `Unclaimed` or refused:

- every current artifact and every current Metal route, because no accepted Metal target-environment schema, payload receipt authority, or live observer exists under ADR 0086;
- schema-18 artifacts after the complete v19 replacement, malformed/unknown provider revisions or schemas, noncanonical/oversize descriptors, missing observations, and exact mismatches;
- device-free attempts to execute a `Plan` cell;
- any variant/delivery whose entries disagree on profile/backend/representation/environment, whose payload lacks a receipt, or whose object/program binding differs;
- nondeterministic or atomic contributor arrival, output-affecting atomic operations, runtime-selected topology/arithmetic, runtime JIT/translation without an exact provider schema and receipt, unproved opaque stages, multiple devices/streams/contexts, and any post-commit plan choice;
- portable bitwise determinism, cross-artifact or recompilation equality, cross-rank or cross-delivery equality, environment widening, changed inputs/bindings, and inference from a live device identifier; and
- provider descriptor schemas whose exact fields, canonicalization, observer, authority, bounds, or revision policy have not separately been accepted.

No new provider ticket is queued here: ADR 0086 deliberately records no concrete Metal observer and already owns the evidence trigger. When that trigger fires, work must split into a provider-specific schema/authority decision and its implementation before Metal changes from `Unsupported`. The implementation carrier must also heal the separate glossary omission of `ExecutionEnvironment::dtype_dispatch`; that wording repair is not evidence for including dtype in the new identity. This packet changes no dependency, accepted ADR, or preserved failed work; after independent exact-commit review passed with no findings, only this ticket's `awaiting-decision` state and its held queue row were added.

## Required implementation perturbations

The implementation ticket must preserve one check per independent subject, then perturb only that subject while leaving the check unchanged:

- change only emitted object bytes under one equal pre-compilation artifact identity; `ArtifactEnvelopeDigest` and `PlanDeterminismSubject` must move while the expansion-cache subject stays equal;
- hold envelope and environment fixed and select another routing rank; only the selected coordinate and subject move;
- hold envelope/rank fixed and select another delivery position; selected objects and subject move even if the kernel-program identity is shared;
- change only one provider descriptor field or provider revision; target-environment and plan-subject identities move, and an old live observation yields `TargetEnvironmentIneligibility::EnvironmentMismatch` or exact provider/schema mismatch;
- change only one topology field; schedule, kernel-program, envelope, and plan-subject identities move while inputs, environment, and route rank remain fixed; and
- replace `AscendingParticipant` with `NondeterministicArrival` or `AtomicAccumulation`, or introduce an output-affecting runtime choice; the IR witness refuses by name before build. Granting permutation must not turn the latter two into a plan-deterministic witness.

Each test must count its typed population, include a positive control, and quote the deliberate-red failure. A single perturbation that moves artifact, route, environment, and topology together does not discharge four checks.

## Exact-base controls and deliberate-red evidence — 2026-08-17

**Measurement — current positive controls.** Each command selected exactly one named test and passed at base `dc105234df0f9fe9bf76461d7fde911aaeb12ffc` plus this ticket-only diff:

```sh
cargo test -p tiler-artifact payload_identity_follows_the_compilation_subject_and_not_the_object -- --nocapture
cargo test -p tiler-artifact a_reached_provider_revision_changes_the_envelope_digest -- --nocapture
cargo test -p tiler-ir every_cooperative_tile_field_separates_scheduled_region_identity -- --nocapture
cargo test -p tiler-ir an_unfixed_arrival_order_consumes_permutation_and_is_refused_by_name -- --nocapture
cargo test -p tiler-runtime --test adapter_route one_artifact_and_pipeline_rebind_c1_extents_and_select_the_aligned_guard_at_sixteen -- --nocapture
cargo test -p tiler-runtime an_identical_profile_is_compatible -- --nocapture
```

**Measurement — independent subjects reached their checks.** Disposable changes were applied one at a time, the assertions were left unchanged, the named test was run, and every source change was restored. Changing only emitted object bytes from `second-link` back to `first-link` under equal `CanonicalArtifactProgramIdentity` failed the disposable envelope-digest watch with `assertion left != right failed: object-only relinking must move the envelope digest`. Reversing only the two portfolio declarations failed the stable-priority check with `left: ["live_row_major", ...]` versus the expected rank-sensitive element `"live_row_major_aligned"` and `StablePriority must select the ≡ 0 (mod 16) variant only at S=16`. Changing only the declared profile descriptor in the current environment-equality control failed with `left: DescriptorMismatch { key: TargetProfileKey("tiler.target.apple-m4") }` and `right: Compatible`; this proves the existing compile-profile half is load-bearing and does not substitute for the future provider-descriptor perturbation required above. Changing only cooperative `rounds` from `2` to its baseline `1` failed with `CooperativeTile { ... rounds: 1, ... } collided with an earlier tile`. Finally, temporarily admitting `NondeterministicArrival` past the named rule failed the unchanged negative control with `called Result::unwrap_err() on an Ok value` whose printed verified schedule retained `arrival: NondeterministicArrival`.

These reds are evidence that the current anchors are reachable, not evidence that the proposed types already exist. The implementation carrier owes the new provider descriptor, envelope-digest type, route-coordinate, plan-subject, and cross-layer movement tests listed above. Final `git status` after restoration names only this ticket.

## Packet verification — 2026-08-17

In addition to the six targeted positive controls and five restored deliberate-red probes above, the ticket-only packet passed:

```sh
tkt lint --format json
make citations
git diff --check
tkt guard tkt/decide-the-adr-0013-plan-determinism-stability-subject \
  --ticket decide-the-adr-0013-plan-determinism-stability-subject \
  --base dc105234df0f9fe9bf76461d7fde911aaeb12ffc \
  --config-ref dc105234df0f9fe9bf76461d7fde911aaeb12ffc \
  --format json
```

Ticket lint reported no diagnostics; every pinned citation and local link resolved; `git diff --check` was empty. The pre-commit guard reported no conflict, no under-declared scope, and the ticket's complete declared scope set; as designed it saw no committed changed file before commit. The same exact-base/config-ref guard is rerun on the reported commit so the ticket-only path is visible. No production file, accepted ADR, ticket status, dependency, queue row, or preserved failed work changed.
