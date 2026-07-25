---
schema: "tiler-doc/v1"
id: "ADR-0071"
kind: "decision"
title: "Use checked builders for shared compiler IR"
topics: ["ir", "rust", "api", "verification"]
catalog_group: "physical-planning-lowering"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.architecture", "tiler.contract.ir", "tiler.contract.artifact-abi"]
evidence: ["tiler.research.semantic-graph.rust-construction-lifecycle", "tiler.research.kernel-ir.structured-kernel-ir-verifier"]
refines: ["ADR-0070"]
ticket: "prototype-shared-compiler-ir-ownership"
---

# 0071: Use checked builders for shared compiler IR

**Status:** accepted. Its "Implementation boundary" was rewritten on 2026-07-25, after four of the things it described as unimplemented landed, into a clause-by-clause maturity record. Superseded statements are marked in place rather than deleted, and clauses the tree has not reached are named there instead of being left for a reader to discover. One Decision clause was amended later the same day: artifact decoding reconstructing values through the same builders is now conditional on decoding yielding an IR value, because the unconditional reading describes a mechanism that cannot be built from bytes. The guarantee beside it is untouched, no other clause moved, and `decision_status` is unchanged; the amendment is marked at the clause and evidenced in the boundary.

## Context

Public fields or unchecked constructors would let external producers forge
cross-layer references and pass malformed index, schedule, kernel, or program
objects to a backend. Restricting all construction to `tiler-compiler` avoids
that failure but makes the compiler privileged and fails to prove Tiler's
consumer-independent toolkit boundary.

## Decision

Each shared target-neutral IR layer exposes a public transactional builder with
private storage. Insertion checks local invariants. Consuming `build()` performs
whole-object verification and returns an opaque immutable verified product or
a typed failure containing diagnostics and recoverable builder ownership.
Closure-based convenience construction delegates to the same builder and
verification path; `build`, not `freeze`, is the terminal vocabulary.

Only `VerifiedIndexRegion`, `VerifiedScheduledRegion`, `VerifiedKernel`,
`VerifiedKernelProgram`, and `VerifiedProgramPortfolio` cross compiler,
backend, artifact, or third-party producer boundaries. Read-only accessors and
iterators expose meaning without exposing arena storage. Verified wrappers do
not implement mutation, unchecked construction, thawing, or mutable access to
their underlying drafts.

Every durable reference is an opaque layer-specific newtype backed by a
checked compact `u32` index. Newtypes live with their domain; there is no
generic identifier module or microcrate. Canonical identity is independent of
transient arena numbering and insertion order wherever the represented
semantics are equivalent.

Each verified structural layer retains the exact identity of the lower
structural layer it refines: schedule to index region and kernel to schedule.
A compiler-owned checked refinement separately binds index structure to a
semantic-region occurrence and exact graph-value mappings; a complete program
binds verified implementations to graph coverage. This separation is governed
by ADR 0072. Artifact decoding reconstructs any IR value it yields through the
same IR builders and verifiers; deserialization cannot manufacture a verified
value or maintain a second verifier authority.

*Amended 2026-07-25 by [`settle-adr-0071-artifact-decoding-through-ir-builders`](../../tickets/settle-adr-0071-artifact-decoding-through-ir-builders.md). The first clause previously read that artifact decoding reconstructs values through the same IR builders and verifiers, with no condition, which reads as an obligation to reconstruct. It is now conditional on decoding yielding an IR value at all. The guarantee that follows the semicolon is unchanged and is the durable half. The Implementation boundary below records the evidence, what the implemented profile does instead, and which ticket owns whether reconstruction should ever exist.*

## Consequences

- External plan producers use the same invariant-preserving path as the Tiler
  compiler.
- Negative compile tests can prove that unverified values and raw identifiers
  cannot cross layer boundaries.
- Recoverability extends ADR 0058's semantic-builder lifecycle principle to
  later IRs without changing the existing semantic API.
- The additional builder and verified-wrapper types are deliberate correctness
  machinery rather than alternate representations.

## Implementation boundary

This section records the maturity each clause above has reached, read at `43f685f`. It is a status record and, with one marked exception, not a second decision: a clause named unrealized below is still accepted and still owed. Implemented support, a tested guarantee, and a type-system reservation are kept apart deliberately.

The exception is the artifact-decoding entry below. Every other entry here reports where the tree stands against an unchanged Decision; that one found a Decision clause whose mechanism cannot be implemented as written, so the clause was amended above and this section carries the evidence rather than filing an unmeetable debt. The amendment narrows a mechanism and preserves the guarantee verbatim, and `decision_status` is unchanged.

**Superseded 2026-07-25 — "Schedule, kernel, and program builders remain unimplemented."** All three exist and all three are on the compile path, not merely declared.

- `tiler_ir::schedule` carries `ScheduledRegionBuilder` with private storage, per-insertion checks, and a consuming `build()` returning an opaque `VerifiedScheduledRegion` or a `ScheduledRegionBuildError` whose `diagnostics()` and `into_parts()` recover the intact builder. [`crates/tiler-compiler/src/physical.rs`](../../crates/tiler-compiler/src/physical.rs) verifies every candidate schedule through it and wraps the result in its own crate-private newtype rather than reimplementing verification.
- `tiler_ir::kernel` carries `VerifiedKernel` and the recoverable `KernelVerificationError`; the same compiler module lowers each verified schedule into one.
- `tiler_ir::program` carries `VerifiedKernelProgram` and the recoverable `KernelProgramVerificationError`, driven from [`crates/tiler-compiler/src/program.rs`](../../crates/tiler-compiler/src/program.rs) and consumed by `tiler-artifact`.

**Superseded 2026-07-25 — "The accepted closure convenience is not part of this first implementation."** [`add-checked-closure-convenience-for-shared-ir-builders`](../../tickets/add-checked-closure-convenience-for-shared-ir-builders.md) landed `IndexRegionBuilder::build_with`, which constructs the builder, runs an authoring closure holding only `&mut`, and consumes it through the same `build()` verifier, so the mutable draft never escapes. Its error composition is the crate-root generic `tiler_ir::CheckedBuildError<Admission, Verification>`, which keeps an admission failure and a recoverable whole-object verification failure distinct without erasing either. **The convenience exists for the index layer only.** The schedule, kernel, and program builders expose no `build_with`; what generalizes today is the error type and a crate-private combinator, which is a reusable shape rather than a delivered convenience on those layers.

**Retained and still exact.** The first public checked-builder implementation is the static-extent `tiler_ir::index` profile. The implemented index verifier accepts ordered typed tensor boundaries but derives no semantic-program or semantic-region correlation identity. Its borrowed static-shape and static-extent views are explicitly optional so the future `ShapeEnv` profile can add symbolic expressions without changing a static fact into a false universal guarantee.

**Retained and still exact.** A selected frozen scalar registry can separately revalidate every reached scalar application and produce region-bound authority evidence containing a provider-independent definition projection and distinct provider-attributed admission provenance. This receipt authenticates the scalar definitions and stored result types used by one exact structural region; it still does not prove that the region implements a selected semantic operation. Operation capabilities and compiler legality evidence must separately bind coordinates, values, effects, and numerical behavior to the authoritative semantic source. Verified index structure and scalar-authority evidence are not semantic-equivalence evidence.

**Amended premise 2026-07-25 — the `ShapeEnv` sentence.** The clause read that symbolic extent roots and index-domain predicates remain follow-up work "because the accepted `ShapeEnv` authority does not yet exist". Half of it now does: [`crates/tiler-ir/src/shape/env.rs`](../../crates/tiler-ir/src/shape/env.rs) implements scoped symbol declarations and typed root bindings as a `pub(crate)` ADR 0074 convention 7 draft, with the constraint environment split out rather than stubbed. The conclusion is unchanged and the reason it survives is stronger than the original premise: nothing on the compile path constructs one, the module carries a crate-level `#![allow(dead_code)]` whose reason says exactly that, and the index module still invents no competing binding system. What is no longer accurate is "does not yet exist"; the exact statement is that the symbol-and-binding half exists, is unreachable outside `tiler-ir`, and has no consumer. `implement-shapeenv-core`, `implement-shapeenv-constraints`, and `implement-shapeenv-index-bindings` own the rest.

**Unrealized clause — `VerifiedProgramPortfolio`.** Decision names five verified types that cross compiler, backend, artifact, or third-party producer boundaries. Four exist in `tiler-ir`. There is no `VerifiedProgramPortfolio` anywhere in the workspace; `ProgramPortfolio` at [`crates/tiler-compiler/src/pipeline.rs`](../../crates/tiler-compiler/src/pipeline.rs) is `pub(crate)` and is not it — a compiler-internal aggregate is not a verified IR product with a checked builder, and reading the name as the clause's subject would report a reservation as implemented support.

**Partially realized clause — retained lower-layer identity.** Decision requires each verified structural layer to retain the exact identity of the layer it refines, "schedule to index region and kernel to schedule". Kernel to schedule is realized: `crates/tiler-ir/src/kernel/model.rs` stores a `schedule_identity: CanonicalScheduledRegionIdentity` on the verified kernel, exposes it, and folds its bytes into `CanonicalKernelIdentity`. Schedule to index region is **not** realized. `crates/tiler-ir/src/schedule/model.rs` declares its own `IndexRegion` struct — a distinct type from `tiler_ir::index::VerifiedIndexRegion`, with different content and a separate canonical identity — and `encode_identity` folds that struct's *content* into `CanonicalScheduledRegionIdentity` without ever referencing `CanonicalIndexRegionIdentity`. The exact check is `grep -rn 'crate::index' crates/tiler-ir/src/schedule/`, which returns one line, a doc-comment cross-reference in `error.rs`; the schedule module reaches the index module in no code path. So `tiler-ir` currently carries two index-region representations and the schedule layer refines the one it declares itself. [`bind-the-scheduled-region-to-the-verified-index-region-identity`](../../tickets/bind-the-scheduled-region-to-the-verified-index-region-identity.md) owns it.

**Superseded 2026-07-25 — "artifact decoding through the same builders" as an unconditional mechanism.** The Decision clause read "Artifact decoding reconstructs values through the same IR builders and verifiers" with no condition attached, and the boundary previously recorded that mechanism as unimplemented and owed. It is not owed, and it is not implementable as stated. The clause is now conditional and this entry records the evidence.

**Fact — decoding yields a validated read view, and no IR value.** [`crates/tiler-artifact/src/program/codec/view.rs`](../../crates/tiler-artifact/src/program/codec/view.rs)'s `decode_artifact` validates framing, manifest and section digests, component schemas, canonical order, and arena closure, re-derives identity from the decoded content rather than reading it from the manifest, and returns a `DecodedArtifact`. The dependency runs the other way from the clause's reading: `ArtifactProgramBuilder` consumes an already-verified `VerifiedKernelProgram` and projects it into the envelope. The exact check is `grep -rn 'KernelProgramBuilder\|ScheduledRegionBuilder\|IndexRegionBuilder\|SemanticProgramBuilder' crates/tiler-artifact/src/`, whose every match is a doc comment or a test fixture that *encodes*; `codec/decode.rs`, `codec/validate.rs`, and `codec/view.rs` name no IR builder at all.

**Fact — the mechanism is structurally unreachable from bytes, not merely unbuilt.** [`crates/tiler-ir/src/program/builder.rs`](../../crates/tiler-ir/src/program/builder.rs)'s `KernelProgramBuilder::new` takes a `&SemanticProgram`, and a `SemanticProgram` is built against a frozen registry holding `Arc<dyn OperationInferencer>`. [`crates/tiler-ir/src/semantic/operation.rs`](../../crates/tiler-ir/src/semantic/operation.rs) declares `OperationInferencer` as a trait whose `infer` derives result facts — behaviour, not data. No byte encoding produces one, so no decoder can drive that builder from an envelope alone without a consumer-supplied registry. Separately, the envelope does not carry the program to begin with: a variant's program reaches it as canonical identity bytes, so a decode proves *which* program an artifact names and cannot resurrect it.

**Inference — the guarantee is met more strongly than the clause described, and stating that is the point of the amendment.** Deserialization cannot manufacture a verified value because it manufactures no IR value at all, and a decoded envelope is a validated read view rather than a second editable authority. That is a stronger position than sharing a verifier, not a weaker one. Recording it as an unimplemented mechanism misreported a deliberate and structurally forced boundary as a debt, which is the confusion between a type-system reservation, implemented support, and a tested guarantee this section exists to prevent.

**What survives as a live constraint.** The amended clause is not decoration. Any future path that yields an IR value out of an artifact must go through `tiler-ir`'s builders and consuming verifiers, and may not introduce a second verifier authority; a decoder that returned a verified product it had not driven through them would violate the clause exactly as before. Because a builder needs a registry that bytes cannot carry, such a path would also have to state the registry dependency at its own API boundary rather than hide it.

**What this entry deliberately does not decide.** Whether the envelope should ever carry a reconstructable program encoding, or whether a decoded artifact is permanently a dispatch record validated against an identity digest, belongs to [`carry-reconstructable-kernel-programs-in-the-neutral-envelope`](../../tickets/carry-reconstructable-kernel-programs-in-the-neutral-envelope.md), which weighs both and also owns reconciling [the artifact ABI contract](../artifact-abi.md) — that document still states the reconstruction requirement normatively while separately recording that the implemented profile does not meet it, and this record does not resolve that divergence from outside its scope. Nothing needs an IR value back today: `prototype-runtime-artifact-validation` is blocked on that same ticket and records that three of its four deliverables fall out of `decode_artifact` while the fourth is reachable by binding-by-identity, which reconstructs nothing; and the expansion cache is keyed by artifact identity and validates and embeds bytes on every hit.

**Consequence with a narrower reach than it reads.** "Negative compile tests can prove that unverified values and raw identifiers cannot cross layer boundaries" is realized for the index layer alone. `crates/tiler-ir/tests/` carries three `trybuild` suites — `index-region`, `shape-evidence`, and `typed-handles` — and none for schedule, kernel, or program. Those three layers' verified products are opaque by construction: private fields with `pub(super)` constructors, which is implemented support. That no out-of-crate forgery of them compiles is not a tested guarantee.

**`implementation_status` stays `partial`.** Four builders, four verified products, the closure convenience on one layer, and both recoverable error boundaries are implemented and exercised. One named verified type does not exist and one identity-retention edge is missing, so the record's own decided behaviour has not reached `implemented`. The artifact-decoding clause no longer counts against it: as amended it is satisfied by the implemented profile, which yields no IR value and therefore reconstructs none outside the builders.

## Alternatives considered

Read-only compiler output postpones the extension boundary. Public data
structures permit invalid states and make compatibility depend on storage
layout. Unchecked constructors rely on convention precisely where artifacts
and backends require proof.
