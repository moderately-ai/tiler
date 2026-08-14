---
id: reconcile-input-ordinal-region-local-and-declared-input-semantics
title: Reconcile InputOrdinal region-local and declared-input semantics
status: blocked
priority: p1
dependencies: [decide-the-schedule-local-input-ordinal-model, decide-the-full-list-access-coordinate-for-out-of-list-references]
related: [decide-the-source-bound-live-row-major-access-surface, admit-symbolic-extents-through-schedule-formation, associate-live-extent-operands-with-symbolic-semantic-interface-axes]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal, implementation/build, implementation/conformance, implementation/runtime, contracts/foundation, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [defect, public-boundary, schedule, identity, shapes]
claimed_from: todo
assignee: worker-reconcile-input-ordinal
lease_expires_at: 1786730018
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
- **Imprecise as originally written — the accepted repair is consequential but did not settle every public coordinate.** The accepted fieldless role retires the role's ordinal payload from every input-bearing schedule and kernel identity. That requires `tiler.schedule.v5` → `v6` and `tiler.kernel.v7` → `v8`; the exact access-coordinate grammar also moves `tiler.compiler.physical-implementation-proposal.v2` → `v3`. The request-subject, kernel-program, artifact-stage, artifact-program, and manifest-schema grammars remain unchanged, and the artifact extent row remains `(InputKey, Axis, AbiType)`. The accepted dependency fixed fieldless roles and compiler projection, but its retained input-only `InputOrdinal` cannot name the intermediate/output negatives required of a full-list extent reference.
- **Verified — the coincidence is not a contract.** Whole-program pointwise currently reads a dense declared prefix, so local and declared ordinals coincide there. Subset, epilogue, fold, and contraction construction already claim populations where they differ. The accepted fieldless model removes that coincidence as an authority: ordered access position is local, and only the checked compiler projection may associate it with a declared program input.
- **False — retaining `InputOrdinal` for every out-of-list reference is not a complete implementable instruction.** Its public contract is dense over boundary input tensors only, while a pointwise epilogue's ordered read run may be `[Intermediate, input 2]` and the required extent negative must name/reject that intermediate plus the final output write. Filtering inputs shifts the coordinate and cannot express either negative. [`decide-the-full-list-access-coordinate-for-out-of-list-references`](decide-the-full-list-access-coordinate-for-out-of-list-references.md) owns the stopped public replacement and exact diagnostic surface.

Reproduce:

```sh
rg -n 'region-local and positional|not an interface key|Which declared input tensor this access binds' crates/tiler-ir/src/schedule/handles.rs crates/tiler-ir/src/schedule/model.rs
rg -n 'The ordinals need not be the dense prefix|program ordinals need not be dense|The recognized ordinal, not the first declared input|declared ordinal is retained' crates/tiler-ir/src/schedule/builder.rs crates/tiler-compiler/src/physical.rs
rg -n 'region-local input and axis|maps that tensor through the stage access|MaterializedOrigin::ProgramInput' crates/tiler-ir/src/kernel/model.rs crates/tiler-artifact/src/program/builder.rs
rg -n 'AssemblyBinding::Input|input_ordinals_are_dense|SparseInputOrdinals|leaves by the position of the read' crates/tiler-compiler/src/program.rs crates/tiler-compiler/src/request.rs crates/tiler-ir/src/schedule/pointwise.rs crates/tiler-ir/src/schedule/pointwise_bf16.rs
rg -n 'tensor_role_comment|LiveRowMajor|InputOrdinal|TensorRole::Input' crates/tiler-metal/src crates/tiler-build/src crates/tiler-conformance/src crates/tiler-runtime/src
```

## Accepted portion and stop condition — 2026-08-14

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

Production is blocked because the dependency also retained input-only
`InputOrdinal` for out-of-list references while requiring an input extent to
name and reject intermediate/output positions in the full access list. That
public coordinate and error surface are not implementable without choosing a
new meaning or type. The exact Pareto-gated replacement is awaiting Tom in
[`decide-the-full-list-access-coordinate-for-out-of-list-references`](decide-the-full-list-access-coordinate-for-out-of-list-references.md); this ticket must not implement ahead of it.

## Required work

- After the awaiting-decision dependency is accepted, re-audit every constructor of its selected access coordinate and every `TensorRole::Input` verifier, encoder, subject binder, kernel/program/artifact consumer, and test. This includes Metal `tensor_role_comment`, docs, and fixtures; build assembly and custom fixtures; conformance construction and role matching; and runtime live-extent fixtures.
- Implement fieldless `TensorRole::Input` and the accepted full-list access coordinate completely. Remove the superseded public coordinate rather than keeping an alias; do not add a declared-input type to shared IR or use positional coincidence as proof. Retained compiler-private declared association fields (`prologue_reads`, `contributor_input`, `NormalizedContractionRead`, and `BoundaryRead::Input`) plus queries such as `NormalizedOutput::input_elements_at` and `NormalizedProgram::agreed_input_elements_at` use one crate-private `DeclaredInputOrdinal`, never retained raw `u32` or the shared access type.
- Eliminate the first-role searches in kernel construction/verification and artifact derivation plus the axis-only selection in live lowering. `InputExtentParameter` directly indexes the complete access/buffer list and checks new public `KernelBuildError::InputExtentAccessOutOfRange`, existing `InputExtentNotInput`, then existing `InputExtentWrongAxis`; rename the public `ArtifactBuildError::ExtentOperandUnbound` field from `ordinal` to `access`. Opaque-call bindings carry their ABI name separately from the exact access coordinate.
- Align defining docs, containing-role docs, intrinsic validation, physical construction, program-stage binding, artifact mapping, canonical encoders, version ledgers, and pins with schedule `v6`, kernel `v8`, and compiler physical-proposal `v3`. Keep kernel-program `v11`, artifact-stage `v3`, artifact-program `v17`, manifest schema 17.0, request-subject `v6`, and the artifact `(InputKey, Axis, AbiType)` extent-row grammar unchanged unless new evidence forces a separate stop.
- Preserve named `InputKey` as the program-interface authority. A local access coordinate reaches it only through the compiler's retained checked subject projection and exact stage access/materialized origin.

## Required evidence

- Sparse declared inputs `1` and `7` map from local access positions `[0, 1]` to the intended `InputKey`s, while the dense-prefix neighbour remains unchanged. Perturb only the retained declared association and quote the compiler `request-binding` failure.
- Preserve and count the full association population: subset `[0, 2]`; later fold over `[1]`; two-of-three contraction; epilogue `[Intermediate, input 2]`; and two mapped reads of one declared input. Each local position maps to exactly the normalized subject member at that position, without filtering or deduplication.
- Reorder the compiler-projected stage bindings while all schedule roles remain fieldless and quote the retained-request-subject refusal. Do not require a standalone public `KernelProgramBuilder` to infer same-role intent the verified kernel does not carry.
- For epilogue `[Intermediate, input 2]`, prove extent position `0` fails the exact not-input diagnostic, position `1` reaches input `2`, the final write fails the same not-input diagnostic, and an absent position fails the exact out-of-range diagnostic. Swap only the axis separately and quote the wrong-axis failure.
- Bind an opaque-call parameter to another valid local access and quote the subject-mismatch refusal; independently perturb `WorkScaling::PerElementOf` and prove it resolves the declared tensor associated with that exact access rather than a first input.
- Perturb schedule, kernel, and physical-proposal domain spellings independently and quote each owning pin failure. Prove unchanged artifact extent-row bytes and manifest schema, then regenerate every downstream value that folds a moved nested identity.

## Closes when

The new awaiting-decision dependency is accepted and consumed, public docs and
all construction/consumption paths agree on one checked access-coordinate
model, the complete association and refusal census above passes, every governed
identity migration is coherent, and
[`decide-the-source-bound-live-row-major-access-surface`](decide-the-source-bound-live-row-major-access-surface.md)
can select an exact source field type without assigning interface authority to
a handle that denies it. Blocking this ticket must not close or silently unblock
that dependent.
