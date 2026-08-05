---
id: record-the-landed-bf16-carrier-in-the-dtype-ledger
title: Move the BF16 carrier and kernel-vocabulary cells onto the landed total maps
status: done
priority: p3
dependencies: []
related: [admit-the-bf16-type-and-carrier-into-every-total-map, move-the-navigation-docs-onto-the-two-contract-key-domains]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, bf16, dtypes]
---
## User-visible outcome

The dtype ledger's BF16 physical-carrier and kernel-vocabulary cells report what
`admit-the-bf16-type-and-carrier-into-every-total-map` actually landed, rather
than the `absent/unsupported` they carried before it.

## Why this is a separate ticket

**Fact.** `admit-the-bf16-type-and-carrier-into-every-total-map` is `done` and
added `KernelType::Bf16` (`crates/tiler-ir/src/kernel/model.rs:112`) and
`StorageScalar::Bf16` (`crates/tiler-ir/src/program/model.rs:350`), each with a
governed tag, a two-byte width derived at one width authority, artifact
`*_from_tag` decoding, a `check_binding_access` pairing refusal, and an
`msl_type` that refuses BF16 by name rather than spelling `bfloat`. It did not
touch `docs/dtype-support.md`.

**Fact, checked at `3adc0689`.** That ledger's *Physical and execution maturity*
table still reads `absent/unsupported` for BF16 under both **Physical carrier
and encoding** and **Kernel vocabulary**.

**Why it was not absorbed by `move-the-navigation-docs-onto-the-two-contract-key-domains`.**
That ticket's sweep found the discrepancy and corrected the flatly false source
claim it had also produced in `docs/roadmap.md` — "no BF16 variant exists in
`KernelType`, `StorageScalar`, `KernelConstant`, or `BinaryOp`". Moving a
*maturity cell* is a different act: it is a positive promotion under this
document's own cell vocabulary and needs the landing commit read in full, not a
grep. A navigation-docs sweep is the wrong authority for it.

## Scope keys

- Read `129d783b` in full before deciding either cell. The candidate reading is
  **implemented mechanism** — family-specific production code exists, no checked
  guarantee is cited for the complete claim — rather than **tested guarantee**,
  because no `VerifiedKernel` can carry a BF16 buffer at all
  (`crates/tiler-ir/src/kernel/verify.rs` derives every buffer's expected element
  type from the region's `ScalarProgram`, every arm of which is F32), so nothing
  constructs a value under either tag.
- Decide the **Kernel vocabulary** cell separately from **Physical carrier and
  encoding**: `KernelConstant` and `BinaryOp` still carry no BF16 variant, so the
  kernel column may be the weaker of the two.
- State the boundary in the section prose, in the idiom the file already uses:
  the tags exist and the refusals are deliberate, and `msl_type` refusing rather
  than spelling `bfloat` is what keeps an unmeasured target capability
  unreachable.
- **ABI and materialization** is a third cell to check rather than assume: the
  artifact decoders gained the tags in the same change.

## Required evidence

- Each moved cell names its construction site and the exact claim class, and each
  unmoved cell says why it did not move.
- `tkt lint` green.

## Closes when

Every BF16 cell this landing bears on reports its actual maturity class, with the
unconstructible-value boundary stated rather than implied.

## Delivered

**The candidate reading held, and it was checked rather than inherited.** `129d783b` was read in full, every site it touched was reread at base `d5960e81`, and the unconstructibility claim was verified at its source rather than from the producing ticket's own summary: `VerifiedKernel` is constructed at exactly one site (`crates/tiler-ir/src/kernel/builder.rs:719`), reachable only through `verify_kernel`, whose `verify_signature` derives every read buffer's expected element type from `schedule.index.scalar_program` — every arm yields `U8` or `F32` — and requires the written buffer to be `F32`, so a BF16-carrying kernel is refused as `BufferContract`. `KernelConstant` and `BinaryOp` were read in full and carry no BF16 variant; `StagedElement` stages `F32` alone.

Three cells moved in the *Physical and execution maturity* table, each `absent/unsupported` before:

- **Physical carrier and encoding → implemented mechanism.** `StorageScalar::Bf16` (`crates/tiler-ir/src/program/model.rs:350`) with `byte_width` 2 at the single width authority, tag `0x03`, and `natural_access_type` pairing it with `KernelType::Bf16`, which `check_physical_storage` requires of an unpacked binding. Not a tested guarantee: nothing carries a BF16 value, because binding requires the kernel buffer's element type to equal the value's.
- **ABI and materialization → implemented mechanism, tags and binding pairing only.** `element_type_from_tag`/`storage_scalar_from_tag` decode `0x06`/`0x03`, and `check_binding_access` (`crates/tiler-artifact/src/program/codec/validate.rs:381`) pairs the carrier with its own access type. The qualifier is load-bearing: `a_bf16_carrier_is_refused_against_a_wider_access_type_and_admitted_against_its_own` records that the *matched* pair is then refused as `BindingComponentMismatch` one step later, so nothing is materialized.
- **Kernel vocabulary → implemented mechanism, type admission only.** `KernelType::Bf16` (`crates/tiler-ir/src/kernel/model.rs:112`) with a doc-fixed meaning, tag `0x06`, and two-byte `element_bytes`, and nothing else in that vocabulary naming BF16. Decided separately from the carrier cell exactly as this ticket asked, and it is the weaker of the two.

Cells deliberately **not** moved: **Backend lowering** stays `absent/unsupported` — `msl_type` refusing `Bf16` by name is evidence that this layer holds no BF16 lowering authority, not a mechanism implementing one, and promoting it would make an unmeasured target capability read as reachable. **Optimizer legality** stays `absent/unsupported` — `index_arithmetic_requirement` classifying `Bf16` as needing no *index* arithmetic is narrower than needing no target capability, and `UNPLANNED_OPERATIONS` still names all three BF16 keys. **Backend execution**, **Runtime semantic validation**, **Target-family dispatchability**, and **Conformance evidence** are untouched by this landing. No semantic-table cell moved: the caller-layer question this ticket flagged is already recorded — the 2026-08-05 BF16 numerical contract moved no cell, and the ledger already said so before this edit.

Two stale assertions were corrected along the way, both about [`bind-the-bf16-contract-refusal-to-the-authoritative-apple9-rows`](bind-the-bf16-contract-refusal-to-the-authoritative-apple9-rows.md), which is `done` (`a136cb0e`) while `docs/dtype-support.md` and `docs/roadmap.md` still described its binding as owed and "separate and live". Both now carry a dated correction citing `the_ledger_rows_refuse_a_strict_bf16_contract_with_their_own_measured_evidence` in `crates/tiler-build/src/metal_declaration.rs`, and both state that the binding moved no cell and no rung — it strengthens an evidence chain under an unchanged refusal. The ledger's case-insensitive negative check was also rewritten: it named the current matches as comment-only, which this landing made false.

**Out of scope, reported rather than absorbed.** `crates/tiler-compiler/src/boundary.rs:669` documents the vocabulary sweep as `every_storage_carrier_has_a_natural_alignment`; the test is named `every_storage_carrier_has_a_representable_alignment` (line 2128). The drift predates `129d783b` and lives in a scope this ticket does not hold.
