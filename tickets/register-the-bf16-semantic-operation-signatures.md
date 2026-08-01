---
id: register-the-bf16-semantic-operation-signatures
title: Register the pure-BF16 constant, multiply, and add operation signatures
status: done
priority: p1
dependencies: []
related: [spike-bf16-through-the-second-dtype-seams, register-the-accepted-built-in-dtype-catalog, own-operation-family-support-matrix, design-the-bf16-computation-and-accumulator-contract]
scopes: [implementation/ir, implementation/compiler, implementation/reference, contracts/navigation, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, semantics, operations]
---
## User-visible outcome

A semantic program can name a pure-BF16 constant, multiply, and add. Today `tiler::bf16@1` is a recognized identity that no operation signature admits, so a BF16 tensor can be described and nothing can be done to it.

## Why this is the second root

**Fact, at `ef3c051`.** `register_builtin_dtype_catalog` registers `tiler::bf16@1` with a complete structural descriptor, and its module documentation states that a row "creates no operation signature, reference evaluator, storage carrier, kernel type, target dispatch fact, or backend lowering". The registered operation keys are all F32-specific: `constant_f32_op`, `multiply_f32_op`, `add_f32_op`.

**Fact.** [The BF16 spike](../spikes/numerics/bf16-second-dtype/README.md) confirmed `SemanticRegistryProvider::register_operation` is public and dtype-neutral, and derived the exact semantics these keys must carry.

**Inference.** These are new operation keys, not widened ones. Operand type is part of an operation's identity under ADR 0026, so `tiler::multiply-bf16@1` sits beside `tiler::multiply-f32@1` rather than replacing it, and nothing may make an F32 operation accept a BF16 operand.

## Implementation keys

- Three keys — `tiler::constant-bf16@1`, `tiler::multiply-bf16@1`, `tiler::add-bf16@1` — each with an inferencer refusing any operand that is not `tiler::bf16@1`.
- The constant's payload attribute is exact BF16 bits, validated against the registered descriptor's width, in the same shape `F32_CONSTANT_BITS_ATTRIBUTE` uses for binary32. A binary32 payload on a BF16 constant is refused.
- Each operation's canonical facts state, separately and explicitly: computation type, accumulator type, intermediate-materialization type, and result type. All four are BF16 here, and **stating them separately is the point** — a future F32 accumulator must be an explicit change to a fact, not the removal of an assumption. The facts also state round-to-nearest-ties-to-even at every observable materialization and the canonical arithmetic NaN payload.
- The normative definition names the ratified RISC-V BF16 operand format and the preserved source id, matching the catalog row. Do not restate the format table.
- **No FMA, no contraction, no reassociation, no mixed precision, no implicit promotion.** Each rejects by typed reason. `design-the-bf16-computation-and-accumulator-contract` owns whether any of them is ever admitted.
- The typed facade (`F32Constant`'s peer) may be added for BF16, but the marker binding and the operation keys are the deliverable; a facade with no registry behind it is not.

## Required evidence

- A program applying `tiler::multiply-bf16@1` to two BF16 values verifies; the same operation applied to an F32 value is refused by name, and to a mixed pair is refused by name.
- An F32 operation applied to a BF16 operand is refused, so registration did not weaken the existing signatures.
- A constant carrying a binary32-width payload is refused.
- The four type facts are readable from the registered operation and are all `tiler::bf16@1`, asserted individually rather than as a group.
- Registering these keys does not make any BF16 program compilable, reference-evaluable, or dispatchable; a test asserts each of those still fails closed.

## Closes when

The three keys are registered with complete facts, every refusal above is observed failing, the operation-family matrix row in `docs/roadmap.md` moves from R1 to R3 for BF16 arithmetic with its evidence stated, `docs/dtype-support.md`'s BF16 `Semantic operation signatures` cell moves off `absent/unsupported`, and no other cell moves.

## Graph maintenance

- Gates `evaluate-bf16-reference-semantics` and `admit-bf16-into-the-schedule-and-kernel-vocabulary`. Independent of the target-profile children.
- Do not re-register the identity; `register-the-accepted-built-in-dtype-catalog` owns it and is `done`.
- The `Cast and convert` row of the operation-family matrix states that admitting any second dtype into a profile forces an explicit conversion operation. This ticket does **not** discharge that; it deliberately admits no BF16/F32 conversion, and the first program needing one blocks on the conversion row rather than acquiring an implicit promotion here.

## Outcome

**Done 2026-08-01.** The three keys are registered with complete facts, every refusal is observed failing, and the two documentation cells moved.

### Registered signatures

`tiler::constant-bf16@1` — arity 0→1, one required `FloatBits` attribute at `BF16_CONSTANT_BITS_ATTRIBUTE` (field 1). Normative definition: `tiler::constant-bf16@1; exact payload in the ratified RISC-V BF16 operand format; source riscv-unprivileged-isa-20260120; tiler::bf16@1`. Facts: computation/accumulator/intermediate-materialization/result all `tiler::bf16@1` (fields 1–4); rounding `none-the-declared-payload-is-already-the-exact-bf16-encoding`; subnormals `preserved-every-subnormal-encoding-denotes-a-distinct-constant`; signed zero `preserved-negative-zero-and-positive-zero-are-distinct-constants`; NaN `preserved-exactly-the-declared-payload-is-not-canonicalized`; infinity/overflow `preserved-both-infinity-encodings-denote-constants-and-no-overflow-arises`; payload rule `exact-bf16-bits` (field 16).

`tiler::multiply-bf16@1` and `tiler::add-bf16@1` — arity 2→1, no attributes, shape rule "match or one operand scalar". Normative definitions name separate multiplication/addition over the same operand format and source id. Both carry one shared arithmetic record: the same four type facts, all `tiler::bf16@1`; rounding `bf16-round-to-nearest-ties-to-even-at-every-observable-materialization`; subnormals `preserved-operands-and-results-in-the-bf16-subnormal-range-are-not-flushed`; signed zero `ieee-754-signed-zero-rules-over-the-bf16-value-set`; NaN `quiet-nan-propagates-and-every-arithmetic-nan-result-is-canonicalized`; canonical NaN payload `0x7fc0` tagged `tiler::bf16@1` (field 9); infinity/overflow `ieee-754-infinity-rules-and-overflow-rounds-to-infinity-under-ties-to-even`; and five `false` fences — mixed precision, implicit promotion, ADR 0015 contraction, fused multiply-add, reassociation (fields 11–15). Neither declares an algebraic capability, deliberately: `tiler::add-f32@1` declares ordered associativity and these withhold it, because a missing declaration reads as unknown rather than as the inverse law.

The four type facts are four separate fields on all three definitions, asserted individually in `every_bf16_definition_states_all_four_types_separately_and_each_is_bf16`. Fields 9 and 11–15 are conditional on the two arithmetic definitions and **absent** on the constant, which has no operand pair to promote, no adjacent rounding to contract, and no contributors to regroup — a `false` there would claim a permission exists and is withheld. Field 16 is conditional the other way. The constant's payload width is read from the registered `tiler::bf16@1` descriptor rather than from a literal, so the validation and the catalog row cannot drift.

### Refusal table

| Refused application | Typed code | Watched failing in |
| --- | --- | --- |
| BF16 arithmetic on an F32 pair (implicit promotion) | `bf16.binary.implicit-promotion` | `a_bf16_arithmetic_refuses_an_f32_operand_pair_as_an_implicit_promotion` |
| BF16 arithmetic on a mixed BF16/F32 pair, either order | `bf16.binary.mixed-precision` | `a_bf16_arithmetic_refuses_a_mixed_operand_pair_by_a_different_name` |
| BF16 constant carrying a binary32 payload | `bf16.constant.bits.format` | `the_bf16_constant_refuses_a_binary32_payload_and_a_wrong_width_payload_separately` |
| BF16 constant carrying a bf16-tagged payload at another width | `bf16.constant.bits.width` | same |
| BF16 constant bits that are not `FloatBits` | `bf16.constant.bits.kind` | `every_structural_refusal_on_the_bf16_family_can_say_no` (schema `attribute-kind` fires first) |
| Mismatched non-scalar shapes on a pure-BF16 pair | `bf16.binary.shape` | `every_structural_refusal_on_the_bf16_family_can_say_no` |
| F32 arithmetic on a BF16 or mixed pair (unweakened) | `binary.type` | `registering_bf16_did_not_weaken_the_existing_f32_signatures` |

FMA is refused by *absence*: no fused BF16 key exists, and the spike's retained record has `metal` rejecting `fma(bfloat, bfloat, bfloat)` outright, so there is no primitive to contract to. Contraction and reassociation are declared `false` facts and are additionally unreachable because no algebraic capability is declared. `design-the-bf16-computation-and-accumulator-contract` owns whether any is ever admitted.

### Perturbations, each observed failing

Four applied one at a time to `crates/tiler-ir/src/semantic/bf16.rs` and reverted: canonical NaN `0x7fc0`→`0x7fc1` (caught by the NaN fact test); the constant width check made vacuous (caught by the width test); the mixed-precision code replaced with the promotion code (caught by the distinct-names test); the accumulator type fact changed to `tiler::f32@1` (caught by the four-type test). Each failed in exactly the intended test with the intended message.

### Accuracy-gate reading

**Fact.** Milestone 1 forbids admitting an operation "before its accuracy contract is canonically serialized and reference-evaluated end to end" only for *transcendental* accuracy contracts (`docs/roadmap.md`, Milestone 1 bullet 5), and Milestone 2 restates it for "any transcendental or GELU". The operation-family matrix places that gate on the `Pointwise transcendentals: Exp, Log, Sin, Gelu, and similar` row, whose trigger is Q-SEM-004. **Inference.** Constant, multiply, and add are not transcendental — they are exact algebraic operations with a fully resolved rounding rule and no approximation envelope — so the gate does not bind them, exactly as it does not bind the four F32 operations already registered under the same reasoning. Nothing here registers an accuracy contract, an ULP tolerance, or an approximation envelope, and `BF16_FACT_ROUNDING` is a rounding rule rather than an accuracy claim.

### Digest movement

The explain request qualifier moved `bae4788d2fc79631` → `b610aff7e1907c00` at `crates/tiler-compiler/src/explain.rs`, because the request subject covers the frozen semantic registry snapshot and it now admits three further operation families. This is the assertion working. Only the semantic half of the subject moved — no capability row, lowering capability, or target declaration names BF16 — and no encoding version advanced. It is the only pin, golden, or fixture that moved.

### Scopes added, and why

Three, following the precedent in [`admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode`](admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode.md):

- `implementation/compiler` — the one-line digest rebaseline plus its comment, and the capability-table test below.
- `implementation/reference` — one test asserting the three keys are not reference-evaluable.
- `contracts/navigation` — **a ticket defect corrected.** This ticket's `Closes when` requires moving cells in `docs/roadmap.md` and `docs/dtype-support.md`, but both map to `contracts/navigation` in `ticketsplease.toml`, and the ticket declared `contracts/foundation`, which contains neither file. Without this the ticket could not have closed on its own terms.

### Two checks that had to change, neither weakened

`catalog::tests::no_registered_operation_admits_a_recognized_but_unsupported_identity` listed `bf16` among identities no operation admits. That is now false by construction, so `bf16` left the population — and a sibling test, `exactly_the_three_bf16_keys_admit_the_bf16_identity`, replaces what it gave up: it counts the admitting set from the frozen registry, compares it against the two arithmetics by name, reaches the constant through its own attribute, and asserts the neighbours `f16` and `f64` are still refused.

`policy::tests::the_capability_table_names_exactly_the_admitted_operations` compared the compiler's capability table against the registry in both directions. The three BF16 keys are subtracted through a named `UNPLANNED_OPERATIONS` list rather than a predicate, and `every_unplanned_operation_is_registered_and_consumes_no_dimension` proves each subtracted name is registered, has no row, resolves to no capability, and consumes no canonical dimension. **Adding BF16 rows would have been the wrong fix**: a row enters each dimension it lists into `is_consumable`'s union, which is what decides whether a *contract* may permit that dimension at all, so it would have widened the build's numerical surface for an operation no target profile can state a contract for.

### Registration did not make BF16 usable below the semantic layer

Checked in three places, each with a live F32 neighbour so the refusal is about BF16 rather than a dead path: `tiler-reference`'s `the_registered_bf16_operations_are_not_reference_evaluable` (all three keys fail `MissingCapability`), `tiler-compiler`'s `a_pure_bf16_program_is_statable_and_refused_at_the_request_boundary` (`compile()` refuses with `phase: "strategy", rule: "dtype-f32"` — the rule that actually says no, reached before the operation-vocabulary check), and the capability-table pair above for dispatch.

### Deliberately not done

- **No BF16/F32 conversion in either direction.** The `Cast and convert` row is not discharged; a program needing a conversion blocks on that row rather than acquiring an implicit promotion here.
- **No reference evaluator** (`evaluate-bf16-reference-semantics`) and **no schedule/kernel vocabulary** (`admit-bf16-into-the-schedule-and-kernel-vocabulary`); both are now unblocked.
- **No `ScalarArithmetic` change.** The spike's blocking seam is untouched and still fail-closed; `admit-a-bf16-scalar-arithmetic-subject` owns it.
- **The standard provider revision stayed at 7.** Its own doc states the revision moves only for a change the content encoding cannot already carry; three new keys are new entries in the projection's ordered map, so the projection carries them, and bumping would invalidate every pinned provenance for existing keys whose admitting authority did not change.
- **`docs/roadmap.md` absence check 3 was extended** to name `register_standard_bf16` and `bf16.rs`. It enumerates registrars by an explicit alternation over an explicit file list, so a registrar missing from either makes the check report a smaller registry than exists and read as a pass.
