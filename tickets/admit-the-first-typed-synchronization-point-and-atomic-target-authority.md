---
id: admit-the-first-typed-synchronization-point-and-atomic-target-authority
title: Admit the first typed synchronization point and atomic target authority
status: in-progress
priority: p1
dependencies: [replace-or-justify-the-barrier-count-axis, represent-cooperative-workgroup-reduction-dataflow]
related: [construct-and-bind-the-first-authoritative-metal-compile-profile]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal, implementation/build, contracts/foundation, contracts/optimizer, contracts/artifacts, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [synchronization, feasibility, target-profiles, correctness]
claimed_from: todo
assignee: worker-sync-authority
lease_expires_at: 1785589250
---
## User-visible outcome

Tiler can represent and verify its first synchronized schedule without treating the number of barrier operations as a target capacity. A synchronized implementation is executable only when one provenance-bearing target fact establishes the exact synchronization realization it consumes; a schedule with no synchronization remains vacuously admitted without such a fact.

## Facts and boundary

**Fact:** `replace-or-justify-the-barrier-count-axis` removes the numeric barrier capability and makes every current KIR barrier intrinsically unauthorized because the implemented schedule owns no synchronization point, phase, placement, participant set, visibility contract, or convergence proof.

**Fact:** the reserved `BarrierSpec` fields for execution scope, memory scope, fenced spaces, and ordering are not a schedule obligation and cannot establish convergence or target support. Equal field shapes do not make those concepts one authority.

**Inference:** independently declared target facts for scope, fence, visibility, or ordering would permit false composition: each component could be supported in some realization while their conjunction is unsupported. The target fact must therefore be atomic over the complete synchronization subject.

**Proposal:** prove the target-neutral contract on the meaningful cooperative workgroup reduction dataflow delivered by `represent-cooperative-workgroup-reduction-dataflow`, with one bounded synchronized schedule and a synthetic governed target authority. A barrier inserted into the current pointwise/global-linear program is eliminated as closing evidence because it is semantically redundant or divergent under predication. Do not promote Metal support from source acceptance or from a backend spelling; `realize-parallel-reduction-strategies-on-metal` owns primary backend evidence.

## Implementation keys

- Add a schedule-owned stable synchronization-point identity and bind it to an explicit phase and placement in the normalized schedule.
- Define the operation kind rather than assuming every synchronization point is a control barrier. Keep asynchronous copies, split-phase barriers, collectives, atomics, and inter-dispatch dependencies distinct until their own contracts are admitted.
- Define the complete participant set and execution scope, visibility obligation, fenced memory spaces, ordering, and convergence proof. State how each field is constructed and verified rather than accepting a caller assertion that a point is convergent.
- Make KIR synchronization reference the exact schedule point it realizes. Verify exact operation kind, participants and scope, phase and placement, visibility, fences, ordering, and convergence before deriving any target requirement.
- Introduce one atomic target synchronization-realization fact over that complete subject, including its availability phase and structurally attributed provenance. Do not let independently true component facts satisfy it, and do not infer it from a language version, successful compilation, backend spelling, or numeric operation count.
- Keep absence canonical: a schedule with no synchronization emits no synchronization requirement, target fact, explain row, or artifact field.
- Include every retained synchronization dimension and authority revision in schedule, KIR, target-profile, feasibility-rule, kernel, program, and artifact identity at the layer that owns it. Recompute domain and schema versions on the tree this work lands into; never copy a pinned value from an independently based branch.
- Preserve hard feasibility separately from cost. Unsupported or missing synchronization authority is `Unknown` or a typed rejection before executable-frontier admission; latency remains a cost fact and cannot establish legality.
- Carry the verified obligation and target realization through the artifact-facing program without creating a caller-declared ABI field or a second editable authority.

## Required evidence

- A zero-synchronization schedule succeeds against a sparse target profile containing no synchronization fact, and explain output contains no manufactured zero row.
- One bounded synchronized schedule reaches verified KIR and artifact construction only with an exact matching atomic target realization.
- Removing the target realization produces `Unknown` or the named fail-closed rejection before executable-frontier admission.
- One test per dimension proves that mismatched point identity, phase, placement, operation kind, participants, execution scope, visibility, fenced spaces, ordering, or convergence cannot satisfy the schedule.
- Identity mutation tests change each retained dimension and the target authority revision independently and observe every identity layer that owns that fact change.
- Canonical encode/decode and adversarial artifact tests prove the synchronization record cannot be reordered, partially omitted, or substituted while retaining identity.
- Every new check is perturbed once and observed failing. Run targeted per-package `cargo nextest run -p ...` and per-package Clippy while iterating, then `make full` once at the completed batch boundary.

## Closes when

The target-neutral vertical has one fully typed synchronized schedule whose KIR refinement, feasibility, explain, identity, and artifact paths agree; zero synchronization remains vacuous; every incomplete or mismatched authority fails closed with a typed cause; focused tests and `make full` pass; and Tom has reviewed the consequential public schedule, target-profile, and artifact boundaries.

## Graph maintenance

This ticket depends on `replace-or-justify-the-barrier-count-axis` for the fail-closed zero-synchronization baseline and on `represent-cooperative-workgroup-reduction-dataflow` for the first schedule that actually consumes synchronization. Its relation to `construct-and-bind-the-first-authoritative-metal-compile-profile` is non-blocking: that profile remains truthful without a synchronization row, and this ticket must not invent Metal evidence to broaden it. `implement-the-single-workgroup-synchronized-reduction-strategy` consumes the accepted point, and `realize-parallel-reduction-strategies-on-metal` owns backend qualification. Update `docs/ir.md`, `docs/compiler/fusion-and-scheduling.md`, `docs/artifact-abi.md`, the identity ledger, and any open-question entry whose status changes.

## Outcome

Delivered. One bounded cooperative workgroup reduction now carries a typed, schedule-owned synchronization point, lowers to a verified structured kernel whose barrier is proven to order its staged handoff, and reaches artifact construction only when one atomic provenance-bearing target fact establishes the exact realization it consumes. A schedule with no synchronization derives no requirement, consults no target fact, and emits no explain row. No Metal support was promoted.

### The point, field by field: how each is constructed and how each is verified

`SynchronizationPoint` lives inside `CooperativeTile` (`crates/tiler-ir/src/schedule/synchronization.rs`), not beside `ReductionTopology` on `KernelSchedule`. A point is placed at a *phase boundary*, and phases exist only inside a tile — so a point in a schedule with no phases is unstatable rather than merely refused, and the field breaks no `KernelSchedule` struct literal. That placement has a second consequence, and it is the ticket's own closing elimination: **a barrier in the pointwise, global-linear program cannot be stated at the schedule layer at all.** It is still refused at the layer where a barrier can actually be written — `KernelDiagnostic::UnexpectedSynchronization`, driven by `a_barrier_the_schedule_does_not_require_is_rejected_explicitly`.

| Field | Constructed | Verified |
| --- | --- | --- |
| `id: SyncPointId` | Producer-declared, dense ascending within the tile | `PointSequence`; the KIR barrier resolves *this* ordinal against the tile |
| `subject: SynchronizationSubject` | Producer-declared | Compared field-by-field against `required_subject(edges)`, which derives it from the tile's own visibility edges — five separate rules so a refusal names the dimension |
| `placement: PhaseBoundary { preceding, following }` | Producer-declared | `Placement`: both phases must exist and be consecutive. A "boundary" spanning a phase is not a program point — that phase's effects would fall on an undetermined side of the fence |
| `participants: ParticipantRange` | Producer-declared | `ParticipantSet`: must equal the tile's participant range |
| `convergence: ConvergenceEvidence` | Producer-declared *evidence class* | `ConvergenceEvidence` refuses `CallerAsserted` outright; `Convergence` then re-derives the named proof from the tile's per-phase participation |
| discharged edges | **Derived, never declared** | Exactly one point per edge: zero is `UndischargedVisibility`, two is `RedundantPoint`, and a point discharging none is also `RedundantPoint` |

**Convergence is where "no caller assertion" is made checkable.** The evidence class exists so that "the caller said so" is a value the model can hold and the verifier can reject by name, rather than a possibility the type system forecloses and no test can drive. `ConvergenceEvidence::CallerAsserted` is refused end to end. The *derivation* the admitted class names is re-checked rather than inherited from the tile's uniform-participation rule; that rule refuses a non-uniform phase first, so the derivation cannot fire end to end today, and it is driven directly instead (`the_convergence_derivation_refuses_a_phase_a_participant_skips`). Stated plainly so a later relaxation of the tile rule breaks this check rather than silently leaving every point convergent by inheritance.

**The kind is defined, not assumed.** `SynchronizationKind` has six variants; only `ControlBarrier` is admitted, and asynchronous copies, split-phase barriers, collectives, atomics, and inter-dispatch dependencies are *statable and refused by name*. Absence was considered and rejected: a target fact is keyed on the kind, so a single-variant vocabulary could not express — let alone refuse — a fact for a different kind satisfying a control barrier's requirement, and the ticket requires that mismatch to be drivable. Each unadmitted variant's doc says why its contract is undefined here.

**Two eliminations that are refusals rather than admissions.** A tile with fewer than two participants stages values it reads back itself, so program order already orders the handoff and a point there is the semantically redundant barrier — `SingleParticipant`. And the derived fence is required *exactly*, not as a superset: fencing device memory as well is a different realization with different target support, and admitting a superset would make two schedules with one meaning two identities.

### The KIR half

`BarrierSpec` gains `point: SyncPointId` and keeps its four spelling fields. They are **not one authority**: the point states the obligation, the spec is the self-contained emission fact a backend needs (it cannot reach the scheduled region from an operation), and `verify::barrier_subject` projects the spec onto the point's subject through one *total* mapping and requires equality — the same discipline that proves a kernel's declared `ResourceRequirements` equal to the derived record. Both projections are exhaustive matches inside `tiler-ir`, so widening either vocabulary is a build error at the one site that has to decide what the new spelling means.

Two operations were added, which `represent-cooperative-workgroup-reduction-dataflow` deliberately withheld: `StagedStore` and `StagedLoad`, each naming the tile **phase** whose declared staged access authorizes it. That phase is the witness — it is what a bounds witness is for a boundary access — and it is also what fixes when the effect happens relative to the visibility edges. `KernelBuilder::declare_staging` now returns a `KernelStagingId` in its own handle space, because a buffer parameter's position is its argument-table ordinal and an allocation's is not.

Five obligations, in the order a failure is most usefully reported: a region whose schedule owns no point contains no barrier and no staged access; every barrier names a declared point and every declared point is realized exactly once in order; every barrier sits at block depth zero; every barrier projects onto its point's subject; and every visibility edge's write precedes, and its read follows, the barrier realizing the point that discharges it. The last is the one the others cannot imply — a body can carry the right point and the right barrier and still read staged values ahead of it.

**The convergence rule is structural, and that is deliberate.** The barrier sits outside both guarded regions rather than inside a predicate that is *provably* uniform today. A predicate that is uniform under `TailPolicy::Exact` stops being one the moment the tail vocabulary admits a masked lane, and a rule resting on that would silently become wrong.

### The canonical body, and where it stops

```text
%gid = global invocation index        %lid = local invocation index
%out = %gid / participants            %par = %gid % participants
%act = %gid < work_items
if (%act) { fold contributors [%par·k, (%par+1)·k) ; staged_store[%lid] }
barrier(point)
if (%act) { if (%lid < commit) { fold staged[0..participants] ; store[%out] } }
```

The invocation split is emitted once at top level because the committing store needs the same output coordinate the producing loads used, and a value defined in a guarded block cannot cross into the next one. The staged store is inside the iteration guard and `grid_threads == work_items` is what makes that sound — every launched invocation satisfies the guard, so every slot the consuming phase reads was written; widening the tail vocabulary must revisit this emission rather than inherit it. `guard_values` gained the commit predicate (`local < commit.count`), admitted only when the commit range starts at participant zero, because `IndexLessThan` selects a prefix and cannot express "equals `k`".

`KernelDiagnostic::CooperativeLoweringShape` refuses every tile outside the one shape this emission has a body for — more than one allocation, more than two phases, a staged span other than one-slot-per-participant and whole-set-read, or a commit range not starting at zero. Representable and lowered stay different claims.

### The atomic target fact

`SynchronizationRealizationFact` (`crates/tiler-compiler/src/target/feasibility.rs`) carries one `SynchronizationSubject`, one `SynchronizationRealization` verdict, and an `Arc<FactSourceProvenance>` supplying phase, authority, validity, and evidence basis, attributed to the declaring profile. `TargetProfileBuilder::declare_synchronization_realization` takes **the whole subject as one argument**; there is deliberately no `declare_barrier_execution_scope` and no `declare_fenced_spaces`, and no accessor yields one dimension of a subject.

Resolution is one equality over the whole value. A profile carrying facts about neighbouring subjects resolves the required one as `NoPath` → `Unknown`. A matching fact declaring `Unrealizable` is a typed rejection carrying the whole refusing fact. A fact admissible only from a later phase is `Unknown`, never `Deferred`: deferral means a runtime can obtain the value before routing commits, and no property vocabulary can ask a device whether it orders a workgroup-scoped acquire-release fence over threadgroup memory.

The verdict is two-valued rather than a presence marker, mirroring `DTypeDispatchability`: a measured negative is a fact worth recording, and a profile that could only stay silent would make "unsupported" and "unmeasured" one state.

**Absence is canonical, and it is enforced at four places.** `derive_requirements` yields `None` for every topology that stages nothing; `region_proposal` composes no requirement from a `None`; `assess` resolves nothing and credits nothing; `record_target_admissions` emits no record. `a_zero_synchronization_candidate_needs_no_synchronization_fact` asserts the absence rather than the success, and the pipeline census test now asserts that no `target.synchronization*` rule key and no `SynchronizationRealization` event exists for a zero-synchronization program — beside the existing assertion that no `target.barriers` row does.

Not composed anywhere else, either: `subprogram_resources` refuses a subprogram whose stages require *different* realizations rather than taking one, because two stages requiring both means neither one alone is the subprogram's requirement.

### Identity execution, complete

| Domain | Step | Why |
| --- | --- | --- |
| `tiler.schedule.v3` | **unchanged** | The point encodes inside the appended `0x35` cooperative payload. No `0x35` region has ever reached a retained identity — the kernel verifier refused every one — so no persisted subject's bytes move. `STRICT_F32_REGION_IDENTITY_HEX` is unchanged and passes. |
| `tiler.kernel.v5 → v6` | **stepped** | `ResourceRequirements` gained `synchronization`, inside a fixed record followed by the value table and the whole body. Barrier and staged-access encodings moved with it at no extra cost: no `v5` kernel could contain either. |
| `tiler.kernel-program.v6` | **unchanged** | Folds the kernel identity by reference; its own layout is unchanged, and the folded bytes carry their own stepped separator. |
| `tiler.artifact-program.v13 → v14` | **stepped** | The same field in the entry's fixed resource record. |
| Neutral manifest `11.0 → 12.0` | **stepped** | Major: an `11.0` reader would consume the presence byte as the input-subnormal tag. |
| `tiler.target-profile.descriptor.v9 → v10` | **stepped** | The synchronization declaration decides verdicts exactly as a bound does. |
| `tiler.target-profile.declaration.v10 → v11` | **stepped** | The row family writes its own separator and count, so every profile's bytes move — including one declaring nothing, which is the point. |
| `…honourability.v4 → v5` | **stepped (key, not revision)** | A vocabulary widening: the rules decide a predicate `v4` could not express. Revision stays `1`. `CapabilityAxis` tag `0x08` stays retired — a subject is matched by equality and has no bound, the same reason numerical honourability is not an axis. |
| Target-requirement component schema `3.0` | **unchanged** | Its vocabulary did not change. |

Every pinned value recomputed on this tree: `STRICT_F32_REGION_IDENTITY_HEX` (unchanged, verified by passing); `physical.rs` `GOVERNED` declaration bytes; `metal_plan.rs` `ARTIFACT_IDENTITY` → `8cb1a5e2…928e951b` and `CACHE_SUBJECT` → `d4493d47…a943cc35bf`; the explain request qualifier `4d9f4773575b6679` → `1ac2bf9aeef5d035`, with the compiler-half derivation recorded in the accreting comment block; the four `crates/tiler-metal/goldens/*.metal` kernel digests (`84a9792d…`→`dcdce33e…`, `0ddb3a98…`→`b4366149…`, `246cc35e…`→`70bd2e38…`, `af5ca4a4…`→`cb0e7727…`) — their *region* digests are unchanged, which is the schedule domain holding; and the artifact domain/schema pins in `crates/tiler/src/route/tests.rs` and the codec tests.

**A zero-synchronization program's identity moving is the intended consequence.** The absence is now a recorded byte rather than an unstated fact, so a cache holding a `v13` subject — which described an entry that could not state the obligation at all — must miss rather than match.

### Evidence, each check watched failing

*Schedule* (`crates/tiler-ir/src/schedule/builder.rs`). `each_schedule_synchronization_rule_refuses_its_own_defect` drives eleven rules from one well-formed fixture, one changed fact each: `UnadmittedKind`, `Placement`, `ParticipantSet`, `ExecutionScope`, `VisibilityScope`, `FencedSpaces`, `Ordering`, `ConvergenceEvidence`, `UndischargedVisibility`, `RedundantPoint` (both directions), `PointSequence`. Plus `SingleParticipant` and the directly driven convergence derivation. `a_zero_synchronization_schedule_derives_no_requirement` and `a_synchronized_tile_derives_one_atomic_realization_requirement` are the positive pair.

*KIR* (`crates/tiler-ir/src/kernel/tests.rs`). `a_cooperative_region_lowers_to_a_staged_fenced_body` asserts the structure rather than describing it: one barrier, at top level, between the two guarded regions, realizing the declared point, with one staged write in the producing phase and two static staged reads in the consuming one. `each_kernel_synchronization_rule_refuses_its_own_defect` carries an explicit **control**: the unchanged hand-built body fails at `ReductionContract`, and each of six perturbations moves the diagnostic to its own rule — `SynchronizationConvergence`, `UndischargedVisibility`, `UnorderedStagedHandoff`, `UnexpectedSynchronization`, `SynchronizationContract`, `StagedAccessEvidence`. That control is what proves each rule fired rather than something upstream of it.

*Target authority* (`crates/tiler-compiler/src/target/feasibility.rs`), nine tests. The zero-synchronization absence; the exact match admitting and naming its provenance; the missing fact as `Unknown`; the declared negative as a typed rejection; **each of the five subject dimensions perturbed once, each satisfying nothing**; the later-phase fact as `Unknown` at compile profile and `Proven` at its own phase (so the refusal is a phase decision, not a dead branch); the declaration reaching the descriptor for every dimension; a contradictory declaration refused; a vacuous fence refused in both directions.

**The composition hazard, demonstrated refused** — `independently_true_component_facts_compose_into_no_permission`. The profile declares five realizations, each `Realized`: a collective, a subgroup arrival, a device-wide publication, a workgroup-and-device fence, and a sequentially consistent ordering. The test first asserts that every dimension of the required subject appears realized *somewhere* in that profile, then asserts the outcome is `Unknown`. A per-dimension authority would admit this candidate.

### Public drafts for Tom — not self-accepted

*New public module `tiler_ir::schedule::synchronization`* — types `SynchronizationKind`, `SynchronizationScope`, `FencedSpaces`, `MemoryOrdering`, `SynchronizationSubject`, `SynchronizationPlacement`, `ConvergenceEvidence`, `SynchronizationPoint`, `SynchronizationRule`; function `required_subject`; handle `SyncPointId`; constant `MAX_COOPERATIVE_SYNCHRONIZATION_POINTS`.

*New public field* — `CooperativeTile::synchronization: Vec<SynchronizationPoint>`. *New method* — `CooperativeTile::discharging_points`. *New variant* — `ScheduledRegionDiagnostic::Synchronization { rule }`.

*New public field on `ResourceRequirements`* — `synchronization: Option<SynchronizationSubject>`. This is the consequential one: it is the field that steps four identity domains.

*`tiler_ir::kernel`* — `BarrierSpec::point`; `OperationView::StagedStore`/`StagedLoad`; handles `KernelStagingId`, `VerifiedStagingId`; `KernelEntityKind::Staging`; `VerifiedKernel::staging_parameter`; `KernelBuilder::staged_store`/`staged_load`; `KernelDiagnostic::{SynchronizationContract, SynchronizationConvergence, StagedAccessEvidence, UnorderedStagedHandoff, CooperativeLoweringShape}`. *Behaviour change* — `KernelBuilder::declare_staging` returns `KernelStagingId` instead of `()`.

*`tiler_compiler::target`* — `SynchronizationSupport`; `TargetProfileBuilder::declare_synchronization_realization`; `TargetProfileBuildError::{DuplicateSynchronizationRealization, VacuousSynchronizationSubject}`.

### Scope, declared rather than escaped

The ticket's scopes gained `implementation/metal`, `implementation/build`, and `contracts/navigation`. The Metal edit is exactly one struct literal and one import in `crates/tiler-metal/src/tests.rs` plus four golden digests — the mechanical consequence of `BarrierSpec` gaining a field, not backend work. `implementation/build` covers the `metal_plan` identity rebaseline; `contracts/navigation` covers `docs/status.md`. Declared rather than escaped, and reported.

### What was deliberately not done

No Metal realization was declared. `crates/tiler-build/src/metal_declaration.rs` still declares no synchronization row, so a cooperative region is `Unknown` against the macOS profile — `realize-parallel-reduction-strategies-on-metal` owns backend evidence. No physical planner produces a cooperative region, so nothing routes one through the compile pipeline yet; that is `implement-the-single-workgroup-synchronized-reduction-strategy`. The Metal emitter's `barrier_call` was not touched and refuses what it always refused.

**Maturity, per the four-claims discipline.** The synchronization authority is a *tested guarantee* as a representation and as a verification: twenty-odd named rules, each driven to failure. The canonical body is *implemented support* at the IR layer — a verified kernel exists — and makes **no behaviour claim**: nothing has executed it, and no target declares it realizable. The target fact is an *architectural seam* with one synthetic governed authority proving the contract, not evidence about any real machine.

### Verification

`cargo fmt --all`; `cargo clippy -p tiler-ir -p tiler-compiler -p tiler-artifact -p tiler-metal -p tiler-build --all-targets -- -D warnings`; `cargo nextest run --workspace` (2145 passed, 5 skipped); `make full` green end to end including `cargo test --workspace --doc`, `RUSTDOCFLAGS="-D warnings" cargo doc`, the release-profile runs (738 passed), `ticketsplease lint` (`ok: no problems found`), and shellcheck. `git diff --check` clean.
