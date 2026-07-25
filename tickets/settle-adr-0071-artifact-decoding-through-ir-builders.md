---
id: settle-adr-0071-artifact-decoding-through-ir-builders
title: Settle whether artifact decoding reconstructs IR through the same builders
status: done
priority: p2
dependencies: []
related: [update-adr-0071-schedule-builder-boundary, prototype-runtime-artifact-validation, carry-the-metal-payload-in-an-artifact-envelope]
scopes: [contracts/decisions]
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

## Outcome

**Outcome 2, with the clause narrowed rather than deleted.** `implementation/artifact` was dropped from this ticket's scopes, as the ticket instructed, because the answer needs no artifact work. Only `docs/decisions/0071-use-checked-builders-for-shared-compiler-ir.md` changed. `decision_status` is untouched at `accepted`.

### Why outcome 1 was rejected, on evidence stronger than the ticket anticipated

The ticket frames "implement the mechanism" as costly — the encoding would have to be complete enough to reconstruct every builder input — and asks for that to be measured before committing. Measuring it found something stronger: the mechanism is not implementable as stated at all, at any encoding cost.

`tiler_ir::program::KernelProgramBuilder::new` takes a `&SemanticProgram`; read at `crates/tiler-ir/src/program/builder.rs:75`. A `SemanticProgram` is built against a frozen registry holding `Arc<dyn OperationInferencer>`, and `crates/tiler-ir/src/semantic/operation.rs:1160` declares `OperationInferencer` as a trait whose `infer` derives result facts from a request. That is behaviour, not data. No byte encoding produces one, so no decoder can drive the program builder from an envelope alone; it would need a consumer-supplied registry, which makes decoding not self-contained. Widening the envelope schema does not reach this — the missing input is code.

The envelope also does not carry the program in the first place. A variant's program reaches it as canonical identity bytes, so a decode proves which program an artifact names and cannot resurrect it. Both halves were read at the source rather than taken from the codec's doc comments, which state the same conclusion.

### The check the ticket required before adopting outcome 2

Neither named consumer needs an IR value back.

`prototype-runtime-artifact-validation` is `blocked`, and its own Blocker 2 records that three of its four deliverables — device-free decoding, integrity validation, typed compatibility classification — fall out of `decode_artifact` plus `identity()`, `features()`, and `payloads()`. The fourth, checked expression evaluation, needs either a promoted dispatch-record projection on `DecodedArtifact` or binding-by-identity, in which the runtime holds the `VerifiedArtifactProgram` it compiled and uses the decoded identity to prove the loaded bytes name it. Neither is IR reconstruction, and `carry-reconstructable-kernel-programs-in-the-neutral-envelope` records that the binding-by-identity path is already implementable with no new public surface.

The expansion cache does not either. ADR 0050 has readers validate framing, embedded key, schemas, manifest, section lengths and digests, and required meanings on every hit, and `docs/artifact-abi.md`'s cache contract publishes an immutable bundle per compilation key. It is keyed by artifact identity and consumes validated bytes; nothing in the protocol needs a `VerifiedKernelProgram`.

### What landed

The Decision clause is now conditional: decoding reconstructs any IR value *it yields* through the same builders and verifiers. The guarantee after the semicolon is unchanged and is the durable half. An italic amendment marker sits under the Decision paragraph recording the previous unconditional wording and pointing at the evidence.

The boundary paragraph became a `Superseded 2026-07-25` entry carrying four labelled paragraphs: the Fact that decoding yields a read view and no IR value, with the reproducible grep and the note that `codec/decode.rs`, `codec/validate.rs`, and `codec/view.rs` name no IR builder; the Fact that the mechanism is structurally unreachable; the Inference that the guarantee is met more strongly than the clause described, so recording it as a debt misreported a forced boundary; and what survives as a live constraint — any future path yielding an IR value must go through the builders, may not add a second verifier authority, and must state its registry dependency at its own API boundary.

Two framing statements the change made stale were repaired rather than left. The section preamble asserted that nothing in Decision changes in it; it now names this one marked exception and says why. The `implementation_status` paragraph counted this clause as an unimplemented Decision mechanism; it no longer does, because as amended the implemented profile satisfies it. `implementation_status` stays `partial` on the two clauses that genuinely remain — no `VerifiedProgramPortfolio` exists, and schedule-to-index-region identity retention is missing.

### What was deliberately not decided, and one divergence left standing

Whether the envelope should ever carry a reconstructable program, or whether a decoded artifact is permanently a dispatch record validated against an identity digest, is `carry-reconstructable-kernel-programs-in-the-neutral-envelope`'s decision. This edit is compatible with either: it records what the implemented profile does and constrains any future reconstruction, and it does not forbid one. The ADR says so in those words, so a reader cannot mistake the amendment for that ticket's answer.

`docs/artifact-abi.md` still states the reconstruction requirement normatively at its ownership boundary while separately recording, under "Where the implemented profile is narrower than this contract", that the implemented profile does not meet it. That divergence from the amended ADR is real and is left standing: the file is `contracts/artifacts`, which this ticket does not hold, and the same `carry-reconstructable-...` ticket already owns making that sentence true or amended. The ADR names the divergence and its owner rather than quietly leaving two contracts disagreeing.

### Gate

`uv run --locked python scripts/docs.py render` passed (183 records), regenerating no catalog line since no frontmatter that the catalogs project changed. `uv run --locked python scripts/check_repository.py` passed complete, including the Rust sub-gate. `git diff --check` clean and `tkt lint` reports no problems.
