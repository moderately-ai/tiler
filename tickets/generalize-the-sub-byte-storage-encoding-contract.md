---
id: generalize-the-sub-byte-storage-encoding-contract
title: Generalize the sub-byte storage encoding contract
status: deferred
priority: p3
dependencies: []
related: [derive-dtype-family-research-tracks-from-the-mature-taxonomy, widen-the-physical-vocabulary-for-per-axis-quantized-component-access, prototype-quantized-value-vertical, own-the-dtype-support-maturity-matrix]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, deferred, storage-encoding, packing]
---
## User-visible outcome

Bit order, cross-byte layout, tail, alignment, unaligned access, neighbour-safe writes, and repacking are answered once for every sub-byte carrier, instead of once per element family that happens to need one.

## Why this is one track rather than a clause in four

**Fact.** [The mature dtype taxonomy](../docs/research/numerics/mature-dtype-taxonomy.md)'s conclusion 6 fixes that "packing belongs to storage/encoding contracts", and its `BitPacked` sketch names element, bits per element, bit order, byte order, row or block alignment, and padding as the parameters. It records that "shape, offset, and stride legality differs among these encodings", that ONNX specifies LSB-first packing for its int2 and int4 tensors, that DLPack describes sub-byte packing and separately flags padded storage, and that other runtimes use byte-padded shell types.

**Inference.** `bool`, `i2/u2`, `i4/u4`, and the OCP FP4 and FP6 elements share every one of those parameters and none of their value semantics. Folding packing into four element tracks would answer the same seven questions four times, and the taxonomy's classification of packing as a `StorageEncodingKey` rather than a `TypeKey` is what says the split is the right one.

**Fact — it has no ledger row on purpose, and the obligation is still recorded.** [The dtype support ledger](../docs/dtype-support.md)'s rows are dtype families and packing is its `Physical carrier and encoding` column. [`own-the-dtype-support-maturity-matrix`](own-the-dtype-support-maturity-matrix.md) names "generalized sub-byte bit order, cross-byte layout, tail, alignment, unaligned access, neighbour-safe writes, and repacking beyond the governed whole-component U4 path" among the surfaces the matrix must make explicit without prematurely implementing. This ticket is that surface's owner.

**Fact — what exists today is one whole-component path that has never been dispatched.** The delivered vertical implements the exact whole-component packed-U4 LSB-first, zero-tail dequantization path. The U4 extraction expression in Metal emission is checked at the string level by `strict_affine_u4_dequantization_is_honoured_on_the_measured_apple_profile` (source contains `& 0x0fu`), is absent from the compiled golden fixtures, and has never been device-dispatched.

**Correction — 2026-08-10.** The prior live Fact claimed the U4 extract expression was checked by a test whose name ends in `_is_refused_on_the_measured_apple_profile`. That suffix was false at this base: the measured-profile test for the U4 path is `strict_affine_u4_dequantization_is_honoured_on_the_measured_apple_profile`, which honours the normal-scale decode (string check includes `& 0x0fu`; `require_declared_realization` succeeds). Historical refusal with `SubnormalFlushInArithmetic` applied when a subnormal scale was still admitted; the metal test comments record that the contract now admits only a positive normal scale. Do not claim refusal of the extract expression on that profile.

## Activation trigger

A selected profile chooses a packed code width, **or** a predicate or sub-byte element acquires a physical carrier. **It has not fired**, and the reason is a recorded selection rather than an absence: the first quantized language-model profile chose **unpacked** `StorageScalar::U8`, and [`widen-the-physical-vocabulary-for-per-axis-quantized-component-access`](widen-the-physical-vocabulary-for-per-axis-quantized-component-access.md) states that the selected profile "needs no new carrier, no new encoding, and no new kernel type".

## Operation-axis intersection

`RQ-OP-02` asks whether bit reinterpretation is a semantic family given that its result depends on a physical representation, and its closure test is "whether two targets with different sub-byte packings can both honour one registered key". That is this contract restated from the operation side, so the two must not be decided separately.

## Closes when

The trigger has fired and the seven packing parameters are contract text with a refusal for every combination the vocabulary does not admit, each refusal watched firing.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-10 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).

## Trigger check log

- 2026-08-04 — **not fired at that base.** The original check recorded two `StorageScalar` variants, `U8` and `F32`; that population has since widened and must not be reused as the trigger.
- 2026-08-09 — **not fired; stale carrier census repaired.** `StorageScalar` now has three variants (`U8`, `F32`, `Bf16`) and `StorageEncoding` already carries the governed `BitPacked`/packed-U4 mechanism. Neither fact fires this ticket: the selected language-model profile still chooses **unpacked U8**, no predicate or new sub-byte element has acquired a selected physical carrier, and the only packed width remains the already-bounded whole-component U4 path. Recheck the selected profile anchor `Physical storage | unpacked` and the complete `StorageScalar`/`StorageEncoding` definitions rather than counting an old enum window.
- 2026-08-10 — **not fired; false refused-test Fact repaired.** The U4 measured-profile test is `strict_affine_u4_dequantization_is_honoured_on_the_measured_apple_profile`, not a name ending in `_is_refused_on_the_measured_apple_profile`. Selection and census unchanged from 2026-08-09: unpacked U8 profile, no new packed width, generalized packing contract still untriggered.
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. Two commands, for the trigger's two halves. Packed width: `rg -n 'fn packed_u[0-9]+\(' crates/` returns **1** line, `crates/tiler-compiler/src/boundary.rs:633:const fn packed_u4() -> BitPackedEncoding {`, so one packed width is still the whole population. The trailing `\(` is load-bearing — unanchored, the same pattern also matches three test functions named `packed_u4_is_lsb_first_…` and inflates a population of one to four. **Watched producing the firing answer:** a scratch `pub(crate) fn packed_u2() -> BitPackedEncoding` makes the anchored command report both. Carrier census: `rg -n 'enum StorageScalar' -A 10 crates/tiler-ir/src/program/model.rs` shows **four** variants — `U8`, `F32`, `Bf16`, `U32` — where the 2026-08-09 entry above records three. That entry's own warning that the population widens and must not be reused as the trigger has now applied to itself; the widening is `U32`, which is not sub-byte and does not fire this ticket, but the recorded count is stale. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
- **Carrier census sized from the type — 2026-08-22; verdict unchanged, still not fired.** The entry above already corrected the 2026-08-09 line that says `StorageScalar` `now has three variants`, and that retired wording stays quoted there. What it did not do is size the census from the type, so this entry replaces its `rg -n 'enum StorageScalar' -A 10` window — a hand-read of a fixed number of lines, which is satisfied by an enumeration that has quietly stopped covering its domain. **The site admits `core::mem::variant_count` and already uses it four times.** `grep -rn 'variant_count::<StorageScalar>()' crates/ --include='*.rs'` returns **4 lines**: `const STORAGE_SCALARS` in `crates/tiler-ir/src/program/model.rs`, `const CARRIERS` in `crates/tiler-ir/src/program/alignment.rs`, another in `crates/tiler-compiler/src/boundary.rs`, and one in `crates/tiler-artifact/src/program/codec/tests/vocabularies.rs`. Each is a `[StorageScalar; variant_count::<StorageScalar>()]` array, so a fifth carrier is a **build error at all four** rather than a count that silently goes stale — which is what the entry above's window could not give. All four sit inside `#[cfg(test)]` modules; that still fails the build under `cargo nextest run --workspace`, which the full gate runs, so the guard is real, and saying which build it fails is the honest form of the claim.

  The census is **four** — `U8`, `F32`, `Bf16`, `U32` — with `assert_eq!(STORAGE_SCALARS.map(StorageScalar::tag), [1, 2, 3, 4])` in `model.rs` proving the array is the vocabulary once each in tag order rather than one carrier repeated. Neither the count nor its newest member fires this ticket: `U32` is not sub-byte, the selected language-model profile still chooses unpacked `U8`, and the packed-width half of the trigger is unchanged at one. The changed answer is a fifth variant that is sub-byte, or a second packed width.
