---
id: admit-the-silu-activation-family
title: Admit the SiLU activation family
status: todo
priority: p1
dependencies: [scope-transformer-nonlinear-normalization-and-reductions]
related: [admit-the-rms-normalization-family, admit-the-softmax-family, own-operation-family-support-matrix, design-attention-program-vertical, numerical-policy-contract]
scopes: [implementation/ir, implementation/reference, implementation/compiler, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, transcendental, activation, language-model, breadth]
---
## User-visible outcome

A program can state `silu(x)` and have it execute — the first transcendental family in this project to reach a backend, and therefore the first end-to-end exercise of the accuracy-contract machinery ADRs 0016 and 0042 accepted and nothing has yet used.

## Why this one first

**Inference — from the [L3′ derivation](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md).** SiLU has one operand, one result, one transcendental, and no reduction. RMS normalization adds an order-sensitive reduction; softmax adds a second reduction, a second combiner family, and a growing symbolic extent. Delivering them in that order means each ticket adds exactly one new obligation, so a failure attributes to the obligation that was added. Delivering softmax first would make the project's first transcendental also its first multi-reduction fused operation.

## Evidence prerequisite

**Fact — the workload's activation is SiLU, and it is not a GELU.** `Qwen3MLP.__init__` at line 91 of the pinned `modeling_qwen3.py` (digest `704c914530530a1acb0b443add1f520404e3ac2c28c0ab7e16f80f86cfe8ccb2`) sets `act_fn = ACT2FN[config.hidden_act]`, `config.json` declares `hidden_act: "silu"`, and the [reference-semantics probe](../spikes/numerics/transformer_reference_semantics/README.md) resolves that name to `torch.nn.modules.activation.SiLU`. The erf-versus-tanh GELU question is a real distinction and is not this workload's; do not import a GELU contract here.

**Measurement — the two conventional spellings are different F32 operations.** Over the probe's boundary corpus, `x / (1 + exp(-x))` reproduces the reference at every input while `x * sigmoid(x)` differs at `-88.0` by one ULP (`0x83354ddb` against `0x83354ddc`). The derivation proposes the division form for that reason. A corpus without an input near the exponential's overflow threshold reports the two identical, which is how this would be got wrong.

**Measurement — Metal has no sigmoid and no SiLU.** `grep -rl sigmoid` over the pinned toolchain's `include/metal` returns nothing and a kernel calling `sigmoid(x)` fails to compile. There is no native intrinsic for the lowering to be tempted by; `air.exp.f32` is the only transcendental involved, and the [emission probe](../spikes/numerics/metal_transcendental_emission/README.md) records that the governed flag set selects it while the compiler default selects `air.fast_exp.f32` instead.

**Fact — volume.** 28 occurrences per forward pass over `[T, 3072]`, or 86,016·`T` scalar evaluations.

## Required delivery

One vertical. It must carry:

- **Reference behaviour.** A governed `OpKey` whose normative reference pins the exact formula `y = x / (1 + Exp(-x))`, including that the division form rather than the sigmoid-product form is the operation, and including the three exact round-to-nearest-ties-to-even boundaries ADR 0024 already fixes for the negation, the addition, and the division. A `tiler-reference` evaluator implementing exactly that.
- **The accuracy contract, which is the substance of this ticket.** A resolved ADR 0042 contract for the subordinate `Exp` over an immutable reference, with its exceptional-value, signed-zero, and subnormal policies stated independently of the error metric. This is [Q-SEM-004](../docs/open-questions.md#q-sem-004--first-profile-transcendental-tuples) instantiated on a named workload and it is decision **D-4** of the derivation. An empirical qualification is not a normative guarantee; if no applicable guarantee or exhaustive evaluation is reachable, the contract says so and the family's evidence class says `empirical` rather than borrowing a bound.
- **Compiler legality.** An `ElementwiseArithmetic` fusion role and an index-access lowering capability. Without a registered role the family yields no fusion legality at all, so it would be an optimization boundary in the middle of every MLP.
- **Metal realization.** A structured-kernel construct for the exponential — none exists today in `crates/tiler-ir/src/kernel/model.rs` — and an emission that selects the intrinsic the resolved accuracy contract admits. Selecting `air.fast_exp.f32` to satisfy a contract stated against the precise family is the substitution ADR 0076 forbids, and the emission probe shows it is one default flag away.
- **Explainable refusal.** A resolved accuracy contract no installed target realization refines must reject with the declaring profile's identity and the refusing fact's measurement boundary, not with a generic unsupported-operation error. Perturb the profile so the refusal actually fires before trusting it.
- **Bounded conformance evidence.** The probe's boundary corpus at minimum: signed zeros preserved (`silu(-0.0)` is `-0.0`), the `-88.73` band where the result is exactly `-0.0` because `exp(-x)` overflowed, `+inf` mapping to `+inf`, and **`-inf` mapping to NaN** — the reference is not total on the extended reals and the evidence must record that rather than repair it. State exactly which inputs the evidence covers and do not generalize to the family.
- **The matrix row.** Update the pointwise-transcendentals row of the [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) in the same change, and correct absence check 1, which will stop returning no output.

## Non-goals

A general `Exp` key, a general `Sigmoid` key, GELU in any form, and any activation this workload does not use. Milestone 1 forbids admitting a transcendental before its accuracy contract is canonically serialized and reference-evaluated end to end, which is why this is a vertical slice and not one more pointwise key — and widening it to a second activation would double the accuracy contracts before the first has been exercised once.

## Reconsideration trigger

Active now: the selected workload evaluates this 28 times per forward pass and no alternative spelling exists. If the workload is superseded, re-derive the activation from the replacement's own `hidden_act` rather than carrying `silu` forward.
