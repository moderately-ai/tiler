---
id: admit-the-rms-normalization-family
title: Admit the RMS normalization family
status: in-progress
priority: p1
dependencies: [scope-transformer-nonlinear-normalization-and-reductions, admit-the-silu-activation-family, admit-the-reindex-and-broadcast-operation-families, implement-the-typed-accuracy-contract-vocabulary, record-the-metal-elementary-function-accuracy-guarantee]
related: [admit-the-softmax-family, implement-parallel-reduction-strategies, own-operation-family-support-matrix, design-attention-program-vertical]
scopes: [implementation/ir, implementation/reference, implementation/compiler, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, normalization, reduction, transcendental, language-model, breadth]
claimed_from: todo
assignee: worker-rms
lease_expires_at: 1785576713
---
## User-visible outcome

A program can state `rms_norm(x, weight, eps)` over a named axis and have it execute — the operation the selected workload performs 113 times per forward pass, and the second-largest requirement in it after the contraction.

## Evidence prerequisite

**Fact — the exact formula, from `Qwen3RMSNorm.forward` at lines 71–76 of the pinned `modeling_qwen3.py` (digest `704c914530530a1acb0b443add1f520404e3ac2c28c0ab7e16f80f86cfe8ccb2`).** `variance = hidden_states.pow(2).mean(-1, keepdim=True)` then `hidden_states * torch.rsqrt(variance + self.variance_epsilon)` then `self.weight * hidden_states.to(input_dtype)`. Three decisions the usual spelling hides: nothing is subtracted, so this is **not** layer normalization; `eps` is inside the `rsqrt` argument, not outside the root; and the operation uses `rsqrt`, not `1 / sqrt`. The [L3′ derivation](../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md) records all three with the F32 consequences.

**Measurement — `eps` is a semantic term and not a guard.** From the [reference-semantics probe](../spikes/numerics/transformer_reference_semantics/README.md): with `eps`, a zero row normalizes to zeros and a subnormal row to a normal `0x02081cb9`; without it, the same rows give NaN and `+inf`. `eps` also changes the result at an ordinary input, so it perturbs every output rather than activating near zero.

**Measurement — the silent-wrongness case.** Squaring overflows at `0x5f7fffff` (≈ 1.845 × 10¹⁹). A row of `1e20` gives a mean of squares of `+inf`, an `rsqrt` of zero, and a result of **all positive zeros** — finite, plausible, and wrong, with no NaN or infinity to reveal it. Whether the operation refuses is decision **D-3** of the derivation and this ticket settles it.

**Fact — volume and extents.** 113 occurrences: 57 over a static extent of 1024 (`input_layernorm` and `post_attention_layernorm` per layer, plus `model.norm`) and 56 over a static extent of 128 (`q_norm` and `k_norm` per layer). One operation, two extent classes: `Qwen3Attention.__init__` at line 195 constructs the per-head norms from the same class. Per forward pass that is 144,384·`T` squared contributors and 729·`T` reciprocal square roots.

**Fact — the broadcast operand.** The weight is `[1024]` against `[T, 1024]`, or `[128]` against `[T, 16, 128]`. `docs/ir.md` admits no implicit broadcasting and the rank-zero scalar admission does not cover it, which is why [`admit-the-reindex-and-broadcast-operation-families`](admit-the-reindex-and-broadcast-operation-families.md) is a dependency rather than a note.

**Measurement — Metal supplies the intrinsic.** The [emission probe](../spikes/numerics/metal_transcendental_emission/README.md) records `air.rsqrt.f32` under the governed flag set and `air.fast_rsqrt.f32` under the compiler default. What is absent is on Tiler's side: no reciprocal square root exists in the structured-kernel vocabulary.

## Required delivery

One vertical. It must carry:

- **Reference behaviour.** A governed `OpKey` carrying a reduced-axis attribute and the exact `eps` bits — `rms_norm_eps` is `1e-06`, not exactly representable in F32, and two normalizations differing only in that constant are different operations that must not share an identity, a cache subject, or a golden. The normative reference pins the mean-of-squares, the `eps` position inside the `rsqrt` argument, the choice of `rsqrt`, and the fact that the weight multiply follows the (F32-identity) conversion rather than preceding it. A `tiler-reference` evaluator implementing exactly that.
- **Compiler legality.** A fusion role for a sum reduction carrying an elementwise squaring prologue. `OrderedReduction` is the shape this was defined for, but it is held by the single registered strict-serial-sum key, so a role for this family is required or it yields no fusion legality at all. The division by the static extent is exact here because 1024 and 128 are powers of two; do not encode that exactness into the formula, because a non-power-of-two extent would then acquire a silent rounding.
- **Order and accumulation.** A strict ordered fold over the canonical contributor sequence unless a registered permission authorizes otherwise, with reassociation and permutation checked independently. The accumulator dtype is explicit and is decision **D-5**, owned by [`implement-parallel-reduction-strategies`](implement-parallel-reduction-strategies.md); consume that authority rather than defaulting from the element dtype.
- **Metal realization.** Structured-kernel constructs for the reciprocal square root and for a prologue-carrying sum, and an emission that selects the intrinsic the resolved accuracy contract admits rather than the one the compiler default would pick.
- **Explainable refusal.** Separate typed refusals for: a non-positive, non-finite, or NaN `eps` (rejected at construction — a zero `eps` is a different operation with a different domain, not a degenerate parameter); an absent, duplicated, or out-of-range reduced axis, naming the violated rule; a reduction topology the order permission does not cover, naming the missing dimension; and an accumulator narrower than the contract allows. Settle **D-3** and, if the answer is a refusal, note that it is a semantic precondition requiring a proof or a costed runtime scan rather than a free guard.
- **Bounded conformance evidence.** The zero row, a subnormal row (which diverges between the CPU reference and the subnormal-flushing qualified Metal row — record the divergence, do not tune it away), a row above the squaring-overflow threshold, both extent classes, and the exact worked example the derivation retains (`x = [3.0, 4.0]`, `w = [1.0, 2.0]`, whose F32 bits are recorded there). State exactly which extents and rows the evidence covers.
- **The matrix row.** Update the reductions row and the transcendentals row of the [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) in the same change.

## Non-goals

Layer normalization, a general `Rsqrt` key, a general mean reduction, and any normalization with a bias or a mean subtraction. The derivation establishes that this workload needs none of them, and widening the family to absorb layer normalization would silently change what an existing occurrence means.

## Reconsideration trigger

Active now: 113 occurrences per forward pass with no alternative spelling. If the workload is superseded, re-derive the formula and the `eps` value from the replacement's own reference rather than carrying these forward — the `eps` position in particular is not shared across architectures.

## Outcome

`tiler::rms-norm-f32@1` is registered as one atomic key, reference-evaluated, given fusion legality and a numerical capability row, and emitted to Metal. The honest ceiling is **R5**: `select_supported_strategy` recognizes two whole-program shapes and neither contains a normalization, so no program compiles through the key and the emission is exercised at the kernel layer. That limit was not widened.

### What the key pins, and what it withholds

Two same-shaped operands and two required attributes. The weight arrives already broadcast, because `docs/ir.md` admits no implicit broadcasting and the rank-zero scalar admission does not cover a per-channel weight — the `[N]` to `[T, N]` widening is a `tiler::broadcast-f32@1` occurrence the program writes, and `rms-norm.f32.weight-shape` is what refuses to absorb it. The reduced axis is a one-element *sequence*, deliberately: a bare integer would make "duplicated" a shape the attribute could not express, and therefore a refusal nothing could reach.

`eps` is carried as exact binary32 `FloatBits`, payload `0x358637bd` for the pinned workload's `1e-06`. Two occurrences differing only in that payload carry different attribute records and therefore different identities; `two_normalizations_differing_only_in_eps_carry_different_attributes` checks it at the binary32 *successor* of the governed payload, the smallest difference that exists.

The reference pins all three decisions the usual spelling hides — nothing subtracted, `eps` inside the reciprocal square root's argument, `Rsqrt` and not `1 / Sqrt` — plus the extent division as a *division* and the weight multiply *after* the identity conversion.

### The accuracy contract, and why it needs no metric

`AccuracyContractForm::Faithful`, and the form is the derivation rather than a fallback. Table 8.1 states `rsqrt` correctly rounded; §8.2 states that either round-ties-to-even or round-toward-zero may be supported. A correctly rounded result under either mode is a member of the faithful pair, and at any argument above the midpoint both members are reachable, so the promised set *is* the faithful set — tight, not conservative.

Two consequences, both load-bearing. `CorrectlyRounded { NearestTiesToEven }` would be a **stronger** claim than the specification supports, and because `refines` proves correctly-rounded-satisfies-faithful along a registered row, the over-claim would be *admitted* rather than rejected: `the_metal_normalization_declaration_is_not_stronger_than_the_specification` constructs it, shows it refines, and is the only thing standing between the build and a claim §8.2 declines to make. And a faithful contract is metric-free, so Gap 1's cross-metric reconciliation does not bind this family at all — this vertical registers **no** second `ScaledMetric` row, and `the_normalization_needs_no_registered_implication_at_all` proves it by stripping the registry empty and watching the normalization still admit while the activation's exponential refuses. Gaps 1 and 4 bind disjoint halves of Table 8.1, exactly as the accuracy record predicted.

### Decision D-3: define, not refuse

Refusal was eliminated on all three routes it could take. Construction sees shapes and attributes, never element values. A proved value domain would need an upper bound on `|x|` no program input supplies. A runtime scan is a *costed operation* — a second full pass over 144,384·`T` contributors per forward pass — whose answer needs either a host readback per occurrence or a device-side validation mechanism the bounded profile does not have. Defining the behaviour is what the pinned formula already means, and reproducing the reference model exactly is the correct outcome. The threshold is `RMS_NORM_F32_SQUARING_OVERFLOW_BITS`, the corpus carries a row above it, and [`scope-a-value-domain-precondition-for-squaring-overflow`](scope-a-value-domain-precondition-for-squaring-overflow.md) owns the deferred capability with its activation trigger.

### A finding the corpus turned up

**Measurement — the workload's own reference implementation does not satisfy the contract Table 8.1 supports.** The retained probe records `torch.rsqrt(1e-6f32)` as `0x4479ffff`. The exact reference is `1000.00000126…`, whose faithful pair is `(0x447a0000, 0x447a0001)`; `0x4479ffff` is one step below that pair, about `1.02` ULP out, and is exactly what the two-rounding `1 / sqrt` composition delivers — the substitution the pinned formula's choice of `rsqrt` exists to exclude. It propagates: the probe's `rms_subnormal_vector` is `0x02081cb9` where this reference gives `0x02081cba`, because the squares of `1e-40` underflow and both rows share `eps` as their reciprocal square root argument. Recorded, not tuned. The derivation reads that measurement as if it were the reference; correcting it is [`correct-the-l3-prime-record-for-the-reference-rsqrt-divergence`](correct-the-l3-prime-record-for-the-reference-rsqrt-divergence.md), which also owes D-3's and D-4's `rsqrt` half's closure — `docs/research/numerics/**` is outside this ticket's declared scopes.

### Bounded conformance evidence

The retained worked example at its recorded bits and every intermediate; a zero row; a signed-zero row; a subnormal row with both divergences named; a row above the squaring-overflow threshold plus the threshold itself and its successor; both workload extent classes at 1024 and 128, uniform and holed, with results that differ between the two so a swapped extent is detectable; an axis-selection row; a weight-association row chosen at an element where the two associations disagree; and a contiguous 512-argument sweep of the reciprocal square root against its exact enclosure. That is the population. Nothing here generalizes beyond it.

### Compiler legality and the structured-kernel constructs

`FusionOperationRole::PrologueCarryingOrderedReduction` is a new role rather than a reuse of `OrderedReduction`: that role's contract is that the operation *is* a fold, and classifying a normalization as one would state that fusing it can only move a fold order when it also carries seven per-point roundings. It counts under the reduction total rather than gaining a count of its own, because the four role counts sum to the member count and a fifth field would move every previously encodable region's content identity.

The capability row adds **contraction** to the reduction dimensions, and the addition is derived: the per-contributor step is `accumulator + x_i * x_i`, so a multiply sits beside the fold's add where a bare serial sum has none. `the_normalization_consumes_the_reduction_dimensions_and_contraction` asserts the difference against the serial sum's row directly.

`UnaryOp::F32Rsqrt` (tag `0x02`), `PointwiseF32Node::Rsqrt` (tag `0x07`), and `ScalarProgram::SquaredSerialSum` (tag `0x26`). `SILU_UNCARRIED_DIMENSIONS` was renamed `ELEMENTARY_UNCARRIED_DIMENSIONS`, because the normalization contains a division and an elementary function too and a constant named for one family while governing two is a stale comment.

**No index-access lowering capability is registered, and the reason is structural rather than a deferral.** A normalization occurrence realizes as *two* regions — a reduction producing a shared row intermediate, then an elementwise pass consuming it — while `GovernedIndexAccess` emits exactly one region per occurrence. Emitting it as one would re-evaluate the whole fold at every output point: about 10⁶ scalar nodes per row at extent 1024, which the index region's structural bounds refuse. [`lower-a-two-region-occurrence-through-one-index-access-capability`](lower-a-two-region-occurrence-through-one-index-access-capability.md) owns the widening. Consequently no `rsqrt` scalar key was minted either: the scalar vocabulary is the index-region lowering's, and adding a key nothing emits would be dead vocabulary.

### Neither identity domain steps

Every new construct is an appended tag on an existing enum, with no field inserted into any repeating record. `UnaryOp::F32Exp` keeps `0x01`; `PointwiseF32Node`'s six existing tags keep `0x01`–`0x06`; `TAG_SCALAR_SERIAL_SUM` through `TAG_SCALAR_STRICT_AFFINE_U4_DEQUANTIZE` keep `0x22`–`0x25`; `FusionRegionStructure` gained no field. The evidence is the whole suite passing unchanged, including every pinned kernel digest and Metal golden, plus two explicit separation checks:
`the_reciprocal_square_root_node_separates_identity_from_the_exponential` and `the_squaring_prologue_reduction_has_its_own_canonical_identity`.

The one pin that moved is the explain request qualifier, `50c735514f5d51ca` → `b8ffa37f3d2dc86b`, and only its *semantic* half — the snapshot admits one more family carrying a second resolved accuracy contract in a form no earlier snapshot contained, plus the registry's first `FloatBits` attribute. No governed scalar key and no lowering capability moved with it.

### Watched failures

Every refusal below was perturbed and observed to fire.

- Axis: `absent`, `duplicated`, `rank` (a second axis), `range`, `type`. Arity refuses under the schema's own `tiler.schema.operand-arity` before the inferencer sees it; the inferencer's arm stays as the direct-call answer.
- `eps`: `zero` (both signed zeros), `negative`, `non-finite` (both infinities), `nan`, `format`. The governed payload is the control in the same test.
- Weight: `weight-shape`, at both a narrow `[N]` weight and a rank-zero scalar.
- Accumulator: a squaring-prologue multi-pass region declaring `F16` or `Bf16` refuses with `NumericalOrAccessRefinement`, with the F32 region as the control. The check is the schedule verifier's single accumulation authority, consumed rather than restated, so the key's declared `tiler::f32@1` fact and the verifier cannot disagree.
- Reduction topology: the squaring prologue applied to a *final* pass refuses, because squaring a partial sum squares an already-folded value.
- Reference: a deliberately coarse enclosure stops establishing the correctly rounded value and the reference answers `UndecidedTranscendentalReference` rather than guessing; a non-positive `eps` refuses in the evaluator as well as at construction.
- Accuracy: `0x4479ffff` and `0x447a0002` both `Violates` against the registered contract while both members of the faithful pair `Conform`; an empty implication registry leaves the normalization admitted and refuses the activation.

### Verification

`cargo fmt --all`; `cargo clippy --workspace --all-targets` minus the three prototypes, `-D warnings`, clean; `cargo nextest run --workspace` 2064 passed; `cargo test --workspace --doc` clean; `tkt lint` clean; `git diff --check` clean.

### Filed

- [`scope-a-value-domain-precondition-for-squaring-overflow`](scope-a-value-domain-precondition-for-squaring-overflow.md) — D-3's deferred capability.
- [`lower-a-two-region-occurrence-through-one-index-access-capability`](lower-a-two-region-occurrence-through-one-index-access-capability.md) — the R6 blocker on the lowering side.
- [`correct-the-l3-prime-record-for-the-reference-rsqrt-divergence`](correct-the-l3-prime-record-for-the-reference-rsqrt-divergence.md) — the derivation edits this ticket's scopes do not reach.
