---
id: decide-the-full-list-access-coordinate-for-out-of-list-references
title: Decide the full-list access coordinate for out-of-list references
status: done
priority: p1
dependencies: [decide-the-schedule-local-input-ordinal-model]
related: [reconcile-input-ordinal-region-local-and-declared-input-semantics, scope-an-in-place-append-into-a-caller-retained-allocation]
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
- **Verified — opaque-call binding validation currently proves ABI-name coverage but not boundary-access coverage.** `crates/tiler-compiler/src/call_abi.rs`, anchors `check_bindings` and `ParameterRole::reads`, requires every declared parameter exactly once and lets several parameters share one role when their storage agrees. It does not receive a region access list, so it cannot prove bounds, access mode, or that every boundary access is represented. `crates/tiler-compiler/src/frontier.rs`, anchor `derive_call_boundary_contract`, then deduplicates roles in provider binding order. That is incompatible with fieldless repeated inputs and with `BoundaryContract::subsumes`, anchor `Both are derived in the verified region's access order`, which pairs repeated facets positionally.
- **Verified — generic ABI `InOut` is not evidence that the current region binding can represent mutation.** `crates/tiler-compiler/src/call_abi.rs`, anchors `Both read and written` and `ParameterLayout::Both`, deliberately keeps a generic in-place declaration. The frontier test `an_in_out_binding_yields_both_a_requirement_and_a_guarantee` calls `derive_call_boundary_contract` directly with one role; it supplies neither the checked regional read prefix nor the distinct owning-write access, so it is a lower-level declaration/derivation test rather than a regional-admission control. `crates/tiler-compiler/src/boundary.rs`, anchor `version-producing step there, together`, requires a value-version boundary dimension and the `KernelProgram` version-producing step to land together. Neither the current one-role binding row nor the proposed one-access row can name both `[read, owning write]`, and treating either position as both would silently conflate two accesses.
- **Verified — compiler-private normalized records and two queries carry a different, declared-interface domain.** `crates/tiler-compiler/src/request.rs` retains declared ordinals as raw `u32` in `prologue_reads`, `contributor_input`, `NormalizedContractionRead`, and `BoundaryRead::Input`; anchors `NormalizedOutput::input_elements_at` and `NormalizedProgram::agreed_input_elements_at` additionally accept public `InputOrdinal` while indexing the declared program input set, not a region access list. Every retained compiler-private declared association and its queries must use a crate-private `DeclaredInputOrdinal`; renaming the query parameter to public `AccessOrdinal` would preserve the original conflation under a new spelling, while keeping raw `u32` would discard type-level authority for no runtime saving.
- **Verified — no surviving public `InputOrdinal` consumer needs a distinct leaf-only domain.** The complete public-API definition census found only `TensorRole::Input.ordinal`, pointwise F32/BF16 leaf fields and builders, and `ReductionTopology::LiveContraction.live_input`; the compiler-private uses are inventoried separately above. The role payload is removed by the accepted dependency. Pointwise leaves are explicitly access-position paired, and live contraction already resolves the input access/buffer that supplies its bound. Density over a pointwise read prefix is an admission rule on this one coordinate, not evidence of a second coordinate domain.
- **Verified — the compiler already retains the declared association authority.** `crates/tiler-compiler/src/physical.rs`, anchors `request_subject: VerifiedRequestSubject` and `verify_region_subject_binding`, stores the exact checked subject after proving each region against it. `crates/tiler-compiler/src/request.rs` retains subset, fold, contraction, epilogue, staged, and repeated-read associations in ordered normalized read records. The implementation may project from that authority; it must not add a second retained association vector.
- **Verified — the public program builder cannot diagnose an intended same-role reorder by itself.** A `KernelProgramBuilder` receives an already verified kernel plus producer-supplied stage accesses. Once two kernel buffers are both fieldless `Input`, the builder can check role, mode, type, extent, and value origin, but the kernel deliberately carries no declared `InputKey` association to compare. Therefore there is no constructible same-role reorder refusal to require. A changed valid association is re-derived positively from its own verified request; an actually foreign compiler wrapper fails the existing request-subject check. The program and artifact builders preserve that checked order and retain their structural role/origin refusals.
- **Verified — the identity, explain, and wire consequences are separable.** Removing the role payload changes every input-bearing scheduled-region and structured-kernel encoding, requiring `tiler.schedule.v5` → `v6` and `tiler.kernel.v7` → `v8`. The compiler proposal identity also encodes boundary roles and opaque-call bindings under `tiler.compiler.physical-implementation-proposal.v2`, so the chosen exact access-coordinate grammar requires `v3`. `crates/tiler-compiler/src/call_registry.rs`, anchors `INPUT_ROLE_PREFIX` and `proposal_subject_is_exact_ordered_and_bounded`, renders an existing opaque proposal as `input#N`; the replacement renders that same subject as `access#N`. Existing `SubjectKind::OpaqueCall` canonical record bytes and rendered text therefore move, requiring `EXPLAIN_SCHEMA_VERSION` 10 → 11 and `EXPLAIN_RENDERER_VERSION` 8 → 9 under `crates/tiler-compiler/src/explain.rs`'s version rule. `tiler.kernel-program.v11`, `tiler.artifact-program.stage.v3`, `tiler.artifact-program.v17`, and manifest schema 17.0 fold the stepped nested identities or already carry `InputKey`; their grammars do not move. `ExtentOperandData` and its wire row remain exactly `(InputKey, Axis, AbiType)`, so no local coordinate crosses the artifact interface.
- **Verified — adjacent source drift since the earlier `bbbf936ad3d8170ec601cd26eda5235cc2ac1d6b` audit does not answer this choice.** `ce3e0d79` added occurrence-bound selected-implementation provenance and explicitly left manifest membership for later. `7ead2c0a` retired the standalone ABI expression subtree key in favour of canonical arena positions, with no artifact-identity move. `6406654c` adjusted that provenance surface's compile-fail test boundary. The exact-current artifact codec/model/verify, compiler selection/session, IR program ABI/builder/error/model/tests/verify, and `docs/artifact-abi.md` paths were re-read; none defines the missing full-list coordinate.

Reproduce:

```sh
rg -n 'Which of a region.s boundary input tensors|served by read|At most one read binds the materialized intermediate' crates/tiler-ir/src/schedule/handles.rs crates/tiler-ir/src/schedule/builder.rs
rg -n 'scheduled_input_rank|find\(\|read\| read.tensor|maps that tensor through the stage access|ReadAddressing::LiveRowMajor' crates/tiler-ir/src/kernel/{builder,verify,lower}.rs crates/tiler-artifact/src/program/builder.rs
rg -n 'bindings: Vec<\(&.static str, TensorRole\)>|TensorRole::Input \{ ordinal \} => request' crates/tiler-compiler/src/call_registry.rs crates/tiler-compiler/src/frontier.rs
rg -n 'check_bindings|ParameterRole::reads|derive_call_boundary_contract|Both are derived in the verified region.s access order' crates/tiler-compiler/src/call_abi.rs crates/tiler-compiler/src/frontier.rs
rg -n 'Both read and written|ParameterLayout::Both|an_in_out_binding_yields_both_a_requirement_and_a_guarantee|version-producing step there, together' crates/tiler-compiler/src/call_abi.rs crates/tiler-compiler/src/frontier.rs crates/tiler-compiler/src/boundary.rs
rg -n 'input_elements_at|agreed_input_elements_at' crates/tiler-compiler/src/request.rs crates/tiler-compiler/src/frontier.rs
rg -n 'request_subject: VerifiedRequestSubject|verify_region_subject_binding|prologue_reads|contributor_input|BoundaryRead|NormalizedContractionRead' crates/tiler-compiler/src/physical.rs crates/tiler-compiler/src/request.rs
rg -n 'InputOrdinal' crates/tiler-ir/src/schedule/{handles,model,pointwise,pointwise_bf16}.rs
rg -n 'accesses.split_last|Returns the ordered read and write accesses|write.mode != AccessMode::Write' crates/tiler-ir/src/schedule/builder.rs crates/tiler-ir/src/kernel/verify.rs
rg -n 'EXPLAIN_SCHEMA_VERSION|EXPLAIN_RENDERER_VERSION|INPUT_ROLE_PREFIX|proposal_subject_is_exact_ordered_and_bounded' crates/tiler-compiler/src/explain.rs crates/tiler-compiler/src/call_registry.rs
git log --oneline bbbf936ad3d8170ec601cd26eda5235cc2ac1d6b..4e10b98066f846ca50de4c97ba6262dade9e0865 -- crates/tiler-artifact/src/program crates/tiler-compiler/src/selection.rs crates/tiler-compiler/src/session.rs crates/tiler-ir/src/program docs/artifact-abi.md
```

## Fixed constraints from the accepted decision

- `TensorRole::Input` becomes fieldless. This packet does not reopen that choice.
- Ordered schedule accesses and corresponding kernel buffers have one exact full-list position. No filtered-input coordinate, role search, first match, axis-only match, or local-equals-declared coincidence is authority.
- `InputKey` remains the program-interface authority. The compiler projects local positions from the already-retained checked `VerifiedRequestSubject`; program and artifact construction preserve the resulting exact stage-access order.
- A live extent reference must be able to name, and then reject, an out-of-range position, an `Intermediate` read, and the final `Output` write. The axis check runs only after exact-position and input-role checks.
- Opaque-call ABI parameter names and region tensor coordinates remain distinct. A call binding carries both; the host admits only `In` → `Read` and `Out` → `Write`, refuses generic ABI `InOut` at this regional binding boundary, proves every boundary access is covered, and derives facets in access-list order. Provider binding order remains proposal identity order. `WorkScaling::PerElementOf` resolves through the named checked access, not through a fieldless role or ABI slot.
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

### Give one `InOut` binding paired `{ read, write }` access coordinates

Eliminated from this packet as a broader mutating-profile capability, not as a
second way to spell the selected coordinate. A paired row could name both the
read prefix member and the distinct owning write, but admitting it would assert
that the output is a new authoritative version in storage whose prior version
the call also reads. The current boundary vocabulary deliberately omits value
version, and `crates/tiler-compiler/src/boundary.rs`, anchor `version-producing step there, together`, requires that dimension
and the `KernelProgram` version-producing step to land together. Adding the
carrier first would therefore make an unverified alias/mutation promise.

[Q-PLAN-015](../docs/open-questions.md#q-plan-015--advanced-buffer-reuse-and-in-place-execution)
already owns general in-place execution, and
[`scope-an-in-place-append-into-a-caller-retained-allocation`](scope-an-in-place-append-into-a-caller-retained-allocation.md)
tracks its existing deferred trigger and the required versioning/recovery
obligations. No new follow-up is filed here. Until that owner opens and lands
the coordinated boundary/program contract, regional opaque binding remains a
single `(parameter, AccessOrdinal)` and rejects `InOut` explicitly.

### Defer

Safe and currently selected while this ticket awaits Tom, but not an
implementation outcome. The existing ordinal contradiction stays fail-closed
and every dependent remains blocked.

## Pareto frontier

### Complete replacement — one public full-list `AccessOrdinal` (recommended)

**Exact public surface.** Remove `InputOrdinal` completely and add the ADR 0074
convention-5b `AccessOrdinal(u32)` in `tiler_ir::schedule`, meaning the exact
position in `ScheduledRegion::index.accesses` and the corresponding structured
kernel buffer list, including the final write. The complete replacement has
these spellings; retaining `ordinal`, `input`, or a raw `u32` at one of these
coordinate sites is not an implementation of this option:

```rust
pub enum PointwiseF32Node {
    Input { access: AccessOrdinal },
    // existing non-input variants unchanged
}

pub enum PointwiseBf16Node {
    Input { access: AccessOrdinal },
    // existing non-input variants unchanged
}

pub enum ReductionTopology {
    LiveContraction {
        live_access: AccessOrdinal,
        live_axis: Axis,
        // existing order and permission fields unchanged
    },
    // existing non-live variants unchanged
}

pub struct InputExtentParameter {
    pub access: AccessOrdinal,
    pub axis: Axis,
}
```

Both expression builders take `input(access: AccessOrdinal)`. Their public
diagnostics become `PointwiseF32ExpressionDiagnostic::SparseAccessOrdinals {
missing: AccessOrdinal }` with stable rule
`pointwise-f32-sparse-access-ordinals`, and
`PointwiseBf16ExpressionDiagnostic::SparseAccessOrdinals { missing:
AccessOrdinal }` with stable rule `pointwise-bf16-sparse-access-ordinals`.
`MissingInput` and its two existing stable rules remain unchanged because they
state that an expression has no input leaf, not a coordinate domain.

Expression builders continue requiring the dense read prefix and schedule
verification continues requiring leaf `i` to name read access `i`; an
input-only consumer may receive a coordinate for the final write only so it can
reject that coordinate precisely. Retained compiler-private declared
association fields and their shape queries move to one
`DeclaredInputOrdinal`, never bare `u32` or public `AccessOrdinal`.

The public artifact error becomes exactly
`ArtifactBuildError::ExtentOperandUnbound { entry: usize, access:
AccessOrdinal, axis: u32 }`. `access` stays typed rather than being flattened to
`u32`; `axis` and every other field and variant remain unchanged. The artifact
wire row still contains no `AccessOrdinal`.

**Exact opaque-call migration.** `OpaqueCallProposal.bindings` becomes the
ordered `Vec<(&'static str, AccessOrdinal)>`. The host derives the complete
ordered boundary-access view — reads followed by the owning write — from the
same retained `VerifiedRequestSubject` and cover write assignment that own the
region, rather than accepting it from the provider or retaining a parallel
association authority. Every ABI parameter is still named exactly once, and
every boundary access must be named by at least one compatible parameter.

Validation is deterministic: retain the existing unknown, duplicate, and
unbound-parameter checks; then check each proposed access for bounds in proposal
order; then refuse an `InOut` parameter before access-mode, complete-coverage,
storage-agreement, or boundary-derivation checks; then check exact direction in
proposal order; then check complete access coverage in access-list order; then
check shared-access storage agreement. Direction compatibility is exactly
`ParameterRole::In` → `AccessMode::Read` and `ParameterRole::Out` →
`AccessMode::Write`; neither the broader `reads()`/`writes()` predicates nor an
extra direction authorizes a regional binding. Distinct ordinary parameters
may target one access. Their encoding and alignment must agree; all `In`
parameters sharing a read must agree on its required layout, and all `Out`
parameters sharing a write must agree on its guaranteed layout.

Generic ABI `ParameterRole::InOut`, `ParameterLayout::Both`, declaration
coherence, and their direct lower-level tests remain. Delete the frontier test
`an_in_out_binding_yields_both_a_requirement_and_a_guarantee`, because its
unchecked direct derivation is not a regional admission. Retain
`call_abi::roles_report_their_access` and
`call_abi::a_layout_must_state_the_direction_its_role_has`, and add the
call-declaration positive
`an_in_place_parameter_with_may_alias_inputs_is_coherent` beside the existing
`an_in_place_parameter_cannot_claim_distinct_results`. No positive regional
`InOut` derivation remains.

The compiler-private binding errors become
`AccessOutOfRange { parameter, access }`,
`InOutRegionUnsupported { parameter: &'static str, access: AccessOrdinal }`,
`AccessModeMismatch { parameter, access, parameter_role, access_mode }`,
`UnboundAccess(AccessOrdinal)`, and
`AccessStorageDisagreement { access, first, second }`; the last completely
replaces role-keyed `RoleStorageDisagreement`. Their stable explain reasons are
respectively `opaque-call.binding.access-out-of-range`,
`opaque-call.binding.inout-region-unsupported`,
`opaque-call.binding.access-mode-mismatch`,
`opaque-call.binding.unbound-access`, and
`opaque-call.binding.access-storage-disagreement`. The enclosing
`OpaqueCallRejectionCause::MalformedBinding` remains unchanged. The `InOut`
fault retains the valid named access so the refusal is attributable without
pretending that coordinate also names the missing opposite-direction access.

Boundary derivation iterates the host access list, not the provider binding
list: each read emits at most one agreed requirement derived only from its `In`
parameters, and the owning write emits at most one agreed guarantee derived
only from its `Out` parameters, preserving access-list order separately in both
facet runs. Binding order remains unchanged in proposal identity and explain.
`WorkScaling::PerElementOf` finds its named parameter, reads that parameter's
exact access, and resolves the element count through the host projection.
Rebinding a parameter to another valid compatible access is a different valid
proposal, not a subject-mismatch refusal.

**Verifier and error surface.** `declare_input_extent` indexes exactly once and
checks in this order: new public unit variant
`KernelBuildError::InputExtentAccessOutOfRange`, existing
`InputExtentNotInput`, then existing `InputExtentWrongAxis`; duplicate
`(access, axis)` remains `DuplicateInputExtent`. Whole-kernel defence retains
`KernelDiagnostic::InputExtentContract`; subject perturbation tests exercise the
three insertion errors independently. Artifact
`ExtentOperandUnbound.ordinal: u32` is replaced by the typed `.access` field
above, but the variant and envelope row remain otherwise unchanged. Opaque-call
admission uses only the independently checkable ABI/access errors above; there
is no `binding-access-subject-mismatch` for two valid local bindings.

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

**Identity/schema.** Schedule `v6`, kernel `v8`, compiler physical proposal
`v3`, explain schema `v11`, and explain renderer `v9` move. The proposal step
covers both fieldless boundary-role bytes and opaque binding records encoded as
framed parameter name plus big-endian `AccessOrdinal`; the explain-schema step
also covers the new typed `InOutRegionUnsupported` refusal record, and the
renderer step covers the existing opaque subject's rendered `input#N` →
`access#N` change. Request subject `v6`, kernel program `v11`, artifact stage `v3`,
artifact program `v17`, manifest schema 17.0, semantic/index identities, the
compilation-explain wrapper `v1`, and the `(InputKey, Axis, AbiType)` extent row
stay at their current domains/grammar. Their *values* and downstream goldens
move wherever they fold a stepped identity or a changed valid `InputKey`
binding; unchanged grammar is not a promise of unchanged value.

The owning pins/goldens include
`schedule::builder::tests::the_strict_f32_region_has_its_recorded_canonical_identity`
and `STRICT_F32_REGION_IDENTITY_HEX`, kernel
`identity_is_independent_of_planning_ordinals_and_separates_content` plus
`ABSENT_SUBGROUP_KERNEL_IDENTITY_HEX`, compiler
`domains::tests::every_tiler_spelled_literal_is_pinned_or_classified` and
`every_pinned_identity_domain_has_its_exact_source_population`, frontier
`the_recognized_region_subjects_keep_their_exact_proposals` and
`GOVERNED_PROPOSALS`, call-registry
`proposal_subject_is_exact_ordered_and_bounded`, explain
`explain_vocabulary_is_append_only_and_versioned` and
`deterministic_trace_is_sealed_and_rendered_separately`, pipeline
`mixed_frontier_records_exact_opaque_call_rejection_detail` and
`an_unregistered_opaque_call_named_on_the_compile_path_is_refused_by_name`, and
every exact `tiler-explain-v8` header assertion in `explain.rs`, `session.rs`,
and `pipeline/tests.rs`. Update the ledgers in `docs/compiler/optimizer.md` and
`docs/artifact-abi.md` from the owning source values; do not copy a stale pin.

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
local-coordinate site with the exact field/diagnostic spellings above; use
compiler-private `DeclaredInputOrdinal` for retained declared associations and
interface queries; add public
`KernelBuildError::InputExtentAccessOutOfRange`; use typed public
`ArtifactBuildError::ExtentOperandUnbound.access: AccessOrdinal`; migrate opaque
bindings and boundary derivation exactly as specified, including the regional
`InOutRegionUnsupported` refusal while retaining generic ABI `InOut`; and step
schedule, kernel, proposal, explain-schema, and explain-renderer domains — or
defer implementation and keep the dependent ticket blocked? No second nondominated
implementation model remains at this base.

## Accepted decision — 2026-08-14

Tom accepted the exact complete replacement in the live Codex conversation,
relayed by the coordinating agent. Implementation must use the public and
private coordinate types, field and diagnostic spellings, opaque-call regional
`InOut` refusal, validation order, identity/version migrations, unchanged wire
grammars, and required controls specified above without amendment.

## Required controls after acceptance

- **Positive declared rebinding.** Compile two independently verified requests whose local region is identical but whose declared association differs. For sparse declared subset `[0, 2]`, local read positions `[0, 1]` reach the intended two `InputKey`s in each request. Schedule and kernel canonical bytes remain identical; request-subject identity changes, the two `CoverAssembly` values bind the corresponding declared inputs, and kernel-program/artifact identity changes where those exact `InputKey` origins are encoded. A changed valid association is not a `request-binding` failure.
- A later fold over declared input `[1]` maps local read position `0` to input `1`, never input `0`. A two-of-three contraction maps its two local positions to the exact declared pair. Rebuild each under a second valid association and assert the same stable-local/changed-interface identity split.
- Preserve and count epilogue `[Intermediate, input 2]`, two mapped reads of one declared input, and dense-prefix all-input neighbours. Repeated local positions remain distinct even when they project to the same declared ordinal and `InputKey`; nothing filters or deduplicates them.
- **Independent request and wrapper negatives.** Pair a semantic program with a different verified request and quote existing `semantic-request-binding`. Separately pass a `VerifiedScheduledRegion` wrapper minted under request A to program verification under request B and quote existing `request-subject`. Do not label a separately verified valid request B a mismatch, and do not demand that a standalone public `KernelProgramBuilder` infer same-role intent its verified kernel does not carry.
- **Independent input-extent negatives.** On epilogue `[Intermediate, input 2]`, access `0` fails `InputExtentNotInput`, access `1` reaches input `2`, the final write fails `InputExtentNotInput`, and an absent access fails `InputExtentAccessOutOfRange`. Swap only the axis on access `1` and quote `InputExtentWrongAxis`.
- **Positive opaque rebinding.** Use two compatible read accesses with distinguishable element counts. Swap which valid access two named `In` parameters target while preserving complete coverage. Both registered proposals admit; schedule/kernel identity stays fixed, proposal identity and the exact opaque subject change, and `WorkScaling::PerElementOf` follows the newly named access. Drive the two exact subjects through the same typed opaque refusal separately and prove their canonical explain records, rendered `access#N` subjects, and trace identities differ. There is no valid-access subject-mismatch refusal.
- **Opaque regional profile controls.** Admit an ordinary two-parameter call over `[read, owning write]` with `In` bound to the read and `Out` bound to the write; assert complete coverage and access-ordered requirement/guarantee facets. Separately declare a coherent generic one-parameter `InOut` ABI with `ParameterLayout::Both` and `Aliasing::MayAliasInputs`, bind it to the valid read in that same two-access region, and quote `InOutRegionUnsupported { parameter: "buffer", access: 0 }` plus `opaque-call.binding.inout-region-unsupported`. Prove that this typed refusal is selected before the otherwise inevitable `UnboundAccess(1)` and before boundary derivation. The generic ABI declaration and its lower-level `reads()`/`writes()` and `Both` controls remain green; no regional positive uses that direct derivation test.
- **Opaque validation and ordering negatives.** Bind `In` to the write and `Out` to a read and quote `access-mode-mismatch` separately; name an absent coordinate and quote `access-out-of-range`; omit one otherwise valid boundary access and quote `unbound-access`. Bind two distinct compatible `In` parameters to one read (and, independently, compatible `Out` parameters to the owning write) and prove admission plus one facet at that access; then perturb only one storage declaration and quote `access-storage-disagreement`. Reverse binding order while keeping access assignments and prove proposal identity changes but boundary facets remain in access-list order.
- Perturb the schedule, kernel, physical-proposal, explain-schema, and explain-renderer spellings independently so each owning pin names its stale row/header. Prove artifact row bytes remain `(InputKey, Axis, AbiType)`, manifest schema remains 17.0, and all downstream values folding a moved identity are regenerated.

## Outcome

The exact replacement is accepted with provenance recorded above. The
implementation ticket consumes the choice without another public signature,
coordinate-domain, diagnostic, identity, or schema decision.
