---
id: design-the-bf16-computation-and-accumulator-contract
title: Design the BF16 computation, accumulator, and conversion contract
status: in-progress
priority: p2
dependencies: [register-the-bf16-semantic-operation-signatures]
related: [spike-bf16-through-the-second-dtype-seams, widen-the-f16-operation-vocabulary-to-contraction-and-reassociation, own-operation-family-support-matrix, redesign-the-delivered-realization-record-from-typed-evidence]
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
