---
schema: "tiler-doc/v1"
id: "ADR-0071"
kind: "decision"
title: "Use checked builders for shared compiler IR"
topics: ["ir", "rust", "api", "verification"]
catalog_group: "physical-planning-lowering"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.architecture", "tiler.contract.ir", "tiler.contract.artifact-abi"]
evidence: ["tiler.research.semantic-graph.rust-construction-lifecycle", "tiler.research.kernel-ir.structured-kernel-ir-verifier"]
ticket: "prototype-shared-compiler-ir-ownership"
---

# 0071: Use checked builders for shared compiler IR

**Status:** accepted

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

Each verified layer retains the exact identity of the lower layer it refines:
schedule to index region, kernel to schedule, and program stage to verified
kernel implementation. Artifact decoding reconstructs values through the same
IR builders and verifiers; deserialization cannot manufacture a verified value
or maintain a second verifier authority.

## Consequences

- External plan producers use the same invariant-preserving path as the Tiler
  compiler.
- Negative compile tests can prove that unverified values and raw identifiers
  cannot cross layer boundaries.
- Recoverability extends ADR 0058's semantic-builder lifecycle principle to
  later IRs without changing the existing semantic API.
- The additional builder and verified-wrapper types are deliberate correctness
  machinery rather than alternate representations.

## Alternatives considered

Read-only compiler output postpones the extension boundary. Public data
structures permit invalid states and make compatibility depend on storage
layout. Unchecked constructors rely on convention precisely where artifacts
and backends require proof.
