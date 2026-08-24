---
id: decide-the-conditional-coverage-authority-for-invocation-gather-validation
title: Decide the conditional-coverage authority for invocation-gather validation
status: todo
priority: p1
dependencies: [carry-the-invocation-gather-requirement-through-refinement]
related: [admit-an-invocation-scoped-gather-index-validation-receipt, decide-how-a-dynamic-bounds-witness-enters-the-schedule-vocabulary, decide-the-data-dependent-index-representation-public-surface]
scopes: [contracts/decisions, contracts/foundation, contracts/optimizer, research/indexing, research/program-planning, research/scheduling]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [decision, needs-tom, gather, validation, fail-closed, public-boundary, identity]
---
## User-visible outcome

Tiler has one accepted typed authority by which a dynamic gather's invocation requirement can enter conditional schedule/program coverage without becoming timeless proof or dispatch authority, with an exact transition that the artifact layer can serialize and the runtime receipt can later discharge.

## Exact-base Facts — `6e713e12`

- **Fact — only proved gather coverage is representable.** `BoundsProofKind::GatherSource` in `crates/tiler-ir/src/schedule/model.rs` carries a `GatherIndexBoundsProof`; `CoveredOccurrence::from_receipt` in `crates/tiler-ir/src/program/model.rs` accepts a completed `IndexRefinementReceipt`; and no conditional-coverage type or identity exists.
- **Fact — dynamic gather refuses before schedule formation.** `Err(RegionVocabularyWall::GatherIndexBoundsUnproved)` in `crates/tiler-compiler/src/physical.rs` is the current stop. The accepted public packet states at `A valid dynamic gather reaches this outcome only after` that the requirement creates no receipt, coverage identity, `CoveredOccurrence`, schedule, artifact, cache subject, or dispatch, leaving the later transition to the receipt work.
- **Fact — accepted authority fixes meaning but not the carrier.** ADR 0108 permits named conditional coverage while preserving ADR 0109's refusal of arbitrary `Unknown`, but accepts no Rust spelling, schedule identity, executable-program state, or transition into and out of conditional coverage.

## Decision packet

Compare the status quo typed refusal; a separately typed conditional schedule/program carrier; runtime or late planning after validation; and any proposed widening of `BoundsProof`, `IndexRefinementReceipt`, `CoveredOccurrence`, `VerifiedSchedule`, or `VerifiedKernelProgram`. Eliminate any option that labels a requirement as proof, gives a packaged route dispatch authority, requires runtime IR reconstruction/JIT or duplicated compiler authority, weakens self-contained AOT embedding, or lets absence mean coverage.

For every survivor specify:

- the exact types, private/public boundary, constructors, state transitions, identity domains and encoded subjects from compiler refinement through the artifact-facing program;
- how source/index/result occurrence bindings and the region-local requirement stay one subject through schedule association, multi-stage program assembly, explanation and cache keys;
- which existing verified types remain proof-only and byte-identical, and which new conditional subjects are incomparable;
- the exact point at which a future runtime receipt may transform conditional authority, and why it cannot do so twice or for a different requirement; and
- the complete unsupported population and error precedence before the artifact boundary.

Apply the repository's Pareto-complete decision gate and obtain Tom's decision; a broad statement that “the receipt ticket owns it” is not an implementable outcome.

## Closing checks and negative controls

- Enumerate every proof/coverage/schedule/program state from its type and name which dynamic arm each total consumer handles; a wildcard, `Option` absence, or reuse of an existing proof tag is a failed packet.
- Independently cross a requirement into `IndexRefinementReceipt`, `IndexRefinementExecutableCoverageIdentity`, `CoveredOccurrence`, and the existing proved `BoundsProofKind::GatherSource`. For each, show the exact type or validation failure the selected design requires.
- Perturb occurrence, region identity, access, source/index/result binding, extent, type and requirement identity at the compiler-to-schedule and schedule-to-program boundaries; each must be distinguishable or refused before packaging.
- Show the artifact-facing subject is sufficient for the artifact owner without importing compiler-private ordinals or behavior, and show a runtime cannot construct a compiler/schedule authority from serialized bytes alone.
- Record exact accepted names, tags/domains, included/excluded public surface, acceptance provenance and the landed contract hashes in the artifact-obligation and implementation tickets.

## Non-goals

Choosing artifact manifest placement/schema, runtime snapshot ownership, receipt errors, Metal emission, mutable or device validation, inline checks, callbacks, assertions, fallback, or generalization beyond gather. Implementing the selected carrier.

## Closes when

Tom has accepted one exact conditional-coverage and artifact-facing authority, every proof-confusion negative control is answered, identities and unsupported states are complete, and the artifact-obligation ticket cites the exact accepted record rather than inferring one.
