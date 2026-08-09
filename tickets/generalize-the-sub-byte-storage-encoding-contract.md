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

**Fact — what exists today is one whole-component path that has never been dispatched.** The delivered vertical implements the exact whole-component packed-U4 LSB-first, zero-tail dequantization path. The U4 extraction expression in Metal emission is checked at the string level by a test whose name ends in `_is_refused_on_the_measured_apple_profile`, is absent from the compiled golden fixtures, and has never been dispatched.

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
