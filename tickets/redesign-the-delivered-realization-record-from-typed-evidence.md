---
id: redesign-the-delivered-realization-record-from-typed-evidence
title: Redesign the delivered-realization record from typed evidence
status: todo
priority: p1
dependencies: [carry-the-honourability-fact-provenance-into-the-artifact-record]
related: [record-delivered-numerical-realization, drive-the-build-orchestrator-from-a-checked-compiler-plan, widen-the-region-realization-to-consumable-dimensions]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/build, contracts/numerics, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [design, artifact, numerics, provenance, api]
---
## User-visible outcome

Tom receives a compile-checked, review-ready design for a required delivered-realization record. The implemented record family represents the compiler-produced scalar-arithmetic policy subject completely over its eleven governed dimensions, preserves canonical full resolved-type identity and structured provenance, and reserves a versioned seam for future operation- or scheme-owned contract families without manufacturing scalar-arithmetic claims for integers, booleans, quantized values, complex values, decimal values, MX values, or owner-namespaced types.

## Why the staged draft must be replaced

- **Fact:** `crates/tiler-artifact/src/program/realization.rs` declares a second `NumericalDimension` with four cases and fixed fields. The compiler authority has eleven cases, while the widened scheduled realization and artifact codec carry the eight dimensions currently consumable by admitted operations.
- **Fact:** compiler honourability is keyed by `(NumericalDimension, ArithmeticType)`. The measured Apple profile preserves `f16` input subnormals and flushes `f32` input subnormals, so `honoured(dimension)` cannot return one correct answer.
- **Fact:** `HonouringMeans::SupportedOnlyUnderDeclaredRelaxation` carries a specific relaxation subject, but `HonouringMeans::key()` collapses every such value to the same text. Opaque key bytes therefore cannot tell a reader which relaxation made a requirement honourable.
- **Fact:** ADR 0076 requires availability phase, authority, validity scope, declaring profile, compiler build, and execution environment. The draft carries only phase and one record-level profile.
- **Fact:** `DeliveredRealizationBuilder::declare` accepts arbitrary bytes and a caller-selected phase. It validates framing, not that a checked compiler plan selected the claim.
- **Inference:** publishing the draft would freeze an incomplete and already-contradicted contract. The dtype key, structured means, provenance, dimension authority, constructor, readers, canonical encoding, and identity all require a coherent replacement.

## Required design

### Compiler-produced policy subjects, never inferred applicability

Define one shared, exhaustive numerical-dimension authority for the eleven governed scalar-arithmetic dimensions. The dimension is numerical-contract vocabulary; the target-aware means of honouring it remains outside semantic IR. Remove the artifact-local and compiler-local lists as independent authorities, and keep exhaustive tag mappings so a new dimension breaks every total encoder and consumer at compile time.

Key the implemented contract by a checked compiler-produced numerical-policy subject. Its first and only implemented subject kind is `ScalarArithmetic`: it carries the canonical full `ResolvedValueType` identity of one dtype-wide arithmetic contract plus the eleven-dimension schema. `TypeKey` alone is insufficient because parameters and ordered encoded components distinguish resolved types within one definition family. The subject cannot be inferred by scanning graph value types, recursively walking encoded components, or guessing from storage width.

Keep operation-specific requirements distinct from that dtype-wide ceiling. A canonical `NumericalObligationKey` identifies the program occurrence and policy locus—input, computation, accumulator, result, component, or materialization—that produced the requirement. One `(subject, dimension)` disposition is therefore `NotRequired` or references a non-empty canonical range of obligation rows, not one evidence row. Each obligation row carries its own required behaviour and evidence reference, so two f32 loci with different legal requirements never collapse merely because they share a type.

Reserve a versioned/tagged record-family seam rather than a universal dtype enum. A future integer, boolean, complex, decimal, quantized, MX, conversion, or owner-defined contract family arrives only with its first producer, consumer, behaviour schema, validation, identity, and lowering evidence. Subject existence follows the checked request's selected governed contract, not the number of obligations: a selected scalar-arithmetic contract always produces one complete subject, even when every dimension is `NotRequired`. A recognized semantic type not governed by any selected scalar contract does not create an additional subject merely by appearing in the program.

### Complete contract and complete assessment coverage

For every scalar-arithmetic subject emitted by the checked compiler plan, carry the resolved contract complete over all eleven dimensions. Separately carry the canonical union of honourability obligations relied upon by every packaged executable variant and stage that routing may select. Never use runtime language such as “actually exercised”: the artifact exists before a route executes.

Every dimension has an explicit assessment disposition: either compiler-produced `NotRequired` for every packaged route or `Required` with a canonical non-empty obligation range. This refines ADR 0076 item 4's earlier “each dimension's means” wording without turning an unconsumed dimension into a fabricated target fact. The review packet includes exact proposed ADR and numerical/artifact contract text; it does not make that proposal operative before ratification.

Each obligation row carries the exact policy subject, dimension, locus key, required behaviour, and evidence reference. Each referenced evidence row carries the declared target behaviour, structured honouring means including any declared-relaxation payload, availability phase, measured-fact authority, validity scope, declaring profile, compiler-build identity, and execution-environment identity. The target/profile declarer produces the fact; the compiler validates and carries the selected evidence rather than becoming its authority.

### Compact canonical representation

Store a canonical sorted slice of versioned subject records. The `ScalarArithmetic` record stores the resolved type identity once and its eleven resolutions and assessment dispositions in dense dimension-indexed arrays whose index conversion is one exhaustive shared match. Store obligation rows as a canonical sorted sparse slice keyed by exact subject, dimension, and locus key; store deduplicated target evidence separately and reference it canonically.

The typed producer builder accepts declarations in arbitrary order, validates them, sorts once, and rejects duplicates. Canonical decode rejects out-of-order or duplicate wire rows; producer call sites do not have to reproduce wire ordering. Decoded views borrow contiguous rows without per-read allocation. Lookup remains allocation-free; choose linear, partitioned, or binary sparse lookup from a targeted benchmark rather than assuming asymptotic performance dominates at the current one-subject scale.

### Honest producer and trust boundary

Specify a borrowed, typed compiler view of the resolved subjects, assessment coverage, and selected evidence. Specify one exhaustive `tiler-build` translation into the artifact representation. The translation matches every subject, dimension, disposition, structured means, and provenance variant; it never reconstructs evidence from flags, target names, neighbouring dtypes, profile digests, or outer value shape.

The artifact builder validates internal consistency and provenance but cannot provide authenticity or re-run compiler consumption analysis: an untrusted producer can write a self-consistent assertion, including a false `NotRequired`. Decode verifies integrity, canonical coverage, references, and associations; it does not upgrade producer assertions into independently proved semantics. Document that boundary. Ordinary checked production consumes the compiler-selected obligations and evidence through `tiler-build`; any retained low-level construction seam accepts typed producer assertions and is named accordingly.

The design must state every cross-check and its proving layer. The compiler proves the policy subject, obligation loci, required behaviours, and `NotRequired` claims from the checked plan. `tiler-build` proves the translated subject and obligation references agree with that compiler view. Artifact construction and decode prove the record's profile equals the artifact subject, every packaged entry/variant references an existing policy subject, and the record's eight overlapping scalar resolutions equal every entry's existing widened `NumericalFacts`; a mismatch rejects. Where the neutral artifact cannot independently derive the arithmetic type from its dispatch record, it validates the encoded association and documents that the compiler/build producer proved its semantic meaning.

Unknown record-family, family-schema, subject-kind, dimension, disposition, means, provenance, or locus tags reject fail-closed. Extensibility means a newer reader implements and validates a new family; an older reader never skips an unknown numerical family while still calling the executable artifact validated.

### Review workflow without crossing the boundary

This ticket owns a compile-checked design packet or bounded spike with exact proposed public signatures, canonical ordering, validation rules, failure vocabulary, and representative call sites. It does not promote production visibility, wire the production builder/decoder, advance schema or identity domains, or rebaseline production pins.

`accept-the-delivered-realization-artifact-surface` reviews the exact proposed IR/compiler/artifact/build boundary after this ticket is done. `wire-the-delivered-realization-record-into-the-artifact` implements the ratified boundary, required terminal state, compiler-to-build-to-artifact path, codec, readers, identity/schema changes, and merged-tree rebaselines.

## Required evidence

- One scalar-arithmetic subject fixture covers all eleven resolved dimensions, `NotRequired`, one required obligation, and multiple distinct required loci on one `(type, dimension)`.
- One profile carries different `f16` and `f32` subnormal evidence without collision.
- Two declared-relaxation means differing only in relaxation payload remain distinct.
- The producer builder accepts shuffled declarations and canonicalizes them; duplicate declarations reject.
- Canonical decode rejects shuffled, duplicate, missing, malformed, profile-mismatched, overlapping-realization-mismatched, dangling-obligation/evidence, unknown-tag, behaviour-mismatched, and incomplete-provenance rows with typed causes.
- Recognized bool, integer, complex, decimal, strict-affine encoded, and MX values do not manufacture additional scalar-arithmetic subjects merely by appearing in a program.
- A selected scalar contract with zero obligations still produces a complete eleven-dimension subject whose dispositions are all `NotRequired`; an unsupported owner-namespaced type cannot create another subject except through checked producer evidence.
- The identity codec distinguishes nominal, parameterized, and encoded full resolved-type identities without claiming those types inhabit the scalar-arithmetic schema.
- Every proposed validation check is perturbed once and observed failing before restoration.
- The design/spike's exact invocation passes together with `tkt lint` and `git diff --check`.

## Closes when

The review packet eliminates the stale draft, defines the exact signatures and call sites across the shared dimension, compiler evidence view, artifact record, and build translation, includes the exact proposed ADR 0076 and numerical/artifact contract corrections, demonstrates the compact representation and adversarial validation in a compile-checked bounded spike or equivalent private draft, and leaves applying those contract changes plus every production public item and wire/schema/identity change for acceptance and downstream implementation.

## Graph maintenance

- This ticket follows `carry-the-honourability-fact-provenance-into-the-artifact-record`, whose producer-side facts must exist before this design can require them.
- `accept-the-delivered-realization-artifact-surface` depends on this review packet and owns Tom's exact public-boundary ratification.
- `wire-the-delivered-realization-record-into-the-artifact` follows acceptance and owns all production integration.
- Qualify `record-delivered-numerical-realization` as historical evidence: its staged four-dimension outcome was valid for its tree and is not the current candidate.
- Keep `express-metal-honourability-in-the-shared-form` and its caller-profile decision visible as the upstream gate through the provenance dependency.
