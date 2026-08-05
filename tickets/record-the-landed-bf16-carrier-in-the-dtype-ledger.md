---
id: record-the-landed-bf16-carrier-in-the-dtype-ledger
title: Move the BF16 carrier and kernel-vocabulary cells onto the landed total maps
status: todo
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
