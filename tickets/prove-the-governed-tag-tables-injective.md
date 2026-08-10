---
id: prove-the-governed-tag-tables-injective
title: Prove the governed tag tables injective
status: done
priority: p2
dependencies: [derive-the-artifact-numerical-and-fenced-space-populations]
related: [prove-the-exhaustible-encoder-injectivity-claims-natively]
scopes: [implementation/ir, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [verification, identity, injectivity, evidence-upgrade]
---
## User-visible outcome

Every governed `tag()` table reached only by an *inexhaustible* identity encoder is backed by an exhaustive pairwise-distinctness test over its whole variant set, so a duplicated tag literal fails the build's gate instead of silently folding two operations, address spaces, or authorities onto one identity.

## Why this exists (found while proving the exhaustible encoders, 2026-08-07)

**Fact — corrected 2026-08-10.** `prove-the-exhaustible-encoder-injectivity-claims-natively` landed exhaustive injectivity tests for 19 canonical-identity encoders, and tag tables actually reached by those tests are covered by them. Its later source audit, at the anchor `The Outcome close language that the enumeration is complete`, withdraws the claim that it classified *every* encoder: kernel `push_index_arithmetic` and artifact `index_arithmetic_tag` / `index_arithmetic_from_tag` are a size-one residual. This ticket absorbs that explicitly delegated residual beside its tag-table census. The artifact round-trip authority is the source anchor `fn every_governed_tag_table_round_trips`; its payload-carrying numerical populations are complete only after [`derive-the-artifact-numerical-and-fenced-space-populations`](derive-the-artifact-numerical-and-fenced-space-populations.md), so this ticket must read that landed result rather than repeating the old blanket seven-table claim.

**Fact — corrected current syntactic census, not yet the owed-set classification.** At base `8a1602693a275e5ab3aeb8207723eb8bb2bc58f0`, `rg -n 'fn tag' crates/tiler-ir/src crates/tiler-artifact/src --glob '*.rs'` finds 65 source definitions: 54 in `tiler-ir` and 11 in `tiler-artifact`. One artifact hit is the generic codec-decoder helper `fn tag<T>(...)`, leaving **64 tag-method source definitions**. That is not the semantic table population: the one definition inside `spelled_rule!` expands for `NanReferenceRule`, `InfiniteReferenceRule`, `DomainErrorRule`, and `FiniteOverflowRule`, so the 64 definitions represent **67 method tables**. The first 2026-08-10 correction named only the first three invocations from a truncated excerpt; the full source file supersedes it with all four. Some are already covered by exhaustive encoder evidence; tables whose source exposes `from_tag` still require the typed complete-population and full unclaimed-byte proof classified below. The old “about 50” estimate and the directory counts below predate later vocabularies and are not a complete current manifest.

**Historical starting population — must be reclassified, not copied into tests.** The filing inventory named the kernel, program/ABI, numerics, schedule, shape, semantic, index, and ten artifact tables, including `FactAuthority` and `FactValidityScope`. The current source-definition census is 12 kernel, 10 program/ABI, 7 numerics, 11 schedule, 5 shape, 7 semantic, and 2 index definitions, plus the same ten artifact definitions: 64 total. The semantic definitions expand to ten tables because `spelled_rule!` produces four, making 67 method tables in the overall expanded manifest. The historical `semantic/ 8` and `index/ 3` counts are false at this base. Before implementation, produce an exact expanded manifest that marks each table as (a) already covered by an exhaustive encoder, (b) covered by a complete left inverse, or (c) owed here, and read every table in category (c) in full. Include the kernel/artifact index-arithmetic residual named above. This classification—not the syntactic count—is the closing population.

**Inference.** `FactAuthority` and `FactValidityScope` deserve first attention: both assign tags deliberately *out of declaration order*. The source anchors `The tags are deliberately not in declaration order` and `` `MeasuredEnvironment` carries `0x05` `` state why, and this is exactly the shape where a hand-checked literal table is easiest to get wrong and hardest to spot in review.

## Exact-base expanded-table classification (2026-08-10)

This classification is against exact base `8a1602693a275e5ab3aeb8207723eb8bb2bc58f0`. It distinguishes the 64 source definitions from the 67 tables they produce. Every named source file was read around the complete enum, tag match, encoder consumer, and correctness-bearing test; the source-safe anchors are the symbols below. A table counts as complete only if its inhabitant list is sized from `variant_count`, its tags are proved pairwise distinct, and a left inverse, where present, is checked over both the claimed and all unclaimed bytes.

**A — already covered by an exhaustive encoder (8 tables).** `StorageEncoding::tag` and `StorageScalar::tag` in `program/model.rs` are reached by the constructible-domain `the_storage_encoding_encoder_is_injective_over_its_constructible_domain` and whole-domain `the_storage_scalar_encoding_is_injective_over_its_whole_domain`. `ApproximationEnvelope::tag` in `numerics.rs` is reached by the type-derived `all_behaviours` / `DimensionBehaviour::encode` proof. `SynchronizationKind::tag`, `SynchronizationScope::tag`, and `MemoryOrdering::tag` in `schedule/synchronization.rs` are all reached by `the_synchronization_subject_encoding_is_injective_over_its_whole_domain`. `ComponentValueDomain::tag` in `semantic/conformance.rs` is reached by `the_component_value_domain_encoding_is_injective_over_its_whole_domain`. `IndexDomainFactSource::tag` in `index/predicate.rs` is reached by the `FACT_SOURCE_CASES` identity comparison in `index/builder/tests.rs`. These tests already have type-derived populations; this ticket leaves them intact.

**B — covered at the base by a complete left inverse under this ticket's closing standard (0 tables).** Several base tests round-trip a hand-sized list or spot-check one or two unknown bytes, including `every_provenance_authority_and_scope_tag_is_total_and_fails_closed` and `every_governed_tag_table_round_trips`. Neither shape proves both a type-derived complete population and refusal of every unclaimed byte. Those tables therefore remain in C; the presence of a source `from_tag` does not by itself prove the left-inverse claim.

**C — owed here (59 tables).** The expanded manifest is:

- kernel/model.rs (12): `KernelType::tag`, `AddressSpace::tag`, `BufferAccess::tag`, `Builtin::tag`, `BinaryOp::tag`, `CompareOp::tag`, `UnaryOp::tag`, `ConvertOp::tag`, `PackedExtractOp::tag`, `ExecutionScope::tag`, `MemoryScope::tag`, `BarrierOrdering::tag`;
- program/model.rs and program/abi.rs (8): `ValueRole::tag`, `MemorySpace::tag`, `AllocationOwnership::tag`, `StageAccessMode::tag`, `RoutingCommitState::tag`, `AvailabilityPhase::tag`, `AbiUnaryOp::tag`, `AbiBinaryOp::tag`;
- numerics.rs (7): `NumericalDimension::tag`, `HonouringMeans::tag`, `PolicyLocus::tag`, `FactAuthority::tag`, `FactValidityScope::tag`, `CompilerBuildRole::tag`, `FactEvidenceBasis::tag`;
- schedule (7): `ContributorArrival::tag`, `StagedElement::tag`, `LocalCoordinateSource::tag`, `ReductionPass::tag`, `ArithmeticType::tag`, `SynchronizationPlacement::tag`, `ConvergenceEvidence::tag`;
- shape (5): `ExtentTerm::tag`, `ExtentRelation::tag`, `BindingSource::tag`, `FactProvenance::tag`, `SourcedExtent::tag`;
- semantic (9 expanded tables): `CanonicalIntegerWidth::tag`, `ReferenceRoundingRule::tag`, `NanReferenceRule::tag`, `InfiniteReferenceRule::tag`, `DomainErrorRule::tag`, `FiniteOverflowRule::tag`, `AccuracyContractForm::tag`, `ReferenceResultClass::tag`, `ConformanceEvidenceClass::tag`. The four `*Rule` tables are the four expansions of the one `spelled_rule!` source definition;
- index (1): `SourcedIndexInteger::tag`;
- artifact (10): `RoutingPolicy::tag`, `ArtifactExecutionPolicy::tag`, `BindingKind::tag`, `StageDependencyReason::tag`, `RecordFamily::tag`, `AssessmentDisposition::tag`, `SectionKind::tag`, `SectionDisposition::tag`, `RouteResourceDimension::tag`, `RouteRequirement::tag`.

The delegated residual adds two encoder/table subjects without changing the 67-method census: kernel `push_index_arithmetic` and artifact `index_arithmetic_tag` / `index_arithmetic_from_tag`. Both range over the `variant_count::<IndexArithmetic>() == 1` population and are proved separately because the kernel encoder and artifact table are independently maintained bytes.

## The work

1. Re-derive the exact 64-source-definition / 67-expanded-method-table manifest above and record why every table is already covered or owed. Include the delegated kernel/artifact index-arithmetic residual. For each owed table, enumerate its variants with an array sized by `core::mem::variant_count` so a widened vocabulary is a build error at the list. `#![cfg_attr(test, feature(variant_count))]` is already declared in both crates.
2. Assert the tags are pairwise distinct and count the population walked, so a shrunk enumeration fails rather than passing vacuously.
3. Where a `from_tag` inverse exists, also assert the round trip and that every unclaimed byte refuses — several already do this and need only the population guard.
4. Watch each new check fail on a planted duplicate literal before trusting it.
5. Do not weaken the existing round-trip tests; these sit beside them.

## Closes when

Every table in the enumeration above has a passing exhaustive distinctness test with a `variant_count`-guarded population, each watched failing on a planted duplicate tag, and any table deliberately left out is named with its reason.

## Implementation evidence (2026-08-10, branch remains in progress for integration)

All 59 C tables now have `variant_count`-sized inhabitant arrays, a nonempty and walked-population assertion, and pairwise tag comparison. Sixteen method tables expose `from_tag`; those additionally round-trip every claimed tag and sweep all 256 bytes to prove that every unclaimed byte refuses. The delegated kernel `push_index_arithmetic` encoder and artifact `index_arithmetic_tag` / `index_arithmetic_from_tag` table have independent type-sized checks, with the artifact table adding a seventeenth complete inverse subject. No production tag, vocabulary, encoder byte, identity domain, schema, version, or public surface changed.

### Subject perturbations

Each non-singleton table was perturbed independently by replacing one production tag literal with another inhabitant's literal, its focused test was run, and the source was restored before the next perturbation. Exact failure text:

```text
KernelType::tag tag 0x01 is shared by Index and Bool
AddressSpace::tag tag 0x01 is shared by Workgroup and Device
BufferAccess::tag tag 0x01 is shared by Write and Read
Builtin::tag tag 0x01 is shared by LocalInvocationIndex and GlobalInvocationIndex
BinaryOp::tag tag 0x01 is shared by IndexMultiply and IndexAdd
UnaryOp::tag tag 0x01 is shared by F32Rsqrt and F32Exp
ConvertOp::tag tag 0x01 is shared by U8ToI32 and CanonicalizeF32Nan
ExecutionScope::tag tag 0x01 is shared by Workgroup and Subgroup
MemoryScope::tag tag 0x01 is shared by Device and Workgroup
ValueRole::tag tag 0x01 is shared by Temporary and Input
AllocationOwnership::tag tag 0x01 is shared by Program and External
StageAccessMode::tag tag 0x01 is shared by Write and Read
RoutingCommitState::tag tag 0x01 is shared by Committed and Preflight
AvailabilityPhase::tag tag 0x01 is shared by ArtifactEvidence and CompileProfile
AbiUnaryOp::tag tag 0x01 is shared by NarrowU16 and Not
AbiBinaryOp::tag tag 0x01 is shared by CheckedSubtract and CheckedAdd
HonouringMeans::tag tag 0x01 is shared by SupportedWithExactEmulation and SupportedExactly
PolicyLocus::tag tag 0x01 is shared by Computation and Input
FactAuthority::tag tag 0x01 is shared by ExternalProfile and GovernedProfile
FactValidityScope::tag tag 0x01 is shared by MeasuredEnvironment and PortableProfile
CompilerBuildRole::tag tag 0x01 is shared by Optimizer and Frontend
FactEvidenceBasis::tag tag 0x01 is shared by Measurement { contexts: [] } and GovernedGuarantee { guarantee: ProvenanceIdentity { key: "governed", revision: 1 } }
ContributorArrival::tag tag 0x01 is shared by NondeterministicArrival and AscendingParticipant
LocalCoordinateSource::tag tag 0x01 is shared by LocalWorkgroupPosition and LocalLinearInvocation
ReductionPass::tag tag 0x01 is shared by Final and Partial
ArithmeticType::tag tag 0x01 is shared by Bf16 and F16
SynchronizationPlacement::tag tag 0x01 is shared by RoundBoundary and PhaseBoundary { preceding: PhaseId(0), following: PhaseId(1) }
ConvergenceEvidence::tag tag 0x01 is shared by CallerAsserted and EveryParticipantReachesThePoint
BindingSource::tag tag 0x01 is shared by InputDimension { input: InputKey("input"), axis: Axis(0) } and Static(Extent(1))
FactProvenance::tag tag 0x01 is shared by FrontendRequired and StaticallyProven
ExtentTerm::tag tag 0x01 is shared by Constant(1) and Symbol(ShapeSymbol { scope: SymbolScope([116, 97, 103]), name: "extent" })
ExtentRelation::tag tag 0x01 is shared by Divisible { dividend: Constant(2), divisor: 1 } and Equal { left: Constant(1), right: Constant(2) }
SourcedExtent::tag tag 0x01 is shared by Symbol(ShapeSymbol { scope: SymbolScope([116, 97, 103]), name: "extent" }) and Static(Extent(1))
CanonicalIntegerWidth::tag tag 0x08 is shared by Bits16 and Bits8
NanReferenceRule::tag tag 0x01 is shared by Refuse and CanonicalNan
InfiniteReferenceRule::tag tag 0x01 is shared by CanonicalNan and SignedInfinity
DomainErrorRule::tag tag 0x01 is shared by Refuse and CanonicalNan
FiniteOverflowRule::tag tag 0x01 is shared by LargestFinite and SignedInfinity
AccuracyContractForm::tag tag 0x01 is shared by Faithful and CorrectlyRounded { rounding: NearestTiesToEven }
ReferenceResultClass::tag tag 0x01 is shared by Nonzero and Finite
ConformanceEvidenceClass::tag tag 0x01 is shared by ExhaustiveFinite and FormalProof
SourcedIndexInteger::tag tag 0x01 is shared by Symbol(ShapeSymbol { scope: SymbolScope([114, 101, 103, 105, 111, 110, 47, 48]), name: "tag-source" }) and Literal(0)
StageDependencyReason tag 0x01 is shared by StorageHandoff and Data
AssessmentDisposition tag 0x01 is shared by Required { first: 0, len: 1 } and NotRequired
SectionKind tag 0x01 is shared by BackendPayloadMetadata and KernelProgramSubject
SectionDisposition tag 0x01 is shared by Optional and Required
RouteRequirement tag 0x01 is shared by BackendFeature(BackendFeatureRequirement { owner: BackendKey("tiler.metal"), key: RouteFeatureKey("tiler.metal.minimum-gpu-family"), version: 1, payload: [1] }) and Resource(RouteResourceRequirement { dimension: SubgroupThreads, required: 32 })
```

`NumericalDimension`'s planted `InputSubnormals` / `ResultSubnormals` collision is rejected even earlier by its pre-existing compile-time guard, with exact failure `error[E0080]: evaluation panicked: assertion failed: CANONICAL_DIMENSIONS[left].tag() != CANONICAL_DIMENSIONS[right].tag()`; the new test supplies the missing `variant_count` population boundary and full unclaimed-byte inverse sweep.

**Unsupported duplicate-literal perturbations (and only these).** Eleven owed method tables have one inhabitant: `CompareOp`, `PackedExtractOp`, `BarrierOrdering`, `MemorySpace`, `StagedElement`, `ReferenceRoundingRule`, `RoutingPolicy`, `ArtifactExecutionPolicy`, `BindingKind`, `RecordFamily`, and `RouteResourceDimension`. The two index-arithmetic subjects are also singleton. A production-literal duplication does not exist in a one-inhabitant match; manufacturing it would require adding a variant and would violate this ticket's authority. Their type-sized arrays still make population widening a compile error at the test list. Where a left inverse exists, its claimed-byte round trip and complete unclaimed-byte refusal are exercised; for the remaining singleton forward maps, injectivity is true by cardinality and the only non-vacuous regression boundary is the type-derived population.
