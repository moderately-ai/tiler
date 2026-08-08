---
id: correct-the-l6-records-stale-storagescalar-variant-count
title: Correct the L6 record's stale StorageScalar variant count
status: todo
priority: p2
dependencies: []
related: [admit-a-storage-carrier-for-integer-program-inputs]
scopes: [research/program-planning]
shared_scopes: []
paths: []
tags: [documentation, dtype]
---
## User-visible outcome

The L6 ingestion record stops asserting a carrier vocabulary that has been wrong since the BF16 landing, so the next reader of the "IN-A has a blocker at the ABI" paragraph is not sent to the wrong line and the wrong count.

## The defect

**Fact, found 2026-08-07 at base `68f1ced6` while auditing [`admit-a-storage-carrier-for-integer-program-inputs`](admit-a-storage-carrier-for-integer-program-inputs.md).** `docs/research/program-planning/complete-model-ingestion-and-execution.md`, in the section "The input boundary: the gather stays inside", asserts:

> **Fact —** the runtime-value boundary has no carrier for it: `StorageScalar` in `crates/tiler-ir/src/program/model.rs:264` has exactly two variants, `U8` and `F32`.

Both halves are false at this base. The enum is at `model.rs:342`, not `:264` — line 264 is inside `StorageEncoding`, an unrelated type — and it has **three** variants: `U8`, `F32`, and `Bf16`. `Bf16` landed under [`admit-the-bf16-type-and-carrier-into-every-total-map`](admit-the-bf16-type-and-carrier-into-every-total-map.md). Reproduce with `grep -n 'pub enum StorageScalar' -A 16 crates/tiler-ir/src/program/model.rs`.

**Fact — the same paragraph's ABI-cost conclusion is also falsified, and that is the load-bearing half.** It closes: "`StorageScalar::tag` participates in canonical encoding, so a third variant is an artifact-ABI change and therefore Tom's." A third variant *already happened* and was not an ABI break: `StorageScalar::tag` appends, and its own comment records that "`U8` and `F32` keep their tags and every field keeps its position, so no previously encodable program's bytes move and the program identity domain does not step."

Measured on the carrier ticket's branch: with a fourth carrier appended at tag `0x04`, `cargo nextest run -p tiler-ir -p tiler-artifact -p tiler-macros -p tiler -p tiler-compiler` ran 2213 tests with 2211 passing and **no golden, pin, or identity test moving**. The two failures were deliberate widening tripwires (a forged-tag test whose `UNASSIGNED_CARRIER` is `0x04`, and a `variant_count`-sized domain assertion), not identity drift.

## Why this is its own ticket

`research/program-planning` is not in the carrier ticket's scopes, and it was held by a live exclusive claim when the defect was found. The correction is a documentation edit with no code consequence, so it does not belong inside the carrier landing either.

## Closes when

The paragraph carries a dated correction in the record's own house style — the false line citation and variant count replaced with anchors, and the "therefore Tom's" ABI conclusion restated against the measured append-only behaviour — with the prior rationale preserved rather than silently overwritten.
