---
id: admit-the-silu-activation-family
title: Admit the SiLU activation family
status: in-progress
priority: p1
dependencies: [scope-transformer-nonlinear-normalization-and-reductions, implement-the-typed-accuracy-contract-vocabulary, record-the-metal-elementary-function-accuracy-guarantee]
related: [admit-the-rms-normalization-family, admit-the-softmax-family, own-operation-family-support-matrix, design-attention-program-vertical, numerical-policy-contract]
scopes: [implementation/ir, implementation/reference, implementation/compiler, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, transcendental, activation, language-model, breadth]
claimed_from: todo
assignee: worker-silu
lease_expires_at: 1785556713
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

## Outcome

**Determination: (c) — blocked on an unmet gate.** The gate is now *located* rather than merely named, its two prerequisites are filed and linked as dependencies, and the harder half of it turned out to be closable from a primary source already retained in this repository. No code was written and no rung moved.

### The gate, derived

**Fact — the accuracy-contract carrier this ticket calls "the substance" does not exist in compiled code.** Two exact checks, run from the repository root, each currently returning nothing:

```sh
grep -rn --include='*.rs' -E 'ulp-reference-gap|UlpMetric|AccuracyContract|CorrectlyRounded|NamedElementaryProfile' crates/
grep -rln --include='*.rs' -i transcendental crates/
```

ADRs 0016 and 0042 are `accepted` and both are `implementation_status: not-started`, which the [support matrix](../docs/roadmap.md#operation-family-support-matrix) transcendentals row already asserts and which these checks confirm at this base. What does exist is the *permission* dimension that presupposes the carrier: `ApproximationEnvelope::Forbidden` in `crates/tiler-ir/src/schedule/numerics.rs` is documented as "approximate intrinsics are forbidden; every elementary function follows its own resolved accuracy contract", and there is no such contract for it to defer to.

**Fact — the missing carrier blocks registration, not merely execution.** Milestone 1 requires Tiler to "canonically serialize and reference-evaluate every enabled transcendental accuracy contract before admitting such an operation to the vertical slice", and the transcendentals row repeats it as that row's trigger. So the gate binds at R3, not at R6: there is no honest "register the key now, resolve the contract later" path. ADR 0016 supplies the structural reason — "transcendental accuracy participates in semantic, plan, artifact, reference, and explain identity" — so a key registered without its contract would carry a *wrong* identity rather than an incomplete one, and adding the contract afterwards would have to change it.

**Inference — the widening is real but it is not this ticket's, and that is a structural claim rather than a size objection.** Three separate reasons, each refutable on its own:

1. **The vocabulary is profile-wide public surface, not SiLU's.** ADR 0042's algebra is owned by [Numerical semantics](../docs/numerical-semantics.md) and is consumed by `Exp` (this ticket and softmax), `Rsqrt` (RMS normalization), and every later elementary function. Fixing it from one activation's call site is the premature-specialization failure the architectural contract names.
2. **Q-SEM-004 selects *tuples*, plural.** Its closure condition is an "operation/dtype/accuracy allowlist with reference and backend conformance evidence" — a profile-level selection that all three L3′ verticals draw from once, not three times.
3. **The evidence that shapes the first cut was unread until this ticket, and it changes the design.** See below: the applicable normative table supplies a *constant rational* bound for the precise family and an *input-dependent formula* for the fast family, which ADR 0042 routes to two different contract forms. A vocabulary designed without that table in hand would have got its first cut wrong.

**Fact — the ticket's stated user-visible outcome is separately blocked, by the limit that already held a sibling admission at R5.** "Have it execute" needs a recognized whole-program shape, and `select_supported_strategy` in `crates/tiler-compiler/src/request.rs` recognizes exactly two. [`admit-the-reindex-and-broadcast-operation-families`](admit-the-reindex-and-broadcast-operation-families.md) is `done` with these same five scopes, landed at R5, and filed [`reach-a-verified-kernel-through-the-structural-families`](reach-a-verified-kernel-through-the-structural-families.md) for exactly this reason. So even with the carrier in hand, this ticket's honest ceiling is R5 and the R6 half needs the same remainder ticket the structural families needed.

### What was found: D-4's backend half closes from a retained source

**Fact — the applicable normative guarantee was already on disk and nobody had read it.** Metal Shading Language Specification v4.1 (2026-06-04), retained at `docs/research/apple-targets/sources/apple-metal-shading-language-specification-v4.1-2026-06-04.pdf` (SHA-256 `41538b30d2f1140a5b2a0c84ce0a9f7b67bf0c707e224cfea0bfe5a44aa26cf5`), chapter 8 "Numerical Compliance", §8.4 "ULPs and Relative Error", Table 8.1, pages 368–370, states for single precision: **`exp` ≤ 4 ulp**, **`x / y` correctly rounded**, `1.0 / x` correctly rounded, `x + y` / `x - y` / `x * y` / `fma` correctly rounded, **`rsqrt` correctly rounded**, `sqrt` correctly rounded, `fmax`/`fmin` 0 ulp. The same entries appear in the retained v4 specification (2025-10-23), the revision matching the pinned toolchain's `-std=metal4.0`.

This is what **D-4** says it closes on — "an applicable normative guarantee ... establishes what the selected Metal intrinsics deliver" — and the [L3′ record](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md) calls D-4 "the single largest gap between this record and an admissible `Softmax`, `RmsNorm`, or `SiLU` key". It does not need a device. The [emission probe](../spikes/numerics/metal_transcendental_emission/README.md) established *which intrinsic* the governed flags select and stopped there by design; [Transcendental accuracy precedents](../docs/research/numerics/transcendental-accuracy-precedents.md) says only that Metal's tables "include ULP bounds, absolute-error regions, input-dependent formulas, and undefined regions" and quotes no entry. The gap was in the reading, not in the evidence.

**Inference — and it confirms this ticket's own numerical claim from the other side.** Under Table 8.1 the addition and the division in `y = x / (1 + Exp(-x))` are both correctly rounded, so the composition's only open tolerance is the exponential's, exactly as ADR 0024 fixes it on the semantic side.

**Fact — three gaps stop the number being adopted as `Ulp(tiler::ulp-reference-gap@1, 4)` without further work**, and each is carried into the filed evidence ticket rather than waved past:

1. **The metric definitions differ.** §8.4 reads: "If x is a real number that lies between two finite consecutive floating-point numbers a and b, without being equal to one of them, then ulp(x) = |b − a|, otherwise ulp(x) is the distance between the two nonequal finite floating-point numbers nearest x. Moreover, ulp(NaN) is NaN." `tiler::ulp-reference-gap@1` resolves the representable case explicitly (the *smaller* gap where predecessor and successor differ) and is defined only for finite `r` and `z`. Apple's second clause is silent at a power of two and defines `ulp(NaN)`. ADR 0042 forbids translating across metric definitions by name, so the bound needs its own metric key with a registered implication, or a derivation that the two agree over the domain in use.
2. **The applicability clause names a flag spelling Tiler does not use.** Table 8.2's caption is "with fast math enabled (which is the default unless you specify `-fno-fast-math`)", making Table 8.1 the non-fast-math table; the governed baseline is `-fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`. The emission probe measured that this selects `air.exp.f32` while the default selects `air.fast_exp.f32`, which is strong evidence that Table 8.1 is the applicable row — and it is an inference, not a quotation.
3. **The table gives accuracy and no exceptional-value contract.** There is no math-function edge-case table. §8.1 says denormal inputs and outputs "**may** be flushed to zero" — permissive, so it licenses neither declaration; §8.3 disables floating-point exceptions; §8.5 says a function in flush-to-zero mode may return any of four results and that "if an operand or result is flushed to zero, **the sign of the zero is undefined**". ADR 0042 requires those policies to be stated independently of the error metric, so for `exp` they remain `Unknown` from the specification.

**Inference — where §8.5's undefined zero sign does and does not bite, because overstating it in either direction is a defect.** It does **not** reach this ticket's `-88.73` band: there `Exp(-x)` overflows to `+inf` and the result is a finite negative divided by infinity, an exact `-0.0` under Table 8.1's correctly-rounded division and §8.1's guarantee that INF is supported with fast math disabled. **Measurement — no subnormal is produced anywhere in F32 SiLU**, computed in the division form over `numpy` F32: `silu(-88.7228)` is `0x82b173cc` (≈ −2.607 × 10⁻³⁷), a *normal* value more than twenty times the minimum normal `1.1754944e-38`, while `silu(-88.73)` is already exactly `0x80000000`. The result drops from normal straight to `-0.0` with no subnormal band for a flush policy to act on. The same run reproduces this ticket's other pinned values — `silu(-88.0)` is `0x83354ddc`, one ULP from the sigmoid-product form's `0x83354ddb`; `silu(-0.0)` is `0x80000000`; `silu(+0.0)` is `0x00000000`; `silu(+inf)` is `0x7f800000`; `silu(-inf)` is `0x7fc00000` — so the ticket's stated corpus is confirmed rather than assumed. §8.5 **does** reach the siblings: softmax's underflow band, where the L3′ measurement records exactly `+0.0` for a far-below-maximum contributor while a flushing target may produce a zero of either sign, and RMS normalization's subnormal row.

### Filed prerequisites

- [`implement-the-typed-accuracy-contract-vocabulary`](implement-the-typed-accuracy-contract-vocabulary.md) — ADR 0042's four contract forms, exact rational tolerances, the bounded predicates and their normalization, `tiler::ulp-reference-gap@1` with its dtype-compatibility rejection, the accuracy-domain predicate language with coverage verification, the five-step result-set composition, canonical serialization into identity, the conservative refinement relation, the classified evidence records, and a certified-enclosure reference evaluation of the predicate. Public surface throughout, so its acceptance is Tom's.
- [`record-the-metal-elementary-function-accuracy-guarantee`](record-the-metal-elementary-function-accuracy-guarantee.md) — the quoted Table 8.1 and Table 8.2 entries with exact provenance, the three gaps above, the §8.5 applicability split, and the Q-SEM-004 and D-4 consequences written where they are read.

Both are now dependencies of this ticket and of [`admit-the-rms-normalization-family`](admit-the-rms-normalization-family.md) and [`admit-the-softmax-family`](admit-the-softmax-family.md), which share the same gate — RMS normalization through `Rsqrt`, softmax through `Exp` again — so the block is structural rather than a status word.

### Deliberately not done

No crate was touched, so no explain digest moved; it remains `0b7759de2d9b5756` at `crates/tiler-compiler/src/explain.rs:3739`. The [support matrix](../docs/roadmap.md#operation-family-support-matrix) row and absence check 1 are **not** updated: both remain accurate — no transcendental operation, evaluator, or structured-kernel construct exists, and the row's own trigger (Q-SEM-004, and Milestone 1's rule) is exactly what this determination confirms is unmet. The `docs/` corrections this reading implies — the D-4 entry, the precedents record's unquoted Metal paragraph, and Q-SEM-004's backend-evidence half — belong to the filed evidence ticket, whose scopes reach them; `research/numerics` and `contracts/numerics` are outside this ticket's declared scopes and were not edited.
