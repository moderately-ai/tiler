---
id: implement-the-adr-0013-plan-determinism-stability-subject
title: Implement the ADR 0013 plan-determinism stability subject
status: in-progress
priority: p1
dependencies: [decide-the-adr-0013-plan-determinism-stability-subject]
related: [decide-the-semantic-order-contract-for-relaxed-contractions]
scopes: [implementation/ir, implementation/artifact, implementation/compiler, implementation/runtime, implementation/build, implementation/frontend, implementation/candle, research/target-profiles, contracts/numerics, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, identity]
claimed_from: todo
assignee: worker-adr0013-carrier
lease_expires_at: 1787084139
---
## Outcome

Implement the exact plan-determinism stability subject Tom accepts under `decide-the-adr-0013-plan-determinism-stability-subject`, including its verified construction, durable identity, artifact/explain projection, and runtime refusal path. This ticket is structurally blocked on that decision and carries no independent authority to choose or revise its public surface.

## Entry condition

Do not begin until the decision dependency is satisfied. Re-read the accepted decision and its governing ADRs at the exact implementation base. If any constructor, field, owner, verification rule, error, identity domain, schema/version, or unsupported population remains unresolved, stop and repair the decision graph rather than inventing a default.

## Required delivery

- Implement only the accepted public and internal types, constructors, accessors, errors, and ownership boundaries.
- Carry and verify the accepted stability subject through every schedule, kernel-program, artifact manifest/codec, explain, cache, and runtime site the decision names.
- Apply every accepted domain/schema/version/provider-revision and pin consequence atomically.
- Add the accepted subject perturbations for artifact digest, selected variant, target environment, and topology, plus the negative execution control for run-dependent selection.
- Record exact-base Facts, unsupported population, gates, perturbation failure text, and landed hash before closure.

## Boundary

This ticket does not decide the target-environment compatibility identity, selected-topology representation, public surface, or schema policy. It does not authorize relaxed contraction semantics or a reassociated schedule. `admit-reassociated-contraction-schedule-alternatives` depends on this carrier so no relaxed plan can claim determinism before the accepted generic subject is implemented.

## Implementation record — 2026-08-18 (worker-adr0013-carrier)

Implemented at branch `tkt/implement-the-adr-0013-plan-determinism-stability-subject` from exact base `48515ddf77e0e1ac533a56defe57eaad27a93c3a`; the implementation commit is `c77aab39`, this record follows it, and the branch head handed to the coordinator is the commit that also reorders three trybuild facade fixture imports for the `make full` fmt floor. Merging, closure, and integration stay with the coordinator.

The four scopes added to the frontmatter (`implementation/build`, `implementation/frontend`, `implementation/candle`, `research/target-profiles`) are scheduling metadata required by the authorized work, not scope expansion: the accepted carrier names `assemble_plan_artifact` (tiler-build), and the new required `RuntimeAdapter` methods reach every adapter implementor — the `tiler` facade's trybuild fixtures and route tests, and the Candle prototype adapter — while the standard-Metal pin mirror lives in `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md`. `contracts/numerics` was declared but needed no edit: `docs/numerical-semantics.md` ("The initial scoped guarantee is **plan deterministic**") already states exactly the subject implemented here.

### Exact-base Fact audit

- **Verified with correction (version steps).** The packet's drafted steps were written before the elementary-numerical-dimensions landing took `tiler.artifact-program.v19` / manifest `19.0` (merged as `841c373e`). Rederived at this base per the packet's own rederivation instruction: artifact `v19 → v20`, manifest `19.0 → 20.0`, guard-and-routing component `1.0 → 2.0`, target-environment compatibility identity `tiler.target-environment-compatibility.v1\0`, runtime plan-subject domain `tiler.runtime.plan-determinism-subject.v1\0`. Anchor: `crates/tiler-artifact/src/program/codec/encode.rs` "**`20.0` carries the ADR 0013 stability-subject records.**".
- **Verified (already discharged, no work).** The stage key already sits at `tiler.artifact-program.stage.v4`, and the glossary dtype repair the packet's queue named had already landed; neither required an edit here.
- **Verified (ADR 0086 population).** No current Metal route can claim `Plan`: the native runtime-translation authority is `Unknown`, so every Metal payload declares `environment: None` and every cell stays `Unclaimed` (`crates/tiler-build/src/metal_assembly.rs`, `metal_plan.rs`, six `assemble_plan_artifact` call sites passing `PlanDeterminismDeclaration::Unclaimed`).

### Rederived pin consequences

Moved, atomically with the step: standard Metal artifact identity `9ec0c149… → 13b2246b2e01f39c9a247ee9d2d4565d3bf743d08de8f3d53a7ed6d6c33fec5f`; expansion-cache subject `a3b00546… → 32477f9dfd68cf586553248c52b638e09029f6e948a03b99b9cfc4574928fff2`; fixed content `77,256 → 77,266` bytes (exactly one environment presence byte + the scope run's eight-byte count + one `Unclaimed` tag); the subgroup fixed-content digest pin; `crates/tiler/src/route/tests.rs` `IDENTITY_DOMAIN` to `v20`; `DomainContainer::PROGRAM_IDENTITY` `7 → 8`; the ledger mirror in `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md` ("What those pins are today"). Held, verified rather than assumed: schedule `v7`, kernel `v9`, kernel-program `v12`, stage `v4`, selected-provider `v3`, proof-sidecar schema and its four domains (the sidecar's stored envelope digest is retyped as `RecordedArtifactEnvelopeDigest` with the wire unchanged), the cache domains, and the 2,181-byte Metal descriptor. `docs/artifact-abi.md` carries the step (new `v20` Fact paragraph, ledger paragraph, manifest field-order Fact, and the domain census updated eighteen → nineteen / seven → eight).

### Accepted subject perturbations (one check per independent subject)

All in `crates/tiler-runtime/tests/adapter_route/determinism.rs` unless named; each holds the sibling coordinates fixed and asserts which subject segments move, with structural parsing of the subject encoding (`parse_subject`) rather than opaque equality:

- **Artifact digest (object relink):** `object_only_relinking_moves_the_envelope_digest_and_the_subject` — equal `CanonicalArtifactProgramIdentity`, different object bytes; only the digest segment moves.
- **Routing rank:** `another_routing_rank_moves_only_the_selected_coordinate` — claimed C1 portfolio at N=16 vs N=8; digest and environment equal, rank (and kernel-program projection) move.
- **Delivery position:** `another_delivery_position_moves_the_subject_with_a_shared_kernel_program` — two-delivery claimed member; delivery moves while rank, digest, environment, and kernel-program identity are shared.
- **Target environment:** `a_moved_provider_descriptor_moves_the_subject_and_filters_a_stale_observation` — one descriptor field moves; the environment segment moves with a matching observation, and a stale live observation is refused as `TargetEnvironmentIneligibility::EnvironmentMismatch`; provider/schema mismatches are refused by name (`declared_provider_and_schema_mismatches_filter_by_name`).
- **Topology:** discharged compositionally because no strictly-single-field verifiable end-to-end dial exists (a verifiable rounds change forces coherent coverage and synchronization moves): the per-field schedule-identity population test `every_cooperative_tile_field_separates_scheduled_region_identity` (pre-existing), the new kernel-layer chain `a_topology_change_separates_schedule_and_kernel_identity_together` (`crates/tiler-ir/src/kernel/tests.rs`), and the runtime tests above prove the envelope→subject link. Recorded honestly as compositional rather than claimed as one single-field test.
- **Negative execution control for run-dependent selection:** a launch reading a governed target property cannot enter a verified program — the pre-existing refusal `AbiNonInterfaceRoot { use_site: GridThreads }` in `crates/tiler-ir/src/program/tests.rs` ("a launch extent must read only interface facts") — and the witness retains the typed backstop arm `RuntimeDependentSelection` for a future launch vocabulary. The arrival item is carried by `a_permutation_permitted_stage_is_refused_as_unfixed_arrival_by_name`: granting permutation is refused as `UnfixedContributorArrival { stage: 0 }` with the exact Display string asserted, because `NondeterministicArrival`, `AtomicAccumulation`, and `SynchronizationKind::Atomic` are already refused by the schedule verifier before a program exists.

Positive controls: `an_attested_claimed_route_carries_one_exact_subject_across_the_commit` (subject minted on `Preflight`, byte-identical at `RoutedDispatch`, structurally parsed), `an_unclaimed_route_carries_no_subject`, the device-free filter pair (claimed cell filters as `Unattested`; the unclaimed sibling routes), and the artifact-layer claim join tests below.

### Artifact-layer join and codec coverage

`crates/tiler-artifact/src/program/tests.rs`: positive claim (`a_published_plan_claim_marks_its_cell_and_moves_artifact_identity` — claimed vs unclaimed twins differ in canonical identity), plus one test per accepted `publish_plan` refusal: out-of-range delivery (`StructuralLimit`/`DeliveryPositions`), `MissingPlanDeterminismWitness`, `MissingTargetEnvironmentDeclaration`, `MissingPayloadPlanDeterminismReceipt`, `PlanDeterminismProgramMismatch`, `PlanDeterminismPayloadMismatch` (uncarried payload and relinked object bytes), `PlanDeterminismEnvironmentMismatch` self-pair (`first_entry == entry`) and cross-entry (`first_entry: 0, entry: 1` over the two-stage partial-window fixture). `crates/tiler-artifact/src/program/codec/tests.rs`: claimed round-trip through the public codec (scope cell, declaration, and identity preserved), agreeing two-entry claim decodes, and forgeries — wrong scope cardinality, `Plan` over an undeclared payload, cross-payload environment disagreement, unknown scope tag, unknown environment-presence tag, zero schema major from the wire, and a spliced descriptor one byte over the governed budget refused as `CodecLimitKind::TargetEnvironmentDescriptorBytes`. `crates/tiler-artifact/src/program/environment.rs` tests: declaration grammar (zero major, exact 64 KiB bound, empty descriptor admitted as a state), reason-code alphabet, exact validation (provider/revision, schema, spelling), and the per-component identity population (baseline plus ten single-component perturbations, all pairwise distinct, population-counted).

### Deliberate-red probes (subject perturbed, assertions untouched, source restored)

1. Identity fold: removing the scope fold from `push_variant` failed `a_published_plan_claim_marks_its_cell_and_moves_artifact_identity` with `assertion 'left != right' failed: a claim that does not move identity is invisible to every cache and pin`.
2. Cross-entry join: removing `publish_plan`'s agreement check failed the cross-entry test with `left: Ok(()) / right: Err(PlanDeterminismEnvironmentMismatch { variant: 0, delivery: 0, first_entry: 0, entry: 1 })`.
3. Decode coherence: early-returning `check_plan_determinism` failed all three codec forgery tests (cardinality, undeclared payload, cross-payload disagreement), each with `left: Ok(ArtifactEnvelope { … })` where the named `ModelObligation` refusal was expected.
4. Witness: accepting permutation failed the tiler-ir test with `a granted arrival freedom must not inherit plan determinism: PlanDeterminismWitness { … }`.
5. Runtime filter: admitting `DeviceFree` for a claimed cell failed the filter test with `the claimed cell filters rather than failing later: runtime.route-requirements: variant 0 requires 1 live-device fact(s), and a device-free loader binds no device`.
6. Subject encoding: swapping rank and delivery failed both coordinate tests via the structural parse (`assertion 'left == right' failed … left: 0 / right: 1`, including `N=8 falls through to the general member`).
7. Object binding: skipping the receipt's object-digest comparison failed the relinked-bytes test with `left: Ok(()) / right: Err(PlanDeterminismPayloadMismatch { variant: 0, delivery: 0, entry: 0 })`.
8. Environment identity: dropping the provider-name field from the identity encoding failed the population test with `provider name and baseline share one identity encoding`.
9. Explain census: filing the witness verdict under `program.plan-verified` failed `every_wired_authority_emits_its_typed_explain_records` with `program.plan-determinism.v1` absent and `program.plan-verified` inflated `2 → 4`.

### Unsupported population

- No positive provider exists: ADR 0086 leaves the Metal native runtime-translation authority `Unknown`, so no shipped artifact declares an environment or claims `Plan`; the whole positive path is exercised by the scalar-host test backend only.
- Witness refusal arms `OutputAffectingAtomic`, `UnverifiedOpaqueStage`, and `RuntimeDependentSelection` are unreachable from any currently constructible verified program (the schedule verifier refuses `Atomic`, whole-program verification enforces exactly one stage owner, and the program builder refuses non-interface launch roots). They are typed backstops whose exhaustive wildcard-free matches turn future vocabulary widening into a build error at the classification site.
- `TargetEnvironmentIneligibility::EnvironmentMismatch` is reachable only under a schema admitting more than one canonical spelling **per class**; the test schema registers two classes to reach it.
- Device evidence: none required and none produced — the carrier admits no positive claim, so nothing here runs on a device.

### Commands

`cargo nextest run --workspace` (3,775 passed), `cargo test --workspace --doc`, `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked --exclude tiler-prototype-run --exclude tiler-prototype-compile --exclude tiler-prototype-candle -- -D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`, `tkt lint`, `make citations`, `git diff --check`, `tkt guard tkt/implement-the-adr-0013-plan-determinism-stability-subject --format json`, and `make full` at the final commit.

### Follow-on

The next framing step over these records (queue item 7's carrier, or any manifest-grammar change) must rederive on top of this landing: artifact `v20 → v21`, manifest `20.0 → 21.0`, and recompute the standard-Metal pins from its merged tree.
