---
id: scope-the-bit-reinterpretation-family-against-its-storage-carrier
title: Scope the bit-reinterpretation family against its storage carrier
status: deferred
priority: p3
dependencies: [generalize-the-sub-byte-storage-encoding-contract]
related: [scope-the-bitwise-and-shift-families, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, storage-encoding, deferred]
---
## User-visible outcome

`RQ-OP-02` is answered: bit reinterpretation is either a semantic family whose identity carries a declared storage encoding, or a physical construct with no semantic key — and whichever it is, the answer is recorded rather than left to whoever first needs a bitcast.

## Why this is deferred rather than open

**Fact — the question, and the falsifiable test that closes it.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s `RQ-OP-02` asks whether bit reinterpretation is a semantic family "given that its result depends on a physical representation", and fixes the test: "whether two targets with different sub-byte packings can both honour one registered key; if they cannot, the semantic classification is refuted."

**Fact — the rank consequence is normative in a primary source.** StableHLO `bitcast_convert` (`stablehlo-spec-v1.18.0`) states that when `num_bits(E') < num_bits(E)` the result gains a trailing axis with `dim(result, R) * num_bits(E') = num_bits(E)`, and that `bits` "returns in-memory representation of a given value, and its behavior is implementation-defined because the exact representation of tensors is implementation-defined". So this is the first family in the inventory whose *semantic* result depends on a physical fact, which is the separation [ADR 0046](../docs/decisions/0046-separate-logical-access-from-storage-addressing.md) exists to protect.

**Fact — [ADR 0018](../docs/decisions/0018-portable-bitwise-nans.md) already accepts the portable-bitwise preserve posture for this construct.** ADR 0018 Decision separates arithmetic NaN canonicalization from bit-preserving movement (views and bit-preserving copies preserve selected source bits), and Consequences state that "Copies, views, and bit reinterpretation do not destroy payload bits." That preserve-bits rule is accepted ADR text, not an open inference. What this ticket still owes is `RQ-OP-02` classification — semantic key whose identity carries a declared storage encoding versus physical construct with no semantic key — and writing that accepted preserve-bits boundary into the record that admits or refuses the family. A non-canonical NaN payload can still reach a bitwise consumer after a bit-preserving reinterpretation; that is a consequence of the accepted preserve rule, not evidence that ADR 0018 has no position.

**Inference — the test cannot be run before the packing contract exists.** Two targets with different sub-byte packings can only be compared once sub-byte packing is a contract rather than one governed U4 extraction expression, which is [`generalize-the-sub-byte-storage-encoding-contract`](generalize-the-sub-byte-storage-encoding-contract.md)'s subject. That ticket is the dependency, not a related note.

## Activation trigger

The packing track delivers a storage-encoding contract that admits at least two distinct packings of one logical width, **and** a named producer requires a reinterpretation. The dtype axis states the same join from its side: the delivery record's dtype peer records `RQ-OP-02` as joining track D-10 and being "D-10's contract restated from the operation side".

## What the work would be, when it starts

Run the two-packing test against the delivered encoding contract and record which way it goes. If the semantic classification survives, state the declared-carrier field in the family's identity and accept that the key is no longer target-neutral, with the consequence for artifact identity written down; if it is refuted, reclassify the construct as physical and say where a caller asks for one. Either way, record ADR 0018's already-accepted preserve-bits boundary in the admit-or-refuse record, and state the rank change as part of shape inference rather than as a note.

## Explicit non-goals

- The packing contract itself, which the dependency owns.
- Bitwise and shift operations, whose family is [`scope-the-bitwise-and-shift-families`](scope-the-bitwise-and-shift-families.md)'s; the coherent spelling for a bitwise operation over a float is a reinterpretation followed by a shift, and that composition is the reason both exist, not a reason to merge them.
- Sub-byte and block-scaled carriers, which the taxonomy marks explicitly unsupported for this family today.

## Closes when

`RQ-OP-02` is answered by the stated test against a real second packing, the ADR 0018 boundary is recorded as a rule in the record that admits or refuses the family, and the taxonomy's `RQ-OP-02` row is updated to name the answer instead of the question.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-16** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-04 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** The packing dependency's own trigger is unfired: the first quantized profile selected unpacked `StorageScalar::U8`, and the one packed construct that exists is the U4 extraction expression, checked at the string level and never dispatched. One packing is not two. Recheck: read the `Trigger check log` of [`generalize-the-sub-byte-storage-encoding-contract`](generalize-the-sub-byte-storage-encoding-contract.md) and re-run the command its last line names.
- 2026-08-09 — **not fired.** The packing dependency still records one selected unpacked U8 carrier and one governed U4 extraction spelling, not two distinct packings of one logical width; no named producer requires a bit reinterpretation. BF16's exact-pattern constant reinterpretation is a backend emission detail, not an admitted semantic bitcast family and does not fire this trigger.
- 2026-08-10 — **not fired.** Still one packing of one logical width and no named bit-reinterpretation producer. U4 emission on the measured Apple profile is now honourable under the normal-scale freedom (`strict_affine_u4_dequantization_is_honoured_on_the_measured_apple_profile`); the 2026-08-05 "checked at the string level and never dispatched" clause is historical emission status and must not be reread as current. That honourability change does not admit a second packing or a semantic bitcast family.
- **Recheck repaired — 2026-08-22; no verdict re-decided here.** The 2026-08-05 entry's recheck was *"read the `Trigger check log` of [`generalize-the-sub-byte-storage-encoding-contract`](generalize-the-sub-byte-storage-encoding-contract.md) and re-run the command its last line names"*. **That log's last line names no command**, so the delegation resolves to nothing runnable and the recheck can only ever be discharged by restating the previous verdict. Stated inline instead, so this ticket carries its own check: the dependency is unfired while exactly one sub-byte packed width is constructed, and

  ```sh
  rg -n 'fn packed_u[0-9]+\(' crates/
  ```

  returns exactly that one line, `crates/tiler-compiler/src/boundary.rs:633:const fn packed_u4() -> BitPackedEncoding {`. The trailing `\(` is load-bearing: unanchored, `fn packed_u[0-9]+` also matches the three test functions named `packed_u4_is_lsb_first_…`, `packed_u4_reads_name_shared_edge_bits_…`, and `packed_u4_whole_value_writes_…`, inflating a population of one to four. **Watched producing the firing answer:** on a scratch copy a second width helper `pub(crate) fn packed_u2() -> BitPackedEncoding` was added, and the anchored command reported both lines. One packing is still not two.
