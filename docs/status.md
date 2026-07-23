---
schema: "tiler-doc/v1"
id: "tiler.portal.status"
kind: "portal"
title: "Project status"
topics: ["status", "orientation"]
related: ["tiler.questions.open", "tiler.roadmap"]
---

# Project status

Tiler has entered a bounded prototype phase. The semantic, optimizer,
scheduling, numerical, artifact, cache, and runtime boundaries have substantial
accepted decisions and bounded executable evidence. An initial untyped
semantic/reference draft exposed incorrect provisional public boundaries; the
dependency-ordered v2 correction is now implemented through the semantic and
reference crate boundary. Graph ownership, recoverable commitment,
output-reachable compaction, origin-bound output selectors, independent type
authority, open operation registration, generic typed values, and exact
reference-capability dispatch are compile-checked for the bounded profile.
The dependent-array shape conformance gate, exact nightly migration, checked
shaped-value implementation, and assembled semantic/reference integration gate
are now complete. A downstream-style test constructs and evaluates the bounded
strict-`f32` program through public APIs, while malformed drafts and malformed
reference results fail closed. The private target-neutral proof now constructs
both a verified two-stage materialized plan and a verified one-stage fused
plan, retains both in a deterministic portfolio, and selects fusion only as a
strict structural Pareto improvement. An adversarial follow-up audit
established that this remains a graph-specific conformance fixture: occurrence
construction, region enumeration, legality evidence, KIR, and artifact-facing
program types are not yet a generic backend-consumable compiler path. Metal
AOT and device execution remain unimplemented, and no compiler API is public
or stabilized. Shared target-neutral IR ownership and checked-construction
boundaries are now accepted by ADRs 0070 and 0071. The compiler's bounded proof
pipeline is compiled as ordinary library code, and the unused backwards
compiler-to-artifact dependency has been removed. The proof-specific structs
remain private until their dedicated generic IR/verifier tickets replace them.
ADR 0072 defines separate graph meaning, reached provider-independent
definitions, admission-provider provenance, and complete registry snapshot
subjects. The corrected semantic implementation computes one deterministic,
iterative, cycle-safe, incrementally bounded authority closure across retained
types, operations, occurrence attributes, definition facts, defaults, and
conformance requirements. Root ingestion and unique discovered authority have
separate governed bounds. Borrowed validation runs the same reachable closure
as commitment and preserves authority failures as typed diagnostics. Completed
programs own one non-forgeable `SemanticIdentity` bundle with named accessors
for graph meaning, definitions, admission provenance, and the registry snapshot;
compiler requests and artifact-construction plans retain it atomically. Region occurrence,
checked refinement,
structural schedule/KIR, complete-plan, and artifact identities remain explicit
obligations of their owning tickets rather than implemented support. The first
public static-extent `tiler_ir::index` builder/verifier is implemented, audited,
and accepted. Its scalar-provider and authority boundary was explicitly
reviewed at implementation commit `32e190f`; it uses generic typed scalar SSA,
canonical attributes, optional static fact views, complete semantic
type-authority evidence, and a separate region-bound scalar-authority receipt.
Semantic lowering equivalence remains a separate checked
capability/refinement obligation. The private compiler proof still has
graph-specific occurrence, normalization, fusion, scheduling, KIR, explain,
and program structures; those are evidence fixtures, not the public compiler.

## Authorized prototype

Tom selected and authorized the bounded strict serial `f32` `Sum` Metal value
proof in [ADR 0055](decisions/0055-use-a-serial-sum-for-the-first-metal-value-proof.md).
The prototype must pass through the documented semantic, reference, optimizer,
schedule, structured-kernel, artifact, and guarded-runtime boundaries; a
handwritten standalone Metal kernel is insufficient. ADR 0065 supersedes ADR
0056's four-crate count by extracting `tiler-reference`; ADR 0067 supersedes
the stable Rust 1.89 floor and selects the exact `nightly-2026-07-19` pin plus
one dependent-array `StaticShape<RANK, EXTENTS>` family. The retained
[nightly shape conformance
harness](../spikes/shapes/nightly-dependent-static-shapes/README.md) now passes
on the governed and adjacent compilers. The checked shaped-value layer now adds
sealed exact/rank evidence, graph-owned same-shape witnesses, and transactional
evidence propagation through the existing builder admission path. The
assembled
[`semantic/reference slice`](../tickets/prototype-semantic-reference-slice.md),
[`materialized target-neutral baseline`](../tickets/prototype-target-neutral-baseline-slice.md)
and
[`target-neutral fusion selection`](../tickets/prototype-target-neutral-fusion-slice.md)
are complete for their bounded claims. The
[`shared compiler IR ownership`](../tickets/prototype-shared-compiler-ir-ownership.md)
slice establishes the accepted module, verifier, builder, and dependency
direction without publishing the graph-specific proof structs.
The semantic identity, canonical index-region, and [`verifier subject-binding`](../tickets/harden-compiler-verifier-subject-binding-and-totality.md) corrections are integrated.
The immediate compiler frontier is the three dependency-satisfied authorities that may proceed in parallel: [`typed explain`](../tickets/prototype-typed-explain-infrastructure.md), [`operation capability registration`](../tickets/prototype-operation-capability-registry.md), and the [`generic index-region reference oracle`](../tickets/prototype-index-region-reference-oracle.md).
Downstream work is split into bounded normalization, generic region formation, checked semantic/index refinement, fusion legality, legal region covers, target feasibility, scheduled-region verification, physical frontiers, complete physical-plan selection, structured KIR, target-neutral kernel programs, artifact-facing programs, and conformance. A reviewed
[`public compiler boundary`](../tickets/prototype-public-compiler-api.md)
follows conformance and is consumed by the inline frontend. Backend feasibility
spikes may proceed when their actual verifier and IR prerequisites exist; they
do not depend on that ergonomic facade merely for ordering.

This chain is an architectural value proof. Separate tickets now track the
neutral artifact codec, Metal lowering and offline driver, runtime validation
and one-way commit, inline proc macro and expansion cache, artifact-family
delivery, embedding measurements, and Candle integration. Completion of the
serial Metal proof still does not by itself complete the inline-DX exit for
Milestone 0B or the broader Milestone 2 product profile.

## Evidence boundary

- Executable spikes validate specific representations, protocols, numerical
  cases, and toolchain behavior.
- Apple compatibility is bounded to measured toolchains and hosts; old-OS and
  device-family runtime coverage is not universal.
- Sound region-accuracy analysis is feasible for a narrow trusted-analyzer
  profile; independent certificate checking remains deferred.
- Multi-device, external-storage, and richer transfer work is deliberately
  deferred, not silently ready.

## Live work state

Ticketsplease is authoritative for current workflow state:

```sh
tkt rollup
tkt ready
tkt tracks
tkt reconcile
```

See the [work-tracking guide](work-tracking.md) before claiming work. The
[roadmap](roadmap.md) describes proposed progression, not completed support.
