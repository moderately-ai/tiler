---
id: decide-the-partitioned-copy-scheduled-region-public-surface
title: Decide the partitioned-copy scheduled-region public surface
status: in-progress
priority: p1
dependencies: [admit-the-concatenate-family-into-the-scheduled-region-vocabulary, accept-the-partitioned-concatenate-realization-law]
related: [admit-the-partitioned-copy-scheduled-region, admit-an-explicit-non-arithmetic-region-and-delivery-state, lower-the-partitioned-copy-region-through-kernel-ir, plan-concatenate-through-one-partitioned-copy-entry]
scopes: [implementation/ir, implementation/compiler, contracts/foundation, contracts/optimizer, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, schedule, concatenate, identity, verification]
claimed_from: todo
assignee: worker-partitioned-copy-review
lease_expires_at: 1787062259
---
## Outcome

A Pareto-complete decision packet fixes the exact public scheduled-region representation for the already-accepted one-region partitioned-copy design, including construction, verification, diagnostics, identity, and the first supported population. It asks Tom one exact question only after an independent derivation confirms that no correctness-bearing API, proof, or identity choice remains implicit.

This ticket prepares the decision; it does not implement a draft surface or treat the accepted semantic topology as acceptance of an unstated Rust API.

## Exact-base Fact audit — 2026-08-17 at `783e9b5b743afafdf4957396dbcfdb2f4c34565c`

Re-read in full: [`admit-the-concatenate-family-into-the-scheduled-region-vocabulary`](admit-the-concatenate-family-into-the-scheduled-region-vocabulary.md), [`accept-the-partitioned-concatenate-realization-law`](accept-the-partitioned-concatenate-realization-law.md), [`repair-the-scheduled-vocabulary-census-and-concatenate-law-standing`](repair-the-scheduled-vocabulary-census-and-concatenate-law-standing.md), and [`admit-an-explicit-non-arithmetic-region-and-delivery-state`](admit-an-explicit-non-arithmetic-region-and-delivery-state.md), plus the current schedule model, builder, diagnostics, request, physical-construction, and identity owners.

1. **Verified — the semantic topology is accepted.** Tom accepted one whole concatenate occurrence as one scheduled `PartitionedCopy` program, one verified KIR, one backend entry, and one dispatch. The accepted law already retains ordered operand members, zero-extent members, repeated occurrences such as `concat(x, x)`, distinct input bindings, and partitioned write-ownership evidence.
2. **False if read as exact public-surface authority — the accepted topology did not select the Rust representation.** The record uses `RegionProgram::Numerical { ... } | PartitionedCopy(...)` conceptually, while the downstream numerical-state decision explicitly says that exact names follow the source audit. No accepted record fixes the public copy program/member/subdomain record names, fields, visibility, exhaustiveness, constructors, accessors, or builder transition.
3. **Verified — current schedule construction is arithmetic-shaped and total.** `IndexRegion`, anchor `pub struct IndexRegion`, carries mandatory `scalar_program: ScalarProgram` and `numerical: NumericalRealization`. `ScheduledRegionBuilder`, anchor `pub struct ScheduledRegionBuilder`, stores both as required `Option` slots; `assemble` refuses either as `IncompleteRegion` and then constructs the one current `IndexRegion` shape.
4. **Verified — current diagnostics do not own partitioned-copy failures.** `ScheduledRegionDiagnostic`, anchor `One deterministic whole-region schedule-verification failure`, has only the current arithmetic/access/proof/topology vocabulary. The implementation ticket requires distinct overlap, gap, overflow, member, prefix, dtype, rank, and correspondence refusals, but no accepted source assigns their exact public variants, payloads, stable `rule()` strings, or precedence.
5. **Verified — current identity has no reserved copy spelling.** Schedule identities open `tiler.schedule.v6\0`; the present `IndexRegion` encoding is the arithmetic-shaped scalar-program-plus-numerical grammar. Neither the accepted topology nor the implementation ticket assigns a copy-program tag, member framing, proof reference encoding, or whether an append-only sum preserves v6 versus requires a coherent domain step.
6. **Verified — the request and physical consumers are arithmetic-only today.** The compiler request subject opens `tiler.compiler.request-subject.v6\0`; request normalization, physical construction, schedule verification, resource derivation, and assembly match the current scalar/numerical form. There is no `RegionProgram` or `PartitionedCopyProgram` type in `crates/` at this base.
7. **Imprecise — “partitioned copy” does not by itself bound the first public population.** The named producer is governed `tiler::concatenate-f32@1` at arities `2..=8`, with static shapes and the accepted partition law. A generic public name could also appear to admit other dtypes, slice/window copies, symbolic partitions, several outputs, or caller-authored partition records. Those populations are not authorized by the accepted concatenate decision and must be explicitly included or excluded.
8. **Verified — implementation cannot derive the exact surface mechanically.** Correct alternatives include different ownership of ordered members and proof subjects, different transactional builder APIs, and different public diagnostic/identity vocabularies. They can all preserve the accepted one-kernel semantic topology while imposing different constructible states, compatibility, and host-memory costs. Choosing among them is a consequential public/identity decision under ADR 0075.

The implementation purpose remains sound. The repair is to split this missing authority into a prerequisite, not to replace the accepted one-region outcome.

## Required decision packet

Apply the repository decision-packet readiness gate at the exact current base. Read every construction, validation, consumption, refusal, identity, schema, and dependency path rather than treating the conceptual enum spelling as settled.

The packet must fix, at minimum:

- the exact public `RegionProgram` sum and the complete copy/member/subdomain/binding records, including visibility, exhaustiveness, constructors, accessors, limits, and transactional builder states;
- the single canonical association between ordered concatenate operands, deduplicated boundary tensors, index-region roots, source/destination subdomains, bounds proofs, and partitioned ownership evidence;
- the initial supported population and every fail-closed exclusion, including dtype, arity, shape source, rank, output count, zero extents, repeated operands, symbolic partitions, slice/window copies, and generic caller-authored copies;
- exact verifier diagnostics, payloads, stable rule strings, precedence, and which malformed states are unrepresentable versus representable-and-refused;
- exact schedule and compiler-request identity tags, framing, old-byte invariants, domain/version consequences, pins, and downstream provenance movement;
- the total consumer migration through request normalization, physical construction, resource derivation, KIR handoff, explanation, and the already-accepted `FloatingPoint | BitPreservingCopy` downstream projection; and
- host runtime and memory bounds for every retained member/proof collection.

Enumerate the genuine frontier: status-quo typed refusal, the narrow governed static-F32 concatenate slice, a broader reusable partitioned-copy surface if independently justified, further bounded research, and typed deferral where applicable. Eliminate any option that lets a caller mint proof authority, infer association from ordering, fabricate numerical state, or silently admit a population the verifier cannot prove.

For every survivor, state the strongest counterargument, reversal evidence, subject perturbations, identity/public consequences, and follow-up graph. Use an independent derivation before presentation because a wrong association or tag can silently misidentify or misroute a program.

## Stop condition

Do not edit production types while this ticket is unresolved. If the source audit exposes another missing proof or downstream authority, file and order that prerequisite rather than embedding it as an implementation detail. Only Tom accepts the exact public surface.

## Closes when

Tom accepts one exact included/excluded public surface, rejects the expansion, or explicitly defers it; the accepted answer is recorded with provenance; the implementation and downstream graph encode every prerequisite; and no API, diagnostic, identity, or population choice remains for the implementer.

## Exact-base Fact audit — 2026-08-18 at `075d2d447b89d8f9b96fe6baa90157334a4359f6`

Audited before writing this packet. The 2026-08-17 audit above was taken at `783e9b5b743afafdf4957396dbcfdb2f4c34565c`; between the two bases `crates/` moved (canonical stage ownership, U32 program-input carrier, precise Metal FP32 requirements, language-model boundary tests), but only three cited files changed, none touching a claim below *(corrected by the 2026-08-18 independent review: the original sentence claimed model.rs was the only changed cited file)*: a doc-comment rewording in `crates/tiler-ir/src/schedule/model.rs` (`what its crate-private KIR classifier derives`); two kernel-domain rows in `crates/tiler-ir/src/domains.rs` (`tiler.kernel-program.stage.v3`, `tiler.kernel-program.v12`) that leave the `tiler.schedule.v6\0` row unmoved; and split-continuation assembly machinery in `crates/tiler-compiler/src/program.rs` (`split_continuation_occurrence`) away from the publishing-copy sites. Every Fact was still re-read from source at this branch's exact base rather than carried forward.

1. **Verified — the semantic topology is accepted.** [`admit-the-concatenate-family-into-the-scheduled-region-vocabulary`](admit-the-concatenate-family-into-the-scheduled-region-vocabulary.md), anchor `Accepted decision — 2026-08-12`, records Tom accepting one whole concatenate occurrence as one scheduled `PartitionedCopy` program, one verified KIR, one backend entry, and one dispatch, retaining operand-ordered members, zero-extent members, `concat(x, x)` as two members over one deduplicated boundary, and the accepted partition law. The law itself is accepted at tag 12 under [`accept-the-partitioned-concatenate-realization-law`](accept-the-partitioned-concatenate-realization-law.md), anchor `The new encoding tag is **12**`.
2. **Verified — the accepted topology did not select the Rust representation.** The acceptance uses the conceptual spelling `RegionProgram::Numerical { ... } | PartitionedCopy(...)`; [`admit-an-explicit-non-arithmetic-region-and-delivery-state`](admit-an-explicit-non-arithmetic-region-and-delivery-state.md), anchor `Exact names follow the source audit`, says so explicitly, and its `Current-base correction — 2026-08-17` assigns the exact spelling to this ticket. No accepted record names the copy record fields, visibility, exhaustiveness, constructors, builder transition, diagnostics, or tags.
3. **Verified — current schedule construction is arithmetic-shaped and total.** `pub struct IndexRegion` in `crates/tiler-ir/src/schedule/model.rs` carries mandatory `scalar_program: ScalarProgram` and `numerical: NumericalRealization`. `pub struct ScheduledRegionBuilder` in `crates/tiler-ir/src/schedule/builder.rs` stores both as required `Option` slots, and its `assemble` refuses each absence as `IncompleteRegion` naming `ScheduleComponent::ScalarProgram` or `ScheduleComponent::NumericalRealization` before constructing the one current `IndexRegion` shape.
4. **Verified — current diagnostics do not own partitioned-copy failures.** `ScheduledRegionDiagnostic` in `crates/tiler-ir/src/schedule/error.rs`, anchor `One deterministic whole-region schedule-verification failure`, carries the shared arithmetic/access/proof rules plus four carried rule enums — `CooperativeTileRule`, `SynchronizationRule`, `ContributorCoverageRule`, `BlockedWorkgroupRule` — and no copy vocabulary. No accepted source assigns the required overlap/gap/overflow/member/prefix/dtype/rank/correspondence refusals, payloads, `rule()` strings, or precedence.
5. **Verified — current identity has no reserved copy spelling.** `encode_identity` in `crates/tiler-ir/src/schedule/model.rs` opens `tiler.schedule.v6\0` and writes shape, accesses, proofs, ownership, `push_scalar_program` (tags `0x22`–`0x2A`, anchor `TAG_SCALAR_SQUARED_SUM_EPILOGUE`), `push_numerical`, then the schedule record. The `v6` step, anchor `declared-input ordinal payload from fieldless input roles` in `crates/tiler-ir/src/schedule/builder.rs`, is the AccessOrdinal reconciliation. Logical-access tags run `0x01`–`0x09` (anchor `TAG_LIVE_ROW_MAJOR`); reduction-topology tag `0x36` is reserved elsewhere and does not constrain the program-tag space. No copy tag exists.
6. **Verified — the request and physical consumers are arithmetic-only today.** `canonical_explain_subject_bytes` in `crates/tiler-compiler/src/request.rs` opens `tiler.compiler.request-subject.v6\0` and encodes recognized outputs through `encode_output_subject`, whose arms carry framed sub-tag strings `serial-sum-f32.v3`, `contraction-f32.v1`, `epilogue-f32.v1`, `staged-family.v2`, and the pointwise sub-tags — every one an arithmetic family. `verify_schedule_with_feasibility` in `crates/tiler-compiler/src/physical.rs` compares `verified.region().index.numerical` against the request realization unconditionally, and `region_numerical_requirements` destructures all eight flat floating-point fields of `ResourceRequirements`. `grep -rn "RegionProgram\|PartitionedCopyProgram" crates/ --include="*.rs"` returns nothing at this base.
7. **Imprecise as a name, verified as a population.** The named producer is governed `tiler::concatenate-f32@1` at arities `MIN_CONCATENATE_OPERANDS..=MAX_CONCATENATE_OPERANDS` (2..=8 in `crates/tiler-ir/src/semantic/concatenate.rs`), F32-only, with static operand shapes — the family, anchor `static_operand_shape`, refuses sourced operands before its result-shape helper — and the accepted partition law. A generic public copy name admits nothing by itself; the population section below states every inclusion and exclusion explicitly.
8. **Verified — implementation cannot derive the exact surface mechanically.** The audit below found at least four genuinely open association/representation choices with correct-looking wrong answers: member geometry on the program versus on each access (the access spelling cannot represent `concat(x, x)` under boundary dedup), offsets carried versus derived (carried offsets create representable overlap/gap states), builder transition shape (one `program` setter versus mutually excluded setters), and read-order canonicality (without a first-reference rule one meaning has many identities). Each is fixed below rather than left to the implementer.

Two findings from this audit repair or sharpen the earlier sections rather than restating them:

- **The `ResourceRequirements` numerical projection cannot stay entirely downstream.** `derive_requirements` in `crates/tiler-ir/src/schedule/model.rs` copies eight numerical fields from `region.index.numerical` inside `ScheduledRegionBuilder::build`, so a copy region cannot be *built* while those fields are mandatory. The accepted downstream record assigns the resource carrier to [`admit-an-explicit-non-arithmetic-region-and-delivery-state`](admit-an-explicit-non-arithmetic-region-and-delivery-state.md); the typed numerical-requirement sum on `ResourceRequirements` must instead land with [`admit-the-partitioned-copy-scheduled-region`](admit-the-partitioned-copy-scheduled-region.md), while KIR, artifact, and delivery arms stay downstream. This is scheduling metadata for authorized work, recorded here because this packet is what the implementer reads; it changes no accepted semantics.
- **With derived offsets, three of the implementation ticket's named perturbations move layers.** [`admit-the-partitioned-copy-scheduled-region`](admit-the-partitioned-copy-scheduled-region.md) requires overlap, gap, and wrong-prefix perturbations to fail by distinct typed rules. Under the recommended surface those states are unrepresentable in the public program — which the packet checklist itself prefers — so the distinct refusals live at the compiler projection that re-derives the accepted index law (displaced roots, reordered operands), not in the intrinsic verifier. The intrinsic verifier's representable-and-refused set is enumerated below.

## Decision readiness result

**Inference — one option dominates; no genuine product fork survives.** The status quo contradicts an accepted decision, every optional/defaulted/fabricated-numerical spelling was already eliminated by the accepted `Numerical | PartitionedCopy` model, per-access geometry cannot represent the accepted `concat(x, x)` population, and a parallel region type duplicates every builder/verifier/identity path for no capability. The one axis with a real alternative — derived versus carried member offsets — is settled by the repository's own rule that unrepresentable beats representable-and-refused, with the carried spelling retained as the named reversal. Under the gate, when one option dominates the packet recommends it rather than manufacturing a choice; what remains for Tom is acceptance of the exact included/excluded surface, not a selection among peers.

**Recommendation — the field-replacing exhaustive `RegionProgram` sum with program-owned member geometry and derived prefix offsets, exactly as specified below.**

**Draft Tom question — deliberately unqueued pending review.** Accept the exact `RegionProgram::{Numerical, PartitionedCopy}` surface below — program-owned members with derived checked prefix offsets, the fieldless `PartitionedCopySource` access map, the closed one-variant `CopyElement`, the eleven `partitioned-copy-*` rules, appended tags `0x2B` and `0x0A` preserving `tiler.schedule.v6`, and the governed static-F32 concatenate population with every listed exclusion — or name the exclusion or spelling to change. **Recommendation: accept as specified.**

This packet does not move status, queue the question, or authorize implementation.

## Recommended public surface

### The exact sum and records

`tiler_ir::schedule` replaces `IndexRegion`'s two fields `scalar_program` and `numerical` with one required field, and the region model gains three types:

```rust
pub struct IndexRegion {
    pub id: RegionId,
    pub iteration_shape: Shape,
    pub accesses: Vec<Access>,
    pub bounds_proofs: Vec<BoundsProof>,
    pub ownership_proof: OwnershipProof,
    pub program: RegionProgram,
}

pub enum RegionProgram {
    Numerical {
        scalar: ScalarProgram,
        numerical: NumericalRealization,
    },
    PartitionedCopy(PartitionedCopyProgram),
}

pub struct PartitionedCopyProgram {
    pub element: CopyElement,
    pub axis: Axis,
    pub members: Vec<CopyMember>,
}

pub struct CopyMember {
    pub source: AccessOrdinal,
    pub extent: u64,
}

pub enum CopyElement {
    F32,
}
```

- **Exhaustiveness.** All three enums are deliberately **not** `#[non_exhaustive]`, under ADR 0074 convention 5b and for the same reason `ScalarProgram` states at its own definition: `tiler-compiler`'s `physical.rs` and `frontier.rs` map the program totally from outside the crate, so a third computation class or a second element format must stop those builds rather than reach a wildcard that answers for a program it was never checked against.
- **Visibility.** All fields are `pub` value data, following the schedule module's stated leaf-descriptor convention (anchor `Why the leaf descriptors expose fields` in `crates/tiler-ir/src/schedule/mod.rs`): no field-level invariant is maintained between calls, every cross-field invariant is proven by `ScheduledRegionBuilder::build`, and struct-literal construction makes an added field a compile error at every producer. There is no unchecked path to a `VerifiedScheduledRegion`.
- **Constructors and accessors.** No `new`-with-validation constructors; the verifier is the validation. Derived accessors only: `RegionProgram::numerical() -> Option<&NumericalRealization>` (the total replacement for every `region.index.numerical` read), `PartitionedCopyProgram::member_offsets() -> Option<Vec<u64>>` returning the checked exclusive prefix sums of `members[..].extent` and `None` on `u64` overflow, `PartitionedCopyProgram::member_source_shape(&Shape, usize) -> Option<Shape>` returning the iteration shape with the copy axis's extent replaced by the member's, `CopyElement::tag() -> u8` (`F32 => 0x01`, exhaustive match per ADR 0074 convention 3), and `CopyElement::storage_bytes() -> u64` (`F32 => 4`, the derivation KIR's load/store width reads). Offsets, source shapes, and destination rectangles are **never stored**: a field a producer could set beside its derivation is a second spelling two regions could disagree in, which is the rule the cooperative tile's underived visibility edges already state.
- **Limits.** New `pub const MAX_PARTITIONED_COPY_MEMBERS: usize = 4_096` beside `MAX_SCHEDULE_ACCESSES`, an enumeration bound in the sense the cooperative bounds are: the verifier walks members once, and the bound keeps that walk and the member frame finite. It is not a population claim; the reachable population stays 2..=8 at the compiler boundary.

### Transactional builder transition

`ScheduledRegionBuilder` replaces its `scalar_program` and `numerical` setters and `Option` slots with one `pub fn program(&mut self, program: RegionProgram) -> Result<(), ScheduleBuildError>` single-assignment slot. `ScheduleComponent::{ScalarProgram, NumericalRealization}` are replaced by `ScheduleComponent::RegionProgram`; `assemble` refuses its absence as `IncompleteRegion` under the unchanged `incomplete-region` rule string; `from_region` maps the field. Consequences this buys: a region carrying a copy program plus a numerical realization, an arithmetic program without one, and an unclassified empty state are unrepresentable in the builder as well as in the region — there is no mutual-exclusion diagnostic because there is no expressible mixed state. The retained-builder recovery contract of `ScheduledRegionBuildError::into_parts` is unchanged.

### The single canonical association

One statement, from which every other relationship is derived rather than restated:

- **Ordered operands ↔ members.** `members[k]` is concatenate operand `k`. Member order is semantic identity; members are never deduplicated, so `concat(x, x)` is two members.
- **Deduplicated boundaries ↔ read accesses.** The region carries one read access per *distinct* source boundary plus the one owning write, and `CopyMember::source` names a read by `AccessOrdinal`, so `concat(x, x)` is one read referenced by two members. Distinctness of boundaries is not expressible inside `tiler-ir` (a schedule access carries no binding) and is therefore the compiler projection's obligation, with the typed correspondence refusal below; what the intrinsic verifier owns is that every ordinal resolves to a read and every read is referenced.
- **Canonical read order.** Reads appear in first-reference order of the member list: the sequence of first occurrences of `members[..].source` must be exactly the dense ascending run `0, 1, 2, ...`. This is the canonicality rule that gives one meaning one identity — without it, permuting the read list and renumbering ordinals spells one program twice.
- **Index-region roots ↔ members.** Root `k` of the accepted law's emitted region (`emit_partitioned_concatenate` in `crates/tiler-ir/src/index/law.rs`, one root per operand with a `linear_combination` offset displacement on the concatenated axis) projects to member `k`. The projection re-derives extents and offsets from the accepted law rather than copying them, and refuses disagreement by typed rule.
- **Subdomains.** Member `k`'s destination subdomain is the iteration shape with the copy axis restricted to `[offset_k, offset_k + extent_k)`, offsets from `member_offsets`; its source subdomain is the whole source, whose shape is `member_source_shape`. Neither is stored.
- **Bounds proofs.** One per access exactly as today, through the unchanged `verify_proof_records`: read `r` carries `BoundsProofKind::LinearRange { element_count }` equal to its derived source element count, and the write carries `LinearRange` equal to `owned_output_positions`, which for the copy's mandatory `ReductionTopology::None` is the work-item count. `bounds_proof_refines_access` gains one arm pairing `LinearRange` with the new read map; the arm is structural (pairing only), because the function's signature carries no access ordinal and the fieldless map cannot say which member-derived count applies — the exact element-count agreement is owned by the `partitioned-copy-source-shape` rule.
- **Ownership evidence.** The write references the region's one `OwnershipProof` with the existing `OwnershipProofKind::OneGlobalInvocationPerOutput { output_count }`, which is literally true of the copy: the iteration domain is the output domain and each invocation performs one guarded store. The partition theorem — the member intervals are pairwise disjoint and jointly exhaustive over the axis extent — is a total function of the members once offsets are derived prefix sums with a checked total equal to the axis extent, so it is re-derived by the verifier and never encoded or caller-supplied; the accepted index law's `WriteOwnershipProofView::PartitionMember` joint evidence is consumed by the projection, not carried into schedule identity. A caller cannot mint proof authority because no proof field exists for a caller to fill.
- **Access maps.** Every copy read carries the new fieldless `LogicalAccess::PartitionedCopySource`; the write carries `LogicalAccess::LinearIdentity`. `PartitionedCopySource` is refused by name in every other program family's admission (the same named-refusal pattern `pointwise_read_map_is_admissible` uses), so the map cannot leak into arithmetic regions.

### Schedule record

A copy region requires `ExecutionBinding::GlobalLinearInvocation`, `TailPolicy::Exact`, `ReductionTopology::None`, `work_items` equal to the iteration-domain element count, and the existing exact launch equalities — all already-encodable spellings, so the schedule half of the encoding is untouched. Zero-extent members stay in identity and execute no access; an all-zero output domain takes the existing `zero_work_skips_dispatch` path.

## Initial supported population and fail-closed exclusions

**Included at the vocabulary level** (intrinsic verifier): F32 element; one concatenated axis in range; ordered members, `2..=MAX_PARTITIONED_COPY_MEMBERS`, each naming a read by ordinal with a `u64` extent; zero-extent members; repeated source ordinals; static shapes (the schedule `Shape` is static by construction); one owning write to a program `Output`; reads from declared `Input` boundaries only; the fixed schedule record above.

**Reachable population at the compiler boundary**: exactly governed `tiler::concatenate-f32@1` occurrences — arity 2..=8, static shapes, the accepted partition law, distinct-input boundary dedup, one occurrence per region, the occurrence's result a declared program output. Nothing else constructs a copy region: the only path from a request to one is the concatenate projection.

**Fail-closed exclusions**, each with its refusing boundary:

| Excluded | Where it fails closed |
| --- | --- |
| other dtypes (bf16, f16, f64, integer copies) | unrepresentable: `CopyElement` has one variant; widening is a deliberate variant + tag + build errors at every encoder and total match |
| slice/window copies, partial-source members | unrepresentable: a member's source subdomain is always its whole source; windowing has no field, and `tiler::slice-f32@1` remains its own separately walled family |
| symbolic/sourced partitions | unrepresentable at schedule (`u64` extents, static `Shape`); refused earlier at the family boundary (`static_operand_shape`) |
| caller-authored generic copies | no public request route reaches a copy region except the concatenate projection; a hand-built region must still pass the intrinsic verifier, which proves geometry but cannot grant a semantic occurrence — subject binding refuses it |
| single-member copies | `partitioned-copy-member-count`; also the family's own `MIN_CONCATENATE_OPERANDS` rationale — a one-member copy would be a second spelling of the publishing copy, which deliberately remains the arithmetic-classified identity-expression region `verify_publishing_copy_binding` checks |
| several outputs | the unchanged single-owning-write region shape |
| writes to intermediates (copy feeding a fused consumer) | `partitioned-copy-write-tensor`; reversal is a future cover decision, not a default |
| reads from intermediates (fused producer feeding the copy) | `partitioned-copy-read-tensor`; same reversal boundary |
| overlap, gaps, wrong prefix offsets | unrepresentable: offsets are derived prefix sums; the only expressible coverage defect is a wrong extent sum, refused by `partitioned-copy-coverage-sum` |
| multi-kernel or N-entry realizations, pointwise-identity substitution, host materialization | out of this vocabulary entirely; owned by the accepted one-kernel decision and the KIR/plan tickets' explicit-alternative rule |

## Exact verifier diagnostics, rule strings, and precedence

`ScheduledRegionDiagnostic` gains one carried-rule variant, following the four existing ones:

```rust
PartitionedCopy {
    rule: PartitionedCopyRule,
},
```

`PartitionedCopyRule` is `#[non_exhaustive]` like its siblings, with these variants and stable `rule()` strings:

| Variant | Rule string | Refuses |
| --- | --- | --- |
| `Topology` | `partitioned-copy-topology` | a copy region whose reduction is not `None`, binding not `GlobalLinearInvocation`, or tail not `Exact` |
| `ReadTensor` | `partitioned-copy-read-tensor` | a read whose boundary is not `TensorRole::Input`, or carrying a component role |
| `WriteTensor` | `partitioned-copy-write-tensor` | an owning write whose boundary is not `TensorRole::Output`, or carrying a component role |
| `MemberCount` | `partitioned-copy-member-count` | fewer than two members or more than `MAX_PARTITIONED_COPY_MEMBERS` |
| `AxisRange` | `partitioned-copy-axis-range` | a copy axis position at or beyond the iteration rank |
| `SourceReference` | `partitioned-copy-source-reference` | a member ordinal that resolves to no access, to the write, or to a read not carrying `PartitionedCopySource` |
| `SourceOrder` | `partitioned-copy-source-order` | first references of member sources not forming the dense ascending run over the reads |
| `UnreferencedSource` | `partitioned-copy-unreferenced-source` | a read no member references |
| `ExtentOverflow` | `partitioned-copy-extent-overflow` | a prefix sum or derived source element count overflowing `u64` |
| `CoverageSum` | `partitioned-copy-coverage-sum` | member extents whose checked sum is not the axis extent — the one representable coverage defect, covering both would-be gap and would-be overlap |
| `SourceShape` | `partitioned-copy-source-shape` | members referencing one read with disagreeing extents, or a read's bounds-proof element count disagreeing with the derived source element count |

Payloads are deliberately none beyond the rule (matching `CooperativeTileRule`'s convention); the recoverable builder plus the rule name locates the defect, and payload-bearing variants can be added additively later because the enum is `#[non_exhaustive]`.

**Precedence** is the deterministic first-failure order of one dispatch arm in `verify_intrinsic`, mirroring the existing gates: shared `IncompleteRegion` (assemble) → `ShapeProductOverflow` → `LaunchCoverage` → `Topology` → shared `AccessCount` (no read, or no trailing write) → shared `AccessContract` (modes, maps, ownership placement, `output_owner` agreement) → `ReadTensor` → `WriteTensor` → shared `BoundsProofCount`/`ProofReference`/`BoundsProof` (one call to the unchanged `verify_proof_records`; the new `LinearRange`/`PartitionedCopySource` refinement arm is *structural* — the fieldless map cannot name the member-derived source element count from inside `bounds_proof_refines_access`, whose signature carries no access ordinal, so the arm admits the pairing and the exact element-count agreement is `partitioned-copy-source-shape`'s below) → `MemberCount` → `AxisRange` → `SourceReference` → `SourceOrder` → `ExtentOverflow` → `CoverageSum` → `SourceShape` → `UnreferencedSource`. *(Corrected by the 2026-08-18 independent review: the original order listed a trailing shared `BoundsProof` refinement step after `UnreferencedSource`, which one unchanged `verify_proof_records` call cannot realize — that function performs count, reference, and refinement checks together.)*

**Unrepresentable versus representable-and-refused.** Unrepresentable: mixed program states, a copy with a numerical realization, an arithmetic region without one, non-F32 elements, stored offsets/shapes/rectangles disagreeing with their derivation, caller-minted ownership evidence, member overlap/gap/wrong-prefix. Representable-and-refused: exactly the eleven rules above plus the shared rules. There is no `partitioned-copy-element` rule, under the stated unreachable-refusal convention (anchor `can a subject reach this rule by any route` in `crates/tiler-ir/src/index/law.rs`): `CopyElement` is closed, so no constructible region presents another element and a rule for it could never be watched failing.

**Projection correspondence refusals** (compiler-owned, named here so no choice remains): the projection from the accepted `PartitionedConcatenate` index region re-derives operand order, per-root extents, and offset displacements, and refuses disagreement by distinct typed rules `partitioned-copy-projection-operand-order`, `partitioned-copy-projection-offset`, `partitioned-copy-projection-extent`, and `partitioned-copy-projection-boundary-dedup`, in the compiler's existing typed-refusal vocabulary. This is where the implementation ticket's overlap, gap, and wrong-prefix perturbations fail — by displacing the emitted roots, exactly the two watched-failing offset displacements the accepted law ticket already carries — because the schedule-level states are unrepresentable.

## Identity: exact tags, framing, old-byte invariants, and pins

### Schedule identity

- Domain: **`tiler.schedule.v6\0` does not step.**
- New program-position tag `TAG_REGION_PARTITIONED_COPY: u8 = 0x2B`, appended after `TAG_SCALAR_SQUARED_SUM_EPILOGUE` (`0x2A`). The `Numerical` arm writes byte-for-byte what `push_scalar_program` followed by `push_numerical` writes today.
- Copy-arm payload, in order: the tag `0x2B`; one element tag byte via `CopyElement::tag()` (`0x01`); the axis as 4 big-endian bytes (`Axis::get`); one framed member run — `push_len` count, then exactly count fixed-width 12-byte records of 4-byte source ordinal plus 8-byte extent, both big-endian. No numerical record follows; the schedule record follows directly.
- New logical-access tag `TAG_PARTITIONED_COPY_SOURCE: u8 = 0x0A`, appended after `TAG_LIVE_ROW_MAJOR` (`0x09`), with no payload.

**Independent injectivity derivation** (derived from the encoder grammar, not from the design intent):

1. *Cross-arm discrimination.* The first program byte separates `0x22`–`0x2A` from `0x2B`, and `0x01`–`0x09` from `0x0A` at the access position. No previously encodable region can carry either new tag, so no old identity can equal a new one and no new reader reinterprets old bytes — the same argument every appended tag from `TAG_REDUCTION_MULTI_PASS` onward records.
2. *Within-arm recoverability.* Element tag and axis are fixed-width at fixed positions; the member run is length-framed with fixed-width records, so every source ordinal and extent is recoverable at a frame-determined position. Two copy programs differing in element, axis, member count, any member's source, or any member's extent differ in these bytes.
3. *Equal meanings encode equally.* Offsets, source shapes, and destination rectangles are derived and never written; the `SourceOrder` rule pins one read order per meaning; member order is itself semantic. So one program meaning has exactly one encoding, and the derived quantities cannot make two equal programs differ.

**Old-byte invariants and pins.** Every previously encodable region is byte-identical, and the controls already exist and passed at this base: `the_strict_f32_region_has_its_recorded_canonical_identity` and `existing_one_committer_schedule_encodings_keep_their_bytes` in `crates/tiler-ir/src/schedule/builder.rs` must stay green **unmodified** through the implementation. The `tiler.schedule.v6\0` row in `crates/tiler-ir/src/domains.rs` does not move; the implementation introduces no new NUL-terminated domain literal, so both crates' domain censuses stay untouched — and would fail loudly if that claim were wrong, which is the census working. Implementation adds one new golden: a pinned canonical identity for an arity-2 copy fixture, plus a `CopyElement` tag-injectivity assertion in the model's existing injectivity test family.

### Compiler-request identity

- Domain: **`tiler.compiler.request-subject.v6\0` does not step.** The recognized-output arm vocabulary gains one appended arm whose framed sub-tag string is exactly `partitioned-copy-f32.v1`, alongside `serial-sum-f32.v3`, `contraction-f32.v1`, `epilogue-f32.v1`, and `staged-family.v2`. Existing subjects are byte-identical because arms are self-framing and the new string is distinct; a subject containing a concatenate occurrence previously did not exist (the family refused under `operation-set` before a subject was built), so no old subject collides.
- The arm's payload (input keys, output key, axis, ordered member extents, occurrence attribution) is fixed by the plan-integration ticket inside this framed arm; whatever it writes is separated from every other arm by the sub-tag before any payload byte is read.
- Downstream provenance: the law sidecar and lowering-registry identities already moved when the accepted law and seven capabilities landed (`7bba54bcb59ec2cc` → `0aa252e0bfa16451`, recorded in [`accept-the-partitioned-concatenate-realization-law`](accept-the-partitioned-concatenate-realization-law.md)); this decision moves no registry identity, no pinned qualifier for existing fixtures, and no semantic snapshot.

### Downstream domains

This surface assigns **no** KIR, artifact, cache, or delivery tags. `tiler.kernel-program.v12`, the artifact schema, and the delivered-realization grammar are owned by the accepted [`admit-an-explicit-non-arithmetic-region-and-delivery-state`](admit-an-explicit-non-arithmetic-region-and-delivery-state.md) projection (`FloatingPoint | BitPreservingCopy`) and [`lower-the-partitioned-copy-region-through-kernel-ir`](lower-the-partitioned-copy-region-through-kernel-ir.md), which must re-derive their own append-only-versus-step answers against their own encoders exactly as the accepted record requires.

### `ResourceRequirements`

Landed with the schedule ticket because `derive_requirements` runs inside `build` and must be total: the eight flat floating-point fields become one required field `numerical: RegionNumericalRequirements`, an exhaustive (5b) sum `FloatingPoint { input_subnormals, result_subnormals, contraction, reassociation, permutation, signed_zero, nan_assumptions, infinity_assumptions } | BitPreservingCopy`. The structural fields — `buffer_bindings`, `threads_per_workgroup`, `local_memory_bytes`, `requires_device_memory`, `index_arithmetic`, `synchronization`, `subgroup` — stay unconditional; a copy still computes coordinates, so `index_arithmetic` remains non-optional exactly as its field comment argues. `region_numerical_requirements` in `crates/tiler-compiler/src/physical.rs` matches the sum: the `FloatingPoint` arm produces today's projection byte-for-byte; the `BitPreservingCopy` arm produces no floating-point numerical requirement, which [`derive-target-numerical-feasibility-from-reached-arithmetic-only`](derive-target-numerical-feasibility-from-reached-arithmetic-only.md) owns consuming as proved absence rather than target silence. `VerifiedScheduledRegion::subnormal_freedom` answers `SubnormalFreedom::Unproven` for the copy arm — the fail-closed answer; a copy-specific freedom claim would need its own KIR-level bit-preservation evidence and belongs to the feasibility ticket if ever needed.

*(Added by the 2026-08-18 independent review.)* The sum's reach is wider than this crate: `ResourceRequirements` is written into the kernel identity by `push_requirements` in `crates/tiler-ir/src/kernel/model.rs` and into the artifact wire format by `push_resources`/`parse_entry` in `crates/tiler-artifact/src/program/`, so landing the sum with the schedule ticket forces same-change edits at those encoders and the decoder. That moves no tag and steps no domain — the `FloatingPoint` arm reproduces today's bytes exactly, and the copy arm is a typed refusal unreachable until the downstream carriers land — but the byte-preservation obligation at each site is stated in the migration census above rather than left to be discovered as a build error.

## Total consumer migration

Every current consumer of the two replaced fields, from `grep -rn "index\.scalar_program\|index\.numerical"` over `crates/` at this base, plus the total-map sites that must state a copy answer rather than inherit one:

| Site | Change |
| --- | --- |
| `crates/tiler-ir/src/schedule/model.rs` | `IndexRegion.program`; copy arms in `encode_identity`, `derive_requirements` (sum), `region_arithmetic_type` (copy → `ArithmeticType::F32` from the element), `subnormal_freedom_of` (copy → `Unproven`); `ResourceRequirements` sum; new tags and types |
| `crates/tiler-ir/src/schedule/builder.rs` | builder slot/setter, `assemble`, the `verify_intrinsic` dispatch arm and copy gate, the `bounds_proof_refines_access` arm, named refusal of the new map in every arithmetic admission, tests |
| `crates/tiler-ir/src/schedule/error.rs` | `ScheduleComponent::RegionProgram`, `PartitionedCopyRule`, the diagnostic variant and `rule()` arm |
| `crates/tiler-ir/src/schedule/mod.rs` | exports and the doc example's construction |
| `crates/tiler-ir/src/schedule/witness.rs` (+ tests) | `RealizationWitness` over the sum; a copy has no fold or freedom site, and the witness must say so explicitly rather than default |
| `crates/tiler-ir/src/kernel/{builder,lower,verify,tests}.rs` | construction sites only at this stage; canonical lowering keeps receiving `Numerical` regions and refuses the copy arm by typed error until [`lower-the-partitioned-copy-region-through-kernel-ir`](lower-the-partitioned-copy-region-through-kernel-ir.md) and the downstream numerics carrier land — silence is not an option because the dispatch match is total |
| `crates/tiler-compiler/src/physical.rs` | every literal `IndexRegion` construction gains the `Numerical` arm; `verify_schedule_with_feasibility` compares the request realization against the `Numerical` arm and, for the copy arm, requires no region-carried realization while the request keeps its complete policy subject; `region_numerical_requirements` sum |
| `crates/tiler-compiler/src/frontier.rs` | `boundary_carrier` total over the sum (copy → the F32 carrier from the element) |
| `crates/tiler-compiler/src/program.rs` (+ tests) | pass-assembly reads (publishing-copy detection) become arm-aware; the publishing copy remains a `Numerical` identity-expression region |
| `crates/tiler-compiler/src/request.rs` | the appended `partitioned-copy-f32.v1` output-subject arm and its recognizer, landed by the projection/plan tickets inside this framing |
| `crates/tiler-reference/src/oracle.rs` | **no change at this base** — *corrected by the 2026-08-18 independent review: the original row claimed region construction sites here, but `grep -rn "ScheduledRegionBuilder\|schedule::IndexRegion" crates/tiler-reference/` returns nothing; the file evaluates `tiler_ir::index::VerifiedIndexRegion` and imports only `schedule::ArithmeticType`. Copy conformance stays bit-identical comparison under the plan ticket* |
| `crates/tiler-conformance/src/bf16_vertical/tests.rs` | construction-site updates |
| `crates/tiler-ir/src/kernel/model.rs` | *(added by the 2026-08-18 independent review)* `push_requirements` writes the eight flat numerical fields of `ResourceRequirements` into the **kernel identity**; with the sum it destructures the `FloatingPoint` arm byte-for-byte, and the `BitPreservingCopy` arm is stated as a typed refusal (unreachable while canonical lowering refuses the copy region upstream) rather than silent — `tiler.kernel-program.v12` bytes for every existing kernel must be proved unmoved by the existing pinned-kernel controls |
| `crates/tiler-artifact/src/program/model.rs` and `crates/tiler-artifact/src/program/codec/{decode,model}.rs` (+ tests) | *(added by the 2026-08-18 independent review)* the artifact wire codec serializes and parses `ResourceRequirements` directly — `push_resources` destructures all fifteen fields and `parse_entry` constructs the record from ten parsed numerical fields — so the schedule ticket's sum forces same-change edits here: the `FloatingPoint` arm stays byte-identical under schema `16.0`, and the `BitPreservingCopy` arm refuses by typed error until [`admit-an-explicit-non-arithmetic-region-and-delivery-state`](admit-an-explicit-non-arithmetic-region-and-delivery-state.md) lands its tagged entry row and schema step |
| `crates/tiler-compiler/src/frontier.rs` `subprogram_resources` | *(added by the 2026-08-18 independent review)* today it inherits stage zero's eight numerical fields, justified because every admitted stage implements one request contract; over the sum it must state its policy explicitly — refuse a subprogram whose stages carry disagreeing numerical arms rather than inherit stage zero's silently, the same refuse-on-disagreement shape its `index_arithmetic` and `synchronization` merges already use |
| builder-setter and literal call sites outside the field-read census | *(added by the 2026-08-18 independent review)* replacing the `scalar_program`/`numerical` setters and the `IndexRegion` fields also migrates every call site the field-read grep cannot see — enumerable with `grep -rn "\.scalar_program(" crates/` plus the schedule-`IndexRegion` literals: production sites in `crates/tiler-conformance/src/{loop_carried,bf16_vertical}.rs`; test and fixture sites in `crates/tiler-metal/src/tests.rs`, `crates/tiler-runtime/tests/adapter_route/fixture.rs`, `crates/tiler-ir/src/program/tests.rs`, `crates/tiler-compiler/tests/{live_contraction_consumes_s,bf16_numerical_contract,materialized_intermediate_epilogue_wall}.rs`, `crates/tiler-compiler/src/pipeline/tests.rs`, `crates/tiler-build/{src/metal_assembly.rs,tests/custom_backend/main.rs}`, `crates/tiler-artifact/src/program/tests.rs`, and the `ResourceRequirements` literal helpers in `crates/tiler-compiler/src/{call_declaration,call_registry,selection,pipeline/trace}.rs`; doc examples in `crates/tiler-ir/src/{kernel,program}/mod.rs`, `crates/tiler-metal/src/lib.rs`, and `crates/tiler-artifact/src/{proof,program}/mod.rs`. All mechanical — the sum makes every one a compile error — but the census is stated so the migration's true extent is a read fact rather than a build-error discovery |

The `UNPLANNED_OPERATIONS` entry for `tiler::concatenate-f32@1` in `crates/tiler-compiler/src/policy.rs` is retired **with its stated reason superseded** by the plan ticket, not by this surface: the entry's two claims have to be separated, because the family stays numerically rowless (it performs no arithmetic) while ceasing to be unplannable, and the policy census tests pin exactly that separation.

## Host runtime and memory bounds

- `PartitionedCopyProgram`: one enum tag, one `Axis`, and a `Vec` of 12-byte members — ≤ 96 payload bytes for the whole governed 2..=8 population, ≤ 48 KiB at the structural bound.
- Verification: O(accesses + members) time, O(reads) auxiliary memory for the reference bitmap, single checked-arithmetic pass for offsets; no enumeration over the output domain.
- `member_offsets`: one O(members) allocation per caller; nothing retains it.
- Identity: 14 fixed bytes plus 12 per member plus one byte per read access, on top of the unchanged region framing.
- Existing regions: the sum adds one discriminant to `IndexRegion` (≤ 8 bytes with alignment); no retained collection, cache, or artifact grows.

## Subject perturbations and negative controls

**Current-state controls, run at this exact base on the clean tree — all passed** (commands in Packet verification below): the law emits one root per operand over one output and partitions by operand rather than by input (`the_concatenate_law_realizes_one_root_per_operand_over_one_output`, `the_concatenate_law_partitions_by_operand_rather_than_by_input`); both schedule identity pins hold; both domain censuses hold; the governed registry holds one capability per admitted arity and refines at every one; and every unplanned operation is registered and rowless. These are the baselines the implementation must keep green unmodified.

**Implementation controls, each perturbing the subject and required to show its failure text:**

- change one member's extent by one → `partitioned-copy-coverage-sum`, while the same region with the compensating axis extent passes;
- drop one member of an arity-2 fixture → `partitioned-copy-member-count`; drop one of arity 3 → `partitioned-copy-coverage-sum`;
- point a member at the write access → `partitioned-copy-source-reference`;
- permute the read list of a two-source fixture without renumbering → `partitioned-copy-source-order`;
- leave a read unreferenced → `partitioned-copy-unreferenced-source`;
- give two members of one read different extents → `partitioned-copy-source-shape`;
- set an extent near `u64::MAX` → `partitioned-copy-extent-overflow`;
- put `ReductionTopology::Serial` on a copy region → `partitioned-copy-topology`;
- write to an `Intermediate` → `partitioned-copy-write-tensor`; read from one → `partitioned-copy-read-tensor`;
- reorder two members (a *legal* but different program) → the canonical identity moves and the projection control below refuses the mismatch — this is the overlap/gap/wrong-prefix class made unrepresentable, tested at its real boundary;
- byte-level: an arity-2 copy region's identity and the pinned strict-F32 region's identity first diverge at the program-position byte (`0x2B` versus `0x24`), and both existing pins stay byte-identical;
- projection: displace one emitted root's offset in the accepted law fixture (the law ticket's two watched-failing displacements) → `partitioned-copy-projection-offset`; reorder operands → `partitioned-copy-projection-operand-order`; the failure text is quoted, not asserted-about.

Each rule above is load-bearing separately; a perturbation that reddens several rules at once does not discharge the others.

## Pareto-complete options

| Option | Disposition | Strongest counterargument / reversal evidence |
| --- | --- | --- |
| Status-quo typed refusal / deferral | **Process fallback, not a survivor.** The `operation-set` wall stays correct, but Tom already accepted the one-region topology; deferral leaves an accepted decision stranded with no new evidence. | Appropriate only if review finds a named gap in this packet. |
| **Field-replacing exhaustive `RegionProgram` sum, program-owned geometry, derived offsets (recommended)** | **Dominant.** Mixed and malformed-coverage states unrepresentable; one meaning, one identity; `v6` preserved; smallest retained surface that carries the accepted population. | Costliest single objection: overlap/gap perturbations move to the projection layer rather than failing in the intrinsic verifier. Reverse if implementation shows the projection cannot distinguish those causes with typed rules — no current evidence suggests it cannot, since the law's displacements are already watched failing. |
| Offset-carrying `CopyMember { source, extent, offset }` | **Eliminated as dominated.** It buys intrinsic-level overlap/gap/wrong-prefix refusals at the cost of a redundant identity field, a representable wrong-prefix state, and a verifier rule (`offsets equal prefix sums`) that exists only to refuse what derivation makes impossible. | Reverse only on the projection evidence above. |
| Per-access geometry (`LogicalAccess` variant carrying offset/extent per read) | **Eliminated as unable to represent the accepted population.** Boundary dedup makes one read serve two members at different offsets (`concat(x, x)`), so per-access offsets cannot state the accepted operand-keyed partition without abandoning dedup, which the acceptance fixed. | None; the acceptance record pins both dedup and operand keying. |
| `ScalarProgram::Copy` variant with retained mandatory `NumericalRealization` | **Eliminated by the accepted decision.** A fabricated or "neutral" realization asks target-arithmetic questions of bit transport and mints an identity for arithmetic no kernel performs. | Reopening requires superseding the accepted 2026-08-12 record. |
| `Option<NumericalRealization>` / applicability flag / empty-profile sentinel | **Eliminated by the accepted decision** — conflates missing with proved inapplicability. | Same. |
| Parallel `CopyRegion` type beside `IndexRegion` | **Eliminated.** Two region kinds at every consumer without a sum's exhaustiveness; duplicates builder, verifier, identity, and requirements plumbing; invites drift between them. | Would compete only if the sum forced an unacceptable migration, and the census above shows the migration is bounded and mechanical outside the named total-map sites. |
| Multi-root schedule region / N single-root regions | **Eliminated.** Reopens the accepted one-region law; the schedule model carries exactly one ownership proof and kernel verification rejects multiple stage writers; the earlier acceptance already rejected Option B on these grounds. | Reopening is a supersession of the accepted law, not a surface choice. |
| Broader reusable copy surface (windows, caller rectangles, other dtypes now) | **Eliminated.** Admits populations with no verifier proof and no accepted authority; slice remains a separately walled family; dtype widening without a registered family is a claim about nothing. | Reopen per-population when a family with its own accepted law demands it; the closed `CopyElement` and absent window fields make each widening a deliberate, identity-visible act. |
| Further bounded research | **Stop condition met.** Every construction, validation, consumption, refusal, and identity path was read at this base; the encoding grammar and association were derived independently; no remaining unknown is answerable by more reading. | Reopen on a concrete contradiction found in review. |

## Downstream graph and dependency repairs

1. [`admit-the-partitioned-copy-scheduled-region`](admit-the-partitioned-copy-scheduled-region.md) implements exactly this surface once Tom accepts it, **including the `ResourceRequirements` numerical sum** (the ownership repair recorded in this packet's audit), and its overlap/gap/wrong-prefix closing perturbations are discharged at the projection boundary as specified above.
2. [`admit-an-explicit-non-arithmetic-region-and-delivery-state`](admit-an-explicit-non-arithmetic-region-and-delivery-state.md) keeps the KIR/artifact/delivery `FloatingPoint | BitPreservingCopy` projection; its resource-carrier sentence is narrowed by the repair above, which the coordinator should mirror into that ticket at dispatch.
3. [`derive-target-numerical-feasibility-from-reached-arithmetic-only`](derive-target-numerical-feasibility-from-reached-arithmetic-only.md) consumes `RegionNumericalRequirements::BitPreservingCopy` as proved absence.
4. [`lower-the-partitioned-copy-region-through-kernel-ir`](lower-the-partitioned-copy-region-through-kernel-ir.md) assigns KIR tags and the specialized ownership verification; nothing here constrains its encoder beyond the accepted one-kernel decision.
5. [`plan-concatenate-through-one-partitioned-copy-entry`](plan-concatenate-through-one-partitioned-copy-entry.md) lands the recognizer, the `partitioned-copy-f32.v1` request arm payload, the `UNPLANNED_OPERATIONS` retirement with its reason superseded, the optimizer census update, and end-to-end conformance.

No new prerequisite tickets were needed: every discovered obligation maps onto an existing node, and the one ownership move is recorded here and in the dispatch path rather than hidden.

## Packet verification — 2026-08-18

The clean source tree plus this ticket-only packet passed:

```sh
cargo nextest run -p tiler-ir -E 'test(the_concatenate_law_realizes_one_root_per_operand_over_one_output) or test(the_concatenate_law_partitions_by_operand_rather_than_by_input) or test(the_strict_f32_region_has_its_recorded_canonical_identity) or test(existing_one_committer_schedule_encodings_keep_their_bytes) or test(every_pinned_identity_domain_still_appears_in_the_source)'
cargo nextest run -p tiler-compiler -E 'test(the_governed_registry_holds_one_capability_per_admitted_concatenate_arity) or test(every_unplanned_operation_is_registered_and_consumes_no_dimension) or test(the_governed_concatenate_lowering_refines_at_every_admitted_arity) or test(every_pinned_identity_domain_has_its_exact_source_population)'
tkt lint
make citations
git diff --check
tkt guard tkt/decide-the-partitioned-copy-scheduled-region-public-surface --format json
```

The first run reported 5 passed, the second 4 passed, with zero failures. Lint, citations, whitespace, and guard results are recorded on the branch; guard output is scope evidence only.

## Independent review — 2026-08-18

Adversarial review at exact base `2dad9a7cd40b644cb58e1b58c6a3d270758d9c5e`, branch `tkt/decide-the-partitioned-copy-scheduled-region-public-surface`, by `worker-partitioned-copy-review`. Every named source was read in full at this base: `crates/tiler-ir/src/schedule/{model,builder,error,mod}.rs`, `crates/tiler-ir/src/index/law.rs` (`emit_partitioned_concatenate`, `ConcatenatePlan::derive`, the unreachable-refusal convention), `crates/tiler-ir/src/semantic/concatenate.rs`, the cited regions of `crates/tiler-compiler/src/{request,physical,frontier,program,policy}.rs`, and the three acceptance records plus the four downstream tickets. Base drift from the packet's `075d2d44` was verified with `git diff --name-only 075d2d44..2dad9a7c`: ticket/queue files plus `crates/tiler-reference/tests/language_model_boundaries.rs` only, and that test file touches no schedule surface (`grep -rln "ScheduledRegionBuilder" crates/tiler-reference/` is empty).

### Per-Fact verdicts on the 2026-08-18 audit

1. **Verified.** Both acceptance anchors resolve and say what is claimed; tag 12 and the `concat(x, x)` two-members-one-boundary ruling are in the records verbatim.
2. **Verified.** `Exact names follow the source audit` and the `Current-base correction — 2026-08-17` assigning the spelling to this ticket both resolve in the downstream record.
3. **Verified.** `pub struct IndexRegion` carries mandatory `scalar_program`/`numerical`; `assemble` refuses each absence as `IncompleteRegion` naming `ScheduleComponent::ScalarProgram`/`NumericalRealization` (builder.rs, `fn assemble`).
4. **Verified.** `ScheduledRegionDiagnostic` carries exactly the four named rule enums and no copy vocabulary; the anchor resolves.
5. **Verified.** `encode_identity` opens `tiler.schedule.v6\0`; scalar tags run `0x22`–`0x2A` ending at `TAG_SCALAR_SQUARED_SUM_EPILOGUE`, access tags `0x01`–`0x09` ending at `TAG_LIVE_ROW_MAJOR`; `0x36` is reserved in the reduction-topology position only; the `v6` step comment (`declared-input ordinal payload from fieldless input roles`) matches the AccessOrdinal reconciliation.
6. **Verified.** `canonical_explain_subject_bytes` opens `tiler.compiler.request-subject.v6\0` with the five named self-framing arm sub-tags; `verify_schedule_with_feasibility` compares `verified.region().index.numerical` against the request realization unconditionally (physical.rs:3257 area); `region_numerical_requirements` destructures all eight floating-point fields exhaustively; the `RegionProgram|PartitionedCopyProgram` grep returns nothing at this base.
7. **Verified.** `MIN_CONCATENATE_OPERANDS = 2`, `MAX_CONCATENATE_OPERANDS = 8`, F32-only under `concatenate.f32.implicit-promotion`, and `static_operand_shape` gates every operand before `concatenate_result_shape`.
8. **Verified.** All four association/representation forks are genuine at this source; the `concat(x, x)`-under-dedup argument for per-access geometry holds (dedup is imperative in [`admit-the-partitioned-copy-scheduled-region`](admit-the-partitioned-copy-scheduled-region.md), `Deduplicate boundary input bindings, never ordered members`, and the KIR ticket fixes one source binding with two members).

The audit preamble's inter-base drift sentence was **imprecise** — `git diff --stat 783e9b5b..075d2d44` over the cited files shows three changed files, not one (`program.rs` +83 split-continuation lines, `domains.rs` +2 kernel-domain rows, plus the model.rs rewording); no cited claim is touched. Repaired in place above.

### Independent derivations reproduced

- **Injectivity of `0x2B`/`0x0A`.** Derived from the encoder grammar independently: every field before the program position is framed and self-delimiting, so two equal byte strings parse to the same program-byte offset, where `0x2B` is disjoint from the only bytes any earlier region writes there (`0x22`–`0x2A`); likewise `0x0A` against `0x01`–`0x09` at the access-map position. Within-arm recoverability from the fixed-width framed member run, and one-encoding-per-meaning from `SourceOrder` plus never-written derived quantities, both close. The 14-fixed-bytes arithmetic (1 + 1 + 4 + 8) and the 12-byte member record (4-byte ordinal + 8-byte extent) check out, as do the 96-byte / 48 KiB bounds.
- **`concat(x, x)` under dedup** — reproduced; see Fact 8.
- **Derived prefix offsets** — reproduced: intervals `[P_k, P_k + e_k)` with `P_k` the exclusive prefix sum are adjacent by construction, so the only representable coverage defect is a wrong total (`CoverageSum`), with `ExtentOverflow` correctly ordered before it; `emit_partitioned_concatenate` emits one root per operand with a `linear_combination` offset displacement on the concatenated axis, and the law record carries the two watched-failing displacements, so the projection layer has its typed refusal sites.
- **`derive_requirements` inside `build`** — reproduced: `build` → `assemble_and_verify` → `derive_requirements(&region)` reading eight mandatory fields of `region.index.numerical`, so a copy region cannot exist before the `ResourceRequirements` sum does; the ownership move is forced. It does not contradict the accepted downstream record's semantics (its item 3 already specifies the sum's exact shape); it narrows one scheduling sentence of that record's graph correction, which downstream repair 2 mirrors.
- **Consumer census** — the packet's grep reproduced exactly, and the table covers every hit. The census *population* was however incomplete: the surface also replaces the builder setters, the `IndexRegion` fields, and the `ResourceRequirements` shape, whose consumers the field-read grep cannot see. Independently derived misses, all repaired into the table above: `push_requirements` in `crates/tiler-ir/src/kernel/model.rs` (kernel identity), `push_resources`/`parse_entry` in `crates/tiler-artifact/src/program/` (artifact wire codec), `subprogram_resources` in `crates/tiler-compiler/src/frontier.rs` (silent stage-zero numerical inheritance), and the setter/literal call-site population across seven further crates. The `crates/tiler-reference/src/oracle.rs` row was **false** (no schedule construction exists anywhere in `tiler-reference`); corrected in place.
- **`SubnormalFreedom::Unproven` for the copy arm** — correct, not over-refusing: the only consumer is Metal emission's `record_subnormal_obligation`, reached per emitted float arithmetic op; a copy emits none and its numerical requirement is the `BitPreservingCopy` absence, so `Unproven` costs nothing at this base and any copy-specific freedom would indeed need KIR-level bit-preservation evidence.
- **`partitioned-copy-f32.v1` payload deferral** — sound: arm sub-tags are length-framed and lead each arm, so cross-arm separation holds whatever the plan ticket writes inside; within-arm injectivity is that ticket's named obligation. The deferral is explicit, not implicit.

### Discrepancies

1. **False row (moderate, repaired):** the `oracle.rs` migration row named construction sites that do not exist.
2. **Incomplete census (moderate, repaired):** kernel-identity encoder, artifact wire codec, `subprogram_resources`, and the setter/literal call-site population were absent from "Total consumer migration" although the first three are total-map sites inside the section's own stated scope, and two of them carry byte-preservation obligations (`tiler.kernel-program.v12`, artifact schema `16.0`).
3. **Unrealizable precedence detail (minor, repaired):** the stated order placed a shared `BoundsProof` refinement step after `UnreferencedSource` while also calling `verify_proof_records` "unchanged" earlier in the order; one unchanged call performs count, reference, and refinement together, and the fieldless map means the new refinement arm cannot compare the member-derived count anyway. Restated: the arm is structural and `partitioned-copy-source-shape` owns exactness.
4. **Imprecise drift sentence (minor, repaired):** see the per-Fact section.
5. **Observation (no repair):** two of `partitioned-copy-topology`'s three clauses are shadowed by shared gates at this base — a copy region with a `BlockedWorkgroup` binding fails `blocked-workgroup-binding-forbidden` (or trips the reduction clause when paired with `CooperativeContraction`), and a `Predicated` tail fails `LaunchCoverage` — so only the reduction clause is independently watchable, which is exactly the one perturbation the packet lists. The clauses are defence-in-depth within one watchable rule, consistent with the convention; a per-clause perturbation demand would find the other two report the shared rules' text.
6. **Observation (no repair):** "the policy census tests pin exactly that separation" reads as present-tense; today `the_capability_table_names_exactly_the_admitted_operations` pins rowless-and-unplanned as one state, so retiring the concatenate entry obliges the plan ticket to restructure that census (a planned-but-rowless state), which the sentence anticipates rather than describes.

### Commands rerun at this base

Both packet nextest commands reproduced exactly — 5 passed and 4 passed, zero failures — plus `tkt lint`, `make citations`, `git diff --check`, and `tkt guard tkt/decide-the-partitioned-copy-scheduled-region-public-surface --format json` on the repaired ticket (results with the commit).

### Verdict

**Ready for Tom with the repairs above, which are made in place.** The recommended surface itself — the field-replacing exhaustive `RegionProgram` sum, program-owned members with derived checked prefix offsets, the fieldless `PartitionedCopySource` map, the closed one-variant `CopyElement`, the eleven rules, tags `0x2B`/`0x0A` preserving `tiler.schedule.v6`, and the governed static-F32 population with its exclusions — survives independent derivation unchanged, and the dominance claim stands: no repaired discrepancy touches the frontier ordering, the eliminations, or the draft question's content. The repairs correct the migration census, one false row, one unrealizable precedence detail, and one imprecise drift sentence.
