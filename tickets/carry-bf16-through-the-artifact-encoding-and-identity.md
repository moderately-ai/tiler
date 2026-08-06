---
id: carry-bf16-through-the-artifact-encoding-and-identity
title: Carry BF16 through the artifact encoding, ABI, and program identity
status: in-progress
priority: p1
dependencies: [admit-bf16-into-the-schedule-and-kernel-vocabulary, redesign-the-delivered-realization-record-from-typed-evidence]
related: [spike-bf16-through-the-second-dtype-seams, accept-the-delivered-realization-artifact-surface]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, artifact, abi, identity]
claimed_from: todo
assignee: agent-bf16-artifact
lease_expires_at: 1785982711
---
## User-visible outcome

A BF16 program encodes to an artifact and decodes back to the same program, with the dtype in its identity. A decoder meeting a BF16 tag it does not understand refuses rather than reading the bytes as `f32`, which at two versus four bytes per element would otherwise misread the whole buffer.

## Why the identity half is the risky half

**Fact, at `ef3c051`.** The artifact layer carries its own tag tables, separate from the IR's: `element_type_tag` / `element_type_from_tag` for `KernelType` and `storage_scalar_tag` / `storage_scalar_from_tag` for `StorageScalar` (`crates/tiler-artifact/src/program/model.rs`), plus `check_binding_access` in `codec/validate.rs`, which pairs a storage scalar with a kernel type in an exhaustive match.

**Fact.** `NumericalFacts` carries `canonical_arithmetic_nan_bits: u32` — a binary32-shaped field on the artifact's numerical record.

**Inference.** These tables are a second encoding of the same vocabulary, deliberately, so the artifact format does not move when the IR's internal representation does. That is why this is its own ticket rather than part of the IR one: a tag added in one place and not the other produces an artifact that encodes and fails to decode, and the failure is at the far end of the pipeline.

**Fact.** The canonical NaN field is `u32`-shaped and a BF16 canonical NaN is 16 bits. Whether it widens, becomes width-tagged, or moves into the realization record is a real design question, and `redesign-the-delivered-realization-record-from-typed-evidence` is already redesigning the record that owns delivered numerical evidence.

**Fact — the redesign has since landed, and the width question survived it unresolved (2026-08-06, read on `main` after the merge).** The record is now `EntryRealization` at artifact version 15 and manifest schema 13.0, and the field it projects from is unchanged: `NumericalRealization.canonical_arithmetic_nan_bits: u32` (`crates/tiler-ir/src/schedule/numerics.rs:237`), encoded big-endian into the artifact at `crates/tiler-artifact/src/program/model.rs:2245`. The dependency below is satisfied and the ordering concern it protected is discharged; the width question is now resolved against the landed record rather than coordinated with an in-flight redesign. Note the kernel IR already carries `CanonicalizeBf16Nan`, so a 16-bit canonical NaN value exists in the vocabulary this record describes.

## Implementation keys

- **The tag tables and `check_binding_access` are no longer this ticket's work** — they land in `admit-the-bf16-type-and-carrier-into-every-total-map`. **Measurement, 2026-08-02 at `3990f9d`, `cargo check --workspace --all-targets`.** `element_type_tag` (`program/model.rs:1737`), `storage_scalar_tag` (`:1758`), and `check_binding_access` (`codec/validate.rs:369`) are exhaustive matches over two deliberately non-`#[non_exhaustive]` vocabularies, so `crates/tiler-artifact` stops compiling the moment `KernelType::Bf16` and `StorageScalar::Bf16` exist. The tags therefore cannot wait for this ticket. On arrival, *verify* that both tables and their `*_from_tag` decoders already carry BF16 with every earlier tag value unchanged, rather than adding them a second time.
- What stays here is everything the tags are *for*: the round trip, the identity, and the refusals below. An artifact encoded before that widening must still decode to the identical program after it.
- The dtype participates in program identity, so a BF16 program and an otherwise identical F32 program have different identities. Assert this directly; it is the property a cache is wrong about if it does not hold.
- Resolve the `canonical_arithmetic_nan_bits` width question rather than widening the field by reflex, against the landed `EntryRealization` record rather than by introducing a second numerical record.
- Decoder refusal for an unknown tag stays a typed refusal at decode time, before any byte is interpreted.

## Required evidence

- Round trip: a BF16 program encodes, decodes, and compares equal, with its element count and byte length recorded.
- An artifact carrying a BF16 tag decoded by a build that does not know it is refused with a typed reason, not misread.
- A BF16 storage scalar paired with an F32 kernel type is refused by `check_binding_access`, observed failing.
- A BF16 program's identity differs from the same program at F32; both are recorded.
- Existing F32 artifacts decode byte-identically, pinned by the existing goldens.

## Closes when

A BF16 program survives the encode/decode round trip, the dtype is in the identity and shown to be, the unknown-tag and mismatched-binding refusals are observed failing, the canonical-NaN width question is resolved rather than deferred silently, F32 artifacts are unchanged, and the `ABI and materialization` cell for BF16 moves.

## Graph maintenance

- Depends on the IR vocabulary existing, and on `redesign-the-delivered-realization-record-from-typed-evidence` because both change the artifact's numerical record and the second is already `todo` with that surface in scope. Landing this first would put a BF16-shaped patch into a record that ticket is replacing.
- The artifact *tags* arrive earlier than that, through `admit-the-bf16-type-and-carrier-into-every-total-map`, which this ticket already depends on transitively. That extraction is a compilation fact and changes none of this ticket's deliverables — see the first implementation key.
- `accept-the-delivered-realization-artifact-surface` is Tom's public-boundary ratification and is `todo`; it is `related` rather than a dependency, but a consequential change to the artifact's public numerical record goes to Tom before acceptance.
- This ticket moves artifact identities. Any spike or prototype citing an artifact identity size will drift; recompute on the merged tree.
