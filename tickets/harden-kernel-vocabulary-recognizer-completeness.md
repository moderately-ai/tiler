---
id: harden-kernel-vocabulary-recognizer-completeness
title: Keep the kernel-IR vocabulary recognizable by its backends
status: done
priority: p2
dependencies: [resolve-non-exhaustive-recognizer-hole]
related: [harden-public-enums-non-exhaustive, resolve-non-exhaustive-recognizer-hole, extend-canonical-identity-encodings-for-reserved-variants]
scopes: [implementation/ir, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, api-hardening, correctness]
---
ADR 0074's convention 5, amended on 2026-07-24 by
`resolve-non-exhaustive-recognizer-hole`, records the twelve `#[non_exhaustive]`
vocabulary enums of `crates/tiler-ir/src/kernel/model.rs` as knowingly
non-conforming under clause 5c: `tiler-metal` is an out-of-crate recognizer that
matches them to decide what it can emit, so the attribute converts every future
IR capability into a silent `UnsupportedOperation` rejection at that backend
rather than a compile error naming the site that must decide.

The enums are `KernelType`, `AddressSpace`, `BufferAccess`, `Builtin`,
`KernelConstant`, `BinaryOp`, `CompareOp`, `ConvertOp`, `ExecutionScope`,
`MemoryScope`, `BarrierOrdering`, and `OperationView`. Remove `#[non_exhaustive]`
from each and make every out-of-crate match over them explicit.

**The retrofit is a net deletion, not a cost.** Read against `crates/tiler-metal/src/emit.rs`
at commit `37f1350`, ten of the emitter's wildcard arms are already unreachable —
`msl_type`, `builtin_parameter`, `emit_operation`, `emit_constant`, `emit_binary`,
`emit_compare`, `emit_convert`, `barrier_call`'s execution-scope and ordering
matches, and the `BufferAccess` match under `AddressSpace::Device` each list every
variant explicitly and then carry a `_` arm that exists only because the attribute
forces it. Three arms are live and each already names in a comment exactly which
known variants it rejects: `address_space_declaration` rejects `Workgroup` and
`InvocationPrivate`, its `Constant` branch rejects `BufferAccess::Write`, and
`fence_flag` rejects `Constant` and `InvocationPrivate`. Transcribe those comments
into explicit patterns so the rejection is a stated capability decision rather
than a fallthrough. `barrier_call`'s match on the `(ExecutionScope, MemoryScope)`
pair keeps its catch-all: a product match needs one whatever the attributes say,
and ADR 0074's convention 5 says so explicitly.

Expect `unreachable_patterns` to fire on any wildcard left behind once the
attribute is gone; the workspace gate denies warnings, so the compiler enumerates
the remaining work. Confirm no consumer outside `tiler-metal` recognizes these
enums before finishing — `tiler-compiler`'s uses of `OperationView` are
`if let`-shaped partial reads (ADR 0074 convention 5a) and must stay that way or
be converted deliberately.

This ticket changes no semantics and no canonical identity: the `tag()` encoders
for these enums live in `tiler-ir` alongside the definitions and already match
exhaustively, which the amendment's measurement confirms is unaffected by the
attribute either way. If a per-variant behaviour genuinely changes, say so
explicitly in the Outcome rather than folding it in.

## Upgrade 2026-07-24: three of these are clause 5b, not 5c

`prototype-artifact-program-model` found a second, stronger reason to remove the attribute from `KernelType`, `AddressSpace`, and `BufferAccess`, and it changes this ticket's priority rather than merely adding detail.

`tiler-artifact` encodes all three into `CanonicalArtifactProgramIdentity`. That is a cross-crate **total map** — every variant must yield its own distinct encoding — which amended convention **5b** makes *mandatory*, not phase-scoped like 5c. The two clauses fail differently: a 5c failure is silently *incomplete* (a supported capability never reaches a backend), while a 5b failure is silently *wrong* (two structurally different subjects sharing identity bytes, which is the hazard convention 3 names). So for these three the attribute is not a judgement call about pre-alpha tolerance — it must go.

Note also that the artifact encoder is a **direct counterexample to the same-crate exemption** recorded on `extend-canonical-identity-encodings-for-reserved-variants`. That exemption is real for the schedule encoders, which sit in the enums' own crate — but it does not generalise, and it stops holding the moment any encoder crosses a crate boundary.

Its interim behaviour, which this ticket should retire: rather than emit a sentinel tag (which would cause exactly the identity collision above), the encoder rejects with `ArtifactDiagnostic::UnrecognizedForeignVariant { subject }`. That is sound but strictly weaker than a compile error — a widened `KernelType` would silently make previously packageable artifacts *unpackageable* instead of failing the build at the site that must decide. Once the attribute is removed, delete that variant and its rejection path; it exists only to compensate for the attribute.

## Outcome (2026-07-27)

`#[non_exhaustive]` removed from `KernelType`, `AddressSpace`, and `BufferAccess`, and every compensation it required is deleted.

**The upgrade note's clause-5b reasoning is what decided it.** These three are encoded into `CanonicalArtifactProgramIdentity` — a cross-crate *total map* — and the two failure modes differ in kind. An incomplete recognizer is silently *incomplete*: a supported capability never reaches a backend. An incomplete total map is silently *wrong*: two structurally different subjects share identity bytes. The attribute made the second unreachable at compile time, which is why this was not a judgement call about pre-alpha tolerance.

### What the compiler confirmed, rather than what I assumed

Removing the attribute produced **exactly the unreachable-pattern warnings the ticket predicted**, which is the check that the wildcards existed only to compensate for it. Deleted in consequence:

- `ArtifactDiagnostic::UnrecognizedForeignVariant` and the `ForeignEnumSubject` enum naming its three subjects, with the re-export.
- The three `*_tag` encoders became infallible `const fn`s returning `u8` rather than `Result<u8, ArtifactDiagnostic>`.
- `MetalEmitError::UnsupportedType`, which had exactly one construction site — the wildcard in `msl_type` — and no other producer.

**The infallibility propagated further than the three functions**, and clippy's `unnecessary_wraps` found each step: `encode_entry`, `encode_variants`, and `push_interface` all had `Result` return types whose only failure source was a tag encoder, and all three are now infallible. That propagation is the honest measure of how much machinery the attribute was costing.

### The Metal recognizers are exhaustive by name now, not by wildcard

`address_space_declaration` matched `AddressSpace::Workgroup | InvocationPrivate` explicitly instead of `_`, and the constant-space arm names `BufferAccess::Write` instead of `_`. A widened `AddressSpace` now stops the build at the backend that has to decide whether the new space has a realization — which is what a recognizer is for. `msl_type` became an infallible `const fn`.

**One rejection is deliberately kept:** a `Write` binding in the constant address space still returns `UnsupportedBufferAccess`. That is a real refusal about Metal's read-only constant space, not a compensation for an open vocabulary, and it is now reached by a named arm rather than a wildcard.

### Not changed

No canonical identity moved: the tag tables and their values are unchanged, and the round-trip test that pins them passes with the `unwrap()`s dropped. `tiler-compiler`'s `OperationView` reads are `if let`-shaped partial reads and were not touched, as the ticket required.
