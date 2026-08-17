---
id: decide-the-data-dependent-index-representation-public-surface
title: Decide the data-dependent index representation public surface
status: awaiting-decision
priority: p1
dependencies: [accept-adr-0108-data-dependent-index-coordinate-siting, name-the-fact-source-on-retained-write-ownership-evidence]
related: [admit-the-selected-data-dependent-index-representation, revise-adr-0108-with-a-complete-data-dependent-index-vertical, admit-an-invocation-scoped-gather-index-validation-receipt]
scopes: [implementation/ir, implementation/reference, implementation/compiler, contracts/foundation, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: [.ticketsplease/decision-queue.md]
tags: [decision, needs-tom, public-boundary, indexing, identity, correctness]
---
## User-visible outcome

Before data-dependent gather enters the verified index/schedule vocabulary, Tiler has one accepted exact public representation, checked address-only read association, proof authority, diagnostic surface, and identity migration. No implementation guesses a Rust spelling or treats a dynamic invocation obligation as a timeless proof.

## Exact-base Fact audit — 2026-08-17, `0bd9e79da13a1c9098a4f67906df9e144a11432f`

The audit read the complete ticket and dependency set, the accepted decision and contract set, and every current construction, validation, compaction, proof, identity, reference, request, schedule, lowering, refusal, registry, and test owner named in the evidence log below. Searches located owners; the verdicts come from reading the complete files. This repairs the discovery record before making a decision. The repair does not change the ticket's purpose.

1. **Fact — verified.** [ADR 0108](../docs/decisions/0108-site-a-data-dependent-index-coordinate-on-the-expression.md) selects an append-only tagged gather access and literally says `This decision accepts no public Rust spelling`. [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) therefore reserves this packet's public boundary for Tom.
2. **Fact — imprecise owner repaired, semantics verified.** `AccessData` and `TensorAccessRef` in `crates/tiler-ir/src/index/model.rs` describe one tensor and one coordinate list. `IndexRegionBuilder::read` in `crates/tiler-ir/src/index/builder.rs` creates a scalar `AccessRead`. The owner is `verify_pointwise_region` in `crates/tiler-ir/src/schedule/builder.rs`, not a nonexistent `verify_pointwise_accesses` in `schedule/model.rs`: it rejects `reads.is_empty() || reads.len() != input_count`, while expression verification pairs scalar leaves to those reads by ordinal. A gather's F32 value read plus its address-only U32 read cannot enter that model without an explicit association rule.
3. **Fact — verified.** No current type states the extra read's owner, order, multiplicity, or whether it is also a scalar value input. A search hit is not the evidence: the complete schedule verifier and construction paths contain no alternative association.
4. **Fact — false twice and repaired.** The discovery said no closed producer can mint `StaticallyProved`; the first packet then narrowed vacuity incorrectly to an empty *index* shape, and the first review repair incorrectly inferred fact source from whichever short-circuit proved the kind. No producer or gather proof type exists today, but two closed proof classes are derivable without reading index data: the **complete result/access domain is empty** when any result extent is zero, and a gathered source extent at least `2^32` contains every exact U32 value. Thus source `[0, 5]`, axis 1, index `[3]` yields result `[0, 3]` and is vacuous even though the index is inhabited. `predicate.rs` says `Program` is the strong claim that the **complete proof population** was literal, and `builder.rs::access_fact_source` scans the complete access domain, boundary shapes, and coordinate expressions independently of proof kind. This packet admits only literal source/index/domain shapes, but a coordinate may still contain a declared symbol: source `[0, 5]`, axis 1, index `[3]`, result/domain `[0, 3]`, and source coordinate `S * d0` under the bound environment determining `S = 1` proves vacuity with `ShapeEnvironment`; source `[2^32, 4]`, axis 0, rank-zero index, result/domain `[4]`, and the same source coordinate proves the U32 universe with the same provenance. Every other case requires invocation validation. The repair widens the static population and fixes provenance without changing the ticket's purpose.
5. **Fact — verified.** `PendingIndexRefinementReceipt` and `IndexRefinementUnknown` are not truthful dynamic-gather results. `CoveredOccurrence::from_receipt` accepts only a completed `IndexRefinementReceipt`; a dynamic obligation must instead stop before executable coverage and must not receive a receipt or executable-coverage identity.
6. **Fact — verified.** The accepted ADR selects no law, access, schedule, proof, registry, or diagnostic tags. The source has advanced since the discovery base: current owners include the landed source-bearing proof and access repairs, so this packet derives from the exact base rather than reusing the `f46ac65c` census.
7. **Fact — repaired maturity claim.** ADR 0108 is accepted. The chronological decision catalog, IR contract, open-question record, and roadmap still called it proposed or returned for revision; this ticket corrects those records. The current source census remains five `IndexNode` variants, three `IndexExprClass` variants, and three `IndexDomainUnknownReason` variants because accepting an architecture did not implement it.
8. **Fact — repaired dependency claim.** Both dependencies are `done`; there is no remaining live scope conflict with `name-the-fact-source-on-retained-write-ownership-evidence` on this base.
9. **Fact — verified.** The existing index-region encoder has direct read/write access tags `0x01`/`0x02`; `IndexRealizationLaw` uses `0x01` through `0x0D`; `LogicalAccess` uses `0x01` through `0x09`. The pending, independently reviewed live-row-major packet reserves schedule tags `0x0A`/`0x0B`, so this packet does not create two reviewable claims to the same byte.
10. **Fact — verified.** The standard realization sidecar currently contains sixteen rows. The governed compiler request subject is version 6 and already folds lowering-registry and realization-registry identities. Adding the gather rows therefore moves those identities and all request/explain qualifiers that contain them, without changing the request-domain grammar.
11. **Fact — verified for admitted literals; the sourced candidate was invalid and is withdrawn.** ADRs 0107 and 0108 admit an F32 source, exact U32 index, one explicit gathered axis, program-input operands, nonzero source rank, rank-zero or higher index, duplicates, a shared result domain, one direct source coordinate for each non-gather source axis in source-axis order, and one complete index coordinate for each index axis in index-axis order. The index value supplies only the omitted gathered-axis coordinate. Current `GatherF32` is registered with `OperationDefinition::new`, is `ShapeInferenceParticipation::LiteralOnly`, and calls `static_operand_shape`. B/C make the index-layer boundary equally narrow: `gather_read` rejects a nonliteral source boundary, nonliteral index boundary, or any nonliteral result/access-domain extent before comparing the concrete gather result shape. Coordinate expressions remain allowed to name symbols admitted by the region's existing `ShapeEnv`; that is coordinate evaluation and proof provenance, not sourced boundary support. The reference oracle therefore receives literal boundary shapes for every admitted Gather and needs no sourced equality, specialization, or second refusal authority.
12. **Fact — verified authority boundary.** `DTypeNotDispatchable` and the `dtype-recognized` reason in `crates/tiler-compiler/src/request.rs` currently refuse the U32 operand before operation lowering on ordinary targets. Admission must recognize U32 only as the gather's address operand; it must not widen the general scalar-arithmetic dispatch set.

### Evidence log

Complete files read at the exact base: root `AGENTS.md`; this ticket; `admit-the-selected-data-dependent-index-representation`; `accept-adr-0108-data-dependent-index-coordinate-siting`; `name-the-fact-source-on-retained-write-ownership-evidence`; `revise-adr-0108-with-a-complete-data-dependent-index-vertical`; `admit-an-invocation-scoped-gather-index-validation-receipt`; the accepted `decide-the-schedule-local-input-ordinal-model`, `reconcile-input-ordinal-region-local-and-declared-input-semantics`, and `decide-the-full-list-access-coordinate-for-out-of-list-references` records; `docs/README.md`; the complete ADR index and ADRs 0046, 0074, 0075, 0107, 0108, and 0109; the relevant complete IR, open-question, roadmap, status, and work-tracking contracts; and the complete implicated files under `tiler-ir::{index,semantic,schedule,kernel}`, `tiler-reference`, and `tiler-compiler` covering model, builder, proof, compaction, law, refinement, sourced values, identity, registry, reference evaluation, request verification, governed lowering, schedule formation, frontier, pipeline presentation, policy, physical refusal, errors, and their correctness-bearing tests.

## Exact common public surface for implementation-capable candidates

The following checked access, proof, requirement, diagnostics, and dynamic stop are common to literal-only frontier surfaces B and C. Schedule association differs only where stated. Tom must select one atomic surface; mixing fields across candidates is not acceptance and keeps the existing typed refusal.

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

`DirectTensorAccessRef<'a>` and `GatherReadAccessRef<'a>` are `Clone + Copy + Debug`, nonconstructible outside the crate. The direct view exposes `tensor() -> VerifiedTensorId`, `coordinates() -> impl ExactSizeIterator<Item = VerifiedIndexExprId> + 'a`, `bounds_proof() -> Option<BoundsProofView>`, and `write_ownership_proof() -> Option<WriteOwnershipProofView>`. The gather view exposes:

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

The builder checks, in this exact precedence: valid source handle; valid index handle; distinct tensor identities; source is a program input; index is a program input; source is exact F32; index is exact U32; source boundary shape is authored wholly literal; index boundary shape is authored wholly literal; source rank is nonzero; axis is in range; derive the one concrete result `Shape` by splicing the now-literal index shape into the now-literal source shape at `axis`; source-coordinate arity equals `source_rank - 1`; index-coordinate arity equals index rank; validate the domain handles in supplied order; reject a duplicate domain dimension; reject the first domain dimension whose extent is not authored literal; construct the concrete domain shape and compare it with the derived result under `GatherDomainShape`; validate both coordinate-handle runs and require every source coordinate to lie in that domain in source-axis order excluding `axis` and every index coordinate to lie in the same domain in index-axis order; then structural limits and allocation. The three literal-shape refusals therefore precede `GatherDomainShape`, and a result shape is never derived by consulting `ShapeEnv`.

Add these exact `IndexBuildError` variants, while retaining existing invalid-handle, foreign-expression, duplicate-domain, out-of-domain-coordinate, and resource variants where named above:

```rust
GatherAliasedTensors { tensor: TensorId },
GatherSourceNotInput { tensor: TensorId },
GatherIndexNotInput { tensor: TensorId },
GatherSourceNotF32 { tensor: TensorId, actual: Arc<ResolvedValueType> },
GatherIndexNotU32 { tensor: TensorId, actual: Arc<ResolvedValueType> },
GatherSourceShapeNotLiteral { tensor: TensorId },
GatherIndexShapeNotLiteral { tensor: TensorId },
GatherSourceRankZero { tensor: TensorId },
GatherAxisOutOfRange { axis: Axis, source_rank: usize },
GatherSourceCoordinateRank { expected: usize, actual: usize },
GatherIndexCoordinateRank { expected: usize, actual: usize },
GatherDomainExtentNotLiteral { dimension: DimensionId },
GatherDomainShape { expected: Shape, actual: Shape },
```

`GatherSourceShapeNotLiteral` and `GatherIndexShapeNotLiteral` mean `SourcedShape::as_static()` returned `None`; an environment that happens to determine every symbol does not turn authored sourced spelling into a literal boundary. `GatherDomainExtentNotLiteral` similarly reports the first supplied result/access-domain dimension whose `SourcedExtent::as_static()` is `None`. The errors retain the caller's exact tensor or dimension handle and do not embed, resolve, or canonicalize a sourced shape.

Whole-region revalidation owns corruption or future internal construction through:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GatherAccessRule {
    SourceRole,
    IndexRole,
    SourceType,
    IndexType,
    SourceShapeLiteral,
    IndexShapeLiteral,
    SourceRank,
    Axis,
    SourceCoordinateRank,
    IndexCoordinateRank,
    DomainExtentLiteral,
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

`GatherAccessRule` is exhaustive inside `tiler-ir` and `#[non_exhaustive]` publicly. Whole-region verification checks `SourceShapeLiteral`, `IndexShapeLiteral`, and `DomainExtentLiteral` in that order before `DomainShape`, mirroring the builder boundary for any future internal construction. The builder errors win for caller input; `GatherAccess` is the later verifier owner and must not collapse into `CoordinateOutOfBounds` because invocation-required data is not an observed bad coordinate.

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

The types below are defined in `crates/tiler-ir/src/index/model.rs` and re-exported exactly once from `tiler_ir::index` by `index/mod.rs`; there is no second module spelling. All fields are private. Each identity is `Clone + Debug + Eq + Hash + Ord + PartialEq + PartialOrd`, has no public constructor or byte conversion, and exposes `pub fn as_bytes(&self) -> &[u8]` only.

```rust
pub struct GatherIndexBoundsProofIdentity(Vec<u8>);
pub struct GatherIndexValidationRequirementIdentity(Vec<u8>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GatherIndexBoundsResolution<'a> {
    StaticallyProved(&'a GatherIndexBoundsProof),
    InvocationValidationRequired(&'a GatherIndexValidationRequirement),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GatherIndexBoundsProofKind {
    VacuousEmptyResultDomain,
    U32RangeContainedBySourceExtent,
}
```

`GatherIndexBoundsProof` exposes exactly `identity() -> &GatherIndexBoundsProofIdentity`, `kind() -> GatherIndexBoundsProofKind`, `facts() -> IndexDomainFactSource`, `region() -> &CanonicalIndexRegionIdentity`, `access() -> VerifiedTensorAccessId`, `source() -> VerifiedTensorId`, `index() -> VerifiedTensorId`, `source_type() -> &ResolvedValueType`, `index_type() -> &ResolvedValueType`, `source_shape() -> &Shape`, `index_shape() -> &Shape`, `result_shape() -> &Shape`, `axis() -> Axis`, `source_extent() -> Extent`, and `domain() -> impl ExactSizeIterator<Item = VerifiedDimensionId> + '_`. `GatherIndexValidationRequirement` exposes the corresponding `identity()`, `region()`, `access()`, source/index/type/shape/result/axis/extent/domain accessors with the same concrete return types and **no** `facts()` or `kind()`: a requirement is not proof evidence. `GatherIndexBoundsResolution` exposes `statically_proved() -> Option<&GatherIndexBoundsProof>` and `invocation_validation_required() -> Option<&GatherIndexValidationRequirement>`. This does not invent a free-floating “domain identity”; the opaque proof or requirement identity binds the exact ordered literal domain itself.

Only verifier-private `derive_gather_index_bounds` mints either object. It first derives `IndexDomainFactSource` **independently of the proof-kind short circuit**, exactly once, from the complete proof subject: the literal result/access-domain, source, index, and result shapes plus every source and index coordinate expression recursively. This mirrors `builder.rs::access_fact_source`. Because B/C reject every nonliteral boundary/domain extent, `ShapeEnvironment` is reachable here only when a coordinate expression names a declared symbol; it remains the correct weak answer even when that expression was unnecessary to the final inequality or empty-domain conclusion.

It then reads the result shape obtained by splicing the index extents into the source at `axis` and applies this closed kind precedence:

1. inspect **all** result extents through the accepted literal/environment fact source; if any is proved zero, mint `VacuousEmptyResultDomain`;
2. otherwise inspect the gathered source extent; a proved value `>= 2^32` mints `U32RangeContainedBySourceExtent`;
3. every other valid gather mints `InvocationValidationRequired`.

The empty-result proof wins even when the source-axis universe proof also holds. Source `[0, 5]`, axis 1, index `[3]`, result/domain `[0, 3]`, and ordinary literal coordinates proves vacuously with `Program`. Replace one source coordinate with `S * d0`, where the already-bound environment determines `S = 1`, and the proof kind remains `VacuousEmptyResultDomain` while `facts()` becomes `ShapeEnvironment`. Likewise source `[2^32, 4]`, axis 0, rank-zero index, result/domain `[4]`, and source coordinate `S * d0` retains `U32RangeContainedBySourceExtent` with `ShapeEnvironment`. No source, index, result, or domain extent is symbolic in either control. The comparison is in exact `u64`/mathematical extent space; it never narrows `2^32` into U32. `IndexDomainFactSource` appears once on the proof, is returned only by `facts()`, and is encoded once as the existing exhaustive `Program = 0x01` / `ShapeEnvironment = 0x02` tag. Every schedule `BoundsProofView` continues to expose its own source through its existing `facts()` total match. No sample, caller assertion, target fact, profile, or reference run can mint either proof kind.

The refinement-bound types live in `crates/tiler-ir/src/index/refinement.rs` and are re-exported from `tiler_ir::index`. `InvocationGatherIndexValidationRequirementIdentity` follows the same opaque-wrapper and `as_bytes()` rule. `InvocationGatherIndexValidationRequirement` exposes exactly `identity() -> &InvocationGatherIndexValidationRequirementIdentity`, `subject() -> &IndexRefinementSubject`, `access() -> VerifiedTensorAccessId`, `source_binding() -> &OperandBinding`, `index_binding() -> &OperandBinding`, `result_bindings() -> &[ResultBinding]`, and `requirement() -> &GatherIndexValidationRequirement`. The result is a slice because the existing refinement contract admits one binding per output root; gather validation requires exactly one and rejects any other count, but the accessor does not create a competing singular projection.

```rust
IndexRefinementVerificationOutcome::InvocationValidationRequired(
    Box<InvocationGatherIndexValidationRequirement>,
)

IndexRefinementOutcome::InvocationValidationRequired(
    Box<PendingInvocationIndexValidation>,
)
```

`tiler_compiler::legality::PendingInvocationIndexValidation` has private fields and exactly `provider() -> &ProviderIdentity`, `revision() -> LoweringCapabilityRevision`, `capability_authority() -> &LoweringCapabilityAuthority`, and `requirement() -> &InvocationGatherIndexValidationRequirement`. `IndexRefinementVerificationOutcome` gains `verified()`, `pending()`, and `invocation_validation_required()` borrowed accessors, each returning `None` for the other two arms. Existing compiler `IndexRefinementOutcome::{refined,pending}` are extended to return `None` for the new arm; it gains `invocation_validation_required()` and `into_invocation_validation_required() -> Option<PendingInvocationIndexValidation>`, while `into_refined()` returns `None` for it. `IndexRefinementVerificationError::GatherValidationRequirementMismatch` owns any disagreement in access, occurrence, ordered bindings, axis, shapes, or types.

A valid dynamic gather reaches this outcome only after provider, law, occurrence, exact `[source, index]` operand binding, one result binding, access association, and region checks. It creates no `IndexRefinementReceipt`, receipt identity, executable-coverage identity, `CoveredOccurrence`, schedule, artifact, cache subject, or dispatch. The separate receipt ticket alone owns validation bytes, immutable snapshots, invocation binding, runtime carriage, and any later transition to executable coverage.

The two levels are intentional rather than an omitted occurrence field. `GatherIndexBoundsProof` and `GatherIndexValidationRequirement` are reusable **region-local access** subjects: the index region has authority for the exact region, access, tensors, types, three shapes, axis, extent, coordinates, and ordered domain, but has no semantic-occurrence handle. The refinement layer is the first owner that has that handle. `InvocationGatherIndexValidationRequirement` frames the local requirement identity together with the exact `IndexRefinementSubject` and ordered source/index/result bindings, while request-to-schedule verification independently cross-checks that same occurrence and association. Therefore the complete ADR occurrence-scoped requirement exists only at the invocation wrapper; a caller cannot substitute a region-local requirement from another occurrence, and the reusable index-region identity does not absorb a graph-local occurrence coordinate.

### Normalized request subject and total compiler consumers

The recognizer cannot bypass request normalization: normalization runs before lowering/refinement, and request subject v6 encodes every `NormalizedOutputSubject`. Both literal implementation-capable surfaces therefore add a boxed arm to the crate-private exhaustive sums:

```rust
NormalizedOutput::Gather(Box<NormalizedGather>)
NormalizedOutputSubject::Gather(Box<NormalizedGatherSubject>)
```

The exact private payloads are:

```rust
struct NormalizedGather {
    input_keys: Vec<InputKey>,
    output_key: OutputKey,
    source_input: DeclaredInputOrdinal,
    index_input: DeclaredInputOrdinal,
    source_shape: Shape,
    index_shape: Shape,
    result_shape: Shape,
    axis: Axis,
    index_access: AccessOrdinal,
    member: SemanticMemberId,
    source_elements: u64,
    index_elements: u64,
    result_elements: u64,
}

struct NormalizedGatherSubject {
    input_keys: Vec<InputKey>,
    output_key: OutputKey,
    source_input: DeclaredInputOrdinal,
    index_input: DeclaredInputOrdinal,
    source_shape: Shape,
    index_shape: Shape,
    result_shape: Shape,
    axis: Axis,
    index_access: AccessOrdinal,
    member: SemanticMemberId,
    source_elements: u64,
    index_elements: u64,
    result_elements: u64,
}
```

These are B's source-side payloads: `index_access == AccessOrdinal::new(1)` and C removes that one field. `source_input != index_input`; they bind exact program-input operands 0 and 1. The scalar-value read is canonical local access 0 and the address-only read local access 1. B stores the companion on the owning source relation/subject; C derives it from canonical order and therefore has no editable ordinal. `output_subject` copies every selected field. There are no graph handles in either payload, so unlike staged/epilogue projection there is no recursive state to clear.

`encode_output_subject` gains a fresh framed `push_slice(bytes, b"gather-f32.v1")` arm and writes, in order: all declared `input_keys`; `output_key`; source declared ordinal; index declared ordinal; source shape; index shape; result shape; axis; member occurrence ordinal; source/index/result element counts; and association spelling tag (`0x01` source-side reference or `0x02` fieldless canonical order). Shapes use `encode_explain_shape`; counts are big-endian U64. The source-side form then writes the big-endian index `AccessOrdinal`; the fieldless form writes no ordinal. This is a new output subtag, so request domain v6 does not step and every old subject byte remains exact. Source key, index key, either declared ordinal, any shape extent, axis, member, element count, association tag, or source-side local ordinal changes the subject bytes independently.

Every total consumer must add an explicit gather arm; no wildcard is permitted to silently dispatch it:

- `NormalizedOutput::serial_sum()` panics for Gather just as for every non-serial arm; `try_serial_sum()`, `pointwise()`, `contraction()`, `epilogue()`, and `staged()` return `None`; new `gather() -> Option<&NormalizedGather>` returns `Some` only for Gather. `fused_prologue_constants` consequently returns `None`, `carries_parametric_broadcast` is false, and `producer_shape_for` returns the gather itself.
- `input_elements_at` returns `source_elements` for `source_input`, `index_elements` for `index_input`, and `None` otherwise; `reads_declared_input` recognizes exactly those ordinals; `max_input_elements` is their maximum; `output_elements` is `result_elements`; `members` returns `[SemanticStage::first(member)]`; `owns_region_members` accepts exactly that singleton.
- `output_subject` projects to `NormalizedGatherSubject`; `encode_output_subject` writes the payload above; the test-only arm census is sized with `variant_count::<NormalizedOutput>()` and moves 5 to 6.
- `physical::spell_output` returns new internal `RegionSpellingKind::Gather(write)` only when the member set is exactly `[SemanticStage::first(member)]`; a nonempty different, cross-output, or multi-occurrence set falls through to the existing `RegionVocabularyWall::PartialCoverage`, while the existing frontier `Coverless` stop owns an empty subject before `spell_output`. No gather reaches the serial/contraction/staged family builders. `published_shape` returns `result_shape`; `declared_input_for_verified_access` maps local access 0 to `source_input`, 1 to `index_input`, and refuses every other position; `verify_region_output_binding` compares member, region 0, the identity `PointwiseF32` scalar program with exactly one F32 leaf, the exact two accesses, all three shapes, axis, association, bounds proof, result count, and ordered declarations. A requirement outcome never reaches these functions.
- `pipeline::output_region_role` reports `"whole-program"` for `NormalizedOutput::Gather`, matching the other one-occurrence whole-output families. There is no `pipeline::strategy_label` owner.
- `frontier::govern_spelling` handles `RegionSpellingKind::Gather(write)` exhaustively. It calls `physical::gather_region`, offers no split/tree alternatives, and computes `result_elements` from the normalized gather: `PhysicalCostEstimate::structural(1, result_elements, 0)` for `ProgramOutput`, or `structural(1, result_elements, result_elements.saturating_mul(4))` for `Materialized` and `MaterializedAndPublished`. The existing generic published-and-consumed path then adds the publishing-copy dispatch and threads without hiding it. This is a frontier proposal/cost for a verified schedule, not a claim of an executable kernel.
- `region.rs` has no `NormalizedOutput` classifier to widen: its `SemanticStage` remains the occurrence/stage atom, and Gather uses its existing single-region `SemanticStage::first(member)`. Cover and partition ownership reach Gather only through the total `members`/`owns_region_members` methods above, so no second region vocabulary or stage count is introduced.

The relation encoder is total too. `encode_access_relation` assigns `LogicalAccess::GatherSource` fresh `0x06` (existing `0x04` stays the unread marker and `0x05` stays parametric broadcast), then writes source shape, result shape, axis, association spelling, optional index ordinal, and index shape. The existing wildcard `0x00` remains a refusal for unhandled future relations and can never encode gather. This makes relation identity injective independently of the request-output encoding.

### Index and reference consumers

Every current user of removed common `TensorAccessRef` methods must match `view()` totally. Index compaction copies a `DirectAccessData` unchanged or remaps both gather tensors, its domain, both coordinate runs, and its retained resolution atomically; proof derivation, predicate construction, law/refinement interface checks, sourced-region checks, and test helpers reach direct-only fields only through `DirectTensorAccessRef`. A gather can never enter write-ownership or ordinary direct-coordinate proof logic. The access view census is typed, and adding a third view stops every internal total match.

`tiler-reference::IndexRegionEvaluator` also matches the view rather than refusing the new arm generically. A direct access keeps its byte-for-byte/current evaluation. A gather evaluates the index-coordinate run first, loads one exact U32 element from the bound index tensor without creating scalar SSA, inserts its exact `u64` value at `axis` among the source-coordinate results, and loads the F32 source element. It checks source/index types and concrete shapes against the verified access; a requirement resolution validates the observed value but mints no receipt or proof, while a static resolution independently checks its proof identity and still bounds-checks defensively. Add:

```rust
IndexRegionEvaluationError::GatherIndexOutOfBounds {
    access: VerifiedTensorAccessId,
    index_offset: usize,
    value: u32,
    extent: u64,
}
```

The index offset is the row-major address in the index operand, not a result coordinate. This error wins after malformed handle/type/shape/coordinate checks and before any source payload read. `oracle::bind_inputs` and `output_plans` always receive literal boundary shapes for an admitted Gather; they keep their existing `SourcedShape::as_static()` refusal for every other family/path and gain no Gather-specific environment resolution or equality rule. Symbols in Gather coordinates are evaluated through the index region's existing bound `ExtentSources`/coordinate-evaluation path. Reference execution is an oracle result only: evaluating a requirement here cannot create compiler executable coverage, cache identity, or dispatch permission.

### Schedule association, ordering, and proof

Only statically proved gathers reach schedule formation, and every admitted gather already owns concrete source, index, result, and domain shapes by construction. Two correct association spellings survive the gate; Tom's selection is atomic with the capability surface. The self-locating source-side spelling is:

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

The lower-state fieldless spelling removes only `index_access`; its canonical rule fixes the owning address read as the first address-only read after the scalar leaf reads, in owner order. Neither spelling grants verification authority merely because `LogicalAccess` is publicly constructible: only the schedule verifier mints a verified schedule.

Use schedule-relation encoder tag `0x0C`. Tags `0x0A` and `0x0B` remain reserved by the earlier live-row-major decision packet until that packet resolves; a gap is preferable to colliding reviewed identities. `encode_access_relation`'s compiler-request tag is separately `0x06`; the domains are distinct and both tags are pinned.

Canonical schedule access order in both spellings is: all scalar value-producing reads in pointwise-leaf order; then one address-only U32 read for each owning `GatherSource` in owner-access order; then the write. In the source-side spelling every `GatherSource` names exactly one later address-only read through `index_access`; in the fieldless spelling the verifier derives the same bijection from the complete access-list order. Every address-only read has exactly one owner; none is a scalar leaf, shared by two gathers, or unreferenced. For the initially admitted one-gather occurrence the source read is access 0, index read access 1, and write access 2. Request binding cross-checks access 0 and 1 against declared semantic operands 0 and 1, including exact types, shapes, and axis.

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
    proof: GatherIndexBoundsProof,
}
```

The fieldless spelling omits `index_access` here too. Use fresh bounds-proof encoder tag `0x03`; current `LinearRange`/`ReductionDomain` bytes at `0x01`/`0x02` do not move. It writes the exact relation fields and the framed `GatherIndexBoundsProofIdentity`; proof kind and fact source remain encoded once inside that opaque proof identity rather than copied into schedule state. Schedule verification cross-checks relation, proof, paired address read, and request occurrence. The ordinary derived index-address map retains its own direct bounds proof.

**Explicit ADR amendment common to B and C.** ADR 0108 literally says the schedule `GatherSource` carries an “index-input ordinal.” The later accepted schedule-local/access-coordinate decisions eliminated that spelling: a declared-program ordinal in shared schedule identity aliases reusable computation with program-interface position, and `DeclaredInputOrdinal` is intentionally compiler-private. B and C therefore do not call the move a clarification. They amend that one ADR 0108 clause so the checked semantic association remains in `NormalizedGather::{source_input,index_input}`, the retained request subject, stage binding, and whole-program identity, while shared schedule identity carries only a local `AccessOrdinal` (B) or canonical local order (C). Preserving the literal older clause would require superseding the later accepted layer boundary and adding a new public declared-input coordinate to shared IR; that option is eliminated as worse on identity canonicality, public surface, and maintenance, and returns only if Tom explicitly reopens that accepted boundary.

### Diagnostics and precedence

Current target verification sees a general U32 input as `DTypeNotDispatchable`/`dtype-recognized`. Admission adds a special F32/U32 gather recognizer; it does not add U32 to `recognized_program_arithmetic` or let U32 become a scalar leaf. On a target that lacks the U32 carrier, the existing target refusal remains first.

On a U32-capable target the order is: request/interface and target type checks; operation/law/provider selection; lowering construction and all builder diagnostics in their order above; occurrence/binding/refinement verification; then the stable compiler reason `gather-invocation-validation-required`. Provider defects, missing governed rows, malformed bindings, and invalid regions therefore remain visible instead of being hidden by the expected dynamic stop. The dynamic reason takes precedence over scheduling, feasibility, target-candidate, artifact, cache, and dispatch diagnostics because none of those stages is entered. It is not reported as `dtype-recognized`, `operation-set`, `IndexRefinementUnknown`, `MissingCapability`, `NoFeasiblePlan`, or a backend miss.

The governed gather lowering uses exact `LoweringEmitError::Occurrence` rules, before builder work, in this order: `gather-operand-arity`, `gather-result-arity`, `gather-axis-attribute`, `gather-result-shape`, and `gather-operand-binding`. The builder's structured errors follow. `GatherValidationRequirementMismatch` follows successful region verification. The successful dynamic outcome then maps to `gather-invocation-validation-required`; it is not an error from either lower layer.

A statically proved literal gather may reach verified schedule formation and the governed frontier offer above, but it has no indirect KIR or backend route. `kernel::lower::addressing` must add an exhaustive `LogicalAccess::GatherSource { .. } => Err(KernelDiagnostic::BodyRefinement)` arm, whose stable rule is `body-refinement`; it must not select `LinearIdentity`, `BroadcastReplication`, or a backend fallback. `physical::lower_structured_kernel` reattributes that as `PhysicalError::Refinement { rule: "body-refinement", .. }`, and current `program::lower` maps the failed region lowering to `ProgramError::Structure { rule: "schedule-verification" }`. Thus no `VerifiedKernel`, `KernelProgram`, artifact, Metal program, cache publication, or dispatch request is produced. The separate indirect-KIR/backend work, not this surface ticket, owns any later executable path.

## Identity, schema, registry, and cache consequences

- Index access: fresh tag `0x03`; frame source tensor ordinal, index tensor ordinal, axis, domain, source coordinates, and index coordinates in exactly that order. Ordinals and axis are big-endian U32; each vector has a big-endian U64 count followed by its big-endian U32 members, matching `push_len` and `encode_u32s`. Existing direct read/write encodings remain byte-for-byte unchanged. `INDEX_REGION_IDENTITY_DOMAIN` remains version 11 because a fresh injective tag adds a value without reinterpreting an old byte.
- Bounds resolution is deterministic from the framed access and exact concrete shapes and is not caller-selected. `tiler.gather-index-bounds-proof.v1\0` frames proof-kind tag (`0x01` empty result, `0x02` U32 universe), the existing fact-source tag **once**, region identity, big-endian verified access/source/index ordinals, canonical source/index types, concrete source/index/result shapes, axis, concrete gathered `Extent`, and the count plus big-endian ordinals of the exact ordered literal domain. `tiler.gather-index-validation-requirement.v1\0` frames the same bindings without proof kind or fact source. `tiler.invocation-gather-index-validation-requirement.v1\0` frames the access requirement identity, refinement-subject identity, source `OperandBinding`, index `OperandBinding`, result-binding count and the one `ResultBinding`. Each variable component is length-framed and every integer big-endian. No sourced boundary spelling, snapshot bytes, observed index values, or invented domain identity enter these domains. A coordinate's existing region encoding already carries its symbolic expression and environment identity; the proof's single fact-source tag records that participation without duplicating those bytes.
- Realization law: fresh `0x0E` followed by the axis `AttributeFieldId`'s canonical big-endian U32, standard row count 16 to 17, new gather row revision 1. Existing row bytes remain exact; the frozen realization-registry identity moves.
- Schedule: fresh relation `0x0C` followed by framed source shape, result shape, big-endian axis, selected association spelling, and index shape; the source-side spelling includes the big-endian local index-access ordinal and the fieldless spelling does not. Proof tag `0x03` follows with the same relation fields and framed `GatherIndexBoundsProofIdentity`; it does not duplicate proof kind or fact source. Per the explicit ADR amendment, neither schedule row encodes the compiler-private declared-input ordinal. Old access/proof bytes remain exact and the schedule identity domain does not step. Only newly schedulable static gather identities are new.
- Compiler request and whole-program identity: fresh output subtag `gather-f32.v1`, compiler access-relation tag `0x06`, and the complete subject fields above, including both declared ordinals. The governed lowering row is revision 1 and moves the frozen lowering-registry identity. Request subject v6 already folds lowering and realization registries, so every request identity and explain qualifier containing either moves. The output subtag and fresh relation tag add previously unencodable values, so old subject bytes remain exact and v6 does not step. Source/index declared association, source/index/axis/shape/member/local-association perturbations each move the request bytes. Stage/program identities continue folding the checked semantic binding even though reusable schedule identity does not.
- Semantic registry: the literal carrier leaves `GatherF32` at participation tag `0x01`; the existing semantic-registry row and semantic graph identities do not move merely because a lower-layer representation is admitted. Claiming sourced support is outside this frontier and cannot reuse these bytes.
- Refinement: static gather gains new subject/resolution/authority values. Dynamic gather receives only the explicit requirement outcome and no receipt or coverage identity.
- Artifact/cache: this ticket changes no artifact or manifest schema. A static literal gather stops at KIR lowering and therefore acquires no artifact, manifest, cache-publication, or dispatch identity. A dynamic gather stops earlier at its requirement. The receipt ticket owns any future compatibility fence.

Implementation must pin all old direct and old request-output bytes before and after, gather field-order injectivity, output and relation subtag distinctness, law row, schedule relation/proof, registry/request cascade, request/program movement under declared association changes, schedule stability under the same changes, and absence of static or dynamic artifact/cache construction. Typed `variant_count` censuses must keep `IndexNode` 5, `IndexExprClass` 3, and `IndexDomainUnknownReason` 3; move `IndexRealizationLaw` 13 to 14, standard realization rows 16 to 17, `NormalizedOutput` 5 to 6, and `RegionSpellingKind` 7 to 8; and size the access-view, schedule-relation, proof-kind, refinement-outcome, diagnostic, and normalized-output-subject populations from their enums. Handwritten lengths are not substitutes.

## Complete unsupported population

Every implementation-capable surface refuses signed, other unsigned, and floating indices; negative-index conventions; clamp, wrap, or truncation; inferred or multiple axes; recursive/nested or multiple indirect reads in one access; an index load exposed as scalar SSA; sharing one address-only read across gathers; coalescing a direct U32 read with a gather address read; non-input source or index tensors; **a nonliteral source boundary shape, a nonliteral index boundary shape, or any nonliteral result/access-domain extent**; rank-zero source; data-dependent result shape; scatter and duplicate-write semantics; mutable-device or zero-copy validation; caller assertions; inline-kernel validation; dynamic dispatch receipts; artifact/runtime carriage; and Metal emission. It also refuses caller-selected address relations, proof kinds, schedule associations, and cache participation. Duplicate gather reads remain allowed and deterministic.

Semantic Gather continues refusing any symbolic source, index, or inferred result extent through the existing `SymbolicOperandUnsupported` before inference. The index-layer `gather_read` independently enforces the three exact literal errors above, so direct index-region authoring cannot bypass that semantic boundary. Symbolic coordinate coefficients/divisors/addends remain admitted only through the existing bound `ShapeEnv` rules and may make proof facts `ShapeEnvironment`; they never make a boundary or domain sourced. Reopening sourced Gather boundary/domain support requires a separate accepted decision with its own complete public, diagnostic, reference, schedule, and identity surface. This packet deliberately supplies no sourced equality, concretization, oracle resolution, or fallback rule to inherit.

## Host memory and runtime comparison

The selected access owns two tensor IDs, one axis, and three bounded coordinate vectors (domain, direct source, index), so retained host storage is `O(result_rank + source_rank + index_rank)` and one enum discriminant beyond direct access. Static proof derivation is O(result-rank) to inspect every result extent, then O(total coordinate-expression size) for complete fact provenance and O(1) for the U32-universe comparison; the proof/requirement record retains the ordered domain and three concrete shapes and is O(total boundary rank), not falsely O(1). Dynamic admission scans or copies no index payload. The receipt ticket's research bound remains separate: validating `T <= 8192` U32 elements would inspect O(T) values and at most 32 KiB of index payload.

`RUSTFLAGS='-Zprint-type-sizes' cargo check -p tiler-ir` on the exact base measured current `LogicalAccess` as 208 bytes (alignment 8), `Shape` as 24 bytes, and `SourcedShape` as 32 bytes. Either proposed gather relation is materially below the existing 208-byte maximum, so neither widens `LogicalAccess`; an `AccessOrdinal` is a U32 and changes no enclosing layout here. The source-side spelling does add four canonical identity bytes per gather, while giving every detached relation consumer O(1) access to its address read. The fieldless spelling saves those four bytes but makes the complete access list and canonical-order algorithm mandatory context in request encoding, sizing, physical binding, schedule, kernel, program, and cost consumers. That is a real maintenance/serialized-memory tradeoff, not a presumed layout win.

A nested tensor-reading expression retains the same tensor, proof, and association facts plus recursive edges and compaction state and is dominated after ADR 0108. A schedule-level declared-input ordinal would add four bytes of interface position per gather relation and proof plus a public type/conversion surface while duplicating the already-retained request association; it was eliminated by the accepted coordinate decision rather than treated as free. Implementation still must measure the resulting `AccessData`, reference wrappers, normalized boxes, `Access`, and diagnostics before landing.

## Pareto-complete decision gate

Every survivor is top-tier on correctness and fail-closed strictness. Transition versus deferral and explicit versus contextual local association are the genuine tradeoffs. The invalid sourced candidates are no longer ranked.

| Atomic surface | Capability and public/schema consequence | Maintenance, host runtime, and memory |
|---|---|---|
| **A. Status quo / typed deferral** | Publishes nothing and preserves the current request refusal, all identities, and every unsupported case. ADR 0108 remains accepted architecture but unimplemented; implementation and receipt stay blocked. | Smallest state and no migration, but no gather reaches index IR. |
| **B. Literal-only + source-side reference** (**recommended**) | Adds the exact public surface above and explicitly amends ADR 0108's schedule-level declared-ordinal clause to preserve the later accepted schedule/program layer boundary. Current semantic participation stays literal-only; dynamic values stop in the exact requirement. | Smallest useful vertical. Four extra relation-identity bytes buy O(1) detached local association; no `LogicalAccess` layout widening measured/derived. Sourced boundary/domain operands remain a named refusal; bound symbolic coordinate expressions remain supported. |
| **C. Literal-only + fieldless canonical association** | Same literal support, amendment, and public proof/outcome surface, but `GatherSource`/proof omit `index_access` and identity selects association tag `0x02`. | Saves four identity bytes per gather. Every detached consumer must retain/reconstruct the complete access order, increasing contextual coupling and lookup work. |

None dominates another on all key dimensions. A has least state but no capability. B is more maintainable and faster for detached consumers; C serializes four fewer bytes. Both B and C stop at KIR `body-refinement`, so neither claims kernel performance or executable support.

Eliminated before ranking:

- static-only admission that drops `InvocationValidationRequired` prevents the separate receipt work from binding a subject and falsely narrows an accepted obligation;
- no-static-producer admission rejects empty result domains and U32-universe cases the verifier can prove soundly;
- retaining ADR's declared index-input ordinal in shared schedule IR duplicates compiler binding authority, cannot identify repeated/sparse local reads by itself, leaks request ABI position into reusable schedule identity, and conflicts with the later accepted coordinate decision; normalization-only retention is therefore an explicit ADR amendment, not a silent substitute;
- address-side `index_owner` was re-evaluated and remains dominated: it reverses semantic ownership, still retains an ordinal, and requires an orphan/duplicate cross-list pass; a verified sidecar duplicates association state and adds a consistency owner; the ADR draft's literal input ordinal cannot replace either local association for repeated reads;
- any sourced-boundary/domain variant is outside B/C: nonliteral source, index, or result/access-domain extents hit the exact builder errors before `GatherDomainShape`, and the reference oracle receives only concrete Gather shapes. Reopen only through a separate accepted sourced-Gather decision; this packet invents no equality, specialization, or oracle authority for it;
- generic indirect sums and nested tensor-reading expressions publish unsupported combinations or contradict ADR 0108's selected carrier;
- caller-minted proof, shareable/unassociated address reads, `Unknown` for the mandatory dynamic requirement, registry identity held fixed, default target/backend fallback, and dynamic progress into dispatch can silently accept or alias a program and are invalid rather than tradeoffs;
- further broad research is not a current surface: sourced support has the exact reopening trigger above, while deferral of the literal surface is represented truthfully by A.

### Strongest counterargument and reversal evidence

For A, the strongest counterargument is that it strands an accepted architecture and blocks the pinned gather workload. Evidence reversing A in its favour would be cancellation of that workload or rejection of implementation transition; perturb by showing no downstream ticket or workload requires the access. For B, breadth is the counterargument: it breaks misleading common accessors, publishes gather-specific proof/requirement vocabulary before dynamic execution, and asks Tom to amend one accepted ADR clause. Evidence reversing B would be a measured detached-consumer design whose four bytes matter more than context, or an accepted reversal that puts declared interface position back into shared schedule identity; perturb by removing `index_access` from each detached consumer and by changing declared binding while holding schedule content fixed. For C, the counterargument is contextual coupling. Evidence reversing it would be a total-consumer census showing every relation consumer already has the canonical access list at no extra state/work; perturb by passing one relation alone to encoding, sizing, physical binding, KIR refusal, program lowering, and cost code.

A named sourced-gather workload is not part of this frontier: it fires the separate sourced-Gather reopening decision, without supplying that decision's answer by default. An accepted second indirect family requiring recursive reads would likewise reopen ADR 0108's carrier rather than be guessed into any survivor.

### Required subject perturbations

Implementation and independent review must show the actual failure text after perturbing each subject, not the assertion:

- reuse index-access tag `0x02`, swap source/index fields, or delete the axis frame; injectivity pins must fail independently while old-direct-byte pins remain green;
- mint `StaticallyProved` for source extent `u32::MAX`, change the threshold from `>= 2^32`, or restore the false index-only vacuity rule; prove source `[0, 5]`, axis 1, index `[3]` as the non-gather-zero control and independently fail when result extent inspection or empty-result precedence is removed;
- with every boundary/domain shape literal, independently replace the symbolic source coordinate `S * d0` by a literal coordinate in the empty-result control and the U32-universe control; `facts()` must move from `ShapeEnvironment` to `Program` while each proof kind stays unchanged, and forcing either symbolic-coordinate case to `Program` must fail;
- make the address read a scalar leaf, share it between two gather owners, move it before a scalar read, point `index_access` at the write, or leave it unreferenced; schedule verification must name the violated association;
- change operand order `[source, index]` to `[index, source]`, bind the index to F32, or change the gathered axis; occurrence/refinement verification must fail before the dynamic stop;
- let `InvocationValidationRequired` construct a receipt, `CoveredOccurrence`, schedule, cache key, or dispatch request; the relevant typed census/negative construction check must fail;
- hold either lowering/realization registry identity at its old value after adding the row, or hold the request qualifier fixed; frozen-registry/request pins must fail;
- author a sourced source boundary, sourced index boundary, and sourced domain dimension independently; they must report `GatherSourceShapeNotLiteral { tensor }`, `GatherIndexShapeNotLiteral { tensor }`, and `GatherDomainExtentNotLiteral { dimension }` respectively before `GatherDomainShape`. Change `GatherF32` participation from `LiteralOnly`, let any sourced gather reach `NormalizedGather`, or teach the reference oracle to resolve a Gather boundary from `ShapeEnv`; the literal participation/refusal controls must fail rather than silently widening the carrier;
- change the normalized request's source key, index key, source/index declared ordinal, source/index/result shape, axis, member, association tag, or source-side index-access ordinal one at a time; each request-subject pin must move while every old output-subject pin remains exact;
- change only `source_input` or `index_input` while holding the local schedule relation fixed: request/program identity and binding must move or refuse while reusable schedule identity remains unchanged; change only `index_access` under B and schedule/request relation identity must move;
- replace the KIR `GatherSource` refusal with identity addressing or omit the `RegionSpellingKind::Gather` frontier arm; the independent body-refinement and exhaustive arm-count controls must fail, while a static gather still records exactly one structural proposal, no parallel variants, and its result-derived dispatch/thread/temporary-byte cost;
- widen `IndexNode`, `IndexExprClass`, or unknown reasons incidentally; typed `variant_count` censuses must fail at compile time.

The packet's two repository checks were deliberately driven red and reverted. A broken ADR target made `make citations` print exactly `FAIL  tickets/decide-the-data-dependent-index-representation-public-surface.md`, then `no tracked file or directory at docs/decisions/0108-does-not-exist.md`, and finally `check-citations: 1 markdown link(s) do not resolve against this tree.` Replacing a dependency with `dependency-that-does-not-exist` made `tkt lint --format json` return `ok: false` with code `missing-dep` and message `depends on missing ticket \`dependency-that-does-not-exist\``. The final green runs use neither perturbation.

### Exact-base mechanical evidence

- `RUSTFLAGS='-Zprint-type-sizes' cargo check -p tiler-ir` passed and printed `LogicalAccess: 208 bytes`, `Shape: 24 bytes`, and `SourcedShape: 32 bytes` at alignment 8.
- `cargo test -p tiler-ir semantic::gather::tests --lib` passed 19 tests; `cargo test -p tiler-reference --test gather_conformance` passed all 10 gather cases.
- The first broad compiler filter matched zero tests and is not counted as evidence. Exact tests `request::tests::parametric_broadcast_request_subject_tag_is_injective`, `request::tests::verified_target_receipt_detects_every_governed_subject_mutation_class`, and `request::tests::a_two_declaration_contraction_keeps_its_v1_subject_bytes` each passed, preserving the current relation/output/subject identity controls this proposal must extend.
- Final packet gates are `tkt lint --format json`, `make citations`, `git diff --check`, and exact-base/config-ref `tkt guard`; production Cargo is unchanged, so the repository's ticket-only green-gate carry remains applicable.

## ADR 0108 application and graph

This packet corrects the accepted maturity in the chronological decision catalog, IR contract, Q-SHAPE-007, roadmap, and decision queue. `docs/status.md` contains no stale ADR 0108 maturity claim and needs no edit. Current 5/3/3 tests remain truthful about implemented maturity. Production comments that call ADR 0108 proposed/returned are stale but sit outside this ticket's no-production edit boundary; `admit-the-selected-data-dependent-index-representation` must correct them in the same implementation sweep if Tom selects B or C. That carrier also owns every typed census, tag, byte pin, negative control, layout measurement, selected identity recalculation, exhaustive `RegionSpellingKind`/`output_region_role` arm, and typed KIR refusal above. If Tom selects A, a separate documentation correction must remove the implication that implementation is merely waiting on spelling while preserving ADR acceptance.

The separate receipt ticket remains the sole owner of dynamic validation and runtime/artifact carriage. The implementation ticket remains blocked until Tom accepts this exact included and excluded surface. This ticket stays `in-progress` through independent exact-commit review and is not moved to `awaiting-decision` by its author.

## Recommendation and exact Tom question

**Proposal — recommend B, literal-only plus the source-side reference and the explicit narrow ADR amendment above.** It is the smallest useful vertical aligned with the family current source actually supports, keeps symbolic operands under their existing named refusal, makes detached association O(1), and preserves the mandatory dynamic stop without inventing receipt authority. The four serialized bytes per gather are preferable to making every relation consumer depend on complete-list context. Sourced participation reopens only after the named authority prerequisite, not merely after a workload request.

After independent exact-commit review, ask Tom one question: **Should Tiler keep A the current typed deferral, or accept the literal-only gather surface with the explicit ADR 0108 schedule-ordinal amendment using B source-side `index_access` (recommended) or C fieldless canonical association?**
