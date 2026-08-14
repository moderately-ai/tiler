---
id: decide-the-full-list-access-coordinate-for-out-of-list-references
title: Decide the full-list access coordinate for out-of-list references
status: awaiting-decision
priority: p1
dependencies: [decide-the-schedule-local-input-ordinal-model]
related: [reconcile-input-ordinal-region-local-and-declared-input-semantics]
scopes: [contracts/foundation, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, identity]
---
## User-visible outcome

Tom accepts or defers one complete public replacement: `AccessOrdinal` names a
member of the complete ordered schedule-access/kernel-buffer list, including a
position that is invalid for an input-only consumer because it names an
intermediate or output. The replacement leaves no producer free to filter
inputs, search for the first matching role, or reinterpret a scalar-expression
leaf/access ordinal as a program-interface key.

## Exact-current-base Fact audit — 2026-08-14, `4e10b98066f846ca50de4c97ba6262dade9e0865`

- **Verified — `InputOrdinal`'s public contract is input-only.**
  `crates/tiler-ir/src/schedule/handles.rs`, anchor `Which of a region's boundary input tensors`, says a region reading `n` input tensors uses the dense prefix `0..n`. It explicitly denies program-interface authority.
- **Verified — pointwise leaves actually index the complete ordered read run.**
  `crates/tiler-ir/src/schedule/builder.rs`, anchor `served by read`, requires one read per leaf and pairs them by position. The same verifier, anchor `At most one read binds the materialized intermediate`, admits an `Intermediate` among those reads. Therefore an epilogue with reads `[Intermediate, declared input 2]` has leaf positions `[0, 1]`; position `0` is not an input tensor under `InputOrdinal`'s defining contract.
- **Verified — the read run and final write form one constructibly ordered access domain.**
  `crates/tiler-ir/src/schedule/builder.rs`, anchors `region.index.accesses.split_last()` and `write.mode != AccessMode::Write`, verifies pointwise reads as the prefix and the write as the final access. `crates/tiler-ir/src/kernel/verify.rs`, anchor `Returns the ordered read and write accesses`, enforces the same split for kernel construction. A full access position can therefore name every read plus the final output write without an invented conversion.
- **False — the accepted Option 3 packet did not fully fix the out-of-list coordinate.**
  [`decide-the-schedule-local-input-ordinal-model`](decide-the-schedule-local-input-ordinal-model.md), anchor `InputExtentParameter { input: InputOrdinal, axis }`, requires an input-extent reference to reject a position naming an intermediate. An input-filtered ordinal cannot name that intermediate, while interpreting it as the full access position contradicts the defining type. This is a consequential public signature and meaning choice, so production must remain stopped.
- **Verified — filtering inputs is not a harmless conversion.** In `[Intermediate, input 2]`, full read position `1` becomes filtered input ordinal `0`; in repeated reads `[input 2 dense, input 2 mapped]`, two full positions would both project to one declared input. A filter therefore changes coordinates, and a later reorder can silently bind a different tensor while all values remain in range.
- **Verified — four live consumers currently search/default instead of resolving an exact coordinate.** `crates/tiler-ir/src/kernel/builder.rs`, anchor `scheduled_input_rank`, searches the first access with an equal role. `kernel/verify.rs`, anchor `find(|read| read.tensor == parameter.tensor)`, repeats that search. `crates/tiler-artifact/src/program/builder.rs`, anchor `maps that tensor through the stage access`, searches the first equal kernel role in the zipped buffer/access run. `kernel/lower.rs`, anchor `ReadAddressing::LiveRowMajor`, selects by axis alone. Fieldless roles make every one of those searches ambiguous; all must become direct checked indexing under either survivor.
- **Verified — an opaque-call parameter name is not a tensor coordinate.** `crates/tiler-compiler/src/call_registry.rs`, anchor `bindings: Vec<(&'static str, TensorRole)>`, separately carries a governed ABI parameter name and the tensor role it binds. `crates/tiler-compiler/src/frontier.rs`, anchor `TensorRole::Input { ordinal } => request`, needs the role ordinal to resolve `WorkScaling::PerElementOf` to a declared tensor. Once the role is fieldless, the name still identifies the call parameter but says nothing about which region access supplies it.
- **Verified — compiler-private normalized records and two queries carry a different, declared-interface domain.** `crates/tiler-compiler/src/request.rs` retains declared ordinals as raw `u32` in `prologue_reads`, `contributor_input`, `NormalizedContractionRead`, and `BoundaryRead::Input`; anchors `NormalizedOutput::input_elements_at` and `NormalizedProgram::agreed_input_elements_at` additionally accept public `InputOrdinal` while indexing the declared program input set, not a region access list. Every retained compiler-private declared association and its queries must use a crate-private `DeclaredInputOrdinal`; renaming the query parameter to public `AccessOrdinal` would preserve the original conflation under a new spelling, while keeping raw `u32` would discard type-level authority for no runtime saving.
- **Verified — no surviving public `InputOrdinal` consumer needs a distinct leaf-only domain.** The complete public-API definition census found only `TensorRole::Input.ordinal`, pointwise F32/BF16 leaf fields and builders, and `ReductionTopology::LiveContraction.live_input`; the compiler-private uses are inventoried separately above. The role payload is removed by the accepted dependency. Pointwise leaves are explicitly access-position paired, and live contraction already resolves the input access/buffer that supplies its bound. Density over a pointwise read prefix is an admission rule on this one coordinate, not evidence of a second coordinate domain.
- **Verified — the compiler already retains the declared association authority.** `crates/tiler-compiler/src/physical.rs`, anchors `request_subject: VerifiedRequestSubject` and `verify_region_subject_binding`, stores the exact checked subject after proving each region against it. `crates/tiler-compiler/src/request.rs` retains subset, fold, contraction, epilogue, staged, and repeated-read associations in ordered normalized read records. The implementation may project from that authority; it must not add a second retained association vector.
- **Verified — the public program builder cannot diagnose an intended same-role reorder by itself.** A `KernelProgramBuilder` receives an already verified kernel plus producer-supplied stage accesses. Once two kernel buffers are both fieldless `Input`, the builder can check role, mode, type, extent, and value origin, but the kernel deliberately carries no declared `InputKey` association to compare. The required reorder negative therefore belongs at the compiler's retained-request-subject projection; the program builder and artifact builder only preserve that checked order and fail structural role/origin mismatches.
- **Verified — the identity and wire consequences are separable.** Removing the role payload changes every input-bearing scheduled-region and structured-kernel encoding, requiring `tiler.schedule.v5` → `v6` and `tiler.kernel.v7` → `v8`. The compiler proposal identity also encodes boundary roles and opaque-call bindings under `tiler.compiler.physical-implementation-proposal.v2`, so the chosen exact access-coordinate grammar requires `v3`. `tiler.kernel-program.v11`, `tiler.artifact-program.stage.v3`, `tiler.artifact-program.v17`, and manifest schema 17.0 fold the stepped nested identities or already carry `InputKey`; their grammars do not move. `ExtentOperandData` and its wire row remain exactly `(InputKey, Axis, AbiType)`, so no local coordinate crosses the artifact interface.
- **Verified — adjacent source drift since the earlier `bbbf936ad3d8170ec601cd26eda5235cc2ac1d6b` audit does not answer this choice.** `ce3e0d79` added occurrence-bound selected-implementation provenance and explicitly left manifest membership for later. `7ead2c0a` retired the standalone ABI expression subtree key in favour of canonical arena positions, with no artifact-identity move. `6406654c` adjusted that provenance surface's compile-fail test boundary. The exact-current artifact codec/model/verify, compiler selection/session, IR program ABI/builder/error/model/tests/verify, and `docs/artifact-abi.md` paths were re-read; none defines the missing full-list coordinate.

Reproduce:

```sh
rg -n 'Which of a region.s boundary input tensors|served by read|At most one read binds the materialized intermediate' crates/tiler-ir/src/schedule/handles.rs crates/tiler-ir/src/schedule/builder.rs
rg -n 'scheduled_input_rank|find\(\|read\| read.tensor|maps that tensor through the stage access|ReadAddressing::LiveRowMajor' crates/tiler-ir/src/kernel/{builder,verify,lower}.rs crates/tiler-artifact/src/program/builder.rs
rg -n 'bindings: Vec<\(&.static str, TensorRole\)>|TensorRole::Input \{ ordinal \} => request' crates/tiler-compiler/src/call_registry.rs crates/tiler-compiler/src/frontier.rs
rg -n 'input_elements_at|agreed_input_elements_at' crates/tiler-compiler/src/request.rs crates/tiler-compiler/src/frontier.rs
rg -n 'request_subject: VerifiedRequestSubject|verify_region_subject_binding|prologue_reads|contributor_input|BoundaryRead|NormalizedContractionRead' crates/tiler-compiler/src/physical.rs crates/tiler-compiler/src/request.rs
rg -n 'InputOrdinal' crates/tiler-ir/src/schedule/{handles,model,pointwise,pointwise_bf16}.rs
rg -n 'accesses.split_last|Returns the ordered read and write accesses|write.mode != AccessMode::Write' crates/tiler-ir/src/schedule/builder.rs crates/tiler-ir/src/kernel/verify.rs
git log --oneline bbbf936ad3d8170ec601cd26eda5235cc2ac1d6b..4e10b98066f846ca50de4c97ba6262dade9e0865 -- crates/tiler-artifact/src/program crates/tiler-compiler/src/selection.rs crates/tiler-compiler/src/session.rs crates/tiler-ir/src/program docs/artifact-abi.md
```

## Fixed constraints from the accepted decision

- `TensorRole::Input` becomes fieldless. This packet does not reopen that choice.
- Ordered schedule accesses and corresponding kernel buffers have one exact full-list position. No filtered-input coordinate, role search, first match, axis-only match, or local-equals-declared coincidence is authority.
- `InputKey` remains the program-interface authority. The compiler projects local positions from the already-retained checked `VerifiedRequestSubject`; program and artifact construction preserve the resulting exact stage-access order.
- A live extent reference must be able to name, and then reject, an out-of-range position, an `Intermediate` read, and the final `Output` write. The axis check runs only after exact-position and input-role checks.
- Opaque-call ABI parameter names and region tensor coordinates remain distinct. A call binding carries both; `WorkScaling::PerElementOf` resolves through the checked region projection, not through a fieldless role.
- No local coordinate appears in the artifact extent row. The artifact receives only the `InputKey` derived through the exact stage access.
- Compiler-private normalized association records and queries over the declared program interface do not accept the public access coordinate or retain bare `u32`. They use one crate-private `DeclaredInputOrdinal` whose scope cannot leak into shared IR; conversion to `usize`/bytes occurs only at the owning lookup/encoder boundary.

## Eliminated candidates

### Keep `InputOrdinal` input-filtered and convert by counting input roles

Eliminated for correctness. Filtering shifts `[Intermediate, input 2]` from full
position `1` to filtered ordinal `0`, cannot express the required intermediate
negative, and turns access reordering into a valid but differently bound
coordinate. A checked filter proves only that an input exists, not that the
caller named the same access.

### Make fieldless consumers search by role, axis, or first match

Eliminated for correctness. Two inputs can share a role and axis, two mapped
reads can reach the same declared input, and a fieldless role has no member
identity. The current builder/verifier/artifact/lowering searches are the defect
surface, not a compatibility route.

### Support live extents only when every read is an input

Correct as a temporary fail-closed staging state, but eliminated as a completed
outcome. It cannot express the accepted intermediate/output perturbations,
leaves epilogue and staged populations unresolved, and does not repair opaque
call bindings. If Tom defers the public choice, this restriction may protect an
intermediate branch, but it cannot close the implementation ticket.

### Use `KernelBufferId` or `VerifiedBufferId` as the durable parameter field

Eliminated as the sole carrier. Those handles include builder/verified-owner
identity and are minted only after construction; retaining one in canonical KIR
would fold transient ownership into a durable subject. A builder method may
accept a handle and normalize it, but the verified `InputExtentParameter` still
needs a stable owner-independent full-list coordinate.

### Retain a leaf-only `InputOrdinal` and add `AccessOrdinal`

Eliminated as dominated under the pre-production complete-replacement rule. An
exhaustive public-definition census found no constructible leaf coordinate that
can diverge from the ordered read/access position: `verify_pointwise_region`
pairs leaf `i` with read `i`, including the staged read in an epilogue, and
`LiveContraction.live_input` names the access that supplies its live bound. The
pointwise dense-prefix rule restricts which access positions its grammar admits;
it does not create another coordinate. Two public newtypes would encode the same
number over every shared population, add conversions and documentation that can
drift, and provide no stricter constructible guarantee than the existing checked
expression and whole-region builders. Evidence of a public construction where a
leaf ordinal and its serving read position differ would reopen this elimination;
none exists at the exact base.

### Keep compiler-private declared ordinals as bare `u32`

Eliminated as dominated by crate-private `DeclaredInputOrdinal`. Both have the
same compact representation and runtime cost, but only the newtype makes it a
type error to pass a local access position into a declared-interface query or
normalized association record. It remains private, so it creates no shared-IR
or public interface authority.

### Defer

Safe and currently selected while this ticket awaits Tom, but not an
implementation outcome. The existing ordinal contradiction stays fail-closed
and every dependent remains blocked.

## Pareto frontier

### Complete replacement — one public full-list `AccessOrdinal` (recommended)

**Exact public surface.** Remove `InputOrdinal` completely and add the ADR 0074
convention-5b `AccessOrdinal(u32)` in `tiler_ir::schedule`, meaning the exact
position in `ScheduledRegion::index.accesses` and the corresponding structured
kernel buffer list, including the final write. Use it for scalar expression
leaves, `ReductionTopology::LiveContraction`, input-extent parameters, and the
coordinate half of opaque-call bindings:

```rust
pub struct InputExtentParameter {
    pub access: AccessOrdinal,
    pub axis: Axis,
}
```

Expression builders continue requiring the dense read prefix and schedule
verification continues requiring leaf `i` to name read access `i`; an
input-only consumer may receive a coordinate for the final write only so it can
reject that coordinate precisely. The compiler-private opaque-call binding is
`(parameter_name, AccessOrdinal)` and resolves that position through the exact
region/read/write projection derived from the retained request subject and
cover write assignment. Retained compiler-private declared association fields
and their shape queries move to one `DeclaredInputOrdinal`, never bare `u32` or
public `AccessOrdinal`.

**Verifier and error surface.** `declare_input_extent` indexes exactly once and
checks in this order: `InputExtentAccessOutOfRange`, existing
`InputExtentNotInput`, then existing `InputExtentWrongAxis`; duplicate
`(access, axis)` remains `DuplicateInputExtent`. Whole-kernel defence retains
`KernelDiagnostic::InputExtentContract`; subject perturbation tests exercise the
three insertion errors independently. Artifact `ExtentOperandUnbound.ordinal`
is renamed to `.access`, but the variant and envelope row remain otherwise
unchanged. Compiler-private opaque-call admission adds distinct stable reasons
`binding-access-out-of-range` and `binding-access-subject-mismatch`; an unknown
parameter remains `UnknownParameter` and is no longer overloaded with a bad
tensor coordinate.

**Correctness and strictness.** One coordinate has one meaning everywhere and
all consumers index directly. Expression and live-extent builders impose their
narrower admissible subsets before retaining a verified value. No input filter,
role search, axis search, or local-to-declared coincidence can change the
referent, and invalid intermediate/output positions remain representable and
are rejected at the owning boundary.

**Maintenance, compatibility, and host cost.** Pre-production complete
replacement removes the contradicted public name rather than retaining an alias
or a second numerically equal type. One `u32` replaces one `u32` at each retained
site, no association vector is added, and host/runtime memory and kernel
performance are unchanged. The workspace-wide public rename is intentional;
there are no external consumers or compatibility aliases in scope.

**Identity/schema.** Schedule `v6`, kernel `v8`, and compiler physical proposal
`v3` move. Request subject, kernel program, artifact stage, artifact program,
manifest schema, semantic/index identities, and the `(InputKey, Axis, AbiType)`
extent row stay at their current domains/grammar; their values and downstream
goldens move only where they fold a stepped nested identity.

**Strongest counterargument.** A full-access type is syntactically available at
an expression site whose legal population is only the dense read prefix.
Existing checked expression and scheduled-region builders make an absent/read-
write mismatch unretainable, so a second type would add no constructible
strictness. Evidence of a public construction whose leaf number can differ from
its serving read/access position would reverse that conclusion and require a
new packet.

## Decision question

Should Tiler accept the complete replacement above — remove public
`InputOrdinal`; use one full-list `AccessOrdinal` at every surviving public
local-coordinate site; use compiler-private `DeclaredInputOrdinal` for retained
declared associations and interface queries; add public
`KernelBuildError::InputExtentAccessOutOfRange`; and rename the public
`ArtifactBuildError::ExtentOperandUnbound` field from `ordinal` to `access` — or
defer implementation and keep the dependent ticket blocked? No second
nondominated implementation model remains at this base.

## Required negative controls after acceptance

- Sparse declared subset `[0, 2]` maps from local read positions `[0, 1]` to the intended `InputKey`s; perturb only the declared association and require compiler `request-binding` refusal.
- A later fold over declared input `[1]` maps local read position `0` to input `1`, never input `0`.
- A two-of-three contraction maps its two local positions to the exact declared pair and refuses a swapped subject map.
- Epilogue reads `[Intermediate, input 2]`: extent position `0` fails `InputExtentNotInput`; position `1` reaches input `2`; the final write position fails `InputExtentNotInput`; an absent position fails `InputExtentAccessOutOfRange`.
- Two local mapped reads of one declared input remain distinct positions that project to the same declared ordinal and `InputKey` without being deduplicated.
- Dense-prefix all-input neighbours retain their ordered binding and make every new assertion fail when the subject, not the assertion, is perturbed.
- Reorder compiler-projected stage bindings while retaining fieldless roles and require the retained request-subject check to fail. Do not demand that a standalone public `KernelProgramBuilder` infer intent absent from its verified kernel.
- Swap an input-extent access while keeping the axis, and separately swap the axis while keeping the access; quote the distinct owning failures.
- Bind an opaque-call parameter to another valid local access and require `binding-access-subject-mismatch`; independently perturb `WorkScaling::PerElementOf` and require it to resolve the declared tensor at that exact access.
- Perturb the schedule, kernel, and physical-proposal domain spellings independently so each domain-pin test names its own stale row. Prove artifact row bytes remain `(InputKey, Axis, AbiType)` and manifest schema 17.0.

## Closes when

Tom accepts the exact replacement and acceptance provenance is recorded. The
implementation ticket then consumes the choice without another public
signature, coordinate-domain, diagnostic, identity, or schema decision.
