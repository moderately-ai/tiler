---
id: carry-reconstructable-kernel-programs-in-the-neutral-envelope
title: Decide what a decoded artifact envelope must reconstruct
status: todo
priority: p1
dependencies: []
related: [prototype-neutral-artifact-codec, prototype-metal-bundle-assembly]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [artifact, serialization, ir]
---
`prototype-neutral-artifact-codec` framed a neutral program section carrying one packaged variant's *canonical kernel-program identity*, not the program. A decoder therefore proves which program an artifact names and cannot resurrect it.

**Fact — the blocker is structural, not an omission.** `tiler_ir::program::KernelProgramBuilder::new` takes a `&SemanticProgram`, and a `SemanticProgram` requires a frozen `SemanticRegistry` holding `Arc<dyn OperationInferencer>` implementations. Neither is representable as bytes, so no codec can rebuild a `VerifiedKernelProgram` from an envelope alone without a consumer-supplied registry.

**Two consequences the codec states rather than approximates.**

- A consumer that needs the program must hold the one it compiled; the envelope binds them by identity.
- A multi-stage program's *stage execution order* is not recoverable, because the envelope orders entries by canonical stage key (as identity does) and does not carry the dependency graph. The codec emits `tiler.artifact.feature.multi-stage-program` for such an artifact and its own reader refuses to read it, which is the fail-closed form of that gap.

**What closes this ticket.** One of: (a) a decided contract that the envelope never reconstructs a program, with the runtime's program-binding path written down and the multi-stage feature replaced by carrying execution order and dependency obligations explicitly; or (b) a neutral program section that a registry-supplied decoder can drive back through `KernelProgramBuilder`, with the registry dependency stated at the API boundary. Either way `docs/artifact-abi.md`'s "A decoder must reconstruct shared IR through its checked builders" sentence must end up true or amended.

**Not closed by** adding fields speculatively: the codec deliberately does not carry a value only the program can establish, because a carried copy would let a forged envelope assert a range no verifier examined.

## Direct evidence: the envelope retains the program's identity, not the program

Found while building the first out-of-crate artifact assembler for `carry-the-metal-payload-in-an-artifact-envelope`.

**Fact — `crates/tiler-artifact/src/program/codec/model.rs:433-437`.** `ArtifactEnvelope::project` fills a variant's program reference with `let content = variant.program.canonical_identity().as_bytes();` and then looks that content up in the section table. The section a `VariantRow::program_section` points at therefore holds the program's **canonical identity bytes**, not any encoding of its stages, values, views, allocations, or dependencies.

**Fact — `codec/model.rs:353-360`.** `VariantRow` carries `program_section: u32`, the guard, the profile, the feasibility rules, the deferred predicates, and the entry rows. Nothing else references the program.

**Inference — a decoded envelope cannot produce a `VerifiedKernelProgram`.** `VariantData` requires one by value, so there is no envelope → `VerifiedArtifactProgram` path today, and one cannot be written without deciding what this ticket exists to decide.

**Consequence, already hit in practice.** A narrow codec API of the shape `decode(bytes) -> Result<VerifiedArtifactProgram, _>` was proposed and approved for the assembler, and is not implementable. What a decoder *can* return is a fully validated envelope view — identity re-derived from decoded content, canonical order, arena closure, section purposes, payload descriptor digests, derived features — everything except the reconstructed program. That is enough for the envelope round-trip proof and is not enough for a runtime that must dispatch what it decoded, which is what makes this ticket load-bearing for `route-the-runtime-proof-through-the-artifact-envelope`.

**The question this sharpens.** Whether the envelope should carry a reconstructable program encoding at all, or whether a decoded artifact is deliberately a *dispatch* record — entries, bindings, launch expressions, payload references — that never rebuilds the IR and validates against the identity digest instead. The second is smaller and may be correct; it is not yet decided and should not be settled by an assembler's convenience.

## Second evidence: the dispatch record is encoded, validated, and unreachable

Found 2026-07-25 while attempting `prototype-runtime-artifact-validation`, which needs "checked expression evaluation" over a decoded artifact.

**Fact — the public read surface.** `DecodedArtifact` in `crates/tiler-artifact/src/program/codec/view.rs` exposes exactly `identity()`, `features()`, `routing()`, `payloads()`, `sections()`, `variant_count()`, and `re_encode()`. No accessor reaches a variant, an entry, a binding, an ABI expression, or a launch contract.

**Fact — the bytes already carry the dispatch record.** `codec/model.rs:341-360`: `EntryRow` carries `stage`, `resources`, `numerical`, `bindings`, `launch`, `payload`, and `entry_key`; `VariantRow` carries `guard`, `profile`, `feasibility_rules`, `deferred`, and `entries`. The decoder validates all of them — expression typing, availability phase, guard predicate type, arena closure, canonical order — and `re_encode()` writes them back, which is what proves nothing was dropped on the way in.

**Inference — option (c) is cheaper than this ticket's framing implies.** The "dispatch record" alternative does not require designing or encoding anything: the encoding exists, the validation exists, the round-trip test exists. What is missing is only a projection — accessors on `DecodedArtifact` over rows that are already `pub(crate)`. That materially changes the cost comparison between the two options this ticket weighs, and it is worth knowing before the decision is taken rather than after.

**Inference — the alternative to promoting anything is binding-by-identity, and it also already works.** Every expression accessor that exists (`VariantRef::applicability_guard`, `EntryRef::bindings`, `BindingRef::accessible_bytes`, `EntryRef::launch_threads`, `AbiExprRef::evaluate`, plus `AbiFactBinder`'s phase enforcement) hangs off `VerifiedArtifactProgram`. A runtime that holds the program it compiled and compares `DecodedArtifact::identity()` against `VerifiedArtifactProgram::canonical_identity()` gets validation-on-every-cache-hit and full checked evaluation with no new public surface at all. That is this ticket's option (a) — "the runtime's program-binding path written down" — and the finding is that the path is already implementable, so option (a) costs documentation rather than code.

**What this does not settle.** Binding-by-identity requires the consumer to already hold the program, which is exactly the case a cold process restart does not satisfy. Whether that case matters for Tiler's first runtime is the substance of the choice and is not answered here.
