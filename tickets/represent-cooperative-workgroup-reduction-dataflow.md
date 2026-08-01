---
id: represent-cooperative-workgroup-reduction-dataflow
title: Represent cooperative workgroup reduction dataflow
status: in-progress
priority: p1
dependencies: []
related: [admit-the-first-typed-synchronization-point-and-atomic-target-authority]
scopes: [implementation/ir, implementation/compiler, implementation/reference, contracts/foundation, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: worker-workgroup
lease_expires_at: 1785586663
---
## User-visible outcome

A target-neutral schedule and KIR can describe the meaningful cross-invocation dataflow a bounded workgroup reduction needs before any synchronization point is admitted: local invocation coordinates, workgroup-shared staging storage, phased writes and reads, explicit lifetimes, and uniform participant convergence.

## Implementation keys

The current schedule has only a global-linear one-output mapping. KIR exposes boundary reads and one write, has no usable workgroup allocation or local-invocation coordinate, and rejects synchronization. Adding a barrier to that program is either semantically redundant or divergent under predication; it cannot prove cooperative execution.

Represent one bounded reduction tile whose participating invocations write disjoint partials to explicit workgroup storage and later consume the complete staged set. Define the participant set, local coordinates, storage shape/alignment/lifetime, phases, uniform reachability, and the exact dependency that requires visibility. Do not add a barrier or claim backend support here; this ticket constructs the dataflow the synchronization authority will govern.

## Required evidence

The verifier accepts one cooperative tile and rejects overlapping writes, out-of-lifetime reads, missing writers, nonuniform phase reachability, invalid local coordinates, insufficient storage, and a staged read with no producing phase. Zero-extent input retains the reducer's explicit identity without entering a barrier. Every check is perturbed once and observed failing.

## Closes when

The cooperative dataflow is explicit and verifier-owned across schedule and KIR, no synchronization or Metal support is overclaimed, exact public drafts are presented to Tom before acceptance, targeted `tiler-ir`/compiler/reference nextest and Clippy pass, and `admit-the-first-typed-synchronization-point-and-atomic-target-authority` can bind a real point to this dataflow.

## Graph maintenance

- Keep the synchronization-authority ticket downstream; this ticket must first make the cross-invocation dependency and uniform convergence meaningful.
- Keep Metal lowering and hardware support downstream of the target-neutral dataflow and synchronization strategies.
- Advance schedule/KIR identity only for encoded semantic changes and rebaseline pins on the merged tree.

## Outcome

Delivered. The cross-invocation dataflow of one bounded workgroup reduction is representable and verifier-owned in the target-neutral schedule and in the structured kernel IR, and it is refused end to end because nothing can yet order its handoff. No barrier was admitted, no Metal lowering was written, and no hardware claim was made.

### The dataflow model, and the eliminations behind its shape

**Where it lives.** `ReductionTopology::CooperativeWorkgroup` carries a `CooperativeTile` (`crates/tiler-ir/src/schedule/cooperative.rs`). The alternatives were eliminated against correctness rather than convenience. A `cooperative: Option<CooperativeTile>` field on `KernelSchedule` was rejected: it places the tile *beside* the topology instead of being one, so a region could declare `Serial` and a tile at once — two contradictory statements of how contributors combine with no rule tying them — and it breaks 39 struct-literal sites across eight crates for a field that would be `None` at every one. A new `ExecutionBinding` variant was rejected: the binding says how execution coordinates map to iteration coordinates and has nowhere to put staging, phases, or lifetimes, so the tile would end up split across two fields nothing forces to agree. A new `ScalarProgram` variant was rejected: the scalar program is *what* is computed per output, and a cooperative fold computes the same function as the serial one. The topology is the right home because it is the physical realization of one reduction, and `ReductionTopology::MultiPass` already carries a storage contract (`ContributorPartition`) inside a topology for the same reason — the two are siblings, one staging across a dispatch boundary and one inside a workgroup.

**Participant set and local coordinates.** `LocalCoordinates { source: LocalCoordinateSource::LocalLinearInvocation, participants: ParticipantRange { first, count } }`. The run must be the dense `0..count` (a run starting elsewhere leaves the workgroup's lower invocations with no coordinate, so nothing could say what they execute) and `count` must equal `schedule.threads_per_workgroup` — the uniform-convergence rule, stated so that any point placed in any phase is one every launched invocation reaches. Multi-dimensional local coordinates are absent rather than reserved: no consumer needs one, and widening `LocalCoordinateSource` is an appended tag rather than a reinterpretation of what a coordinate already means.

**Storage.** `WorkgroupStaging { id, element: StagedElement, slots, live_from, live_through }`. `StagedElement` is a vocabulary of its own rather than an `ArithmeticType` because the allocation needs a *storage width* to derive its local-memory requirement, and `ArithmeticType` deliberately carries none ("two formats can share a width and differ in bias, special values, or encoding", `crates/tiler-ir/src/schedule/numerics.rs:319-321`). The lifetime is declared rather than inferred from the accesses, so a phase reading an allocation the tile considers dead is a rejectable statement instead of a silent extension of its life.

**Phases and staged accesses.** `CooperativePhase { id, participation, writes, reads }`, with every staged access a `StagedSpan { stride, offset, count }`: participant `l` addresses the `count` slots at `stride * l + offset`. That form was chosen because it makes disjointness and coverage decidable *by enumeration* rather than by a modular argument, and it covers both shapes a bounded tile needs — one slot per participant on the producing side (`stride 1, count 1`) and the whole staged set read by every participant on the consuming side (`stride 0, count = participants`). `participation` is per phase precisely so a nonuniformly reached phase is expressible and therefore rejectable; a model that could not state the divergence could not refuse it.

**Convergence and commit.** Every phase's `participation` must equal the tile's participant range, and `commit` must name exactly one participant. The commit field is what makes `OwnershipProofKind::OneGlobalInvocationPerOutput` *true* of a workgroup running several invocations over one output: the proof's statement was already right, and the tile supplies the derivation the old `output_count == work_items` check stood in for. This is why **no new ownership evidence class was needed** — and that matters, because `OwnershipProofKind` is encoded untagged at a fixed offset inside the schedule identity, so a new variant there would have been a `tiler.schedule.v3 -> v4` step.

**The visibility dependency.** `CooperativeTile::visibility_edges()` derives one `VisibilityEdge { staging, produced_in, consumed_in }` per triple whose producing phase is strictly earlier. That is the entire content of "the exact dependency that requires visibility", and it is what `admit-the-first-typed-synchronization-point-and-atomic-target-authority` binds a real point to. Edges are deliberately *not* encoded into identity: they are a total function of the phases and accesses already encoded, so encoding them would add bytes no two distinguishable tiles differ in and would give a producer a second place to state a fact the verifier derives.

### Domain step: none required, appends only, with per-tag reasoning

Both identity domains hold. `tiler.schedule.v3` and `tiler.kernel.v5` are unchanged, and the workspace's pinned identities are the evidence: `STRICT_F32_REGION_IDENTITY_HEX` (`crates/tiler-ir/src/schedule/builder.rs`) and `ARTIFACT_IDENTITY` (`crates/tiler-build/src/metal_plan.rs:790`) both still match, and all workspace tests pass unmodified except for one match arm in a test-local name table.

| Construct | Encoding | Why no earlier subject's bytes move |
| --- | --- | --- |
| `ReductionTopology::CooperativeWorkgroup` | topology tag `0x35` | `0x31`–`0x34` keep their tags and field positions, exactly as `0x33` and `0x34` were appended. A reader reaching `0x35` is reading a region the earlier vocabulary could not express. |
| `Builtin::LocalInvocationIndex` | builtin tag `0x02` | `GlobalInvocationIndex` keeps `0x01`; the admitted-builtin list is length-framed, so a second entry lengthens a *new* kernel's list and moves nothing in an existing one. Same shape as `UnaryOp::F32Rsqrt`'s `0x02`. |
| `StagingParameter` list | written last in `encode_identity`, and **not written at all when empty** | The block encoding preceding it is fully self-framing, so after it the decoder is at a determined offset and "bytes remain" *is* the presence tag; a nonempty list carries its own length. Injectivity is preserved and a kernel that stages nothing encodes exactly the bytes it encoded before the list existed. An unconditional `push_len(0)` would have added eight bytes to every kernel ever encoded and forced `v5 -> v6`. |
| `ResourceRequirements.local_memory_bytes` | unchanged field, already encoded | Only its *value* changes, and only for cooperative regions, which did not previously exist. `derive_requirements` still yields `0` for every other topology. |
| `StagedElement`, `LocalCoordinateSource`, `PhaseId`, `StagingId`, `ParticipantRange`, `StagedSpan` | encoded only inside the `0x35` payload | Reachable only from the new tag. |

No field was inserted into any repeating record, no existing encoding was remapped, and nothing in `docs/artifact-abi.md` or `docs/status.md` needed to change — so `contracts/artifacts` was not added to this ticket's scopes.

### The seven required rejections, each perturbed once and watched failing

Every fixture below is the same well-formed `[2, 6] -> [2]` tile; each test changes exactly one fact, so the diagnostic names the rule the change violated. All in `crates/tiler-ir/src/schedule/builder.rs` tests.

| Required rejection | Perturbation | Diagnostic |
| --- | --- | --- |
| overlapping writes | producing span `stride: 1 -> 0` | `cooperative-staging-conflict` |
| out-of-lifetime reads | `live_through: 1 -> 0` | `cooperative-staging-lifetime` |
| missing writers | `slots: 3 -> 4` | `cooperative-staging-coverage` |
| nonuniform phase reachability | consuming phase `participation.count: 3 -> 2` | `cooperative-phase-participation` |
| invalid local coordinates | `participants.first: 0 -> 1` | `cooperative-local-coordinates` |
| insufficient storage | `slots: 3 -> 2` | `cooperative-staging-capacity` |
| staged read with no producing phase | read moved from phase 1 into phase 0 | `cooperative-staged-producer` |

Five further rules, each likewise perturbed: `cooperative-structural-limit` (a 65-phase tile, and an allocation one slot past `MAX_COOPERATIVE_STAGING_SLOTS` — driven because a bound nothing has been seen to trip is a bound that might not be reached at all), `cooperative-no-visibility-edge` (reads cleared — a tile that stages values nobody reads performs no cooperation), `cooperative-participant-convergence` (`threads_per_workgroup: 3 -> 6`), `cooperative-commit-ownership` (`commit.count: 1 -> 3`), and `cooperative-contributor-split` (`contributors_per_partition: 2 -> 3`). A separate test drives the ownership derivation directly: declaring `output_count: 6` instead of `2` fails `proof-reference`, which is the check that stops a cooperative region claiming one owned position per invocation and sizing its output three times too large.

### Zero-extent input

`a_zero_extent_reduction_keeps_its_identity_without_a_tile` (schedule) and `a_zero_extent_reduction_commits_its_identity_without_a_loop_or_a_barrier` (kernel). The existing authority is unchanged and is what the tests assert against: every reduction arm of the intrinsic verifier requires `empty_identity_bits == 0.0_f32.to_bits()`, and `emit_reduction`'s `plan.contributors == 0` arm commits that constant with no loop. The kernel test confirms zero `SerialLoop`s, zero `Barrier`s, zero staging declarations, `local_memory_bytes == 0`, and a stored value of `F32Bits(0x0000_0000)`. A cooperative tile over the same domain is refused as `cooperative-empty-contributor-domain`: an empty reduction commits its identity from one invocation, so a tile there would declare a visibility edge over values no participant produces.

### The KIR half, and why it stops where it does

KIR gained the two things the ticket named as missing, and one refusal.

- **A usable local invocation coordinate.** `Builtin::LocalInvocationIndex`. `verify_signature` now *requires* it for a cooperative region and *forbids* it otherwise; both directions are tested (`builtin-contract`).
- **A usable workgroup allocation.** `StagingParameter`, declared through `KernelBuilder::declare_staging` into a list separate from the buffer parameters, and proven against the region's tile by ordinal, element type, address space, and slot count (`staging-contract`, driven in six directions including a region that stages nothing declaring storage). The separation is a **correctness requirement, not a preference**: a buffer parameter's position *is* its argument-table ordinal (`VerifiedKernel::declared_buffers`), so putting workgroup storage in that list re-bases every later ordinal and changes what an existing signature position means. Keeping the lists apart also keeps `BufferAccess` a two-value vocabulary — staging is read *and* written by the workgroup, which no parameter access mode expresses, and widening `BufferAccess` would have been an artifact-ABI change in a crate this ticket does not hold.
- **A derived refusal.** `KernelDiagnostic::UndischargedVisibility`. A region whose tile carries any visibility edge is rejected before any body is derived, and `lower_scheduled_region` refuses it before inserting a single operation. This is not a placeholder: the barrier vocabulary is refused intrinsically, and no schedule owns a synchronization point a barrier could be matched to, so there is no construct a cooperative kernel could contain that would discharge the edge. Emitting a body realizing the tile's phases would have meant *authoring a race*, which is why no canonical lowering exists.

**What KIR deliberately did not gain: staged load and store operations.** They would need a staging witness class and a phase-tagged effect, and both are only correct in the presence of the point that orders them. `replace-or-justify-the-barrier-count-axis` already assigned that composition to the synchronization ticket — "independently asserted component facts are not composable evidence" — so reserving the operations here would be a type-system reservation dressed as implemented support. `declare_staging` therefore returns no handle, and says why: nothing in the current vocabulary can address the storage, so a handle would reference something no operation can reach.

### Relaxed verifier rules, each justified

Two of the four rules the tiled-contraction stop report tabulated were relaxed, and only as far as the derivation required.

- **"Buffer count is exactly `reads + 1`" is unchanged.** The stop report assumed staging would be buffer parameters; it must not be, per the ordinal argument above, so the parameter-count rule keeps its exact form and workgroup storage is counted in its own list.
- **`AddressSpace::Workgroup` as a buffer parameter moved from conditionally admitted to always rejected.** `verify.rs`'s old arm admitted one when `derived.local_memory_bytes > 0` — dead code while that value was hardcoded to zero, and now reachable and *wrong*. The rule is about the binding namespace, not about whether local memory is needed, which is why the test that drives it uses a cooperative region that genuinely requires workgroup storage.
- **Admitted builtins are no longer a function of `ExecutionBinding` alone.** The binding fixes the global coordinate; the tile adds the local one. Required rather than permitted, so a tile whose kernel cannot name its participants is refused.
- **The ownership and write-bounds checks read owned output positions rather than work items.** `owned_output_positions` returns `work_items` for every topology in which one invocation owns one output, and `work_items / participants` for a cooperative tile. This is the one place a cooperative region's shape changes an existing rule, and it fails closed: a work-item count that is not a multiple of the participant count yields `None` and rejects.
- **`verify_contributor_loop` states no obligation for a cooperative tile.** The fold's shape is decided together with the synchronization point that separates its phases; stating one now would be a contract written against a body nothing can produce.

### The reference crate's role: none, and the reason is checkable

**Fact.** `tiler-reference` consumes `NumericalRealization` and nothing else from the schedule (`crates/tiler-reference/src/conformance.rs:38-41`); it has no notion of a reduction topology. **Fact.** `ReferenceNumericalConformance::from_realization` refuses any realization permitting reassociation with `UnsupportedReferenceContract::ReassociationPermitted` (`conformance.rs:189`), because such a contract admits a result set rather than one value. **Inference.** A cooperative tile *is* a reassociation of the declared contributor sequence and the verifier admits the topology only when `permits_reassociation` holds, so the reference evaluator refuses a cooperative region's contract by construction — the oracle could not answer even if a body existed. No `tiler-reference` change was warranted and none was made; the refusal already has a test at `conformance.rs:410`.

**Maturity, stated per the four-claims discipline.** The cooperative dataflow is a *tested guarantee* as a representation: the schedule verifier proves eleven named properties of it and each is driven to failure. It is an *architectural seam* toward execution: the visibility edges are the exact obligation a synchronization point will be bound to. It is *not implemented support* — no kernel body, no emission, no target. And it makes **no behaviour claim at all**: the tile has no observable semantics today because nothing realizes it. **Inference, not measurement:** a cooperative tile with partition `P` denotes the same value as a two-dispatch `MultiPass` split with the same partition, since both fold the same contiguous contributor ranges in the same order; confirming that bitwise needs an executable body and belongs downstream.

### Public drafts for Tom

Every item below is a draft presented for acceptance, not self-accepted.

*New public types in `tiler_ir::schedule`* — `CooperativeTile`, `CooperativePhase`, `WorkgroupStaging`, `StagedWrite`, `StagedRead`, `StagedSpan`, `LocalCoordinates`, `LocalCoordinateSource`, `ParticipantRange`, `StagedElement`, `VisibilityEdge`, `PhaseId`, `StagingId`, `CooperativeTileRule`.

*New public variants* — `ReductionTopology::CooperativeWorkgroup { partition, tile, axes, order, accumulation, permits_reassociation, permits_permutation }`; `ScheduledRegionDiagnostic::CooperativeTile { rule }`; `Builtin::LocalInvocationIndex`; `KernelDiagnostic::StagingContract`; `KernelDiagnostic::UndischargedVisibility`; `KernelLimitKind::Staging`.

*New public functions and methods* — `CooperativeTile::visibility_edges`, `CooperativeTile::local_memory_bytes`, `ParticipantRange::end`, `ParticipantRange::contains_range`, `StagedElement::storage_bytes`, `StagedElement::tag`, `LocalCoordinateSource::tag`, `CooperativeTileRule::rule`, `PhaseId::{FIRST, new, get}`, `StagingId::{FIRST, new, get}`, `schedule::cooperative_tile`, `schedule::cooperative_local_memory_bytes`, `KernelBuilder::declare_staging`, `VerifiedKernel::staging`.

*New public type in `tiler_ir::kernel`* — `StagingParameter`.

*New public constants* — `MAX_COOPERATIVE_PARTICIPANTS`, `MAX_COOPERATIVE_STAGING_SLOTS`, `MAX_COOPERATIVE_PHASES`, `MAX_COOPERATIVE_PHASE_ACCESSES`, `MAX_KERNEL_STAGING`. These are verification bounds that keep enumeration finite, not hardware claims; a target's own workgroup-thread and local-memory axes are what refuse a launch it cannot run.

*Behaviour change to an existing public rule* — `AddressSpace::Workgroup` is no longer admissible as a `BufferParameter` address space under any condition.

### Feasibility composition (unchanged, and checked)

A cooperative region's nonzero `local_memory_bytes` is already fail-closed against a target: `physical.rs:1696-1699` composes it as `AxisRequirement::new(CapabilityAxis::LocalMemoryBytes, requirements.local_memory_bytes)`, and that axis carries `CapabilityRelation::AtMost` (`target/feasibility.rs:283`). No compiler change was needed, and none was made.

### Verification

`cargo fmt --all`; `cargo clippy -p tiler-ir -p tiler-compiler -p tiler-reference --all-targets -- -D warnings`; `cargo nextest run --workspace` (2127 passed, 5 skipped); `make full` green end to end, including `cargo test --workspace --doc`, `RUSTDOCFLAGS="-D warnings" cargo doc`, the release-profile reference and compiler runs (728 passed), `ticketsplease lint` (`ok: no problems found`), and shellcheck. `git diff --check` clean.
