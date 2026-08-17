---
id: decide-the-data-dependent-index-representation-public-surface
title: Decide the data-dependent index representation public surface
status: in-progress
priority: p1
dependencies: [accept-adr-0108-data-dependent-index-coordinate-siting, name-the-fact-source-on-retained-write-ownership-evidence]
related: [admit-the-selected-data-dependent-index-representation, revise-adr-0108-with-a-complete-data-dependent-index-vertical, admit-an-invocation-scoped-gather-index-validation-receipt]
scopes: [implementation/ir, implementation/reference, implementation/compiler, contracts/foundation, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: [.ticketsplease/decision-queue.md]
tags: [decision, needs-tom, public-boundary, indexing, identity, correctness]
claimed_from: todo
assignee: worker-gather-surface
lease_expires_at: 1786969799
---
## User-visible outcome

Before data-dependent gather enters the verified index/schedule vocabulary, Tiler has one accepted exact public representation, checked address-only read association, proof authority, diagnostic surface, and identity migration. No implementation guesses a Rust spelling or treats a dynamic invocation obligation as a timeless proof.

## Exact-base Fact audit — 2026-08-17, `0bd9e79da13a1c9098a4f67906df9e144a11432f`

The audit read the complete ticket and dependency set, the accepted decision and contract set, and every current construction, validation, compaction, proof, identity, reference, request, schedule, lowering, refusal, registry, and test owner named in the evidence log below. Searches located owners; the verdicts come from reading the complete files. This repairs the discovery record before making a decision. The repair does not change the ticket's purpose.

1. **Fact — verified.** [ADR 0108](../docs/decisions/0108-site-a-data-dependent-index-coordinate-on-the-expression.md) selects an append-only tagged gather access and literally says `This decision accepts no public Rust spelling`. [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) therefore reserves this packet's public boundary for Tom.
2. **Fact — verified.** `AccessData` and `TensorAccessRef` in `crates/tiler-ir/src/index/model.rs` describe one tensor and one coordinate list. `IndexRegionBuilder::read` in `crates/tiler-ir/src/index/builder.rs` creates a scalar `AccessRead`. `verify_pointwise_accesses` in `crates/tiler-ir/src/schedule/model.rs` requires `reads.len() == input_count` and pairs reads and scalar input leaves by ordinal. A gather's F32 value read plus its address-only U32 read cannot enter that model without an explicit association rule.
3. **Fact — verified.** No current type states the extra read's owner, order, multiplicity, or whether it is also a scalar value input. A search hit is not the evidence: the complete schedule verifier and construction paths contain no alternative association.
4. **Fact — imprecise and repaired.** The discovery said no closed producer can mint `StaticallyProved`. No such producer or gather proof type exists today, but two closed O(1) proof cases are derivable from accepted types without reading index data: an empty index domain is vacuous, and a gathered source extent at least `2^32` contains every exact U32 value. Every other case requires invocation validation. The packet below fixes one private minting authority and those two cases only.
5. **Fact — verified.** `PendingIndexRefinementReceipt` and `IndexRefinementUnknown` are not truthful dynamic-gather results. `CoveredOccurrence::from_receipt` accepts only a completed `IndexRefinementReceipt`; a dynamic obligation must instead stop before executable coverage and must not receive a receipt or executable-coverage identity.
6. **Fact — verified.** The accepted ADR selects no law, access, schedule, proof, registry, or diagnostic tags. The source has advanced since the discovery base: current owners include the landed source-bearing proof and access repairs, so this packet derives from the exact base rather than reusing the `f46ac65c` census.
7. **Fact — repaired maturity claim.** ADR 0108 is accepted. The chronological decision catalog, IR contract, open-question record, and roadmap still called it proposed or returned for revision; this ticket corrects those records. The current source census remains five `IndexNode` variants, three `IndexExprClass` variants, and three `IndexDomainUnknownReason` variants because accepting an architecture did not implement it.
8. **Fact — repaired dependency claim.** Both dependencies are `done`; there is no remaining live scope conflict with `name-the-fact-source-on-retained-write-ownership-evidence` on this base.
9. **Fact — verified.** The existing index-region encoder has direct read/write access tags `0x01`/`0x02`; `IndexRealizationLaw` uses `0x01` through `0x0D`; `LogicalAccess` uses `0x01` through `0x09`. The pending, independently reviewed live-row-major packet reserves schedule tags `0x0A`/`0x0B`, so this packet does not create two reviewable claims to the same byte.
10. **Fact — verified.** The standard realization sidecar currently contains sixteen rows. The governed compiler request subject is version 6 and already folds lowering-registry and realization-registry identities. Adding the gather rows therefore moves those identities and all request/explain qualifiers that contain them, without changing the request-domain grammar.
11. **Fact — verified semantic boundary.** ADRs 0107 and 0108 admit exactly an F32 source, exact U32 index, one explicit gathered axis, program-input operands, a static semantic result, nonzero source rank, rank-zero or higher index, duplicates, a shared result domain, one direct source coordinate for each non-gather source axis in source-axis order, and one complete index coordinate for each index axis in index-axis order. The index value supplies only the omitted gathered-axis coordinate.
12. **Fact — verified authority boundary.** `DTypeNotDispatchable` and the `dtype-recognized` reason in `crates/tiler-compiler/src/request.rs` currently refuse the U32 operand before operation lowering on ordinary targets. Admission must recognize U32 only as the gather's address operand; it must not widen the general scalar-arithmetic dispatch set.

### Evidence log

Complete files read at the exact base: root `AGENTS.md`; this ticket; `admit-the-selected-data-dependent-index-representation`; `accept-adr-0108-data-dependent-index-coordinate-siting`; `name-the-fact-source-on-retained-write-ownership-evidence`; `revise-adr-0108-with-a-complete-data-dependent-index-vertical`; `admit-an-invocation-scoped-gather-index-validation-receipt`; `docs/README.md`; the complete ADR index and ADRs 0046, 0074, 0075, 0107, 0108, and 0109; the relevant complete IR, open-question, roadmap, status, and work-tracking contracts; and the complete implicated files under `tiler-ir::{index,semantic,schedule}`, `tiler-reference`, and `tiler-compiler` covering model, builder, proof, compaction, law, refinement, sourced values, identity, registry, reference evaluation, request verification, governed lowering, schedule formation, policy, physical refusal, errors, and their correctness-bearing tests.

## Recommended exact public surface

This is one atomic surface. Partial acceptance is a rejection and keeps the existing typed refusal.

### Checked index access sum

Replace private `AccessData` with the private checked sum `Direct(DirectAccessData)` or `GatherRead(GatherReadAccessData)`. Existing direct construction and encoding remain byte-for-byte unchanged.

Keep public `TensorAccessRef<'a>` as the common borrowed wrapper with only:

```rust
pub fn id(self) -> VerifiedTensorAccessId;
pub fn mode(self) -> AccessMode;
pub fn domain(self) -> impl ExactSizeIterator<Item = VerifiedDimensionId> + 'a;
pub fn view(self) -> TensorAccessView<'a>;

#[derive(Clone, Copy, Debug)]
pub enum TensorAccessView<'a> {
    Direct(DirectTensorAccessRef<'a>),
    GatherRead(GatherReadAccessRef<'a>),
}
```

`TensorAccessView` is exhaustive: reference and compiler consumers must prove they considered every admitted access kind. The common wrapper's current `tensor()`, `coordinates()`, `bounds_proof()`, and `write_ownership_proof()` are removed because none has one truthful meaning for both variants. This unpublished pre-alpha repository has no compatibility requirement that outweighs that correctness boundary.

`DirectTensorAccessRef<'a>` and `GatherReadAccessRef<'a>` are `Clone + Copy + Debug`, nonconstructible outside the crate. The direct view exposes `tensor()`, `coordinates()`, `bounds_proof()`, and `write_ownership_proof()` with their existing return types. The gather view exposes:

```rust
pub fn source(self) -> VerifiedTensorId;
pub fn index(self) -> VerifiedTensorId;
pub fn axis(self) -> Axis;
pub fn source_coordinates(
    self,
) -> impl ExactSizeIterator<Item = VerifiedIndexExprId> + 'a;
pub fn index_coordinates(
    self,
) -> impl ExactSizeIterator<Item = VerifiedIndexExprId> + 'a;
pub fn bounds_resolution(self) -> GatherIndexBoundsResolution<'a>;
```

A gather view's common `mode()` is always `AccessMode::Read`. The two operands must be distinct program-input boundaries. They may refer to storage that a future alias model proves equivalent, but this admission neither detects nor authorizes byte aliasing. One `TensorId` cannot play both semantic roles.

### Authoring, validation, and diagnostics

The only authoring entry point is:

```rust
pub fn gather_read(
    &mut self,
    source: TensorId,
    index: TensorId,
    domain: &[DimensionId],
    source_coordinates: &[IndexExprId],
    index_coordinates: &[IndexExprId],
    axis: Axis,
) -> Result<ScalarValueId, IndexBuildError>;
```

The result is one F32 scalar value. Loading the U32 index is intrinsic to this compound access and never creates a scalar SSA value. Repeating an identical call interns the whole access atomically and returns the same scalar value. A separately authored direct U32 read is a distinct access and does not satisfy, merge with, or share the gather's address read.

The builder checks, in this exact precedence: valid source handle; valid index handle; distinct tensor identities; source is a program input; index is a program input; source is exact F32; index is exact U32; source rank is nonzero; axis is in range; source-coordinate arity equals `source_rank - 1`; index-coordinate arity equals index rank; domain has no duplicate dimensions and has the statically derived gather-result shape; every source coordinate is in the domain in source-axis order excluding `axis`; every index coordinate is in the same domain in index-axis order; then structural limits and allocation.

Add these exact `IndexBuildError` variants, while retaining existing invalid-handle, foreign-expression, duplicate-domain, out-of-domain-coordinate, and resource variants where named above:

```rust
GatherAliasedTensors { tensor: TensorId },
GatherSourceNotInput { tensor: TensorId },
GatherIndexNotInput { tensor: TensorId },
GatherSourceNotF32 { tensor: TensorId, actual: Arc<ResolvedValueType> },
GatherIndexNotU32 { tensor: TensorId, actual: Arc<ResolvedValueType> },
GatherSourceRankZero { tensor: TensorId },
GatherAxisOutOfRange { axis: Axis, source_rank: usize },
GatherSourceCoordinateRank { expected: usize, actual: usize },
GatherIndexCoordinateRank { expected: usize, actual: usize },
GatherDomainShape { expected: Shape, actual: Shape },
```

Whole-region revalidation owns corruption or future internal construction through:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GatherAccessRule {
    SourceRole,
    IndexRole,
    SourceType,
    IndexType,
    SourceRank,
    Axis,
    SourceCoordinateRank,
    IndexCoordinateRank,
    DomainShape,
    SourceCoordinateScope,
    IndexCoordinateScope,
    BoundsResolution,
}

IndexRegionDiagnostic::GatherAccess {
    access: TensorAccessId,
    rule: GatherAccessRule,
}
```

`GatherAccessRule` is exhaustive inside `tiler-ir` and `#[non_exhaustive]` publicly. The builder errors win for caller input; `GatherAccess` is the later verifier owner and must not collapse into `CoordinateOutOfBounds` because invocation-required data is not an observed bad coordinate.

`IndexRealizationLaw` gains variant `Gather { axis_attribute: AttributeFieldId }` and `pub const fn IndexRealizationLaw::gather_f32() -> Self`, which fixes `GATHER_AXIS_ATTRIBUTE`. The public field is the inspection surface; no parallel law-view type is added. Use fresh encoder tag `0x0E` and one standard `tiler::gather-f32@1` row at revision 1.

The governed compiler lowering registry gains a revision-1 gather capability and this exact facade, delegating to the builder and emitting no scalar U32 operation:

```rust
pub fn gather_read(
    &mut self,
    source: TensorId,
    index: TensorId,
    domain: &[DimensionId],
    source_coordinates: &[IndexExprId],
    index_coordinates: &[IndexExprId],
    axis: Axis,
) -> Result<ScalarValueId, LoweringEmitError>;
```

### Closed proof authority and mandatory dynamic stop

Add opaque, private-field owned types `GatherIndexBoundsProof` and `GatherIndexValidationRequirement`, and these read surfaces:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GatherIndexBoundsResolution<'a> {
    StaticallyProved(&'a GatherIndexBoundsProof),
    InvocationValidationRequired(&'a GatherIndexValidationRequirement),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GatherIndexBoundsProofKind {
    VacuousEmptyIndexDomain,
    U32RangeContainedBySourceExtent,
}
```

Only the verifier-private `derive_gather_index_bounds` mints either object. There is no public constructor, setter, proof parameter, unsafe escape, or boolean assertion. The proof and requirement both bind the exact region identity and access ordinal, source and index tensor ordinals and exact types, axis, gathered source extent, and domain identity. `GatherIndexBoundsProofIdentity` and `GatherIndexValidationRequirementIdentity` are opaque digest wrappers returned by `identity()`; `kind()` is the only proof classification accessor. The remaining binding accessors expose the fields above for diagnostics and cross-checking, never mutation.

Derivation order is closed:

1. `VacuousEmptyIndexDomain` when any exact index-shape extent is zero;
2. `U32RangeContainedBySourceExtent` when the gathered source extent is at least `2^32`;
3. `InvocationValidationRequired` for every other valid gather.

The second case uses exact mathematical extent comparison; it does not narrow `2^32` into U32. The first wins when both apply, so proof identity is canonical. No data sample, profile, caller claim, target property, or reference run can mint static proof.

At semantic refinement the requirement is rebound to the exact occurrence, ordered operand bindings `[source, index]`, and result binding as opaque, private-field `InvocationGatherIndexValidationRequirement`. It exposes `subject()`, `access()`, `source_binding()`, `index_binding()`, `result_binding()`, and `requirement()` borrowed accessors. Add exact outcomes:

```rust
IndexRefinementVerificationOutcome::InvocationValidationRequired(
    Box<InvocationGatherIndexValidationRequirement>,
)

IndexRefinementOutcome::InvocationValidationRequired(
    Box<PendingInvocationIndexValidation>,
)
```

Compiler-owned `PendingInvocationIndexValidation` has private fields and borrowed `provider()`, `revision()`, `capability_authority()`, and `requirement()` accessors. Existing `verified()`/`pending()` or `refined()`/`pending()` accessors return `None` for the new arm; add `invocation_validation_required()` to both outcomes and `into_invocation_validation_required()` to the compiler outcome. Add `IndexRefinementVerificationError::GatherValidationRequirementMismatch` for a requirement whose access, occurrence, ordered bindings, axis, shapes, or types disagree. A valid dynamic gather returns the requirement outcome only after all provider, law, occurrence, and binding checks. It creates no `IndexRefinementReceipt`, no `IndexRefinementReceiptIdentity`, no `IndexRefinementExecutableCoverageIdentity`, no `CoveredOccurrence`, schedule, artifact, cache subject, or dispatch attempt. The separate receipt ticket owns validation bytes, immutable snapshots, invocation binding, runtime carriage, and any later transition to executable coverage.

### Schedule association, ordering, and proof

Only statically proved gathers reach schedule formation. Add:

```rust
pub enum LogicalAccess {
    // existing variants unchanged
    GatherSource {
        source_shape: Shape,
        result_shape: Shape,
        axis: Axis,
        index_access: AccessOrdinal,
        index_shape: Shape,
    },
}
```

Use schedule-relation encoder tag `0x0C`. Tags `0x0A` and `0x0B` remain reserved by the earlier live-row-major decision packet until that packet resolves; a gap is preferable to colliding reviewed identities.

Canonical schedule access order is: all scalar value-producing reads in pointwise-leaf order; then one address-only U32 read for each owning `GatherSource` in owner-access order; then the write. Every `GatherSource` names exactly one later address-only read through `index_access`; every address-only read is named by exactly one gather source; no address-only read may be a scalar leaf, be shared by two gathers, or remain unreferenced. For the initially admitted one-gather occurrence the source read is access 0, the index read is access 1, and the write is access 2. Request binding cross-checks access 0 and 1 against ordered semantic operands 0 and 1, including exact types, shapes, and axis.

Schedule verification owns these failures rather than the broad `AccessContract` or `NumericalOrAccessRefinement` buckets:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GatherAddressReadRule {
    IndexNotLater,
    IndexMode,
    IndexRelation,
    IndexUsedAsScalarLeaf,
    IndexShared,
    IndexUnowned,
    OccurrenceBinding,
    ProofMismatch,
}

ScheduledRegionDiagnostic::GatherAddressRead {
    source_access: Option<AccessOrdinal>,
    index_access: AccessOrdinal,
    rule: GatherAddressReadRule,
}
```

Both public rule enums are `#[non_exhaustive]`; their crate-internal matches are total. `source_access` is `None` only for `IndexUnowned`; `index_access` still names the orphan. Verification reports owner range/order first, then mode, relation, scalar-leaf use, sharing, orphaning, occurrence binding, and proof mismatch. U32 typing belongs to the compiler's exact occurrence-binding cross-check because a standalone scheduled `Access` intentionally carries no dtype.

The index address relation is verifier-derived, never caller-selected: equal result/index shape uses `LinearIdentity`; rank-zero index uses `ScalarBroadcast`; otherwise canonical `BroadcastReplication` projects the inserted index axes in their original order. The source relation is `GatherSource`. Existing direct source coordinates map the non-gather axes in source order; the loaded U32 supplies exactly the omitted axis.

`BoundsProofKind` gains variant:

```rust
GatherSource {
    source_shape: Shape,
    result_shape: Shape,
    axis: Axis,
    index_access: AccessOrdinal,
    index_shape: Shape,
    proof: GatherIndexBoundsProofKind,
}
```

Use fresh bounds-proof encoder tag `0x03`; current `LinearRange`/`ReductionDomain` bytes at `0x01`/`0x02` do not move. Schedule verification cross-checks the relation, proof, paired address read, and request occurrence. The ordinary derived index-address map retains its own existing direct bounds proof.

### Diagnostics and precedence

Current target verification sees a general U32 input as `DTypeNotDispatchable`/`dtype-recognized`. Admission adds a special F32/U32 gather recognizer; it does not add U32 to `recognized_program_arithmetic` or let U32 become a scalar leaf. On a target that lacks the U32 carrier, the existing target refusal remains first.

On a U32-capable target the order is: request/interface and target type checks; operation/law/provider selection; lowering construction and all builder diagnostics in their order above; occurrence/binding/refinement verification; then the stable compiler reason `gather-invocation-validation-required`. Provider defects, missing governed rows, malformed bindings, and invalid regions therefore remain visible instead of being hidden by the expected dynamic stop. The dynamic reason takes precedence over scheduling, feasibility, target-candidate, artifact, cache, and dispatch diagnostics because none of those stages is entered. It is not reported as `dtype-recognized`, `operation-set`, `IndexRefinementUnknown`, `MissingCapability`, `NoFeasiblePlan`, or a backend miss.

The governed gather lowering uses exact `LoweringEmitError::Occurrence` rules, before builder work, in this order: `gather-operand-arity`, `gather-result-arity`, `gather-axis-attribute`, `gather-result-shape`, and `gather-operand-binding`. The builder's structured errors follow. `GatherValidationRequirementMismatch` follows successful region verification. The successful dynamic outcome then maps to `gather-invocation-validation-required`; it is not an error from either lower layer.

## Identity, schema, registry, and cache consequences

- Index access: fresh tag `0x03`; frame source tensor ordinal, index tensor ordinal, axis, domain, source coordinates, and index coordinates in exactly that order. Ordinals and axis are big-endian U32; each vector has a big-endian U64 count followed by its big-endian U32 members, matching `push_len` and `encode_u32s`. Existing direct read/write encodings remain byte-for-byte unchanged. `INDEX_REGION_IDENTITY_DOMAIN` remains version 11 because a fresh injective tag adds a value without reinterpreting an old byte.
- Bounds resolution is deterministic from the framed access and exact shapes and is not an independently caller-encoded choice. `tiler.gather-index-bounds-proof.v1` frames proof kind, region identity, access ordinal, source/index tensor ordinals and canonical types, axis, source extent, and domain identity. `tiler.gather-index-validation-requirement.v1` frames the same fields without proof kind. `tiler.invocation-gather-index-validation-requirement.v1` frames the access-level requirement identity, refinement-subject identity, and canonical ordered source/index/result bindings. Each length-prefixes every variable-width component and writes fixed integers big-endian, matching the repository identity convention. No snapshot bytes or observed values enter any of the three.
- Realization law: fresh `0x0E` followed by the axis `AttributeFieldId`'s canonical big-endian U32, standard row count 16 to 17, new gather row revision 1. Existing row bytes remain exact; the frozen realization-registry identity moves.
- Schedule: fresh relation `0x0C` followed by framed source shape, result shape, big-endian axis, big-endian index-access ordinal, and index shape; proof tag `0x03` followed by the same association and one proof-kind byte (`0x01` empty, `0x02` U32-universe). Old access/proof bytes remain exact and the schedule identity domain does not step. Only newly schedulable static gather identities are new.
- Compiler: the governed gather lowering row is revision 1 and moves the frozen lowering-registry identity. Request subject v6 already folds both registries, so every request identity and explain qualifier containing either registry moves. This is a value cascade, not a request-domain or explain-schema reinterpretation; no version step is justified.
- Refinement: static gather gains new subject/resolution/authority values. Dynamic gather receives only the explicit requirement outcome and no receipt or coverage identity.
- Artifact/cache: this ticket changes no artifact or manifest schema. A static gather can acquire downstream identities only after later backend work. A dynamic gather reaches neither artifact identity nor cache lookup/publication. The receipt ticket owns any future compatibility fence.

Implementation must pin all old direct bytes before and after, the new gather field-order injectivity, law-row distinctness, schedule relation/proof distinctness, registry/request cascade, and the absence of dynamic artifact/cache construction. Typed `variant_count` censuses must move `IndexNode` 5 unchanged, `IndexExprClass` 3 unchanged, `IndexDomainUnknownReason` 3 unchanged, `IndexRealizationLaw` 13 to 14, standard realization rows 16 to 17, and the relevant access-view, schedule-relation, proof, refinement-outcome, and diagnostic enums by their exact new populations. Handwritten length claims are not substitutes.

## Complete unsupported population

This surface refuses signed, other unsigned, and floating indices; negative-index conventions; clamp, wrap, or truncation; inferred or multiple axes; recursive/nested or multiple indirect reads in one access; an index load exposed as scalar SSA; sharing one address-only read across gathers; coalescing a direct U32 read with a gather address read; non-input source or index tensors; rank-zero source; data-dependent result shape; scatter and duplicate-write semantics; mutable-device or zero-copy validation; caller assertions; inline-kernel validation; dynamic dispatch receipts; artifact/runtime carriage; and Metal emission. It also refuses caller-selected address relations, proof kinds, schedule associations, and cache participation. Duplicate gather reads remain allowed and deterministic. Static sourced semantic extents remain allowed only where the existing exact `ShapeEnv` resolves the two proof predicates; an unresolved gathered extent takes the dynamic requirement rather than invented authority.

## Host memory and runtime comparison

The selected access owns two tensor IDs, one axis, and three bounded coordinate vectors (domain, direct source, index), so retained host storage is `O(result_rank + source_rank + index_rank)` and one enum discriminant beyond current direct access. Static proof derivation is O(index-rank) only to detect a zero extent, then O(1) for the U32-universe comparison; the proof/requirement record is O(1). Dynamic admission does not scan or copy index data here. The receipt ticket's accepted research bound remains separate: validating `T <= 8192` U32 elements would inspect O(T) values and at most 32 KiB of index payload.

A nested tensor-reading expression would retain the same tensor, type, proof, and association facts plus recursive expression edges and compaction state; it cannot use less asymptotic host memory and adds traversal to every expression consumer. A static-only or no-static-producer slice saves one small resolution variant but rejects valid accepted programs and does not remove the two-operand access state. Exact `size_of` values are implementation evidence, not a stable public contract; implementation must measure the old and new `AccessData`, `TensorAccessRef`, `LogicalAccess`, `Access`, and diagnostic layouts on the repository toolchain and record the delta before landing.

## Pareto-complete decision gate

Every surviving candidate is top-tier on correctness and fail-closed strictness. The complete tagged replacement is the sole nondominated candidate.

| Candidate | Disposition | Correctness, maintenance, and host consequences |
|---|---|---|
| Status quo / typed deferral | Dominated | Correctly refuses all gather refinement but delivers none of accepted ADR 0108; the complete surface preserves the same dynamic stop and adds the two sound static cases. |
| Static-only tagged access | Dominated | Correct for two cases, but silently dropping or rejecting the accepted invocation obligation prevents the receipt dependency from binding to a stable subject. It saves only one small outcome variant. |
| Tagged access with no static producer | Dominated | A truthful dynamic stop, but unnecessarily rejects empty-index and full-U32-range cases that exact shapes prove in O(1). |
| **Complete tagged access, two closed static proofs, mandatory dynamic stop** | **Sole frontier** | Preserves old direct bytes, gives every read and proof one checked owner, supports every case justified by current authority, and stops before execution whenever runtime evidence is required. |
| Generic nonrecursive indirect-access sum | Dominated | Can be made correct but publishes operand/type/axis combinations with no accepted semantics, increasing validation, enum, identity, and host state without another supported program. |
| Tensor-reading `IndexNode` / nested expression | Dominated | Can be made correct only by adding recursive read, reachability, proof, compaction, identity, reference, and public-expression machinery. ADR 0108 rejected that larger carrier for this one bounded family. |
| Further bounded research | Dominated | No unresolved owner, authority, identity, or negative control prevents an exact decision. More reading cannot select a smaller correct surface than the checked sum above. |

Caller-minted proof, an unassociated or shareable address read, `Unknown` for a mandatory dynamic obligation, default target/backend fallback, old request-registry identity, or dynamic progress into schedule/dispatch are eliminated rather than ranked because each can silently accept or misidentify a program.

### Strongest counterargument and reversal evidence

The strongest counterargument is breadth: this breaks common `TensorAccessRef` methods and publishes gather-specific access, proof, requirement, schedule, error, and refinement vocabulary before any dynamic gather can execute. The exact checked sum is nevertheless smaller than preserving misleading common methods or publishing a generic indirect language, and the separate outcome is the minimum state that makes the already-accepted receipt dependency possible without forging proof.

Evidence that would reverse the recommendation is an accepted second indirect family requiring recursive/nested reads, a public consumer that must inspect an access without region context and cannot total-match the view, or a demonstrated representation in which one checked address read can serve multiple gathers while preserving distinct occurrence, relation, proof, and request identity with less state. None exists on this base. Such evidence would reopen the carrier rather than be guessed into this admission.

### Required subject perturbations

Implementation and independent review must show the actual failure text after perturbing each subject, not the assertion:

- reuse index-access tag `0x02`, swap source/index fields, or delete the axis frame; injectivity pins must fail independently while old-direct-byte pins remain green;
- mint `StaticallyProved` for source extent `u32::MAX`, change the threshold from `>= 2^32`, and remove the empty-domain precedence; proof-kind and boundary tests must fail independently;
- make the address read a scalar leaf, share it between two gather owners, move it before a scalar read, point `index_access` at the write, or leave it unreferenced; schedule verification must name the violated association;
- change operand order `[source, index]` to `[index, source]`, bind the index to F32, or change the gathered axis; occurrence/refinement verification must fail before the dynamic stop;
- let `InvocationValidationRequired` construct a receipt, `CoveredOccurrence`, schedule, cache key, or dispatch request; the relevant typed census/negative construction check must fail;
- hold either registry identity at its old value after adding the row, or hold the request qualifier fixed; frozen-registry/request pins must fail;
- widen `IndexNode`, `IndexExprClass`, or unknown reasons incidentally; typed `variant_count` censuses must fail at compile time.

The decision packet itself has two load-bearing repository checks. Temporarily breaking a new local link must make `make citations` name that unresolved target; temporarily replacing a real dependency with a nonexistent ticket must make `tkt lint --format json` report `missing-dep`. Both perturbations are reverted before the final green runs.

## ADR 0108 application and graph

This packet corrects the accepted maturity in the chronological decision catalog, IR contract, Q-SHAPE-007, and roadmap. `docs/status.md` contains no stale ADR 0108 maturity claim and needs no edit. Production source comments and current 5/3/3 tests remain truthful about what is implemented, but comments that call ADR 0108 proposed/returned must be corrected by `admit-the-selected-data-dependent-index-representation` in the same implementation sweep. That carrier also owns every typed census, tag, byte pin, negative control, layout measurement, and request/registry recalculation above.

The separate receipt ticket remains the sole owner of dynamic validation and runtime/artifact carriage. The implementation ticket remains blocked until Tom accepts this exact included and excluded surface. This ticket stays `in-progress` through independent exact-commit review and is not moved to `awaiting-decision` by its author.

## Recommendation and exact Tom question

**Proposal — recommend the sole nondominated complete tagged surface above.** It is aligned with correctness first because no caller can mint proof, every address-only read has one checked owner, and every dynamic case stops. It is aligned with maintainability because it uses an exhaustive public sum and explicit gather-only types instead of infecting the expression language or publishing unsupported genericity. It is aligned with host performance because proof is O(1) after a bounded shape scan, dynamic admission copies no payload, and the retained state is linear only in ranks already represented by the access.

After independent exact-commit review, ask Tom one question: **Accept this exact atomic public surface, identity migration, diagnostic precedence, and unsupported boundary, or reject it and retain the current typed no-admission refusal?**
