---
id: design-the-bf16-computation-and-accumulator-contract
title: Design the BF16 computation, accumulator, and conversion contract
status: in-progress
priority: p2
dependencies: [register-the-bf16-semantic-operation-signatures]
related: [spike-bf16-through-the-second-dtype-seams, widen-the-f16-operation-vocabulary-to-contraction-and-reassociation, own-operation-family-support-matrix, redesign-the-delivered-realization-record-from-typed-evidence, land-the-bf16-conversion-and-accumulator-adr, probe-the-bf16-contraction-pragma-on-the-metal-runtime-path]
scopes: [research/numerics, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, design, dtype, bf16, numerics, accumulator, conversion]
claimed_from: todo
assignee: worker-bf16-accum
lease_expires_at: 1785574773
---
## User-visible outcome

A decided contract for what a BF16 program may compute *in*, accumulate *in*, and convert *to* — so the first workload wanting an F32 accumulator under BF16 storage, or a BF16/F32 conversion, has an accepted answer instead of acquiring one by default. This is a research and design ticket ending in an ADR or a recorded deferral, not an implementation.

## Why this is separate from the pure-BF16 vertical

**Fact.** [The BF16 spike](../spikes/numerics/bf16-second-dtype/README.md) deliberately admits only pure-BF16 constant, multiply, and add, with computation, accumulator, intermediate, and result types all BF16 and all stated separately. It reserves nothing as an ambient default, and `register-the-bf16-semantic-operation-signatures` carries that forward.

**Inference.** That is the whole reason this question needs its own ticket: the pure vertical is correct and useless for the workloads that actually want BF16. Reduced-precision formats exist to be *stored* narrow and *accumulated* wide, so the first real BF16 workload will want an F32 accumulator, and the difference between "we decided BF16 accumulation" and "nobody said, so it inherited the value type" is invisible in the code and decisive in the results.

**Measurement — the obvious lowering does not exist.** Finding 29 of the [Apple numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md) records that `metal` rejects `bfloat v6 = fma(v3, v4, v5)`: MSL has no `bfloat` overload of `fma`, so `fma(bfloat, ...)` returns `float` and emits `fpext`/`air.fma.f32`/`fptrunc`. A source-level fused BF16 operation would therefore measure F32 arithmetic wearing `bfloat` operands.

**Measurement — the contraction defence differs by dtype.** Finding 28 records that under `safe` with `-ffp-contract=fast`, `f16` fuses and `bf16` does not, so an `f16` conclusion may not be carried across.

**Measurement — the runtime compiler contracts regardless.** Finding 30 records that the runtime compiler contracts under `relaxed` and `fast` at all three widths whatever the offline flag says. Any unfused-contraction guarantee derived from an offline measurement does not hold on the runtime path.

## Questions this must decide, each with its elimination stated

- **Is a wider accumulator a property of the operation, of the contract, or of the schedule?** The three differ in what carries the identity and in whether two plans with different accumulators are the same program. The operation's registered facts already state an accumulator type separately, which is a hint but not a decision.
- **Does a BF16 program with an F32 accumulator need an explicit conversion operation at each boundary, or is the accumulator internal to one operation?** The `Cast and convert` row of the operation-family matrix states that admitting any second dtype into a profile forces an explicit conversion, and that no implicit promotion exists after semantic admission. Decide whether an internal accumulator is an exception to that or an instance of it.
- **What are the rounding, overflow, and exceptional-value semantics of a BF16/F32 conversion in each direction?** Narrowing needs a rounding mode and an overflow rule; widening is exact and total for BF16 specifically, and the design must say that this is a property of BF16's shared exponent field rather than a general float-widening fact.
- **Can a fused or contracted BF16 operation be admitted at all, given that no `bfloat` FMA exists?** If the only realization promotes through F32, then admitting it means admitting a mixed-precision operation, which is a different contract from a fused BF16 one. Say which is being proposed, and do not let the name imply the other.
- **What does an unsupported combination return?** Every rejected tuple needs a typed reason, and the reason must distinguish "this is not defined" from "this target cannot do it".

## Required evidence

- Each option tested against correctness, performance, and long-term maintainability, with the eliminated ones stated so a reader can refute the elimination rather than only the conclusion.
- A small end-to-end tensor-program example per surviving option, showing inputs, operations, resolved value types, computation and accumulator types, the numerical contract, and the observable result — including at least one input where the options give **different** bits, since options that never differ are not a decision.
- The measurement boundary of every Apple fact cited, and an explicit statement of which claims are portable and which are not.
- No implementation. A tested prototype under `spikes/` is welcome as evidence and is not acceptance.

## Closes when

Each question above is answered with its elimination, or is explicitly deferred with the evidence that would close it and a reconsideration trigger; the surviving design is written up with worked examples; the public-boundary consequences are identified and taken to Tom rather than self-accepted; and the outcome is an accepted ADR, a recorded deferral, or a bounded experiment — not an open note.

## Graph maintenance

- Depends on the pure-BF16 signatures existing, so the design has a concrete thing to widen rather than a hypothetical.
- Blocks nothing in the pure vertical by construction. The vertical must remain shippable with all four types equal to BF16.
- Do **not** widen `register-the-bf16-semantic-operation-signatures` to carry any of this. That ticket's value is that it reserves nothing.
- Later triggers stay explicit and separate: F16 and F64 verticals, integer and boolean families, quantized and MX compound schemes, OCP reduced formats, and vendor formats each need their own selection through the dtype-addition recipe in `docs/dtype-support.md`. Do not create a generic "support every dtype" implementation ticket from this one.

## Outcome

**The design is complete and every one of the five questions is answered with a stated elimination.** The derivation, the eliminations, the worked examples with differing bits, the portable-versus-host-bound split, the public-boundary list, and a drafted ADR body live in [BF16 computation, accumulator, and conversion](../docs/research/numerics/bf16-computation-accumulator-and-conversion.md). The evidence is a bounded experiment: six new stages and three new perturbations in [the BF16 second-dtype spike](../spikes/numerics/bf16-second-dtype/README.md), run under that spike's own README invocation.

### Dispatch correction — the ADR file is out of scope, and that is why it is a separate ticket

**Fact, reproducible in one line.** `ticketsplease.toml` maps `docs/decisions/[0-9]*.md` to `contracts/decisions` and `docs/decisions/README.md`, `docs/dtype-support.md`, and `docs/roadmap.md` to `contracts/navigation`. This ticket declares `research/numerics` and `contracts/numerics` exclusively and `project/tickets` shared, so writing an ADR file or editing the decision catalog from this branch is a guard escape of exactly the class [the spike's finding 8](../spikes/numerics/bf16-second-dtype/README.md) records. `contracts/navigation` was additionally held by `admit-the-silu-activation-family`, in-progress at the time, so the catalog edit would have collided with live work. [`land-the-bf16-conversion-and-accumulator-adr`](land-the-bf16-conversion-and-accumulator-adr.md) carries both and holds both scopes; the drafted ADR is written to be landed verbatim.

### The five answers

1. **A wider accumulator is a property of the operation**, carried in its registered definition facts and hence its identity, as `CONTRACTION_F32_FACT_ACCUMULATOR_TYPE` already is. *Schedule* eliminated on correctness — two folds differing only in accumulator width return different bits, so a planner choosing between them would price meaning, which the contract forbids by name. *Numerical contract* eliminated structurally — that contract speaks for exactly one `ArithmeticType` and honourability is keyed by an arithmetic subject, so an operation naming two widths names two subjects and no one contract can speak for it; and every contract dimension is a permission, which does not change what an operation means. *Per-node attribute under one key* eliminated on what facts are for — ADR 0087 admits its structure attribute precisely because the numerical signature does not vary with it, and the accumulator *is* the signature; a per-node accumulator leaves the definition pointing at an attribute instead of stating a fact, and forces one registered evaluator to mean two arithmetics. **A boundary on the answer:** the landed pointwise family's accumulator field is structurally present and semantically degenerate — a binary add performs no fold — so the question does not become real until a *reducing* BF16 operation exists, and none does.
2. **An internal accumulator is an instance of the no-implicit-promotion rule, not an exception**, and ADR 0009's own alternatives-considered already decided it: graph-level casts for every scalar step inside a reduction would encode the operation's internal iteration in the public graph. The matrix rule governs conversions between two *values*; an operation's internal roles are not two values. **And for BF16/F32 the two spellings agree bit for bit at a pointwise boundary** — widening is exact and a BF16 product needs 16 significand bits where binary32 holds 24 — so the conversion family is the prerequisite and a BF16 accumulating key is not. The equivalence stops inside a reduction, where the explicit spelling would materialize an `M·N·K` intermediate the reduction does not have.
3. **Two separate conversion families, one per direction, carrying disjoint field sets.** Widening is exact and total and therefore carries **no** rounding, overflow, or NaN-mapping field — the ADR 0041 rule that an exact conversion carries no rounding rule. The property is BF16-specific: BF16 and binary32 share an exponent width and bias and BF16's trailing field is a prefix of binary32's, so a subnormal widens to a **subnormal**. Binary16's widening is also exact but by a *different* argument — its exponent range is strictly inside binary32's, so it renormalizes and its subnormals become binary32 **normals** — and findings 24 and 25 measure that difference as the reason `bf16` flushes on the qualified Apple row where `f16` does not. Narrowing carries round-to-nearest-ties-to-even, overflow to a signed infinity at the inclusive midpoint above the largest finite BF16 value, NaN canonicalization to `0x7fc0`, gradual underflow, and preserved signed zero; three of those change an answer in the sweep and the fourth is forced, because payload truncation is not total.
4. **A fused BF16 operation is not admissible; what is admissible is a mixed-precision operation that says so.** Finding 29 establishes the promoted route is the only one MSL offers, and the promoted route is **not** the correctly rounded BF16 fused result — one ulp apart on the derived witness and on 21,546 of 262,144 swept triples. The same route *is* exact for one multiply or add (0 of 524,288), because the double-rounding bound `q >= 2p + 2` is `24 >= 18` — an inequality the retained Apple record and `crates/tiler-metal/src/target.rs` both already state at this pair — and the fused multiply-add is not an operation that bound covers. **Proposed name: mixed-precision, never fused-BF16.**
5. **Five typed outcomes discriminated by what would fix them** — not defined, malformed request, defined but unimplemented, unhonourable on this target, unknown on this target — with the first owned by the semantic registry and the last two by the declaring target profile, so a registry answer and a target answer cannot collapse. `VariantIneligibility`'s own "the repairs differ" rule is the precedent copied. One new obligation: a rejected conversion must name its **direction**, because the two directions are two families.

### The differing-bits examples

- **Accumulator, no promotion and no target involved.** `0x3f80` (BF16 `1.0`) folded with four copies of `0x3b00` (`2^-9`) returns **`0x3f80`** at a BF16 accumulator and **`0x3f81`** at a binary32 one. One BF16 ulp at `1.0` is `2^-7`, so each addend alone rounds away; four sum to exactly one ulp. Control: with one contributor both return `0x3f80`.
- **Fused multiply-add.** `a = 0x3fc0` (`1.5`), `b = 0x3fb2` (`1.390625`), `c = 0xb300` (`-2^-25`). `192 × 178 = 267 × 2^7`, so the exact product is exactly a BF16 halfway point whose lower quantum count 133 is **odd**; the exact `a·b + c` is strictly below it and rounds to **`0x4005`**, while binary32 rounds the sum back onto the tie and ties-to-even sends it up to **`0x4006`**. Control: moving `c` to `2^-20` makes both routes return `0x4005`.
- **Narrowing.** `0x3f80c000` narrows to `0x3f81` under nearest-even and `0x3f80` under truncation; `0x3f808000` narrows to `0x3f80` under ties-to-even and `0x3f81` under ties-away; `0x7f7fffff` narrows to the infinity `0x7f80`.

### Deferrals, each with its closing evidence and trigger

- **Is an unfused BF16 contract honourable on any Apple runtime path?** Finding 30 measures the runtime compiler contracting at all widths with no `MTLCompileOptions` counterpart; finding 10 records `#pragma METAL fp contract(off)` as an accepted mechanism the probe deliberately did not adopt. Closes with a runtime probe using the pragma against its own unperturbed neighbour: [`probe-the-bf16-contraction-pragma-on-the-metal-runtime-path`](probe-the-bf16-contraction-pragma-on-the-metal-runtime-path.md). Trigger: before any BF16 contract declaring contraction forbidden is offered to a Metal profile.
- **Can a chain separate native `bfloat` arithmetic from binary32-precision evaluation on a device?** Finding 24 names this as the experiment that would separate its two surviving hypotheses and records that none has been taken. A *candidate* is now derived — the accumulator witness above is exactly a two-operation chain whose intermediate rounding is the difference — with the obstacle stated: in MSL an intermediate assigned to a `bfloat` variable is rounded by the language's own typing, so a null result would be about the source rather than about the hardware. No ticket, because nothing in the contract depends on the answer.
- **Is a truncating narrowing ever wanted?** Separated from nearest-even in 32,704 of 65,536 patterns. Closes with a named producer or consumer; trigger is a frontend importing weights produced by truncation.
- **Does the conversion family generalize beyond BF16/F32?** Every derivation rests on BF16-and-binary32-specific inequalities. Closes with the second float pair a workload selects; trigger is the F16 vertical.

### A correction landed along the way

`spikes/numerics/bf16-second-dtype/src/bf16.rs` argued that a host-`f32` oracle would be wrong because `f32`'s 24-bit significand "does not exceed twice `bf16`'s 8-bit significand by enough to make the second rounding innocuous", and the README's finding 5 inferred from it that such an oracle "would agree with a double-rounding implementation *because it shares the defect*". The inequality is backwards: `2p + 2` at `p = 8` is 18 and `24 >= 18` **holds**, which is exactly why finding 24 of the Apple record and `crates/tiler-metal/src/target.rs` both say no single operation can expose an `f32` intermediate. Stage 2 now checks it directly. Both texts are corrected with the original rationale preserved, and the conclusion survives with a different reason: the exact-rational oracle is right because it does not rest on that bound, which covers neither the fused case nor an accumulation.

### Public-boundary consequences, listed for Tom and not self-accepted

Two new conversion operation keys, one per direction, with disjoint field sets; a typed conversion-contract vocabulary in the semantic layer; a `(value type, accumulator type)` key for a reducing BF16 family if a workload asks; fact-field vocabulary growth for a mixed-precision record whose computation type differs from its result type, and whether it reuses the landed BF16 field numbering or starts its own; whether `ScalarArithmetic` gains a second subject at all, which `admit-a-bf16-scalar-arithmetic-subject` owns; and the fifth refusal class plus the obligation that a conversion refusal name its direction.

### Verification

`cargo fmt`, `cargo clippy --release --all-targets` clean, and one release run of the spike in 3.8 s with every stage agreeing and all ten perturbations detected. `tkt lint`, `git diff --check`, `tkt guard --base eaef53c`, and `make full` were run on the completed branch.
