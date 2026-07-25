---
id: expose-the-dispatch-record-on-a-decoded-artifact
title: A decoded artifact must carry enough to dispatch from bytes alone
status: done
priority: p0
dependencies: []
related: []
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, runtime, spine]
---
Implements the decision Tom made on `carry-reconstructable-kernel-programs-in-the-neutral-envelope`: **a decoded envelope is a dispatch record.** That decision is recorded and not yet carried out, which is why the runtime proof still bypasses the envelope.

**Fact — what a consumer can read today.** `DecodedArtifact`'s complete public surface is `identity`, `features`, `routing`, `payloads`, `sections`, `variant_count`, and `re_encode` (`crates/tiler-artifact/src/program/codec/view.rs`).

**Fact — what it cannot read.** The entry symbol: `decode_metadata` is `pub(crate)` at `codec/payload.rs:292`, so the `PayloadMetadata` carrying entry mappings and transport slots is unreachable. And the per-variant entries: `EntryRow`, `BindingData` and `LaunchData` are encoded, validated and round-tripped, but `pub(crate)` with no projection.

**Inference — this is why the bypass survives.** A worker attempting `route-the-runtime-proof-through-the-artifact-envelope` concluded the runner must reach the producer's assembler code, and proposed sharing it via a library target or a new crate. That diagnosis is right about the symptom and wrong about the cause: if a consumer needs the producer's *code* to consume an artifact, the artifact is not the interface. **The bytes are supposed to be the interface.** Sharing assembler code would have worked around this gap and left it in place.

**Consequence — this must NOT be closed by sharing producer code.** No library target on a prototype, no assembler crate, no duplicated assembly. If the dispatch record is complete, a runner reads a file the producer wrote and needs nothing else.

## Scope

Project the rows that already exist: per-variant entries with their bindings (transport slot, accessible-byte expression) and launch contract (grid, workgroup, preconditions), and the payload metadata a loader needs to find its entry symbol. Accessors over validated rows, not new encoded facts — `decode_artifact` already proved them.

**One fact is genuinely missing rather than merely unprojected**, and it must be named rather than invented: `BindingData` carries no value or view reference, and the stage reaches the envelope only as an opaque content key, so a decoded envelope cannot say **which buffer a slot addresses**. Tom's decision records this and accepts it. Decide explicitly whether to encode that reference now or to define the dispatch contract as slot-ordered — and if slot-ordered, say what makes the order authoritative, because a consumer binding by position needs that guarantee stated rather than assumed.

A carried value reference would be asserted by the producer rather than re-derived by the decoder. State that asymmetry wherever the record is documented.

## Closes when

A consumer holding only encoded bytes can name every entry, its bindings and their transport slots, its launch geometry, and its backend entry symbol; the which-buffer question is decided and stated; `route-the-runtime-proof-through-the-artifact-envelope` needs no producer code; and `uv run --locked python scripts/check_repository.py` passes.

## Outcome

**Landed: the dispatch record is projected, and the which-buffer fact is encoded rather than assumed.**

### The which-buffer decision: encode the reference; slot-ordering was rejected

The ticket offered two options. **Slot-ordering was rejected on evidence, not preference.** A binding's slot order is the *kernel signature's*, and the kernel is precisely what a decoded envelope does not carry — `VariantRow::program_section` names a section holding the program's canonical identity bytes and nothing else. So "state what makes the order authoritative" cannot be satisfied honestly: the authority is a fact the format omits, and a consumer binding by position would have no way to check the position means what it assumed. Binding the wrong buffer produces a silently wrong tensor rather than a refusal, which `AGENTS.md` forbids outright.

**What landed instead** is a per-binding `BindingTarget`, matched exhaustively with no wildcard (ADR 0074 convention 3), in three classes that are different instructions to a loader rather than shades of one:

- `ProgramInput(&InputKey)` — bind the host buffer given for this interface key.
- `ProgramOutput(&[OutputKey])` — one buffer, published under **every** key listed.
- `Internal` — allocate it; no interface name exists.

**Why the output case is a set and not one key.** `SemanticProgramBuilder::output_resolved` (`crates/tiler-ir/src/semantic/program.rs:620-643`) rejects a repeated output *key* and not a repeated *value*, and `KernelProgramBuilder::push_output` (`crates/tiler-ir/src/program/builder.rs:377-414`) does the same, so two named outputs may publish one materialized value and therefore one buffer. A single-key target would have named whichever key declaration order put first, and a loader would have allocated a second buffer for the other name and never written it. `a_value_published_under_two_names_carries_both_in_its_binding_target` is the regression test, over a new dual-output fixture.

**Why not a program-value handle.** An arena ordinal is the transient fact artifact identity replaces with canonical content keys everywhere else, and the shared IR's own canonical value key (`tiler_ir::program::model::canonical_keys`) is crate-private with no read view publishing it. The interface is what the envelope already carries and identity already folds.

### The asymmetry, stated where the record is documented

`crates/tiler-artifact/src/program/codec/view.rs` now has a section — *Which facts a decoder re-derives, and which it takes on trust* — saying it plainly. Framing, integrity, canonical form, schema, closure, expression typing and phase, and artifact identity are re-derived from content. A binding's target, like its element type, address space, alignment and accessible range, is a fact about the packaged program, which is not carried.

One correction to the ticket's framing: the producer does **not** assert it. `ArtifactProgramBuilder::check_bindings` *derives* it from the program's own stage access, the same way it already derived alignment and value role, so a producer cannot state a correspondence its plan contradicts. What is weaker than re-derivation is that the proof happened on the writing side; what binds it to these bytes is artifact identity, so a forged envelope restating a target becomes a different artifact — and only a consumer that compares identity has rejected it.

### Wire format

New encoded fact, so both versions moved and each says why at its site: `MANIFEST_SCHEMA` `2.0` → `3.0` (major, because a reader admitting `minor <= implemented` would otherwise accept a manifest whose binding rows it cannot parse) and `ARTIFACT_DOMAIN` `v1` → `v2` (so a `v1` and a `v2` encoding of two different artifacts cannot collide on one identity).

The binding row's `value_role` tag was **removed**, not kept alongside. The role is a total function of the target, and two encoded fields saying one thing is the drifting-second-authority hazard this codec avoids elsewhere; `BindingRef::value_role()` survives as a derived accessor.

### Two defects found and fixed that the ticket did not name

**`codec/payload.rs` claimed the artifact layer "proves the mapping covers exactly the backend entry keys the artifact's executable entries name". Nothing proved it at all** — neither the builder nor the decoder correlated the two tables. An artifact could carry a payload mapping none of the entries it realized, decode clean, and fail at a loader that could not resolve a symbol. `check_entry_mappings` now proves coverage and that each mapping places exactly as many transport slots as its entry has bindings; the doc records what was false. `prototypes/serial-sum-compile/src/bundle.rs:120-155` was already re-deriving the coverage half itself, which is the producer compensating for a missing artifact-layer obligation.

**A decoded envelope published no descriptor-to-object association.** `DecodedArtifact::payload_object(index)` now does, from `payload_content` — which the decoder already validated and which no reader could otherwise recover, since the section table is content-addressed and deduplicates equal objects.

### Refusals added, and which are untested

`PartialBindingView` (a binding addressing part of a value: the record carries an extent and no offset, so the target would be right and the placement guessed), `AliasedInternalBinding` (two bindings of one entry addressing one unnamed internal value), `UnnameableBindingTarget`, and four decode-time rejections.

Three are **not covered by a test**, each with an exact reason recorded at the variant:

- `UnnameableBindingTarget` is **unreachable**. `KernelProgramBuilder::push_value`'s `check_origin` (`crates/tiler-ir/src/program/builder.rs:479-507`) admits only `(ProgramInput, Input)`, `(Internal, Temporary)` and `(Internal, Output)`. Verified by probe: constructing the pair fails with `KernelProgramBuildError::ValueRoleOrigin`. It is kept for the reason `ArtifactDiagnostic::UnrecognizedForeignVariant` is kept — the guarantee lives in another crate's builder rather than in a type this one matches on.
- `PartialBindingView` and `AliasedInternalBinding` need a kernel declaring a `TensorRole::Intermediate` buffer, and `grep -rn "TensorRole::Intermediate" crates/tiler-artifact` is empty. Split into [`carry-the-byte-offset-of-a-partial-binding-view`](carry-the-byte-offset-of-a-partial-binding-view.md), which builds that fixture and removes the first refusal by carrying the offset.

### Retraction

An earlier reading of `crates/tiler-ir/src/program/verify.rs` alone concluded that `tiler_ir` "proves no correspondence between a value's role and its origin". That is **wrong** — the correspondence is enforced at insertion in `builder.rs::check_origin`, not in `verify.rs`. A comment in `binding_target` asserting the weaker claim was written and then corrected. The failed search was evidence the search was wrong.

### Measured

`uv run --locked python scripts/check_rust.py` and `uv run --locked python scripts/check_repository.py` pass on this branch. `cargo nextest run --workspace` is 787 passing, up from 779 at the base commit `311db2f`; `tiler-artifact` alone is 159, up from 152.

The headline evidence is `a_decoded_artifact_carries_everything_one_dispatch_needs`: it starts from `decode_artifact(&bytes)` and reads the interface, binds `AbiFacts` from the decoded shapes, evaluates the guard and both launch extents, reads every binding's target, access mode, alignment and accessible bytes, and resolves the entry's backend symbol, transport slots and exact object bytes — holding no `VerifiedArtifactProgram`, no semantic program, no registry, and no producer code.

### Not closed here

The **runtime crate's own documentation is now stale in three places** and its `resolve_object` cardinality workaround is unnecessary. Left untouched because this ticket holds `implementation/artifact` only; split into [`route-the-runtime-loader-through-the-dispatch-record`](route-the-runtime-loader-through-the-dispatch-record.md) with the exact sites.

`docs/artifact-abi.md`'s status paragraph ("Every item in that module is `pub(crate)` … so no crate outside `tiler-artifact` can encode or decode an artifact") was **already stale before this change**, from the 2026-07-25 promotion. It is `contracts/artifacts` scope and is not made worse here, but a reader will now find it wrong about more.

Whether `route-the-runtime-proof-through-the-artifact-envelope` can drop its remaining dependency on `share-the-serial-sum-artifact-assembler` is that ticket's call: the symbol, bindings and launch geometry half of its producer-code dependency is gone, and the `expected` identity a cold consumer binds against is a separate question it owns.
