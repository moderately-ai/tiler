---
id: carry-reconstructable-kernel-programs-in-the-neutral-envelope
title: Decide what a decoded artifact envelope must reconstruct
status: review
priority: p1
dependencies: []
related: [prototype-neutral-artifact-codec, prototype-metal-bundle-assembly, settle-adr-0071-artifact-decoding-through-ir-builders, prototype-runtime-artifact-validation, route-the-runtime-proof-through-the-artifact-envelope]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, serialization, ir, needs-tom]
claimed_from: todo
assignee: agent-sidecar
lease_expires_at: 1784999937
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

## Third evidence: the dispatch record is incomplete, and one inference above is retracted

Read at `725f18b` by reading each file in full rather than searching it.

**Retraction — "what is missing is only a projection" is wrong as a claim about dispatch.** The second-evidence block above infers that option (c) "does not require designing or encoding anything" because "the encoding exists, the validation exists, the round-trip test exists". The first half of that is right about *description* and wrong about *dispatch*, and the difference decides how expensive option (c) actually is.

**Fact — `crates/tiler-artifact/src/program/model.rs:353-361`.** The encoded binding row is `BindingData { kind, element_type, address_space, access, alignment, value_role, accessible_bytes }`. It carries no reference to a materialized value, a view, an allocation, or a byte window.

**Fact — `crates/tiler-artifact/src/program/model.rs:906-928`.** `BindingRef::value()` and `BindingRef::window()` do not read that row. They call `access_ref()`, which is `EntryRef::stage().accesses().nth(self.binding)` — a walk into the packaged `VerifiedKernelProgram`. The binding-to-value correspondence is *stage access order*, a property of the kernel's signature, and the envelope carries the stage only as an opaque `StageSubject` content key (`codec/model.rs:146-149`, `341`).

**Inference — a decoded envelope cannot say which buffer a slot addresses.** Given two `BindingKind::Buffer` slots whose `value_role` is `Input`, nothing decoded distinguishes them: not the role, not the element type, not the size formula, and not the slot index, because slot order follows the kernel signature rather than the artifact's named interface. A runtime handed only bytes therefore knows a slot's transport, type, address space, access mode, alignment, role, and required byte range, and does not know what to bind to it. That is not a projection gap; it is a missing encoded fact, and supplying it is exactly the "adding fields" this ticket's opening cautions against doing speculatively.

**Consequence for the cost comparison.** Option (c) needs a per-binding value reference — which named interface entry, or which internal allocation, plus the byte window inside it — and therefore a manifest schema step, an encoder, a decoder, a re-proven obligation that the reference agrees with the packaged program, and its own identity contribution. It remains cheaper than option (b), and it is not free.

**Fact — the multi-stage refusal is real and is a second gap of the same kind** (`codec/model.rs:82-99`). `FEATURE_MULTI_STAGE_PROGRAM` is emitted when a variant dispatches more than one stage and is absent from `SUPPORTED_FEATURES`, so this build's reader refuses such an envelope. Its own comment states the reason: the section carries a program's canonical identity, not its dependency graph, so declaration order is not execution order.

## The residual choice is genuinely open, and one earlier framing of it does not survive

**Considered and withdrawn — "option (b) adds capability for nobody".** The argument was that any consumer able to supply a registry could have kept the program instead. It is false. `SemanticProgramBuilder::try_standard()` and `SemanticRegistryBuilder`'s standard providers build a frozen registry *from code*, not from bytes, so a cold process that links `tiler-ir` can hold the registry without ever having held the program. Option (b) is therefore a real capability for programs over standard operations, and fails closed for a program whose admission provenance names a provider the consumer's registry does not hold — which the envelope's reached-provenance subject already makes checkable. Recording this because a reader who reached the same shortcut should see it refuted rather than repeat it.

**Fact — what option (b) would cost.** `crates/tiler-ir/src/program/builder.rs:75` — `KernelProgramBuilder::new(semantic: &SemanticProgram)`. `crates/tiler-ir/src/semantic/program.rs:402-410` — `SemanticProgramBuilder` holds a `FrozenSemanticRegistry`; `crates/tiler-ir/src/semantic/registry.rs:983` — that is an `Arc<FrozenRegistryData>` over live `Arc<dyn OperationInferencer>` implementations. So option (b) requires the envelope to carry a complete encoding of the semantic graph *and* of the kernel program's stages, values, views, allocations, and dependencies, and requires the reading process to link the shared IR and construct a registry that covers every reached provider.

**Fact — the accepted layout does not settle it either way.** ADR 0056's retained clause is "The runner depends on the artifact contract and live Metal bindings, never the compiler" — not *never the shared IR*, and any runtime linking `tiler-artifact` already links `tiler-ir` transitively. `prototype-runtime-artifact-validation` states "The runtime path must not import semantic IR, optimizer state, backend internals, or proof-sidecar semantics", which would exclude option (b) outright, but that is a ticket rather than an accepted contract and cannot settle an architectural boundary on its own.

**Fact — the cold case is the accepted shape of the first runtime proof, not a hypothetical.** `prototype-metal-runtime-proof` runs `serial-sum-run` as a separate executable over a bundle `serial-sum-compile` wrote. It is not exercised today: `route-the-runtime-proof-through-the-artifact-envelope` records that the landed `prototypes/serial-sum-run` bypasses the envelope entirely, compiling and dispatching in one process, and its `Cargo.toml` depends on `tiler-compiler`, `tiler-ir`, `tiler-metal`, and `tiler-reference` accordingly. So the question below is about the runtime that ticket will build, not about the spike that exists.

## Blocked on one atomic decision (needs Tom)

**When a Tiler process loads an artifact it did not compile, does it rebuild the kernel program from the envelope, or dispatch from a record the envelope carries explicitly and never rebuild IR?**

Take the fixture program `sum((input * 2.0) + 1.0)` over `[2, 3]`, packaged as one variant with one entry and two buffer bindings.

- **Rebuild (option b).** The envelope carries the semantic graph and the whole kernel program; the loader links `tiler-ir`, freezes a registry covering the reached providers, and drives `KernelProgramBuilder`, getting back a `VerifiedKernelProgram` verified by the same authority that verified the original. *Enables:* re-validating a program against its semantic source, a cache that rehydrates plans, and every existing `VerifiedArtifactProgram` accessor working on a decoded artifact for free. *Prevents:* a loader that does not link the shared IR; and it fails closed — correctly, but opaquely to a user — for any program naming a provider the loader's registry lacks. *Costs:* the envelope becomes a serialization of the shared IR, so every IR change is a wire-format change.
- **Dispatch record (option c, recommended).** The envelope carries what a dispatch needs and nothing more: it already carries entries, resources, numerical realization, bindings, launch contract, and payload references, and it would gain the per-binding value reference and the stage execution order the two gaps above name. Validation against the artifact identity replaces reconstruction. *Enables:* a loader that needs only `tiler-artifact` and the device bindings, and a wire format that changes when dispatch changes rather than when the IR does. *Prevents:* any consumer that wants an IR value back out of an artifact, permanently and by design.

**Recommendation: the dispatch record**, on the grounds that the envelope's whole purpose is delivery to a process that did not build it, and that making the wire format a serialization of a research-phase IR would couple every IR revision to artifact compatibility. **Counterpoint, stated because it is real:** a carried value reference is a fact the envelope *asserts* and the reading process cannot re-derive, whereas reconstruction would re-prove it. The mitigation is that the reference is folded into artifact identity and its agreement with the packaged program is proven by the artifact verifier at build time — the same posture every other envelope row already has — but that is a weaker guarantee than reconstruction and must be recorded as one rather than glossed.

## What this ticket now owns, and what it does not

Its `scopes` gained `contracts/artifacts`, because either answer amends `docs/artifact-abi.md`: the ownership-boundary sentence "A decoder must reconstruct shared IR through its checked builders", and item 3 of "Where the implemented profile is narrower than this contract".

The ADR-side propagation is **not** this ticket's. `settle-adr-0071-artifact-decoding-through-ir-builders` already owns the identical clause in ADR 0071's Decision and its "Unrealized clause" paragraph, and the two must give the same answer.

If the decision is the dispatch record, the encoding work the two gaps above name is a follow-up ticket over `implementation/artifact` and `contracts/artifacts` and is deliberately **not** filed yet: filing an implementation ticket for one branch of an undecided question would convert a proposal into a plan.

## Decision — Tom, 2026-07-25

**Decided: a decoded envelope is a DISPATCH RECORD, not a reconstruction.** It carries entries, bindings and launch expressions as encoded facts a decoder projects, validated against the packaged program's identity digest. It never rebuilds a `VerifiedKernelProgram`.

Full IR reconstruction was excluded on evidence rather than preference: `KernelProgramBuilder::new` requires a `SemanticProgram`, which requires a frozen registry of `Arc<dyn OperationInferencer>` — behaviour, not data. No serialization format carries that, so the option was impossible at any encoding cost rather than merely expensive.

**The cost is accepted with its weakness stated.** A decoded envelope currently cannot say which buffer a slot addresses: `BindingData` carries no value or view reference, and the stage reaches the envelope only as an opaque content key. So this needs a new encoded fact plus a schema step, not an accessor — the ticket's earlier 'only a projection' inference was retracted for exactly this reason. A carried value reference is asserted by the producer rather than re-derived by the decoder, and that asymmetry must be stated wherever the record is documented rather than left for a reader to discover.

This is what lets a loader dispatch without linking `tiler-compiler` or rebuilding a semantic graph, which is the artifact layer's stated purpose.
