---
id: admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold
title: Admit a governed maximum scalar key for the softmax's shifting fold
status: done
priority: p2
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages, accept-the-governed-maximum-scalar-key, register-the-softmax-realization-law]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`tiler.scalar` carries a governed per-point binary32 `maximum`, so the softmax's row-maximum fold has a scalar operation to be realized as. Without it the softmax's realization law cannot be written at all: its first stage is a fold whose combiner no registered scalar spells.

## Why it was split out rather than landed with the reciprocal square root

**Fact.** [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md) landed `rsqrt-f32` and deliberately did not land this one. The reciprocal square root's fact record is fully determined by already-registered facts; this key's is not, and the undetermined field is `SCALAR_FACT_NAN_RESULT_RULE`.

**The open question, stated so it can be answered by reading rather than guessed.** `SCALAR_FACT_NAN_RESULT_RULE`'s own documentation (`crates/tiler-ir/src/index/scalar.rs`) says every governed scalar states the field and that the absence of `SCALAR_FACT_CANONICAL_NAN_BITS` never carries meaning on its own. The two existing values are `CANONICAL_ARITHMETIC_NAN_PROFILE` -- the operation installs the governed payload -- and `DECLARED_PAYLOAD_PRESERVED`, which is a statement about a *constant's* declared payload. A maximum fits neither:

- `BinaryOp::F32Maximum`'s documentation (`crates/tiler-ir/src/kernel/model.rs:396-418`) states that it "performs no arithmetic" and "selects one of its operands' bit patterns rather than computing a new value", which is why it carries no rounding obligation. A rule naming the canonical arithmetic NaN would claim it *installs* a payload it never computes.
- `tiler::softmax-f32@1`'s own `SOFTMAX_F32_FACT_NAN_BEHAVIOUR` (`crates/tiler-ir/src/semantic/softmax.rs:619-621`) says a quiet NaN "propagates through both folds and poisons the whole row and every arithmetic-NaN result is canonicalized" -- a statement about the *composition*, whose canonicalization comes from the arithmetic steps downstream of the maximum rather than from the maximum itself.

So the key needs a third value in that vocabulary -- an operand-payload-selecting rule -- and minting one is a decision about the published fact vocabulary rather than a mechanical registration. Guessing it would put a wrong claim into a record an out-of-crate reference capability reads through the published field identifiers.

## What must be settled, and where the evidence is

1. The NaN-result rule's spelling, and whether a *signalling* NaN operand is in scope. IEEE 754-2019 `maximum` and ADR 0023's two-family separation are the authorities; `crates/tiler-metal/src/emit.rs` already emits the exact fixup built from ordered comparisons, and `crates/tiler-reference/src/softmax/tests.rs` carries `the_two_extrema_families_are_indistinguishable_through_the_pinned_formula`.
2. Whether the key's name encodes the family. `tiler::softmax-f32@1` pins `Maximum` and deliberately not `MaximumNumber` (`softmax.rs:261`, with the D-2 elimination in that module's header). A bare `maximum-f32` naming one of two families is admissible only if the number-preferring sibling can never later be registered under a name that reads as its complement.
3. The signed-zero ordering. Both Tiler extrema families order `-0.0 < +0.0`; the reference model's `torch.max` does not, and the record at `spikes/numerics/transformer_reference_semantics/results/2026-08-01-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv` carries the four `torch_max_of_signed_zeros_*` rows. Nothing in the decision rests on the reference's behaviour; the fact record must state Tiler's own.

## Closes when

A governed `maximum` scalar key is registered with a fact record whose NaN-result rule is derived from the authorities above rather than chosen, the new vocabulary value is documented beside `SCALAR_FACT_NAN_RESULT_RULE`, and the key lands as a labelled draft with its own acceptance node parked for Tom.

## Outcome — 2026-08-06

`tiler.scalar::maximum-f32@1` is registered as the twelfth governed key and labelled a draft, with [`accept-the-governed-maximum-scalar-key`](accept-the-governed-maximum-scalar-key.md) parked for Tom. Commit `c02d4f7d` on `tkt/admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`, base `dd9def76` — that commit carries all code, tests, and tickets, and is the tree `make full` was run against; the branch tip is one further commit correcting this hash reference and touching no gate-carry path. **The most useful finding is that this ticket's central premise is false**, and the "Closes when" clause about a new vocabulary value is therefore satisfied by showing the value is not needed rather than by minting one.

### The NaN-result rule — derived, and it is an existing value

**The key names `CANONICAL_ARITHMETIC_NAN_PROFILE`. There is no third value, because the operand-payload-selecting rule this ticket expected would be false.**

Three agreeing authorities, in the repository's own evidence order:

1. **ADR 0023's Decision section**, of both extrema families and beside the `-0.0 < +0.0` requirement: "Portable-bitwise NaN results use the existing canonical arithmetic-NaN contract."
2. **`docs/numerical-semantics.md`, "Min and max"**, whose only subject is the two families: "Under portable-bitwise conformance, a produced NaN follows the canonical arithmetic-NaN contract."
3. **Both delivered realizations, inspected.** `maximum_helper` (`crates/tiler-metal/src/emit.rs`) returns `0x7fc00000` directly on its unordered arm — its own comment says "rather than by producing some NaN and relying on a later canonicalization" — and `maximum_f32` (`crates/tiler-reference/src/softmax.rs`) returns `f32::NAN`, which is that pattern. Neither propagates an operand's payload.

**Where the premise fails.** This ticket read `BinaryOp::F32Maximum`'s "performs no arithmetic … selects one of its operands' bit patterns" as implying the operation installs no payload. Those are different claims, and `canonicalize-nan-f32` already separates them: it is documented as a named typed conversion "deliberately not arithmetic", computes nothing, reproduces every non-NaN pattern verbatim including the sign of a zero, and names this profile. The maximum's *ordered* arm installs nothing — but `SCALAR_FACT_NAN_RESULT_RULE` decides the payload of a **NaN result**, which the profile value therefore does not overclaim. This ticket's own reading of `SOFTMAX_F32_FACT_NAN_BEHAVIOUR` (that the canonicalization comes from arithmetic downstream) also does not survive: the reference canonicalizes only at the final multiply, and the maximum's own NaN answer is already the canonical pattern.

**Signalling NaN: in scope, same answer, no clause of its own.** An sNaN operand makes the pair unordered exactly as a qNaN does, so the value contract is identical and both realizations reach it with no special case. The invalid-operation signal IEEE 754 would raise is outside Tiler's observable contract, which `docs/numerical-semantics.md` fixes as value-only (`RaiseNoFlag`).

**Evidence boundary, recorded rather than papered over.** IEEE Std 754-2019 is `metadata-only` in `docs/research/numerics/sources` — purchased and not redistributable — so its clause text for `maximum` is not readable from this tree. What the repository holds is `docs/research/numerics/floating-point-extrema-precedents.md`'s reading: the families propagate NaN and order `-0.0 < +0.0`. That record states **no** payload rule and **no** sNaN rule, which is why the payload is derived from Tiler's own accepted contract and not cited to the standard.

### The name — `maximum-f32`, with its argument

The bare spelling is admissible because the number-preferring sibling's name is already its complement in the standard's own vocabulary: IEEE 754-2019 spells the two `maximum` and `maximumNumber`, ADR 0023 carries them over as `Maximum` and `MaximumNumber`, and under this module's naming rule (spec name, kebab-cased, width appended, as `rsqrt-f32` and `divide-f32` are) the sibling spells `maximum-number-f32`. It can never later read as the wrong family. `maximum-propagating-f32` was eliminated for diverging from the name ADR 0023 anchors on. The real hazard — that a bare `maximum` does not separate this family from `f32::max` or `fmax`, which are both the *other* family — is carried in the registered normative definition, which names the family, the NaN rule, the zero ordering, and both excluded spellings, and which is part of the encoded definition. `the_maximum_shares_the_exact_bit_pattern_fact_record` asserts each of those five clauses is present.

### The signed-zero ordering — Tiler's own fact

`-0.0 < +0.0`, stated in the key's rustdoc and in its normative definition as this operation's own contract under ADR 0023. The reference's `torch.max`/`torch.amax` rows are cited as **contrast only** and nothing rests on them; the rustdoc says so explicitly.

### The fact record, and the sharing

Byte-identical to `canonicalize-nan-f32`'s, through the shared `exact_bit_pattern_f32_scalar_facts` (renamed from `canonicalize_nan_f32_facts`). Both operations select rather than compute, so all three fields agree: `exact-binary32-bits`, the canonical profile with its declared payload, and no contraction field. This is the sharing `elementary_f32_scalar_facts` already does for the exponential and the reciprocal square root, on the same stated ground. The keys stay distinct definitions — different arity, different conformance identity, different `project_reached` bytes — and that is asserted.

### Tests, and the watched failures

Three new tests in the rsqrt landing's idiom: `the_maximum_shares_the_exact_bit_pattern_fact_record`, `the_maximum_refuses_a_foreign_operand_a_mixed_pair_and_a_third_operand` (a uniform `bf16` pair and a *mixed* `f32`/`bf16` pair both reach the inferencer's `tiler.scalar.operand-type`; a third operand reaches the contract's `OperandArity` before the inferencer runs — this is the first binary governed key whose operands could disagree with each other rather than only with `f32`), and `the_maximum_has_no_semantic_counterpart` (no semantic `maximum-f32`, `maximum-number-f32`, or `minimum-f32`, and no sibling extrema scalar). Four existing fact tests gained the key so the populations they name include it.

### Identity — one pin moved, and it is the expected one

Registering the key widens `CanonicalScalarRegistrySnapshotIdentity` and every whole-snapshot provenance derived from it, and leaves reached-only projections alone. `the_landed_one_reader_chain_identities_are_unchanged_byte_for_byte` (`crates/tiler-ir/src/index/law.rs`) pins the exact length and a SHA-256 of three realized sequence identities captured on base `dd9def76`, and all three are unchanged — which covers this widening because a region identity carries the projection of the scalars it *reaches*.

Exactly one pinned identity moved, as the rsqrt landing's did: `explain.rs`'s `deterministic_trace_is_sealed_and_rendered_separately`, `6f153efeb2da5bb1` → `9478647f38ab8df5`. **`implementation/compiler` was added to this ticket's scopes for that one-line update** — no other live claim holds it, checked against `tkt list --status in-progress` and the branch list. **Coordinator note:** that pin's own comment requires recomputation on the tree the change lands in rather than a copy from a producing branch. If anything else moves the frozen semantic, scalar, lowering-capability, or semantic-realization authorities before this merges, recompute with `cargo nextest run -p tiler-compiler -E 'test(deterministic_trace_is_sealed_and_rendered_separately)'` and take the `left` value.

### Filed

[`accept-the-governed-maximum-scalar-key`](accept-the-governed-maximum-scalar-key.md) (awaiting-decision) and [`register-the-softmax-realization-law`](register-the-softmax-realization-law.md) (todo, no dependencies — both walls landed here).

### Checks

`cargo fmt --all --check`, `make lint` (workspace clippy less the three prototypes, `-D warnings`), `make doc` (`RUSTDOCFLAGS="-D warnings"`), `cargo nextest run --workspace` (2899 passed, 7 skipped), `cargo test --workspace --doc`, `make full` (green end to end), `tkt lint`, and `git diff --check`. `tkt guard tkt/admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold` returns exit 0 with `"conflict": false` and `"under_declared": []` against this ticket's three declared scopes; the sibling ticket's Outcome explains why guarding against *its* narrower declaration alone reports an escape.
