---
id: settle-adr-0071-artifact-decoding-through-ir-builders
title: Settle whether artifact decoding reconstructs IR through the same builders
status: todo
priority: p2
dependencies: []
related: [update-adr-0071-schedule-builder-boundary, prototype-runtime-artifact-validation, carry-the-metal-payload-in-an-artifact-envelope]
scopes: [contracts/decisions, implementation/artifact]
shared_scopes: []
paths: []
tags: [documentation, decisions, artifact, ir]
---
ADR 0071's Decision states: "Artifact decoding reconstructs values through the same IR builders and verifiers; deserialization cannot manufacture a verified value or maintain a second verifier authority." The first half is unimplemented and the second holds vacuously, and the two must be separated before anything depends on either.

**Fact — what decoding does today.** `crates/tiler-artifact/src/program/codec/view.rs`'s `decode_artifact` validates framing, manifest and section digests, component schemas, canonical order, and arena closure, and re-derives identity from the decoded content rather than reading it from the manifest, so a forged manifest cannot present a chosen identity. It returns a `DecodedArtifact`, a validated read view over an `ArtifactEnvelope`.

**Fact — what it does not do.** It reconstructs no `VerifiedKernelProgram`, and no path in `tiler-artifact` calls a `tiler_ir::program`, `tiler_ir::kernel`, or `tiler_ir::schedule` builder. The dependency runs the other way: `ArtifactProgramBuilder` consumes an already-verified `VerifiedKernelProgram` and projects it into the envelope. Decoding is a validated read, not a rebuild.

**Inference — why the guarantee is currently free.** "Deserialization cannot manufacture a verified value" is true because deserialization manufactures no IR value at all. That is a stronger position than the clause describes, not a weaker one, and it costs nothing while every consumer of a decoded artifact wants bytes and facts rather than IR. It stops being free the moment something needs an IR value back out of an artifact — a cache rehydrating a plan, a runtime re-validating a program against its semantic source, or a cross-process compile boundary.

## The decision this ticket owns

One of two outcomes, not a third that leaves the clause as decorative text.

**Implement the mechanism.** Decoding gains a path that feeds decoded content through `tiler_ir`'s builders and consuming verifiers, so a decoded program is verified by the same authority that verified the original. The cost is that the artifact encoding must be complete enough to reconstruct every builder input, which is a real constraint on the envelope schema and probably on what the artifact stores; measure that before committing to it.

**Supersede the clause and state the stronger property instead.** ADR 0071 would record that artifact decoding produces a validated read view and deliberately reconstructs no verified IR value, that the no-second-authority guarantee is met by having no reconstruction path rather than by sharing one, and that any future reconstruction must go through the IR builders — which converts the clause from an unimplemented mechanism into a live constraint on future work. This is the cheaper outcome and is probably correct today; do not adopt it without checking whether `prototype-runtime-artifact-validation` or the expansion cache needs an IR value back.

The scope carries both `contracts/decisions` and `implementation/artifact` because the second outcome is a decision edit alone while the first is artifact work; drop whichever the answer does not need rather than holding both through the branch.

## Closes when

ADR 0071's "Unrealized clause — artifact decoding through the same builders" paragraph in its Implementation boundary is replaced by either an implemented mechanism or an explicit in-body supersession of the Decision sentence, the renderer has run, and the full gate passes.
