---
id: reconcile-input-ordinal-region-local-and-declared-input-semantics
title: Reconcile InputOrdinal region-local and declared-input semantics
status: in-progress
priority: p1
dependencies: [decide-the-schedule-local-input-ordinal-model, decide-the-full-list-access-coordinate-for-out-of-list-references]
related: [decide-the-source-bound-live-row-major-access-surface, admit-symbolic-extents-through-schedule-formation, associate-live-extent-operands-with-symbolic-semantic-interface-axes, scope-an-in-place-append-into-a-caller-retained-allocation]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal, implementation/build, implementation/conformance, implementation/runtime, contracts/foundation, contracts/artifacts, research/verification, research/target-profiles, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [defect, public-boundary, schedule, identity, shapes]
claimed_from: todo
assignee: worker-reconcile-input-ordinal
lease_expires_at: 1786743680
---
## User-visible outcome

One checked meaning governs the exact local access coordinate and fieldless
`TensorRole::Input` across schedule construction, kernel lowering,
program-stage binding, artifact extent rows, identity, and compiler subject
binding. A region-local position is never treated as a program-interface key,
and a declared program ordinal is never accepted where an exact ordered access
position is required.

## Exact-current-base Fact audit — 2026-08-14, `4e10b98066f846ca50de4c97ba6262dade9e0865`

The narrow source paths originally cited behind the first seven Facts are
byte-identical to the earlier
`bbbf936ad3d8170ec601cd26eda5235cc2ac1d6b` audit base. That is not a complete
relevant-source claim: artifact codec/model/verification, compiler
selection/session, IR program ABI/builder/error/model/tests/verification, and
`docs/artifact-abi.md` changed under `ce3e0d79`, `7ead2c0a`, and `6406654c`.
Those exact-current paths were re-read. Their selected-implementation provenance
and retired ABI-expression-key changes are orthogonal to this coordinate defect,
but the earlier broad statement that authority-bearing implementation did not
change was false and is replaced here.

- **Verified — the defining type says region-local.** `crates/tiler-ir/src/schedule/handles.rs`, anchor `The ordinal is *region-local and positional*`, requires a region reading `n` inputs to use every ordinal in `0..n` exactly once, explicitly says the handle is not an interface key, and assigns named-input binding to stage accesses.
- **Verified — the containing role says the opposite.** `crates/tiler-ir/src/schedule/model.rs`, anchor `Which declared input tensor this access binds`, says `TensorRole::Input.ordinal` is a declared program-input ordinal, may differ from the access position, and is resolved against the program interface.
- **Verified — intrinsic verification admits sparse declared ordinals.** `crates/tiler-ir/src/schedule/builder.rs`, anchors `The ordinals need not be the dense prefix` and `two distinct ascending program ordinals need not be dense`, deliberately accepts ordinals such as `1` and `7` and calls them program ordinals.
- **Verified — the physical compiler supplies declared ordinals.** `crates/tiler-compiler/src/physical.rs`, anchor `The recognized ordinal, not the first declared input`, copies normalized declared-input ordinals into `TensorRole::Input`; helper tests project `0, 1, 7`, the end-to-end positive admits later input `1`, and forged `0`/`7` roles refuse request binding. Separately, `crates/tiler-ir/src/schedule/builder.rs`, anchors `two distinct ascending program ordinals need not be dense` and `the input ordinal is part of scheduled-region identity`, admits sparse roles intrinsically and pins their schedule-identity effect.
- **False as originally written — program assembly resolves the role directly, while artifact mapping is positional.** `crates/tiler-compiler/src/program.rs`, anchor `AssemblyBinding::Input(ordinal)`, converts `TensorRole::Input.ordinal` directly to a semantic program input index. Separately, `crates/tiler-artifact/src/program/builder.rs`, anchor `maps that tensor through the stage access`, matches the kernel buffer role, zips buffers with stage accesses, then reads `MaterializedOrigin::ProgramInput { key }`. The original Fact omitted the live compiler consumer and therefore understated the defect.
- **Verified — the same public type carries two constructibly distinct coordinates.** `crates/tiler-ir/src/schedule/pointwise.rs` and `pointwise_bf16.rs`, anchors `input_ordinals_are_dense` and `SparseInputOrdinals`, require expression leaf ordinals to be the dense local prefix. `request.rs`, anchor `leaves by the position of the read`, mints them from read-list position. `builder.rs`, anchor `A read's position and its boundary role are separate facts`, simultaneously allows the access role to name a sparse declared input.
- **Verified — the compiler retains both facts, but assembly does not project their checked association.** Normalized pointwise, epilogue, fold, and contraction subjects carry ordered reads plus declared ordinals. `verify_schedule_with_feasibility` proves the region against that exact request subject before `physical.rs::VerifiedScheduledRegion` stores both values. `CoverAssembly` lacks a typed projection/accessor from the retained subject and therefore reuses the schedule role as the program binding.
- **Imprecise as originally written — the accepted repair is consequential but did not settle every public coordinate.** The accepted fieldless role retires the role's ordinal payload from every input-bearing schedule and kernel identity. That requires `tiler.schedule.v5` → `v6` and `tiler.kernel.v7` → `v8`; the exact access-coordinate grammar also moves `tiler.compiler.physical-implementation-proposal.v2` → `v3`. Existing opaque-call explain subjects change from `input#N` to `access#N`, so `EXPLAIN_SCHEMA_VERSION` moves 10 → 11 for canonical record bytes and `EXPLAIN_RENDERER_VERSION` moves 8 → 9 for the existing rendered spelling. The request-subject, kernel-program, artifact-stage, artifact-program, compilation-explain wrapper, and manifest-schema grammars remain unchanged, and the artifact extent row remains `(InputKey, Axis, AbiType)`. Their values still move when they fold a stepped identity or a changed valid `InputKey` binding. The accepted dependency fixed fieldless roles and compiler projection, but its retained input-only `InputOrdinal` cannot name the intermediate/output negatives required of a full-list extent reference.
- **Verified — the coincidence is not a contract.** Whole-program pointwise currently reads a dense declared prefix, so local and declared ordinals coincide there. Subset, epilogue, fold, and contraction construction already claim populations where they differ. The accepted fieldless model removes that coincidence as an authority: ordered access position is local, and only the checked compiler projection may associate it with a declared program input.
- **False — retaining `InputOrdinal` for every out-of-list reference is not a complete implementable instruction.** Its public contract is dense over boundary input tensors only, while a pointwise epilogue's ordered read run may be `[Intermediate, input 2]` and the required extent negative must name/reject that intermediate plus the final output write. Filtering inputs shifts the coordinate and cannot express either negative. [`decide-the-full-list-access-coordinate-for-out-of-list-references`](decide-the-full-list-access-coordinate-for-out-of-list-references.md) owns the stopped public replacement and exact diagnostic surface.
- **Verified — current `InOut` evidence bypasses regional access coverage, and the mutating profile is deliberately unmodelled.** `crates/tiler-compiler/src/call_abi.rs`, anchors `Both read and written` and `ParameterLayout::Both`, defines a valid lower-level in-place declaration. `crates/tiler-compiler/src/frontier.rs`'s `an_in_out_binding_yields_both_a_requirement_and_a_guarantee` calls derivation directly with one tensor role and never proves the distinct regional read and owning-write accesses. `crates/tiler-compiler/src/boundary.rs`, anchor `version-producing step there, together`, requires a value-version dimension and a `KernelProgram` version-producing step together. The decision packet's single `AccessOrdinal` row therefore supports regional `In` → `Read` and `Out` → `Write` only and must refuse `InOut` distinctly before coverage or derivation; Q-PLAN-015 remains the owner of broader in-place execution.

Reproduce:

```sh
rg -n 'region-local and positional|not an interface key|Which declared input tensor this access binds' crates/tiler-ir/src/schedule/handles.rs crates/tiler-ir/src/schedule/model.rs
rg -n 'The ordinals need not be the dense prefix|program ordinals need not be dense|The recognized ordinal, not the first declared input|declared ordinal is retained' crates/tiler-ir/src/schedule/builder.rs crates/tiler-compiler/src/physical.rs
rg -n 'region-local input and axis|maps that tensor through the stage access|MaterializedOrigin::ProgramInput' crates/tiler-ir/src/kernel/model.rs crates/tiler-artifact/src/program/builder.rs
rg -n 'AssemblyBinding::Input|input_ordinals_are_dense|SparseInputOrdinals|leaves by the position of the read' crates/tiler-compiler/src/program.rs crates/tiler-compiler/src/request.rs crates/tiler-ir/src/schedule/pointwise.rs crates/tiler-ir/src/schedule/pointwise_bf16.rs
rg -n 'EXPLAIN_SCHEMA_VERSION|EXPLAIN_RENDERER_VERSION|INPUT_ROLE_PREFIX|proposal_subject_is_exact_ordered_and_bounded' crates/tiler-compiler/src/explain.rs crates/tiler-compiler/src/call_registry.rs
rg -n 'Both read and written|ParameterLayout::Both|an_in_out_binding_yields_both_a_requirement_and_a_guarantee|version-producing step there, together' crates/tiler-compiler/src/call_abi.rs crates/tiler-compiler/src/frontier.rs crates/tiler-compiler/src/boundary.rs
rg -n 'tensor_role_comment|LiveRowMajor|InputOrdinal|TensorRole::Input' crates/tiler-metal/src crates/tiler-build/src crates/tiler-conformance/src crates/tiler-runtime/src
```

## Accepted boundary — 2026-08-14

Tom accepted the dependency packet's Option 3 in the live Codex conversation,
relayed by the coordinating agent. Replace
`TensorRole::Input { ordinal: InputOrdinal }` completely with fieldless
`TensorRole::Input`. Ordered schedule-access and kernel-buffer position is the
sole coordinate for members already in those lists. The compiler projects
local-to-declared association from the already-retained checked
`VerifiedRequestSubject`, `CoverAssembly` consumes that authority, and
`InputKey` is reached only through the exact stage access/materialized origin.
No public declared-input ordinal, positional coincidence, search for a first
input, or second retained association vector is accepted.

Tom then accepted the exact complete-replacement surface in
[`decide-the-full-list-access-coordinate-for-out-of-list-references`](decide-the-full-list-access-coordinate-for-out-of-list-references.md)
in the live Codex conversation on 2026-08-14, relayed by the coordinating
agent. That acceptance removes the stop condition: public `InputOrdinal` is
retired in favour of full-list `AccessOrdinal`, declared-interface association
stays compiler-private, and the exact diagnostics, opaque-call rules, and
identity steps in this ticket are authorized together.

## Required work

- After the awaiting-decision dependency is accepted, re-audit every constructor of its selected access coordinate and every `TensorRole::Input` verifier, encoder, subject binder, kernel/program/artifact consumer, and test. This includes Metal `tensor_role_comment`, docs, and fixtures; build assembly and custom fixtures; conformance construction and role matching; and runtime live-extent fixtures.
- Implement fieldless `TensorRole::Input` and the accepted full-list access coordinate completely. Remove the superseded public coordinate rather than keeping an alias; do not add a declared-input type to shared IR or use positional coincidence as proof. The exact public spellings are `PointwiseF32Node::Input { access: AccessOrdinal }`, `PointwiseBf16Node::Input { access: AccessOrdinal }`, both builders' `input(access: AccessOrdinal)`, and `ReductionTopology::LiveContraction { live_access: AccessOrdinal, live_axis, .. }`. Rename both public `SparseInputOrdinals { missing: u32 }` variants to `SparseAccessOrdinals { missing: AccessOrdinal }` and their stable rules to `pointwise-f32-sparse-access-ordinals` and `pointwise-bf16-sparse-access-ordinals`; retain `MissingInput` and its rules. Retained compiler-private declared association fields (`prologue_reads`, `contributor_input`, `NormalizedContractionRead`, and `BoundaryRead::Input`) plus queries such as `NormalizedOutput::input_elements_at` and `NormalizedProgram::agreed_input_elements_at` use one crate-private `DeclaredInputOrdinal`, never retained raw `u32` or the shared access type.
- Eliminate the first-role searches in kernel construction/verification and artifact derivation plus the axis-only selection in live lowering. `InputExtentParameter { access: AccessOrdinal, axis: Axis }` directly indexes the complete access/buffer list and checks new public unit variant `KernelBuildError::InputExtentAccessOutOfRange`, existing `InputExtentNotInput`, then existing `InputExtentWrongAxis`; duplicates remain keyed by `(access, axis)`. The public artifact error is exactly `ArtifactBuildError::ExtentOperandUnbound { entry: usize, access: AccessOrdinal, axis: u32 }`. Its wire row remains unchanged.
- Migrate `OpaqueCallProposal.bindings` completely to ordered `(parameter_name, AccessOrdinal)` pairs. Derive the complete read-prefix plus owning-write access view from the retained checked request subject and cover assignment, never from provider order or a parallel authority. Keep every ABI parameter exactly once; check access bounds in proposal order; then refuse generic ABI `InOut` as compiler-private `InOutRegionUnsupported { parameter: &'static str, access: AccessOrdinal }` with stable reason `opaque-call.binding.inout-region-unsupported` before mode, coverage, storage, or boundary derivation; admit only exact regional `In` → `Read` and `Out` → `Write`; require every boundary access at least once; permit distinct compatible ordinary parameters on one access; and compare encoding/alignment plus required layouts among `In` readers and guaranteed layouts among `Out` writers. Build at most one agreed requirement for each read and one agreed guarantee for the owning write in access-list order, while keeping provider binding order in proposal identity. Resolve `WorkScaling::PerElementOf` through the named exact access. Also add compiler-private `AccessOutOfRange`, `AccessModeMismatch`, `UnboundAccess`, and `AccessStorageDisagreement` binding errors and their `opaque-call.binding.*` explain reasons; remove role-keyed `RoleStorageDisagreement`. Retain generic `ParameterRole::InOut` and `ParameterLayout::Both`, keep `call_abi::roles_report_their_access` and `call_abi::a_layout_must_state_the_direction_its_role_has`, add lower-level `call_declaration::an_in_place_parameter_with_may_alias_inputs_is_coherent`, and delete frontier's unchecked `an_in_out_binding_yields_both_a_requirement_and_a_guarantee` rather than citing it as regional evidence. A paired `{ read, write }` binding is broader mutating-profile work under [Q-PLAN-015](../docs/open-questions.md#q-plan-015--advanced-buffer-reuse-and-in-place-execution) and its existing deferred tracking ticket; it requires the value-version boundary dimension and `KernelProgram` version-producing step together and is not implemented here. Do not add `binding-access-subject-mismatch`: rebinding to another valid compatible access is another valid proposal.
- Align defining docs, containing-role docs, intrinsic validation, physical construction, program-stage binding, artifact mapping, canonical encoders, version ledgers, and pins with schedule `v6`, kernel `v8`, compiler physical-proposal `v3`, explain schema `v11`, and explain renderer `v9`. The explain-schema step governs the changed opaque binding record and new `InOutRegionUnsupported` refusal record; the renderer step governs `input#N` → `access#N`. Update `STRICT_F32_REGION_IDENTITY_HEX`, `ABSENT_SUBGROUP_KERNEL_IDENTITY_HEX`, `GOVERNED_PROPOSALS`, `proposal_subject_is_exact_ordered_and_bounded`, `explain_vocabulary_is_append_only_and_versioned`, `deterministic_trace_is_sealed_and_rendered_separately`, the exact opaque-call pipeline subjects, every `tiler-explain-v8` header assertion, `crates/tiler-compiler/src/domains.rs`, `docs/compiler/optimizer.md`, and `docs/artifact-abi.md`. Keep kernel-program `v11`, artifact-stage `v3`, artifact-program `v17`, manifest schema 17.0, request-subject `v6`, compilation-explain wrapper `v1`, and the artifact `(InputKey, Axis, AbiType)` extent-row grammar unchanged unless new evidence forces a separate stop; regenerate values that fold moved nested identities or changed valid interface bindings.
- Preserve named `InputKey` as the program-interface authority. A local access coordinate reaches it only through the compiler's retained checked subject projection and exact stage access/materialized origin.

## Required evidence

- Sparse declared inputs `1` and `7` map from local access positions `[0, 1]` to the intended `InputKey`s, while the dense-prefix neighbour remains unchanged. Rebuild an independently verified request with a different valid declared association: schedule and kernel bytes stay identical, request-subject identity changes, `CoverAssembly` reaches the new intended keys, and kernel-program/artifact identities change. This is a positive rebinding control, not a `request-binding` refusal.
- Preserve and count the full association population: subset `[0, 2]`; later fold over `[1]`; two-of-three contraction; epilogue `[Intermediate, input 2]`; and two mapped reads of one declared input. Each local position maps to exactly the normalized subject member at that position, without filtering or deduplication.
- Independently perturb the checked owners that can actually disagree. Pair a semantic program with the wrong verified request and quote existing `semantic-request-binding`; separately pass a `VerifiedScheduledRegion` wrapper minted under request A to program verification under request B and quote existing `request-subject`. Do not require a standalone public `KernelProgramBuilder` to infer same-role intent the verified kernel does not carry.
- For epilogue `[Intermediate, input 2]`, prove extent access `0` fails `InputExtentNotInput`, access `1` reaches input `2`, the final write fails `InputExtentNotInput`, and an absent access fails `InputExtentAccessOutOfRange`. Swap only the axis separately and quote `InputExtentWrongAxis`.
- For opaque calls, swap two named `In` parameters across two valid compatible read accesses with distinguishable element counts. Both registered proposals admit; proposal identity and the exact opaque subject change, binding order stays identity-bearing, and `WorkScaling::PerElementOf` follows the parameter's newly named exact access. Drive both exact subjects through the same typed refusal separately and prove their canonical explain records, rendered `access#N` subjects, and trace identities differ. Admit the ordinary `[read, owning write]` control with one `In` and one `Out`, proving complete coverage plus access-ordered requirement/guarantee facets. Then declare a coherent one-parameter generic `InOut`/`Both` ABI with `Aliasing::MayAliasInputs` over the same two-access region, bind it to the valid read, and quote `InOutRegionUnsupported { parameter: "buffer", access: 0 }` and `opaque-call.binding.inout-region-unsupported`; prove this fires before `UnboundAccess(1)` and boundary derivation while the lower-level generic ABI tests stay green. Separately perturb `In` onto the write and `Out` onto a read (`access-mode-mismatch`), an absent coordinate (`access-out-of-range`), and complete coverage (`unbound-access`). Bind two distinct compatible `In` parameters to one read and, independently, compatible `Out` parameters to the owning write; prove one access-ordered facet in each case, then perturb only storage and quote `access-storage-disagreement`. No valid-access subject-mismatch diagnostic exists.
- Perturb schedule, kernel, physical-proposal, explain-schema, and explain-renderer domain spellings independently and quote each owning pin/header failure. Prove unchanged artifact extent-row bytes, manifest schema 17.0, and unchanged-domain grammar, then regenerate every downstream value that folds a moved nested identity or changed interface binding.

## Implementation record — 2026-08-16

- The public replacement is complete: `AccessOrdinal` is the only shared local
  coordinate, `TensorRole::Input` is fieldless, and no live Rust source under
  `crates/`, `prototypes/`, or `spikes/` retains `InputOrdinal`. Pointwise
  leaves, live contraction, live extent operands, kernel buffers, artifact
  derivation, and opaque calls all use the exact ordered access position.
- `VerifiedScheduledRegion::declared_input_at` projects an exact local access
  through its retained checked `VerifiedRequestSubject` to private
  `DeclaredInputOrdinal`. `CoverAssembly` consumes that projection; artifact
  construction then reaches `InputKey` only through the exact stage access and
  `MaterializedOrigin::ProgramInput`. The existing conformance populations for
  subset `[0, 2]`, later fold `[1]`, two-of-three contraction, staged epilogue,
  and repeated mapped reads all pass with the intended values.
- Live extent declaration directly indexes the full access list. The mixed
  epilogue control proves staged access `0` and the final write refuse as
  `InputExtentNotInput`, access `1` succeeds, access `3` refuses as
  `InputExtentAccessOutOfRange`, and the wrong axis refuses independently as
  `InputExtentWrongAxis`. The artifact envelope remains `(InputKey, Axis,
  AbiType)` and its codec/schema population remains unchanged.
- Opaque-call proposals retain ordered `(parameter, AccessOrdinal)` bindings.
  Admission checks bounds, rejects regional `InOut`, checks exact direction,
  complete access coverage, and shared-access storage, then derives one facet
  per access in access-list order. Swapping two compatible reads changes both
  proposal identity and `PerElementOf` work resolution. The lower-level generic
  `InOut`/`Both` ABI stays constructible, while regional admission refuses it
  before the missing-write coverage error; Q-PLAN-015 remains the owner of a
  future versioned mutating boundary.
- Identity moved coherently to schedule v6, kernel v8, physical proposal v3,
  explain schema v11, and renderer v9. Kernel-program v11, artifact-stage v3,
  artifact-program v17, manifest 17.0, request-subject v6, and compilation-
  explain wrapper v1 remain unchanged. The regenerated standard Metal artifact
  identity is `3e113510c0eeb090968a8b7adb445f6bd64ed4c66b4b3ad440f032716e147be5`,
  its cache subject is
  `5ffb36be5c5714e9982153717d2af2adaef440afc4bf6144c81b3859757771a7`,
  and fixed content is 77,062 bytes.
- Live path-dependent spikes were migrated too. The scalar CPU vertical checks
  against the current crates. The Kani encoder shim reports 29 copied items in
  sync and `cargo kani --harness push_tensor_role_injective` verifies the whole
  three-role fieldless population with 0 of 322 checks failed (six unreachable
  checks, 0.667 s verification on the recorded host).

## Verification — 2026-08-16

Positive gates completed on the implementation worktree:

```sh
cargo fmt --all -- --check
cargo nextest run -p tiler-ir -p tiler-compiler -p tiler-artifact -p tiler-metal -p tiler-build -p tiler-conformance -p tiler-runtime
# 2,778 passed; 3 skipped
cargo test -p tiler-ir -p tiler-compiler -p tiler-artifact -p tiler-metal -p tiler-build -p tiler-conformance -p tiler-runtime --doc
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps -p tiler-ir -p tiler-compiler -p tiler-artifact -p tiler-metal -p tiler-build -p tiler-conformance -p tiler-runtime
cd spikes/target-profiles/scalar-cpu-vertical && cargo check
cd spikes/verification/kani-encoder-injectivity && ./guard.sh
cd spikes/verification/kani-encoder-injectivity && cargo check
cd spikes/verification/kani-encoder-injectivity && cargo kani --harness push_tensor_role_injective
```

Subject perturbations were applied one at a time and restored:

- forcing every compiler access projection to `AccessOrdinal::FIRST` made the
  sparse association control fail as
  `InvalidCompilerOutput(Program(CoreVerification(UnusedValue)))`;
- replacing each opaque-call access in proposal identity with zero made
  `swapped_opaque_read_bindings_remain_distinct_and_resolve_exact_work` fail
  `assertion left != right failed` because the two
  `ImplementationProposalIdentity` values collided;
- forcing live extent declaration to inspect access zero made
  `an_extent_operand_names_one_exact_epilogue_access` fail at `access 1 is the
  exact declared-input read`;
- reverting only the schedule encoder to `tiler.schedule.v5` made
  `every_tiler_spelled_literal_is_pinned_or_classified` refuse that literal as
  absent from `PINNED_IDENTITY_DOMAINS`.

## Closes when

The new awaiting-decision dependency is accepted and consumed, public docs and
all construction/consumption paths agree on one checked access-coordinate
model, the complete association and refusal census above passes, every governed
identity migration is coherent, and
[`decide-the-source-bound-live-row-major-access-surface`](decide-the-source-bound-live-row-major-access-surface.md)
can select an exact source field type without assigning interface authority to
a handle that denies it. Blocking this ticket must not close or silently unblock
that dependent.
