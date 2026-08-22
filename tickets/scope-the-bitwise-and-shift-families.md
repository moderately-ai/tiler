---
id: scope-the-bitwise-and-shift-families
title: Scope the bitwise and shift families
status: deferred
priority: p3
dependencies: [define-the-integer-numerical-contract-and-honourability-subject]
related: [scope-the-bit-reinterpretation-family-against-its-storage-carrier, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, integers, bitwise, deferred]
---
## User-visible outcome

Bitwise and shift operations over integers have a stated signature — including the one field the ecosystem routinely omits, the shift amount's type and admissible range — rather than being reachable only as whatever a backend happens to spell.

## Why this is deferred rather than open

**Fact.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-15 is atomic per operation, one or two operands, "identical resolved integer types; shift amount type and range are part of the signature", and fixes that "an out-of-range shift is a defined result or a refusal, never undefined; arithmetic and logical right shift are separate operations".

**Fact — the logical/bitwise split is a decided posture, not an open one.** The taxonomy records that StableHLO uses `and`, `or`, `not`, `xor` for both the boolean and the integer case and resolves by operand type, while TOSA and the Array API separate them; it follows TOSA and the Array API, so "neither family's signature contains 'depending on the operand type'". F-14 logical operations over predicates is therefore a different family with a different track.

**Fact — bitwise over a float is intentionally invalid, and that must not become work.** The taxonomy's enumerated intentionally-invalid list includes "a bitwise or shift operation over a floating-point type", with the reason: "the coherent spelling is F-04 followed by F-15; admitting the direct form would silently fix a storage representation inside an arithmetic family." An intentionally invalid case must never become a ticket, so this ticket carries the refusal and never the widening.

**Inference — the blocker is the integer family, not this one.** Every operand and result here is an integer, and no general integer arithmetic operation is registered; the honourability subject an integer family would be declared at does not exist. Scoping bitwise before that would state a signature no target can be asked about.

## Activation trigger

The integer track's trigger fires — a named tensor workload selects an exact width, an operation family, an overflow behaviour, a storage, a target, and a corpus — **and** that workload's operation list includes a bitwise or shift operation. Quantized code arithmetic does not fire it; the ledger excludes quantized codes from firing the integer trigger by name, and this ticket inherits that exclusion.

## What the work would be, when it starts

State per operation: the identical-integer-type admissible set, the shift amount's own type and admissible range as a signature field rather than a convention, the out-of-range result or refusal decided rather than inherited, arithmetic and logical right shift as separate keys, the exact-bit-manipulation oracle at the declared width, and the scalar kernel emission. Then state the two refusals as refusals — a float operand, and a predicate operand, the second belonging to F-14.

## Explicit non-goals

- Logical operations over predicates, which are the predicate track's.
- Bit reinterpretation, which is [`scope-the-bit-reinterpretation-family-against-its-storage-carrier`](scope-the-bit-reinterpretation-family-against-its-storage-carrier.md)'s; this family consumes its result and does not subsume it.
- Any admission of a bitwise operation over a float, which is intentionally invalid.

## Closes when

Each operation has a signature naming its shift-amount field and its out-of-range rule, an exact oracle, and an emission — or the family is recorded as unneeded by any named integer consumer.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-21** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-15 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** The integer track's trigger is unfired and no bitwise or shift operation is named by any workload. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
- 2026-08-09 — **not fired.** U32 index operands and U8 quantized-code carriers now exist in bounded roles, but the integer numerical-contract track remains deferred and no named workload includes an integer bitwise or shift operation. Those carrier admissions are explicitly not the general integer arithmetic consumer this trigger requires.
- **Recheck restored — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — was carried forward unmet. Restored from this log's own history rather than invented: the most recent command this log names is `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u`, and run at this base it returns **50** unique keys. A result other than the 50 recorded here is the changed answer. This census counts **unique governed keys** through `sort -u`, not lines of output. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
