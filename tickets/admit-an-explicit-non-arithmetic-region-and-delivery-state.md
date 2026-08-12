---
id: admit-an-explicit-non-arithmetic-region-and-delivery-state
title: Admit an explicit non-arithmetic region and delivery state
status: awaiting-decision
priority: p1
dependencies: [admit-the-concatenate-family-into-the-scheduled-region-vocabulary]
related: [admit-the-partitioned-copy-scheduled-region, derive-target-numerical-feasibility-from-reached-arithmetic-only]
scopes: [implementation/ir, implementation/artifact, implementation/build, implementation/compiler, contracts/foundation, contracts/numerics, contracts/artifacts, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, public-boundary, numerics, artifacts, identity, strict, decision, needs-tom]
---
## Outcome

A scheduled region, KIR entry, and artifact can state either arithmetic with its complete numerical realization or a bit-preserving non-arithmetic computation for which arithmetic numerical requirements and delivery are explicitly not applicable. Invalid mixed states are unrepresentable; no optional field, default profile, silent absence, or inferred strict realization exists.

## Required boundary

Use exhaustive typed sums at every owning boundary, conceptually `RegionProgram::Arithmetic { scalar, numerical } | PartitionedCopy(...)`, structural requirements plus `NumericalRequirements::{NotApplicable, Arithmetic(...)}`, and an equally explicit artifact delivery form. Exact names follow the source audit, but arithmetic without numerics, copy with numerics, and an unclassified empty state must be impossible.

Preserve the caller's stated program contract as request meaning without asking a target to honour arithmetic a copy never executes. Mixed programs retain complete numerical delivery for arithmetic entries and explicitly classify copy entries as not applicable. Decode and construction reject unknown tags and inconsistent cross-entry claims.

## Identity and compatibility

Read every schedule, KIR, artifact, delivery, codec, cache, proof, and build consumer before choosing the encoding. Preserve legacy bytes only with an injectivity proof; otherwise step the owning domain or manifest schema deliberately and update ledgers and pins. Pre-alpha status is not permission to let an old reader reinterpret a new non-arithmetic record.

## Source-first Fact audit — 2026-08-12

Audited at exact main `a0779d0f5b54f94c94474d2df73f54d41f6cd8e5`.

1. **Verified — the current state is inseparable and arithmetic-shaped.** `IndexRegion`, anchor `pub struct IndexRegion`, carries mandatory `scalar_program: ScalarProgram` and `numerical: NumericalRealization`. `ScheduledRegionBuilder::assemble` refuses either absence. `derive_requirements`, anchor `The numerical realization is carried forward whole`, copies eight numerical behaviours into flat `ResourceRequirements` fields.
2. **Verified — KIR repeats and verifies the same statement.** `KernelData`, anchor `The assembled, not-yet-verified structured kernel`, carries mandatory `NumericalRealization`; `KernelBuilder::numerical` is required; canonical lowering copies `schedule.index.numerical`; kernel identity writes the realization before the resource record. A copy cannot omit it without changing the owning type and every total verifier/encoder.
3. **Verified — artifact entries and delivered policy assume every entry is numerical.** `NumericalFacts` is mandatory in each fixed entry row; `EntryRef::numerical` is total; `EntryRealization` carries eight behaviours; `ArtifactProgramBuilder::check_subject` reads the first stage's realization and rejects any other stage that differs. `tiler-build::realization::translate` binds every packaged entry to the sole scalar-arithmetic subject.
4. **Verified but insufficient — `AssessmentDisposition::NotRequired` already exists.** It says no packaged route consumes one dimension of a declared scalar-arithmetic subject. It does **not** say that one entry performs no floating-point transformation, and every `EntryPolicyBinding` still binds an entry to a scalar subject. Reusing the disposition as an entry kind would conflate program-wide obligation absence with per-entry computation class.
5. **Verified — the caller's contract remains meaningful on a copy-only request.** The verified request and `DeliveredRealizationEvidence` retain one complete scalar-arithmetic policy subject even when the reached operation-capability rows consume no dimensions. A copy entry therefore need not delete or default the caller's contract; it must state separately that this entry does not use it.
6. **False in the current graph — a generic non-arithmetic state can land before its computation.** No concrete non-arithmetic `ScalarProgram`, scheduled region, or KIR exists. The sole named consumer is the accepted `PartitionedCopy` concatenate schedule, but [`admit-the-partitioned-copy-scheduled-region`](admit-the-partitioned-copy-scheduled-region.md) currently depends on this ticket. Landing a public `NotApplicable` arm first would create a constructible or retained state with no verified computation that can justify it. The schedule computation must own the classification and the downstream records must derive it.
7. **Imprecise — `Arithmetic` is not the right public partition name.** `StrictSerialMaximum` selects and canonicalizes floating-point values without performing arithmetic in the narrow add/multiply sense, yet its target numerical behaviour remains relevant. The MECE axis is whether an entry performs a value-changing floating-point computation or is verifier-proved bit-preserving, not whether one can colloquially call an instruction arithmetic.
8. **Verified — a bit-preserving copy still performs index arithmetic.** `ResourceRequirements::index_arithmetic` is mandatory because every region computes coordinates. `NotApplicable` applies only to the floating-point numerical contract; it must not erase index arithmetic, memory, launch, synchronization, or bounds requirements.
9. **Verified — old schedule bytes can survive, artifact rows cannot safely do so by omission.** A `RegionProgram` encoder can emit every existing numerical arm exactly as `push_scalar_program` followed by `push_numerical` does and append a fresh partitioned-copy tag. The artifact manifest instead parses one fixed entry row as resources, then ten numerical fields, then bindings. Omitting those fields or using an empty profile-key sentinel under schema `16.0` lets an old reader consume following binding bytes as numerical fields. This requires an explicit tagged row and a manifest major compatibility fence, not a clever sentinel.
10. **Verified — no target fact should be consulted for bit transport.** A verified partitioned copy owes exact type equality and a KIR proof that every output bit pattern is loaded and stored unchanged. That includes NaNs, signed zero, infinities, and subnormals without converting them into target honourability questions. Backend compilation flags remain payload-compilation provenance; they do not turn the copy into floating-point numerical delivery.

## Revised decision packet — 2026-08-12

The top-level boundary should be computation-specific and derived, not a caller-set generic applicability flag.

### Recommended exact model

1. The concrete schedule ticket introduces the first inhabited sum, conceptually `RegionProgram::Numerical { scalar: ScalarProgram, realization: NumericalRealization } | PartitionedCopy(PartitionedCopyProgram)`. Existing numerical programs retain their exact meaning and bytes. `PartitionedCopy` carries its real ordered members and proof subject; there is no empty `NonArithmetic`, `Other`, or caller-authored `NotApplicable` arm.
2. Downstream layers project one closed numerical-use sum, conceptually `EntryNumerics::FloatingPoint(NumericalRealization) | BitPreservingCopy`. `BitPreservingCopy` is minted only after schedule/KIR verification proves exact source/destination type agreement and a body containing no value-changing floating-point operation or conversion. It is not a bool, `Option`, default, or backend assertion.
3. `ResourceRequirements` keeps structural requirements unconditional and replaces its eight flat floating-point fields with a typed numerical-requirement sum. The bit-preserving arm removes only floating-point honourability predicates; buffer, device-memory, index-arithmetic, synchronization, launch, and bounds obligations remain complete.
4. KIR carries and identity-encodes the same derived numerical-use class. Whole-kernel verification compares it with the scheduled `RegionProgram` and the canonical body. A backend cannot relabel an arithmetic kernel as a copy or introduce arithmetic while retaining the copy classification.
5. Artifact entry rows use an explicit tagged `EntryNumerics::{FloatingPoint(NumericalFacts), BitPreservingCopy}`. `EntryRef` exposes the enum rather than a falsely total `numerical()` result. The delivered-realization record retains the caller's scalar-arithmetic policy subject and its per-dimension `NotRequired` dispositions, while its entry-binding table becomes a total tagged sum: every entry is either bound to one scalar subject or explicitly classified `BitPreservingCopy`.
6. A copy-only artifact still carries the selected target profile and the caller's complete policy subject. A mixed artifact binds only floating-point entries to that subject. `ArtifactProgramBuilder` no longer derives portfolio truth from the first stage or requires all stages to have equal per-entry numerical use; it cross-checks every entry against the delivered record instead.
7. Target feasibility walks reached `RegionProgram` values. It asks numerical honourability only for `Numerical` arms and treats `PartitionedCopy` as a proved absence of floating-point requirements, never as target silence or a strict default.

### Graph correction

The concrete schedule state must land before its downstream delivery projection:

- move the `RegionProgram` sum and `PartitionedCopy` arm into [`admit-the-partitioned-copy-scheduled-region`](admit-the-partitioned-copy-scheduled-region.md);
- make this ticket the KIR/resource/artifact/delivered-record carrier and depend on that concrete schedule ticket; and
- make [`derive-target-numerical-feasibility-from-reached-arithmetic-only`](derive-target-numerical-feasibility-from-reached-arithmetic-only.md) depend on the concrete schedule classification rather than on a dead generic state.

No mock, placeholder region, fake realization, or test-only computation is introduced to satisfy the dependency graph.

### Identity and compatibility consequence

- Preserve every existing scheduled-region byte by keeping the numerical arm's current scalar-program-plus-realization encoding and appending a fresh partitioned-copy program tag; `tiler.schedule.v5` need not step if the implementation proves that property.
- Re-derive the KIR domain from the selected encoding. An append-only bit-preserving branch may preserve all old kernel bytes, but the implementation may not assume that before the exact encoder and length calculation agree.
- Step the artifact manifest schema from `16.0` to a new major and the artifact identity domain from `tiler.artifact-program.v16` because a fixed executable-entry row gains a required tagged sum. Existing readers must reject before parsing the new row, and every newly minted artifact identity/pin moves coherently.
- Step the delivered-realization domain if its entry-binding grammar gains an explicit tag. Prefer that readable tagged sum over a magic subject-index sentinel even if a sentinel could be made injective; pre-production compatibility does not justify a second hidden wire vocabulary.
- Do not step semantic, request, cache-container, or payload domains merely because their nested artifact/schedule subjects change. Recompute transitive values and move only owning grammars.

## Ranked options

1. **Concrete `PartitionedCopy` schedule plus derived `FloatingPoint | BitPreservingCopy` state through KIR and artifact.** Best correctness and fail-closed behaviour, MECE over the first real population, no dead state, one O(1) discriminator per entry, and total matches make future computation classes explicit build breaks.
2. **A generic `Applicable | NotApplicable` numerical enum landed before a consumer.** It can be made type-safe, but it is semantically weaker, permits an unproved reason-free absence, and creates a public state whose only initial implementation refuses it. It saves no meaningful runtime or memory.
3. **Keep mandatory `NumericalRealization` and designate one strict value as copy-neutral.** Smallest diff but incorrect: it asks target arithmetic questions about bit transport, can reject a legal copy, misstates delivery, and creates an alternate identity for no computation.
4. **Use `Option<NumericalRealization>`, an empty profile key, or all-`NotRequired` dispositions as the entry class.** Reject: each conflates missing with proved inapplicability, and the wire spellings are unsafe or ambiguous.
5. **Defer classification to Metal or infer it from an arithmetic-free emitted body.** Reject: the backend would become the authority over verified program meaning, other backends could disagree, and artifact identity would omit the distinction.

## Strongest counterpoint and reversal evidence

The recommended explicit sums and artifact major step touch many total maps and deliberately cold-miss every newly minted artifact, while the first consumer is one F32 copy family. That cost is real. It is still preferable to a fake realization or a generic absence because the touched maps are exactly the consumers that must not silently assume arithmetic forever. Reverse to a narrower artifact-only side table only if a complete encoding spike proves old entry rows remain byte-identical, old readers reject every new copy artifact before entry parsing, the side table gives every entry exactly one classification, and no consumer must join independently ordered tables to recover it. No current row shape meets those conditions.

## Decision request

Accept the concrete computation-specific model and graph correction above; revise the classification or compatibility strategy; or keep partitioned-copy scheduling blocked. Acceptance authorizes the public shape and dependency repair, not implementation around any unfinished schedule, KIR, artifact, or feasibility prerequisite.

## Closes when

All construction and consumption sites are total over the new sum; missing, defaulted, and contradictory states fail closed; arithmetic records remain byte-for-byte or deliberately versioned; and independent subject perturbations prove every new discriminator and cross-check is load-bearing.
