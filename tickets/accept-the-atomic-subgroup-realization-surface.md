---
id: accept-the-atomic-subgroup-realization-surface
title: Accept the atomic subgroup realization surface
status: todo
priority: p1
dependencies: [minimize-and-prove-the-atomic-subgroup-public-surface-before-acceptance]
related: [admit-an-atomic-subgroup-realization-subject-to-target-profiles]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

Tom accepts or revises the labelled-draft Rust spelling of the atomic subgroup realization he accepted as a model on 2026-08-11.

## Decision boundary

[ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) routes new public types and builder methods to Tom. The 2026-08-11 packet on [`admit-an-atomic-subgroup-realization-subject-to-target-profiles`](admit-an-atomic-subgroup-realization-subject-to-target-profiles.md) accepted the *model*. This node is the spelling landed at `5cd61fbe` (rebased tip `eecc4002`). Only Tom closes it.

## Ready decision packet — 2026-08-13

The minimizing dependency re-audited the surface at exact base `b2ab50f278616a1ad8f171184a16d60ae7e608ff`, removed the unconsumed decoder and unreachable error, privatized the raw transfer tag, marked both growing enums non-exhaustive, and added a verified `Some(subgroup)` kernel-identity subject. The exact repaired source commit is `595ddea1f47b167a9cb6d017f4ce5d10e0c1413a`. This packet is ready for Tom only at that named commit.

## Exact included surface

**`tiler_ir::schedule`.**

- `SubgroupWidth(u32)` has private storage; derives `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, `Ord`, `PartialEq`, and `PartialOrd`; and exposes `new(u32) -> Result<Self, SubgroupRealizationError>` plus `get() -> u32`.
- `SubgroupTransfer` is `#[non_exhaustive]`, has the one public variant `InRangeXorShuffle`, derives `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, `Ord`, `PartialEq`, and `PartialOrd`, and exposes only `key() -> &'static str` as a public method. Its raw identity tag is private. Downstream partial classification therefore retains a wildcard while construction of the known unit variant remains public.
- `SubgroupRealizationError` is `#[non_exhaustive]`, derives `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, `Ord`, `PartialEq`, and `PartialOrd`, and has exactly `ZeroWidth` and `UnsupportedWidth`. It exposes `rule() -> &'static str` and implements `Display` and `std::error::Error`.
- `SubgroupRealizationSubject` has private `width`, `arithmetic`, and `transfer` storage; derives `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, `Ord`, `PartialEq`, and `PartialOrd`; and exposes the fallible `new`, the three value getters `width`, `arithmetic`, and `transfer`, plus `encode(&mut Vec<u8>)`. `encode` is the single public authority used by cross-crate identity consumers.
- `ResourceRequirements.subgroup: Option<SubgroupRealizationSubject>` is the one public field added to the existing caller-constructed descriptor. `None` is canonical absence; `Some` is one complete subject.

**`tiler_compiler::target`.**

- `SubgroupSupport::{Realized, Unrealizable}` and `SubgroupRealizationResolution::{Realized, Unrealizable, Unknown}` each derive `Clone`, `Copy`, `Debug`, `Eq`, and `PartialEq`.
- `TargetProfileBuilder::declare_subgroup_realization(subject, support, TargetFactSource)` and `declare_measured_subgroup_realization(subject, support, TargetCompileProfileMeasurementSource)` return `Result<(), TargetProfileBuildError>` and refuse a second row for one subject and phase transactionally.
- `TargetProfile::subgroup_realization(subject, available_phase) -> SubgroupRealizationResolution` uses exact whole-subject equality. Silence, a neighbour, and a fact available only after the queried phase are `Unknown`; there is no invented query or deferred promise.
- `TargetProfileBuildError::DuplicateSubgroupRealization` is the existing non-exhaustive error type's public refusal variant. The verdict is not part of the uniqueness key, so both an exact duplicate and a contradiction refuse rather than letting canonical sort order choose a winner.

There is no additional public trait, namespace, field getter, conversion, default, serde implementation, unchecked constructor, or mutable view in this packet.

## Exact exclusions and unsupported population

- No public `SubgroupTransfer::tag`, no `SubgroupTransfer::from_tag`, and no `SubgroupRealizationError::UndefinedTransfer`. A decoder returns only with the first schema that owns real subgroup bytes and can test unknown-tag refusal end to end.
- No per-field target setters, boolean support flag, default fact, inherited target-family fact, or generic wrong-backend guess.
- No KIR subgroup operation, no admitted topology that derives `ResourceRequirements.subgroup = Some(_)`, no subgroup memory scope, and no row on the governed or standard Metal profile.
- No present-subject artifact resource encoding. The current artifact writer deliberately ignores `subgroup` and its decoder constructs `None`; therefore this packet accepts no subgroup artifact schema or round-trip guarantee.
- No typed transfer-neighbour evidence. `SubgroupTransfer` has one inhabitant. The kernel test pins `InRangeXorShuffle`'s tag at the final subject byte, but does not misdescribe a raw unknown byte as a second typed subject. A second transfer must add its own semantics, constructor rule, exhaustive arms for the same-crate `tag`, `key`, and `transfer_defines_width` authorities, a typed identity neighbour, and any owning decoder evidence. The public enum's growth marker does not relax those same-crate total matches.
- The crate-private checked fact, resolution, feasibility evidence, refusal, and unknown records remain compiler implementation details rather than additions to this public packet.

## Correctness and identity evidence

**Fact.** Construction rejects width zero and rejects every XOR-shuffle width that is not a power of two at least two. Every currently recognized `ArithmeticType` is constructible at a valid width. Whole-subject equality, not component composition, is the only positive target match.

**Fact.** Complete and checked target descriptors encode width, arithmetic, transfer, support verdict, phase, authority, validity, and source at their owning layers. Width and arithmetic neighbours move both descriptors; independently true neighbours compose into no permission; exact duplicates and contradictions refuse before insertion.

**Fact.** Silent target profiles write no subgroup section, so `COMPLETE_PROFILE_DESCRIPTOR_DOMAIN` stays `tiler.target-profile.declaration.v11` and `PROFILE_DESCRIPTOR_DOMAIN` stays `tiler.target-profile.descriptor.v10`. The already-landed feasibility rule-set step to `v6` remains the only identity-domain step: assessment now decides a predicate `v5` could not state.

**Fact.** The kernel identity test builds and verifies a real prospective `ResourceRequirements.subgroup = Some(_)` through the ordinary refinement gate. A silent pointwise kernel retains the exact pre-subgroup identity pin. A present subject appends exactly seven bytes — presence `01`, big-endian `u32` width, `ArithmeticType` tag, then transfer tag — and width `32`/`64` and arithmetic `F32`/`Bf16` move the whole identity independently. `InRangeXorShuffle` is pinned as the final `01`; there is no second typed transfer to compare. `KERNEL_DOMAIN` stays `tiler.kernel.v7`: every currently derivable kernel is absent and keeps its bytes, while a present subject was not previously derivable.

**Fact.** Corrupting the production kernel encoder to omit the subject fails its encoder/reservation equality backstop (`left: 825`, `right: 831`). Corrupting production width encoding makes the compiler consumer fail with `the width dimension does not reach the complete descriptor`; corrupting arithmetic encoding gives the corresponding arithmetic failure; changing the private transfer tag makes the kernel suffix pin report final byte `255` instead of `1`. These perturb the producer, not the assertions.

**Fact.** An out-of-crate API fixture constructs `InRangeXorShuffle` and partially classifies it with the wildcard required by `#[non_exhaustive]`. Removing the production attribute makes that wildcard an `unreachable_patterns` error. Independently adding a temporary transfer makes all three same-crate total authorities — `tag`, `key`, and `transfer_defines_width` — fail to compile until the new semantics are defined.

**Inference.** The public `encode` and `key` helpers are the narrowest maintainable shared authorities: kernel/target/frontier identities consume `encode`, while physical errors and explanations consume `key`. Removing either would duplicate governed mappings across crates. The raw tag and inverse have no such consumer and are correctly absent.

## Identity, schema, and compatibility consequence

No existing canonical byte changes, no domain constant changes, no artifact schema changes, and no runtime or kernel behavior changes in the minimizing dependency. The source-level public delta relative to the earlier labelled draft removes one public method and one public error variant, makes one method private, and marks both remaining growing enums non-exhaustive. The two attributes are source-breaking for hypothetical downstream exhaustive matches because they require wildcard arms; that is the intended ADR 0074 compatibility contract for vocabularies with no external total recognizer. Tiler is `0.0.0`, unpublished, and has no external consumer; the in-workspace compiler consumer builds against the repaired surface.

Adding `ResourceRequirements.subgroup` was the original implementation's source-breaking field addition for callers constructing the record. That consequence is real even though every admitted schedule derives `None`; it belongs to the spelling Tom is deciding and is not hidden by the minimizing repair.

## Decision-packet option gate

- **Keep the pre-repair labelled draft.** Rejected as dominated: the public inverse and raw tag have no production consumer, `UndefinedTransfer` is unreachable, both growing enums violate ADR 0074 convention 5a, and none adds correctness, maintainability, or host/runtime value. Its only advantage is allowing hypothetical downstream exhaustive matches over today's one-transfer vocabulary, precisely the source pattern convention 5a reserves for a public total recognizer that does not exist here.
- **Remove `key` or subject `encode` as well.** Rejected as incomplete: live cross-crate explanation and identity consumers would duplicate the mapping or require a larger public-boundary redesign. That is worse for correctness and maintenance without shrinking the actual authority.
- **Invent a second transfer or an artifact decoder now.** Rejected as unsupported expansion: neither has admitted semantics or an owning schema. It would turn the evidence gap into production vocabulary rather than close it.
- **Remove or privatize the whole family.** Rejected as a consequential replacement of the accepted model and the public target-profile input surface. It would require redesigning current cross-crate target construction and is not a narrower implementation of this decision.
- **Defer exact-surface acceptance.** Correct but dominated at the current evidence boundary: it retains the same labelled public items, blocks dependent work, and names no experiment that would change a presently required method. Reconsider if the first additional transfer, first subgroup KIR operation, or first artifact schema demonstrates that the subject or decoder ownership needs another shape.
- **Accept the exact narrowed surface.** The sole nondominated candidate: it keeps every live authority, removes every speculative public item found by the census, conforms to ADR 0074, moves no existing identity or schema, and states the one-transfer evidence limit explicitly.

## Recommendation

**Proposal — accept the exact narrowed surface at `595ddea1f47b167a9cb6d017f4ce5d10e0c1413a`.** Strongest counterargument: because no admitted schedule yet derives `Some`, acceptance precedes the first executable subgroup consumer and the first artifact round trip; either could expose pressure on the subject spelling. `SubgroupTransfer` being non-exhaustive also prevents downstream exhaustive matches over today's sole variant, but no such total recognizer exists and ADR 0074 deliberately reserves that pattern for a closed vocabulary. Evidence that would reverse the recommendation is a concrete second transfer or schema whose correct construction cannot use the private-tag/public-subject-encode split without duplication or ambiguity. The recorded triggers ensure that evidence reopens the exact boundary instead of being silently absorbed.

## Closes when

Tom accepts the exact `595ddea1f47b167a9cb6d017f4ce5d10e0c1413a` surface, accepts it with named exclusions, or requests a named revision. Only Tom closes this ticket.
