---
id: record-the-implemented-artifact-envelope-in-the-contract
title: Record the implemented artifact envelope in docs/artifact-abi.md
status: in-progress
priority: p2
dependencies: []
related: [prototype-neutral-artifact-codec]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, artifact]
claimed_from: todo
assignee: agent-record-the-implemented-artifact-envelope-in-the-contract
lease_expires_at: 1784932575
---
`docs/artifact-abi.md` states "artifact codec unimplemented" and "Canonical envelope serialization, backend payloads, integrity validation, and public artifact APIs remain unimplemented." `prototype-neutral-artifact-codec` implemented a bounded canonical lockstep codec, but deliberately did **not** edit that contract: the codec landed as a crate-private draft under ADR 0074 convention 7, its facade is unaccepted, and the ticket held only `implementation/artifact`.

**What this ticket must state, once the facade is accepted.**

- The envelope framing this build writes: fixed 69-byte header (magic `TILERART`, envelope format, canonical encoding version, governed digest algorithm tag, total length, manifest length, section count, manifest digest), one canonical manifest, then length-delimited sections.
- That the manifest is written in canonical content order for every set-meaning collection, so two artifacts with equal identity encode to equal bytes, and that a well-formed but non-canonical encoding is rejected rather than normalized.
- That the governed digest algorithm is named by an explicit tag and never inferred from a digest width.
- That the initial section vocabulary has exactly one governed purpose, the packaged variant's canonical kernel-program identity, and that backend metadata and code sections are `prototype-metal-bundle-assembly`'s versioned extension.
- The required-feature mechanism and the four keys this build derives, including the one it emits and refuses to read.
- What the envelope deliberately excludes: the frozen registry snapshot (ADR 0072), presentation-only declaration order, backend payload bytes, and a reconstructable kernel program.

**Do not** mark the contract implemented while the surface is `pub(crate)`; the accurate statement is that a bounded lockstep codec exists behind an unaccepted facade.

## Outcome

`docs/artifact-abi.md` gains an "Implemented envelope profile" section recording exactly what `crates/tiler-artifact/src/program/codec/` writes and reads at commit `1f78223`, verified by reading every file in that module in full. `implementation_status` moves `spike-only` → `partial`; the status line and lede now say that serialization, canonical form, and integrity validation are implemented behind an unaccepted `pub(crate)` facade, and name what a consumer can and cannot do — which is, respectively, read an identity now derived from the canonical envelope, and nothing else.

**Maturity claims are kept apart** as `AGENTS.md` requires: the framing, manifest, section framing, feature mechanism, and rejection vocabulary are implemented; the section-purpose vocabulary and payload descriptor are type-system reservations; a `pub` facade is an unshaped seam; and each labelled Measurement is a tested guarantee over named fixtures rather than a universal claim.

**Recorded.** The 69-byte header field by field with offsets; the manifest's domain tag, its own schema, the four component schemas, and its field order; the meaning-versus-canonical ordering rules; identity derived from the envelope through one encoder, with the inference that equal identity implies equal bytes *because* three closure checks reject an unreached expression, an unrealized payload, and an unreferenced section; the four derived feature keys including `multi-stage-program`, which this build emits and refuses to read; the single governed section purpose carrying canonical identity bytes rather than a digest of them; the three governed digest domains verbatim, with prefix-freedom; the four deliberate exclusions; the governed budgets; and the rejection vocabulary grouped by the boundary that refuses.

**Left open for `prototype-metal-bundle-assembly`:** whether a bundle's identity is content-addressed over compilation inputs or over emitted payload bytes. The contract states the section machinery exists and that the decision is that ticket's.

**Four code/contract disagreements found; the contract is right in three.**

1. The section digest omits the section's purpose from its pre-image, which the contract's own derivation includes. Sufficient inside a complete envelope, insufficient for a standalone content address — which is what a code section will want. → `bind-section-purpose-and-schema-into-the-section-descriptor`, together with the descriptor's missing per-section schema and required/optional disposition.
2. The backend payload descriptor carries no compatibility-contract reference. The variant's target profile substitutes only while a payload is unshared, and nothing enforces that. → `carry-a-compatibility-contract-reference-on-the-payload-descriptor`.
3. The identity block writes `H(domain || bytes)` for five layered subjects that are implemented as canonical *byte encodings*, not hashes — which is ADR 0074 convention 2, so the code is right and the contract's block was the stale side. Its placeholder domain spellings match no encoder, and `crates/tiler-ir/src/schedule/model.rs` writes `b"tiler.schedule.v1"` without the NUL every sibling uses. → `decide-whether-layered-subject-digests-exist-as-hashes`.
4. A decoder does not reconstruct shared IR through its checked builders, because `KernelProgramBuilder::new` needs a `SemanticProgram` needing live inferencers. Structural, already owned by `carry-reconstructable-kernel-programs-in-the-neutral-envelope`; recorded, not re-filed.

The absent optional-section mechanism is **not** filed: the contract already conditions it on exposing the format outside a lockstep release, so it is a deferred question with a stated trigger.

**Reported and ticketed, not edited.** `docs/research/artifacts/target-neutral-artifact-envelope.md` (`research/artifacts`) still reads `implementation_status: "spike-only"` and closes with "Production serialization … remain unimplemented"; `docs/status.md` and `docs/roadmap.md` (`contracts/navigation`) still list the neutral artifact codec as pending work. All three are outside this ticket's `contracts/artifacts` scope and are owned by `reconcile-stale-neutral-artifact-codec-status`, which carries the constraint that none of them may out-run `docs/artifact-abi.md`'s deliberate `partial`.

Every fact in the new section was re-verified by reading before integration: the 69-byte header offset by offset against `encode.rs`; the three digest domain constants verbatim; `MAX_ENVELOPE_BYTES`, `MAX_MANIFEST_BYTES`, `MAX_SECTION_BYTES`, `MAX_SUBJECT_BYTES`, `MAX_TEXT_BYTES`, `MAX_FEATURES`, `MAX_INTERFACE_ENTRIES`, and `MAX_INTERFACE_SHAPE_RANK`; the four governed feature keys; the `#![allow(dead_code, reason = …)]` and `pub(crate)` surface; and all 38 rejection variants against `error.rs`, which the table covers exactly with no omission and no invention. The `b"tiler.schedule.v1"` NUL anomaly is the sole such site among the 30 domain constants in `crates/`.
