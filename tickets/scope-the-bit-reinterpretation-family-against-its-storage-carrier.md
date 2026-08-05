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

**Fact — its tension with [ADR 0018](../docs/decisions/0018-portable-bitwise-nans.md) is stated and unresolved.** ADR 0018 canonicalizes arithmetic NaNs so a bitwise result is portable; a bit-preserving reinterpretation that canonicalized would not be bit-preserving, and one that did not would let a non-canonical payload reach a bitwise consumer. The taxonomy's proposed resolution — canonicalization is a property of arithmetic result materialization, which this family is not — is "an inference from two accepted positions, not an accepted rule", and it belongs in the ADR that eventually admits the family.

**Inference — the test cannot be run before the packing contract exists.** Two targets with different sub-byte packings can only be compared once sub-byte packing is a contract rather than one governed U4 extraction expression, which is [`generalize-the-sub-byte-storage-encoding-contract`](generalize-the-sub-byte-storage-encoding-contract.md)'s subject. That ticket is the dependency, not a related note.

## Activation trigger

The packing track delivers a storage-encoding contract that admits at least two distinct packings of one logical width, **and** a named producer requires a reinterpretation. The dtype axis states the same join from its side: the delivery record's dtype peer records `RQ-OP-02` as joining track D-10 and being "D-10's contract restated from the operation side".

## What the work would be, when it starts

Run the two-packing test against the delivered encoding contract and record which way it goes. If the semantic classification survives, state the declared-carrier field in the family's identity and accept that the key is no longer target-neutral, with the consequence for artifact identity written down; if it is refuted, reclassify the construct as physical and say where a caller asks for one. Either way, land the ADR 0018 boundary as a rule rather than as this record's inference, and state the rank change as part of shape inference rather than as a note.

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
