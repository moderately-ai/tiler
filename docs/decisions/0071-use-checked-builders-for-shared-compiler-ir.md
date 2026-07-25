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

**Status:** accepted. The decision is unchanged. Its "Implementation boundary" was rewritten on 2026-07-25, after four of the things it described as unimplemented landed, into a clause-by-clause maturity record. Superseded statements are marked in place rather than deleted, and clauses the tree has not reached are named there instead of being left for a reader to discover.

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
by ADR 0072. Artifact decoding reconstructs values through the same IR builders
and verifiers; deserialization cannot manufacture a verified value or maintain
a second verifier authority.

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

This section records the maturity each clause above has reached, read at `43f685f`. It is a status record and not a second decision: nothing in Decision changes here, and a clause named unrealized below is still accepted and still owed. Implemented support, a tested guarantee, and a type-system reservation are kept apart deliberately.

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

**Unrealized clause — artifact decoding through the same builders.** Decision states that artifact decoding reconstructs values through the same IR builders and verifiers. [`crates/tiler-artifact/src/program/codec/view.rs`](../../crates/tiler-artifact/src/program/codec/view.rs)'s `decode_artifact` validates framing, manifest and section digests, schema, canonical order, and arena closure, and re-derives identity rather than reading it from the bytes — but it returns a `DecodedArtifact` read view and reconstructs no `VerifiedKernelProgram` through `tiler_ir::program`. The clause's *guarantee* — deserialization cannot manufacture a verified value — holds today, and vacuously: decoding manufactures no IR value at all. Its stated *mechanism* is unimplemented, and the two must not be conflated, because the guarantee stops being free the moment anything needs an IR value back out of an artifact. [`settle-adr-0071-artifact-decoding-through-ir-builders`](../../tickets/settle-adr-0071-artifact-decoding-through-ir-builders.md) owns deciding which way that resolves.

**Consequence with a narrower reach than it reads.** "Negative compile tests can prove that unverified values and raw identifiers cannot cross layer boundaries" is realized for the index layer alone. `crates/tiler-ir/tests/` carries three `trybuild` suites — `index-region`, `shape-evidence`, and `typed-handles` — and none for schedule, kernel, or program. Those three layers' verified products are opaque by construction: private fields with `pub(super)` constructors, which is implemented support. That no out-of-crate forgery of them compiles is not a tested guarantee.

**`implementation_status` stays `partial`.** Four builders, four verified products, the closure convenience on one layer, and both recoverable error boundaries are implemented and exercised. One named verified type does not exist, one identity-retention edge is missing, and one Decision mechanism is unimplemented, so the record's own decided behaviour has not reached `implemented`.

## Alternatives considered

Read-only compiler output postpones the extension boundary. Public data
structures permit invalid states and make compatibility depend on storage
layout. Unchecked constructors rely on convention precisely where artifacts
and backends require proof.
